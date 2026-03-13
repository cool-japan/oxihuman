// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Curvature-adaptive Loop subdivision that refines only high-curvature regions.

#![allow(dead_code)]

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
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(v: [f32; 3], t: f32) -> [f32; 3] {
    [v[0] * t, v[1] * t, v[2] * t]
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

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Configuration for curvature-adaptive Loop subdivision.
pub struct AdaptiveSubdivideConfig {
    /// Dihedral angle threshold in radians; faces above this get refined. Default 0.3.
    pub curvature_threshold: f32,
    /// Maximum recursion depth. Default 3.
    pub max_levels: u32,
    /// Smooth boundary edges. Default true.
    pub smooth_boundary: bool,
}

impl Default for AdaptiveSubdivideConfig {
    fn default() -> Self {
        Self {
            curvature_threshold: 0.3,
            max_levels: 3,
            smooth_boundary: true,
        }
    }
}

/// Result of adaptive subdivision.
pub struct AdaptiveSubdivideResult {
    pub mesh: MeshBuffers,
    pub refined_faces: usize,
    pub total_passes: u32,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Compute the face normal for a triangle with vertices p0, p1, p2.
pub fn face_normal(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> [f32; 3] {
    let e1 = sub3(p1, p0);
    let e2 = sub3(p2, p0);
    normalize3(cross3(e1, e2))
}

/// Compute the dihedral angle between two face normals (in radians, 0..PI).
pub fn dihedral_angle(n1: [f32; 3], n2: [f32; 3]) -> f32 {
    let d = dot3(normalize3(n1), normalize3(n2)).clamp(-1.0, 1.0);
    d.acos()
}

/// Build a per-vertex face adjacency list.
///
/// Returns a `Vec<Vec<u32>>` of length `n_verts`; entry `i` is the list of
/// face indices that reference vertex `i`.
pub fn build_face_adjacency(indices: &[u32], n_verts: usize) -> Vec<Vec<u32>> {
    let mut adj = vec![Vec::new(); n_verts];
    let n_faces = indices.len() / 3;
    for f in 0..n_faces {
        let i0 = indices[f * 3] as usize;
        let i1 = indices[f * 3 + 1] as usize;
        let i2 = indices[f * 3 + 2] as usize;
        if i0 < n_verts {
            adj[i0].push(f as u32);
        }
        if i1 < n_verts {
            adj[i1].push(f as u32);
        }
        if i2 < n_verts {
            adj[i2].push(f as u32);
        }
    }
    adj
}

/// Return the maximum dihedral angle across all three edges of a face.
///
/// `positions` is the full vertex buffer; `face` is the triangle ([v0, v1, v2]);
/// `adj_faces` is the complete index list expressed as `[[v0,v1,v2]; n_faces]`.
pub fn face_max_dihedral_angle(
    positions: &[[f32; 3]],
    face: [u32; 3],
    adj_faces: &[[u32; 3]],
) -> f32 {
    let [a, b, c] = face;
    let n_self = face_normal(
        positions[a as usize],
        positions[b as usize],
        positions[c as usize],
    );
    let edges = [(a, b), (b, c), (c, a)];
    let mut max_angle: f32 = 0.0;

    for (ea, eb) in edges {
        // Find a neighbour that shares exactly this edge (opposite direction).
        for adj in adj_faces {
            let [fa, fb, fc] = *adj;
            // Skip self.
            if fa == a && fb == b && fc == c {
                continue;
            }
            // Share edge if the neighbour contains both ea and eb.
            let verts = [fa, fb, fc];
            if verts.contains(&ea) && verts.contains(&eb) {
                let n_adj = face_normal(
                    positions[fa as usize],
                    positions[fb as usize],
                    positions[fc as usize],
                );
                let angle = dihedral_angle(n_self, n_adj);
                if angle > max_angle {
                    max_angle = angle;
                }
                break;
            }
        }
    }
    max_angle
}

/// Subdivide only the marked (true) triangles using Loop midpoint insertion.
/// Un-marked triangles pass through unchanged.  Returns a unified mesh.
pub fn loop_subdivide_marked(mesh: &MeshBuffers, face_mask: &[bool]) -> MeshBuffers {
    let n_faces = mesh.indices.len() / 3;
    let mut new_positions = mesh.positions.clone();
    let mut new_uvs = mesh.uvs.clone();
    let mut new_indices: Vec<u32> = Vec::new();

    // Edge midpoint cache: (min, max) → new vertex index.
    let mut edge_mid: std::collections::HashMap<(u32, u32), u32> = std::collections::HashMap::new();

    let mut get_or_create_mid =
        |positions: &mut Vec<[f32; 3]>, uvs: &mut Vec<[f32; 2]>, a: u32, b: u32| -> u32 {
            let key = (a.min(b), a.max(b));
            if let Some(&idx) = edge_mid.get(&key) {
                return idx;
            }
            let pa = positions[a as usize];
            let pb = positions[b as usize];
            let mid = scale3(add3(pa, pb), 0.5);
            let idx = positions.len() as u32;
            positions.push(mid);
            if !uvs.is_empty() {
                let ua = uvs[a as usize];
                let ub = uvs[b as usize];
                uvs.push([(ua[0] + ub[0]) * 0.5, (ua[1] + ub[1]) * 0.5]);
            }
            edge_mid.insert(key, idx);
            idx
        };

    for f in 0..n_faces {
        let v0 = mesh.indices[f * 3];
        let v1 = mesh.indices[f * 3 + 1];
        let v2 = mesh.indices[f * 3 + 2];

        let should_refine = face_mask.get(f).copied().unwrap_or(false);

        if should_refine {
            // Create midpoints for the three edges.
            let m01 = get_or_create_mid(&mut new_positions, &mut new_uvs, v0, v1);
            let m12 = get_or_create_mid(&mut new_positions, &mut new_uvs, v1, v2);
            let m20 = get_or_create_mid(&mut new_positions, &mut new_uvs, v2, v0);

            // Four sub-triangles.
            new_indices.extend_from_slice(&[v0, m01, m20]);
            new_indices.extend_from_slice(&[m01, v1, m12]);
            new_indices.extend_from_slice(&[m20, m12, v2]);
            new_indices.extend_from_slice(&[m01, m12, m20]);
        } else {
            new_indices.extend_from_slice(&[v0, v1, v2]);
        }
    }

    let n_verts = new_positions.len();
    let new_normals = if new_positions.len() > mesh.normals.len() {
        // Recompute normals for the whole mesh.
        let mut mesh_tmp = mesh.clone();
        mesh_tmp.positions = new_positions.clone();
        mesh_tmp.indices = new_indices.clone();
        mesh_tmp.normals = vec![[0.0, 0.0, 1.0]; n_verts];
        compute_normals(&mut mesh_tmp);
        mesh_tmp.normals
    } else {
        mesh.normals.clone()
    };

    MeshBuffers {
        positions: new_positions,
        normals: new_normals,
        tangents: vec![[1.0, 0.0, 0.0, 1.0]; n_verts],
        uvs: new_uvs,
        indices: new_indices,
        colors: None,
        has_suit: mesh.has_suit,
    }
}

/// Perform curvature-adaptive Loop subdivision.
pub fn adaptive_subdivide(
    mesh: &MeshBuffers,
    cfg: &AdaptiveSubdivideConfig,
) -> AdaptiveSubdivideResult {
    let mut current = mesh.clone();
    let mut total_refined = 0usize;
    let mut total_passes = 0u32;

    for _pass in 0..cfg.max_levels {
        let n_faces = current.indices.len() / 3;
        if n_faces == 0 {
            break;
        }

        // Build face list as [[v0,v1,v2]; n_faces] for adjacency queries.
        let face_list: Vec<[u32; 3]> = (0..n_faces)
            .map(|f| {
                [
                    current.indices[f * 3],
                    current.indices[f * 3 + 1],
                    current.indices[f * 3 + 2],
                ]
            })
            .collect();

        // Compute per-face max dihedral angle and build mask.
        let mut face_mask = vec![false; n_faces];
        let mut any_marked = false;
        for f in 0..n_faces {
            let angle = face_max_dihedral_angle(&current.positions, face_list[f], &face_list);
            if angle > cfg.curvature_threshold {
                face_mask[f] = true;
                any_marked = true;
            }
        }

        if !any_marked {
            break;
        }

        let refined_count = face_mask.iter().filter(|&&b| b).count();
        total_refined += refined_count;
        total_passes += 1;
        current = loop_subdivide_marked(&current, &face_mask);
    }

    AdaptiveSubdivideResult {
        mesh: current,
        refined_faces: total_refined,
        total_passes,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::MeshBuffers;

    fn flat_mesh() -> MeshBuffers {
        // Two triangles forming a flat quad in XY plane.
        // 0-(1,0)-2-(0,1)-1-(0,0) etc.
        let positions = vec![
            [0.0_f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let normals = vec![[0.0, 0.0, 1.0]; 4];
        let uvs = vec![[0.0, 0.0]; 4];
        let indices = vec![0u32, 1, 2, 0, 2, 3];
        MeshBuffers {
            positions,
            normals,
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 4],
            uvs,
            indices,
            colors: None,
            has_suit: false,
        }
    }

    fn folded_mesh() -> MeshBuffers {
        // Two triangles forming a 90-degree fold along the X axis.
        let positions = vec![
            [0.0_f32, 0.0, 0.0], // 0 – shared edge start
            [1.0, 0.0, 0.0],     // 1 – shared edge end
            [0.5, 1.0, 0.0],     // 2 – left face tip (XY plane)
            [0.5, 0.0, 1.0],     // 3 – right face tip (XZ plane)
        ];
        let normals = vec![[0.0, 0.0, 1.0]; 4];
        let uvs = vec![[0.0, 0.0]; 4];
        // Face 0: verts 0,1,2 → normal points roughly +Z
        // Face 1: verts 1,0,3 → normal points roughly +Y
        let indices = vec![0u32, 1, 2, 1, 0, 3];
        MeshBuffers {
            positions,
            normals,
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 4],
            uvs,
            indices,
            colors: None,
            has_suit: false,
        }
    }

    // ── face_normal ──────────────────────────────────────────────────────────

    #[test]
    fn face_normal_xy_plane_points_up() {
        let n = face_normal([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((n[2] - 1.0).abs() < 1e-5, "expected +Z normal, got {:?}", n);
    }

    #[test]
    fn face_normal_xz_plane_points_in_y() {
        let n = face_normal([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        // cross(+X, +Z) = -Y
        assert!(n[1] < 0.0, "expected -Y component, got {:?}", n);
    }

    #[test]
    fn face_normal_is_unit_length() {
        let n = face_normal([0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 3.0, 0.0]);
        let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!((l - 1.0).abs() < 1e-5);
    }

    // ── dihedral_angle ───────────────────────────────────────────────────────

    #[test]
    fn dihedral_angle_perpendicular_is_half_pi() {
        let n1 = [0.0_f32, 0.0, 1.0];
        let n2 = [0.0_f32, 1.0, 0.0];
        let angle = dihedral_angle(n1, n2);
        assert!(
            (angle - std::f32::consts::FRAC_PI_2).abs() < 1e-5,
            "got {}",
            angle
        );
    }

    #[test]
    fn dihedral_angle_flat_is_zero() {
        let n = [0.0_f32, 0.0, 1.0];
        let angle = dihedral_angle(n, n);
        assert!(angle.abs() < 1e-5, "got {}", angle);
    }

    #[test]
    fn dihedral_angle_anti_parallel_is_pi() {
        let n1 = [0.0_f32, 0.0, 1.0];
        let n2 = [0.0_f32, 0.0, -1.0];
        let angle = dihedral_angle(n1, n2);
        assert!((angle - std::f32::consts::PI).abs() < 1e-5, "got {}", angle);
    }

    // ── build_face_adjacency ─────────────────────────────────────────────────

    #[test]
    fn build_face_adjacency_single_triangle_vertex0() {
        let indices = vec![0u32, 1, 2];
        let adj = build_face_adjacency(&indices, 3);
        assert_eq!(adj[0], vec![0u32], "vertex 0 should be in face 0");
    }

    #[test]
    fn build_face_adjacency_two_triangles() {
        let indices = vec![0u32, 1, 2, 0, 2, 3];
        let adj = build_face_adjacency(&indices, 4);
        // vertex 0 is in face 0 and face 1
        assert_eq!(adj[0].len(), 2);
        // vertex 3 is only in face 1
        assert_eq!(adj[3], vec![1u32]);
    }

    // ── adaptive_subdivide ───────────────────────────────────────────────────

    #[test]
    fn adaptive_subdivide_flat_grid_zero_refined_faces() {
        let mesh = flat_mesh();
        let cfg = AdaptiveSubdivideConfig {
            curvature_threshold: 0.1, // very low threshold — flat faces should produce 0 angle
            max_levels: 3,
            smooth_boundary: true,
        };
        let result = adaptive_subdivide(&mesh, &cfg);
        assert_eq!(
            result.refined_faces, 0,
            "flat mesh should produce no refined faces"
        );
        assert_eq!(result.total_passes, 0);
    }

    #[test]
    fn adaptive_subdivide_folded_mesh_refines_fold() {
        let mesh = folded_mesh();
        let cfg = AdaptiveSubdivideConfig {
            curvature_threshold: 0.1,
            max_levels: 2,
            smooth_boundary: true,
        };
        let result = adaptive_subdivide(&mesh, &cfg);
        assert!(
            result.refined_faces > 0,
            "folded mesh should produce refined faces"
        );
    }

    #[test]
    fn adaptive_subdivide_result_has_valid_indices() {
        let mesh = folded_mesh();
        let cfg = AdaptiveSubdivideConfig::default();
        let result = adaptive_subdivide(&mesh, &cfg);
        let n_verts = result.mesh.positions.len() as u32;
        for &idx in &result.mesh.indices {
            assert!(idx < n_verts, "index {} out of range {}", idx, n_verts);
        }
    }

    #[test]
    fn loop_subdivide_marked_increases_face_count() {
        let mesh = flat_mesh();
        let face_mask = vec![true, true];
        let result = loop_subdivide_marked(&mesh, &face_mask);
        // Each marked triangle becomes 4 → 2 * 4 = 8
        assert_eq!(result.indices.len() / 3, 8);
    }

    #[test]
    fn loop_subdivide_marked_partial_refinement() {
        let mesh = flat_mesh();
        // Only refine first face.
        let face_mask = vec![true, false];
        let result = loop_subdivide_marked(&mesh, &face_mask);
        // First face → 4 triangles; second face → 1 triangle = 5 total.
        assert_eq!(result.indices.len() / 3, 5);
    }
}
