// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Feature-preserving QEM (Quadric Error Metric) decimation that locks feature edges.

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

use crate::mesh::MeshBuffers;
use crate::normals::compute_normals;

// ---------------------------------------------------------------------------
// Math helpers
// ---------------------------------------------------------------------------

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[inline]
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l < 1e-10 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

fn face_normal_raw(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> [f32; 3] {
    normalize3(cross3(sub3(p1, p0), sub3(p2, p0)))
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Configuration for feature-preserving decimation.
pub struct FeatureDecimateConfig {
    /// Fraction of triangles to keep (0.0..1.0). Default 0.5.
    pub target_ratio: f32,
    /// Edges sharper than this angle (degrees) are locked. Default 30.0.
    pub feature_angle_deg: f32,
    /// Lock boundary edges. Default true.
    pub boundary_lock: bool,
    /// QEM error above this is never collapsed. Default f32::MAX.
    pub max_error: f32,
}

impl Default for FeatureDecimateConfig {
    fn default() -> Self {
        Self {
            target_ratio: 0.5,
            feature_angle_deg: 30.0,
            boundary_lock: true,
            max_error: f32::MAX,
        }
    }
}

/// Result of feature decimation.
pub struct FeatureDecimateResult {
    pub mesh: MeshBuffers,
    pub collapsed: usize,
    pub locked_edges: usize,
    pub final_error: f32,
}

// ---------------------------------------------------------------------------
// Public functions
// ---------------------------------------------------------------------------

/// Count the number of triangles with all three distinct vertex indices.
pub fn count_valid_triangles(indices: &[u32]) -> usize {
    let n = indices.len() / 3;
    let mut count = 0usize;
    for f in 0..n {
        let a = indices[f * 3];
        let b = indices[f * 3 + 1];
        let c = indices[f * 3 + 2];
        if a != b && b != c && a != c {
            count += 1;
        }
    }
    count
}

/// Compute the dihedral angle (radians) for each edge in the mesh.
///
/// The edge key is `(min(v0,v1), max(v0,v1))`.  The returned angle is the
/// angle between the normals of the two faces that share the edge, or `0.0`
/// for boundary edges (only one adjacent face).
pub fn compute_edge_dihedral(mesh: &MeshBuffers) -> HashMap<(u32, u32), f32> {
    let n_faces = mesh.indices.len() / 3;

    // Collect the face normal for each face.
    let face_normals: Vec<[f32; 3]> = (0..n_faces)
        .map(|f| {
            let a = mesh.indices[f * 3] as usize;
            let b = mesh.indices[f * 3 + 1] as usize;
            let c = mesh.indices[f * 3 + 2] as usize;
            face_normal_raw(mesh.positions[a], mesh.positions[b], mesh.positions[c])
        })
        .collect();

    // Map each canonical edge → list of adjacent face indices.
    let mut edge_faces: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for f in 0..n_faces {
        let verts = [
            mesh.indices[f * 3],
            mesh.indices[f * 3 + 1],
            mesh.indices[f * 3 + 2],
        ];
        for e in 0..3 {
            let a = verts[e];
            let b = verts[(e + 1) % 3];
            let key = (a.min(b), a.max(b));
            edge_faces.entry(key).or_default().push(f);
        }
    }

    let mut result = HashMap::new();
    for (edge, faces) in edge_faces {
        if faces.len() == 2 {
            let d = dot3(face_normals[faces[0]], face_normals[faces[1]]).clamp(-1.0, 1.0);
            result.insert(edge, d.acos());
        } else {
            result.insert(edge, 0.0);
        }
    }
    result
}

/// Return the set of locked (canonical min,max) edge pairs.
///
/// An edge is locked if its dihedral angle exceeds `angle_threshold_deg` or
/// if it is a boundary edge and `boundary_lock` is true.
pub fn mark_feature_edges(
    mesh: &MeshBuffers,
    angle_threshold_deg: f32,
    boundary_lock: bool,
) -> HashSet<(u32, u32)> {
    let threshold_rad = angle_threshold_deg.to_radians();
    let n_faces = mesh.indices.len() / 3;

    // Build edge → face count.
    let mut edge_face_count: HashMap<(u32, u32), usize> = HashMap::new();
    for f in 0..n_faces {
        let verts = [
            mesh.indices[f * 3],
            mesh.indices[f * 3 + 1],
            mesh.indices[f * 3 + 2],
        ];
        for e in 0..3 {
            let a = verts[e];
            let b = verts[(e + 1) % 3];
            let key = (a.min(b), a.max(b));
            *edge_face_count.entry(key).or_insert(0) += 1;
        }
    }

    let dihedral_map = compute_edge_dihedral(mesh);
    let mut locked = HashSet::new();

    for (edge, count) in &edge_face_count {
        if boundary_lock && *count == 1 {
            locked.insert(*edge);
            continue;
        }
        if let Some(&angle) = dihedral_map.get(edge) {
            if angle > threshold_rad {
                locked.insert(*edge);
            }
        }
    }

    locked
}

/// Squared-distance error proxy for an edge collapse (a → b).
pub fn edge_collapse_error(pos_a: [f32; 3], pos_b: [f32; 3]) -> f32 {
    let d = sub3(pos_a, pos_b);
    dot3(d, d)
}

/// Merge vertex `b` into vertex `a`, update all index references, and remove
/// degenerate triangles.
pub fn collapse_edge(positions: &mut Vec<[f32; 3]>, indices: &mut Vec<u32>, a: u32, b: u32) {
    // Remap all references to b → a.
    for idx in indices.iter_mut() {
        if *idx == b {
            *idx = a;
        }
    }

    // Remove degenerate triangles.
    let n = indices.len() / 3;
    let mut keep = vec![true; n];
    for f in 0..n {
        let v0 = indices[f * 3];
        let v1 = indices[f * 3 + 1];
        let v2 = indices[f * 3 + 2];
        if v0 == v1 || v1 == v2 || v0 == v2 {
            keep[f] = false;
        }
    }

    let new_indices: Vec<u32> = keep
        .iter()
        .enumerate()
        .flat_map(|(f, &k)| {
            if k {
                vec![indices[f * 3], indices[f * 3 + 1], indices[f * 3 + 2]]
            } else {
                vec![]
            }
        })
        .collect();

    *indices = new_indices;

    // Mark vertex b as "removed" by moving it to a (already done above).
    // The position slot stays (we don't compact to avoid index remapping
    // in the caller's loops).
    let _ = positions; // positions not compacted here
}

/// Feature-preserving decimation.
pub fn feature_decimate(mesh: &MeshBuffers, cfg: &FeatureDecimateConfig) -> FeatureDecimateResult {
    let mut positions = mesh.positions.clone();
    let mut indices = mesh.indices.clone();

    let initial_triangles = count_valid_triangles(&indices);
    let target = ((initial_triangles as f32) * cfg.target_ratio.clamp(0.0, 1.0)) as usize;

    let locked = mark_feature_edges(mesh, cfg.feature_angle_deg, cfg.boundary_lock);
    let locked_edges = locked.len();

    let mut collapsed = 0usize;
    let mut final_error = 0.0_f32;

    loop {
        let current = count_valid_triangles(&indices);
        if current <= target {
            break;
        }

        // Collect all non-locked edges present in the current index buffer.
        let mut candidate_edges: HashMap<(u32, u32), f32> = HashMap::new();
        let n_faces = indices.len() / 3;
        for f in 0..n_faces {
            let v = [indices[f * 3], indices[f * 3 + 1], indices[f * 3 + 2]];
            if v[0] == v[1] || v[1] == v[2] || v[0] == v[2] {
                continue;
            }
            for e in 0..3 {
                let a = v[e];
                let b = v[(e + 1) % 3];
                let key = (a.min(b), a.max(b));
                if locked.contains(&key) {
                    continue;
                }
                let err = edge_collapse_error(positions[a as usize], positions[b as usize]);
                if err <= cfg.max_error {
                    candidate_edges.entry(key).or_insert(err);
                }
            }
        }

        if candidate_edges.is_empty() {
            break;
        }

        // Pick the edge with lowest error.
        let Some((&(ea, eb), &err)) = candidate_edges
            .iter()
            .min_by(|x, y| x.1.partial_cmp(y.1).unwrap_or(std::cmp::Ordering::Equal))
        else {
            break;
        };

        collapse_edge(&mut positions, &mut indices, ea, eb);
        collapsed += 1;
        if err > final_error {
            final_error = err;
        }
    }

    // Rebuild a clean MeshBuffers.
    let n_verts = positions.len();
    let mut result_mesh = MeshBuffers {
        positions: positions.clone(),
        normals: vec![[0.0, 0.0, 1.0]; n_verts],
        tangents: vec![[1.0, 0.0, 0.0, 1.0]; n_verts],
        uvs: if mesh.uvs.len() == mesh.positions.len() {
            let mut u = mesh.uvs.clone();
            u.resize(n_verts, [0.0, 0.0]);
            u
        } else {
            vec![[0.0, 0.0]; n_verts]
        },
        indices,
        colors: None,
        has_suit: mesh.has_suit,
    };
    compute_normals(&mut result_mesh);

    FeatureDecimateResult {
        mesh: result_mesh,
        collapsed,
        locked_edges,
        final_error,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::MeshBuffers;

    /// Build a flat 2×2 grid (4 triangles) with no sharp features.
    fn flat_grid() -> MeshBuffers {
        //  3──2
        //  │╲ │╲
        //  │ ╲│ ╲
        //  0──1──? (but we use 4 verts × 2 tris)
        let positions = vec![
            [0.0_f32, 0.0, 0.0], // 0
            [1.0, 0.0, 0.0],     // 1
            [2.0, 0.0, 0.0],     // 2
            [0.0, 1.0, 0.0],     // 3
            [1.0, 1.0, 0.0],     // 4
            [2.0, 1.0, 0.0],     // 5
        ];
        let n = positions.len();
        let indices = vec![0u32, 1, 3, 1, 4, 3, 1, 2, 4, 2, 5, 4];
        MeshBuffers {
            positions,
            normals: vec![[0.0, 0.0, 1.0]; n],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; n],
            uvs: vec![[0.0, 0.0]; n],
            indices,
            colors: None,
            has_suit: false,
        }
    }

    /// A 90-degree fold: two triangles sharing an edge, each in a perpendicular plane.
    fn folded_mesh() -> MeshBuffers {
        let positions = vec![
            [0.0_f32, 0.0, 0.0], // 0
            [1.0, 0.0, 0.0],     // 1
            [0.5, 1.0, 0.0],     // 2  (XY plane)
            [0.5, 0.0, 1.0],     // 3  (XZ plane)
        ];
        let n = positions.len();
        let indices = vec![0u32, 1, 2, 1, 0, 3];
        MeshBuffers {
            positions,
            normals: vec![[0.0, 0.0, 1.0]; n],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; n],
            uvs: vec![[0.0, 0.0]; n],
            indices,
            colors: None,
            has_suit: false,
        }
    }

    // ── count_valid_triangles ────────────────────────────────────────────────

    #[test]
    fn count_valid_triangles_all_valid() {
        let indices = vec![0u32, 1, 2, 3, 4, 5];
        assert_eq!(count_valid_triangles(&indices), 2);
    }

    #[test]
    fn count_valid_triangles_degenerate_excluded() {
        // Second triangle is degenerate (two same indices).
        let indices = vec![0u32, 1, 2, 3, 3, 5];
        assert_eq!(count_valid_triangles(&indices), 1);
    }

    #[test]
    fn count_valid_triangles_empty() {
        assert_eq!(count_valid_triangles(&[]), 0);
    }

    // ── mark_feature_edges ───────────────────────────────────────────────────

    #[test]
    fn mark_feature_edges_flat_mesh_no_sharp_edges() {
        let mesh = flat_grid();
        // Disable boundary lock; threshold 30 deg → no interior edges should be sharp on a flat grid.
        let locked = mark_feature_edges(&mesh, 30.0, false);
        assert!(
            locked.is_empty(),
            "flat mesh has no sharp feature edges: {:?}",
            locked
        );
    }

    #[test]
    fn mark_feature_edges_folded_mesh_marks_shared_edge() {
        let mesh = folded_mesh();
        // Shared edge (0,1) has ~90° dihedral; threshold 30° → should be locked.
        let locked = mark_feature_edges(&mesh, 30.0, false);
        let key = (0u32, 1u32);
        assert!(
            locked.contains(&key),
            "shared fold edge should be locked: {:?}",
            locked
        );
    }

    #[test]
    fn mark_feature_edges_boundary_lock_enabled() {
        let mesh = flat_grid();
        let locked = mark_feature_edges(&mesh, 30.0, true);
        // All boundary edges should be locked.
        assert!(!locked.is_empty(), "boundary edges should be locked");
    }

    #[test]
    fn mark_feature_edges_boundary_lock_disabled() {
        let mesh = flat_grid();
        let locked = mark_feature_edges(&mesh, 30.0, false);
        // Flat grid has no sharp interior edges and boundary lock is off.
        assert!(locked.is_empty());
    }

    // ── collapse_edge ────────────────────────────────────────────────────────

    #[test]
    fn collapse_edge_removes_degenerate_triangles() {
        let mut positions = vec![[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let mut indices = vec![0u32, 1, 2];
        // Collapse edge 0→1: vertex 1 becomes vertex 0 → degenerate triangle 0,0,2 → removed.
        collapse_edge(&mut positions, &mut indices, 0, 1);
        assert_eq!(
            count_valid_triangles(&indices),
            0,
            "triangle should be degenerate after collapse"
        );
    }

    #[test]
    fn collapse_edge_remaps_indices() {
        let mut positions = vec![
            [0.0_f32, 0.0, 0.0], // 0
            [1.0, 0.0, 0.0],     // 1
            [0.5, 1.0, 0.0],     // 2
            [1.5, 1.0, 0.0],     // 3
        ];
        let mut indices = vec![0u32, 1, 2, 1, 3, 2];
        // Collapse 1 into 0 → 1 becomes 0.
        collapse_edge(&mut positions, &mut indices, 0, 1);
        // All refs to vertex 1 should now be 0.
        assert!(
            !indices.contains(&1u32),
            "no references to collapsed vertex 1"
        );
    }

    // ── edge_collapse_error ──────────────────────────────────────────────────

    #[test]
    fn edge_collapse_error_same_position_is_zero() {
        let p = [1.0_f32, 2.0, 3.0];
        assert!(edge_collapse_error(p, p).abs() < 1e-6);
    }

    #[test]
    fn edge_collapse_error_unit_distance() {
        let a = [0.0_f32, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        assert!((edge_collapse_error(a, b) - 1.0).abs() < 1e-6);
    }

    // ── feature_decimate ────────────────────────────────────────────────────

    #[test]
    fn feature_decimate_flat_grid_reduces_faces() {
        let mesh = flat_grid();
        let initial = count_valid_triangles(&mesh.indices);
        let cfg = FeatureDecimateConfig {
            target_ratio: 0.5,
            feature_angle_deg: 30.0,
            boundary_lock: false,
            max_error: f32::MAX,
        };
        let result = feature_decimate(&mesh, &cfg);
        let final_count = count_valid_triangles(&result.mesh.indices);
        assert!(
            final_count < initial,
            "decimation should reduce triangle count"
        );
    }

    #[test]
    fn feature_decimate_respects_target_ratio() {
        let mesh = flat_grid();
        let cfg = FeatureDecimateConfig {
            target_ratio: 0.5,
            feature_angle_deg: 30.0,
            boundary_lock: false,
            max_error: f32::MAX,
        };
        let initial = count_valid_triangles(&mesh.indices);
        let result = feature_decimate(&mesh, &cfg);
        let final_count = count_valid_triangles(&result.mesh.indices);
        let actual_ratio = final_count as f32 / initial as f32;
        // Allow ±1 triangle of slack.
        assert!(actual_ratio <= 0.75, "ratio {} too high", actual_ratio);
    }

    #[test]
    fn feature_decimate_locked_edges_survive() {
        let mesh = folded_mesh();
        // Use boundary_lock: true so boundary edges protect the fold faces from
        // being entirely collapsed when targeting an extreme ratio.
        let cfg = FeatureDecimateConfig {
            target_ratio: 0.1, // try to decimate heavily
            feature_angle_deg: 30.0,
            boundary_lock: true,
            max_error: f32::MAX,
        };
        let result = feature_decimate(&mesh, &cfg);
        // The fold edge (0,1) is ~90°; must be locked → locked_edges > 0.
        assert!(
            result.locked_edges > 0,
            "feature + boundary edges must be locked"
        );
        // With boundary_lock the fold's boundary edges are protected; at
        // least one valid triangle survives.
        assert!(
            count_valid_triangles(&result.mesh.indices) >= 1,
            "at least one face should survive"
        );
    }

    #[test]
    fn feature_decimate_boundary_lock_keeps_edges() {
        let mesh = flat_grid();
        let cfg = FeatureDecimateConfig {
            target_ratio: 0.1,
            feature_angle_deg: 90.0, // high threshold → only boundary locked
            boundary_lock: true,
            max_error: f32::MAX,
        };
        let result = feature_decimate(&mesh, &cfg);
        assert!(result.locked_edges > 0, "boundary edges should be locked");
    }
}
