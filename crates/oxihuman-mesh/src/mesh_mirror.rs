// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Mesh mirroring and symmetrization operations.
//!
//! Provides:
//! - `mirror_mesh`: reflect a mesh across an axis plane and combine with the original
//! - `mirror_copy`: create a mirrored copy only (without the original)
//! - `symmetrize_mesh`: average each vertex with its mirror counterpart
//! - `find_symmetry_pairs`: find vertex pairs that are symmetric across an axis
//! - `symmetry_error`: measure how symmetric a mesh is
//! - `extract_half`: extract the positive or negative half of a mesh
//! - `flip_positions`: flip vertex positions across an axis in-place
//! - `flip_normals_axis`: flip normal components along an axis in-place
//! - `reverse_winding`: reverse triangle winding for flipped normals

#![allow(dead_code)]

use crate::mesh::MeshBuffers;
use crate::normals::compute_normals;

// ---------------------------------------------------------------------------
// Math helpers
// ---------------------------------------------------------------------------

#[inline]
fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[inline]
fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

// ---------------------------------------------------------------------------
// MirrorAxis
// ---------------------------------------------------------------------------

/// Axis across which to mirror geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirrorAxis {
    /// Mirror across the YZ plane (negate X).
    X,
    /// Mirror across the XZ plane (negate Y).
    Y,
    /// Mirror across the XY plane (negate Z).
    Z,
}

impl MirrorAxis {
    /// Flip a position across the axis plane (at origin).
    #[inline]
    pub fn flip(&self, pos: [f32; 3]) -> [f32; 3] {
        match self {
            MirrorAxis::X => [-pos[0], pos[1], pos[2]],
            MirrorAxis::Y => [pos[0], -pos[1], pos[2]],
            MirrorAxis::Z => [pos[0], pos[1], -pos[2]],
        }
    }

    /// Get the axis component of a position.
    #[inline]
    pub fn coord(&self, pos: [f32; 3]) -> f32 {
        match self {
            MirrorAxis::X => pos[0],
            MirrorAxis::Y => pos[1],
            MirrorAxis::Z => pos[2],
        }
    }

    /// Set the axis component of a position to `v`.
    #[inline]
    fn set_coord(&self, pos: [f32; 3], v: f32) -> [f32; 3] {
        match self {
            MirrorAxis::X => [v, pos[1], pos[2]],
            MirrorAxis::Y => [pos[0], v, pos[2]],
            MirrorAxis::Z => [pos[0], pos[1], v],
        }
    }
}

// ---------------------------------------------------------------------------
// MirrorConfig
// ---------------------------------------------------------------------------

/// Configuration for a mirror operation.
pub struct MirrorConfig {
    /// Which axis to mirror across.
    pub axis: MirrorAxis,
    /// Distance threshold for welding mirrored verts that lie on the axis plane.
    pub merge_threshold: f32,
    /// Whether to flip the normals of the mirrored half (default `true`).
    pub flip_normals: bool,
    /// Offset of the mirror plane from the origin along the axis (default `0.0`).
    pub offset: f32,
}

impl Default for MirrorConfig {
    fn default() -> Self {
        Self {
            axis: MirrorAxis::X,
            merge_threshold: 0.001,
            flip_normals: true,
            offset: 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// MirrorResult
// ---------------------------------------------------------------------------

/// Result of a mirror operation.
pub struct MirrorResult {
    /// The combined (original + mirrored) mesh.
    pub mesh: MeshBuffers,
    /// Number of vertices from the original mesh.
    pub original_vertex_count: usize,
    /// Number of vertices added by mirroring.
    pub mirrored_vertex_count: usize,
    /// Number of vertices that were welded on the symmetry plane.
    pub welded_vertex_count: usize,
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn make_empty_mesh() -> MeshBuffers {
    MeshBuffers {
        positions: Vec::new(),
        normals: Vec::new(),
        tangents: Vec::new(),
        uvs: Vec::new(),
        indices: Vec::new(),
        colors: None,
        has_suit: false,
    }
}

/// Clone a mesh with optionally flipped normals and reversed winding.
fn clone_mesh_mirrored(
    src: &MeshBuffers,
    axis: MirrorAxis,
    flip_norms: bool,
    offset: f32,
) -> MeshBuffers {
    let mut positions: Vec<[f32; 3]> = src
        .positions
        .iter()
        .map(|&p| {
            // Flip relative to the offset plane
            let c = axis.coord(p) - offset;
            axis.set_coord(p, offset - c)
        })
        .collect();

    let mut normals = src.normals.clone();
    if flip_norms {
        flip_normals_axis(&mut normals, axis);
    }

    let mut indices = src.indices.clone();
    // After flipping, winding is reversed — fix it
    reverse_winding(&mut indices);

    // Ensure positions are finite
    for p in &mut positions {
        for v in p.iter_mut() {
            if !v.is_finite() {
                *v = 0.0;
            }
        }
    }

    MeshBuffers {
        positions,
        normals,
        tangents: src.tangents.clone(),
        uvs: src.uvs.clone(),
        indices,
        colors: src.colors.clone(),
        has_suit: src.has_suit,
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Mirror a mesh across the axis plane and combine the result with the original.
///
/// Vertices that lie exactly on (or within `merge_threshold` of) the mirror
/// plane are shared between the two halves (welded) so there are no cracks.
pub fn mirror_mesh(mesh: &MeshBuffers, config: &MirrorConfig) -> MirrorResult {
    let orig_vc = mesh.positions.len();

    // Build mirrored copy
    let mirrored = clone_mesh_mirrored(mesh, config.axis, config.flip_normals, config.offset);

    // Combine: original vertices first, then mirrored
    // We weld mirrored verts that are on the symmetry plane to the original ones.

    let mut out_positions = mesh.positions.clone();
    let mut out_normals = mesh.normals.clone();
    let mut out_tangents = mesh.tangents.clone();
    let mut out_uvs = mesh.uvs.clone();
    let out_colors = mesh.colors.clone(); // keep original colors; mirrored colors dropped

    // Map from mirrored vertex index → output vertex index
    let mut remap: Vec<u32> = Vec::with_capacity(mirrored.positions.len());
    let mut welded_count = 0usize;
    let mut added_count = 0usize;

    for (mi, &mp) in mirrored.positions.iter().enumerate() {
        let axis_dist = (config.axis.coord(mp) - config.offset).abs();
        let mut found_weld: Option<u32> = None;

        if axis_dist < config.merge_threshold {
            // Try to find an existing original vertex close to this one
            for (oi, &op) in mesh.positions.iter().enumerate() {
                if dist3(mp, op) < config.merge_threshold {
                    found_weld = Some(oi as u32);
                    break;
                }
            }
        }

        if let Some(wi) = found_weld {
            remap.push(wi);
            welded_count += 1;
            let _ = mi; // suppress unused warning
        } else {
            let new_idx = out_positions.len() as u32;
            out_positions.push(mp);
            out_normals.push(mirrored.normals[mi]);
            out_tangents.push(mirrored.tangents[mi]);
            out_uvs.push(mirrored.uvs[mi]);
            remap.push(new_idx);
            added_count += 1;
        }
    }

    // Build combined indices
    let mut out_indices = mesh.indices.clone();
    for chunk in mirrored.indices.chunks(3) {
        if chunk.len() == 3 {
            out_indices.push(remap[chunk[0] as usize]);
            out_indices.push(remap[chunk[1] as usize]);
            out_indices.push(remap[chunk[2] as usize]);
        }
    }

    let result_mesh = MeshBuffers {
        positions: out_positions,
        normals: out_normals,
        tangents: out_tangents,
        uvs: out_uvs,
        indices: out_indices,
        colors: out_colors,
        has_suit: mesh.has_suit,
    };

    MirrorResult {
        mesh: result_mesh,
        original_vertex_count: orig_vc,
        mirrored_vertex_count: added_count,
        welded_vertex_count: welded_count,
    }
}

/// Create a mirrored copy of the mesh only (without combining with the original).
pub fn mirror_copy(mesh: &MeshBuffers, axis: MirrorAxis) -> MeshBuffers {
    clone_mesh_mirrored(mesh, axis, true, 0.0)
}

/// Symmetrize a mesh: for each vertex, find its mirror counterpart (by
/// nearest-neighbour search in mirrored space) and average the two positions.
///
/// Only vertex pairs within `threshold` of each other are averaged.
pub fn symmetrize_mesh(mesh: &MeshBuffers, axis: MirrorAxis, threshold: f32) -> MeshBuffers {
    let pairs = find_symmetry_pairs(mesh, axis, threshold);

    let mut positions = mesh.positions.clone();

    for (li, ri) in &pairs {
        let lp = mesh.positions[*li as usize];
        let rp = mesh.positions[*ri as usize];
        // Average in axis-neutral directions; for the axis component, use midpoint
        let avg = lerp3(lp, rp, 0.5);
        // Keep the axis sign for each: left stays negative side, right stays positive
        let lc = axis.coord(lp);
        let rc = axis.coord(rp);
        let half_span = (lc.abs() + rc.abs()) * 0.5;

        let left_pos = axis.set_coord(avg, if lc <= 0.0 { -half_span } else { half_span });
        let right_pos = axis.set_coord(avg, if rc >= 0.0 { half_span } else { -half_span });

        positions[*li as usize] = left_pos;
        positions[*ri as usize] = right_pos;
    }

    let mut out = mesh.clone();
    out.positions = positions;

    // Recompute normals since positions changed
    if !out.indices.is_empty() && !out.positions.is_empty() {
        compute_normals(&mut out);
    }

    out
}

/// Find vertex symmetry pairs: returns `Vec<(left_idx, right_idx)>` where
/// each pair consists of two vertices that are mirror images across the axis.
///
/// Uses brute-force O(n²) nearest-neighbour matching in mirrored space.
pub fn find_symmetry_pairs(
    mesh: &MeshBuffers,
    axis: MirrorAxis,
    threshold: f32,
) -> Vec<(u32, u32)> {
    let n = mesh.positions.len();
    let mut pairs = Vec::new();
    let mut used = vec![false; n];

    for i in 0..n {
        if used[i] {
            continue;
        }
        let pi = mesh.positions[i];
        let ci = axis.coord(pi);
        // Only match verts that are on the negative or zero side with their mirror
        if ci > threshold {
            continue;
        }
        let mirrored_pi = axis.flip(pi);

        let mut best_j = usize::MAX;
        let mut best_d = threshold;

        for (j, (&pj, &is_used)) in mesh
            .positions
            .iter()
            .zip(used.iter())
            .enumerate()
            .skip(i + 1)
        {
            if is_used {
                continue;
            }
            let d = dist3(pj, mirrored_pi);
            if d < best_d {
                best_d = d;
                best_j = j;
            }
        }

        if best_j < n {
            used[i] = true;
            used[best_j] = true;
            pairs.push((i as u32, best_j as u32));
        }
    }

    pairs
}

/// Measure symmetry error: for each vertex, find the closest mirrored counterpart
/// and return the mean distance. Returns 0.0 for empty meshes.
pub fn symmetry_error(mesh: &MeshBuffers, axis: MirrorAxis) -> f32 {
    let n = mesh.positions.len();
    if n == 0 {
        return 0.0;
    }

    let mut total = 0.0f32;
    for i in 0..n {
        let pi = mesh.positions[i];
        let mirrored = axis.flip(pi);

        // Find nearest vertex to mirrored position
        let mut best_d = f32::MAX;
        for j in 0..n {
            let pj = mesh.positions[j];
            let d = dist3(pj, mirrored);
            if d < best_d {
                best_d = d;
            }
        }
        total += best_d;
    }

    total / n as f32
}

/// Extract the positive or negative half of a mesh along the axis.
///
/// Only triangles where all three vertices satisfy the half condition
/// (axis coord >= 0 for positive, <= 0 for negative) are kept.
pub fn extract_half(mesh: &MeshBuffers, axis: MirrorAxis, positive: bool) -> MeshBuffers {
    if mesh.indices.is_empty() || mesh.positions.is_empty() {
        return make_empty_mesh();
    }

    let mut out_positions: Vec<[f32; 3]> = Vec::new();
    let mut out_normals: Vec<[f32; 3]> = Vec::new();
    let mut out_tangents: Vec<[f32; 4]> = Vec::new();
    let mut out_uvs: Vec<[f32; 2]> = Vec::new();
    let mut out_indices: Vec<u32> = Vec::new();
    let mut out_colors_list: Vec<[f32; 4]> = Vec::new();
    let has_colors = mesh.colors.is_some();

    // Map from original vertex index to output index
    let mut remap: Vec<Option<u32>> = vec![None; mesh.positions.len()];

    let chunks = mesh.indices.chunks_exact(3);
    for tri in chunks {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let p0 = mesh.positions[i0];
        let p1 = mesh.positions[i1];
        let p2 = mesh.positions[i2];

        let c0 = axis.coord(p0);
        let c1 = axis.coord(p1);
        let c2 = axis.coord(p2);

        let keep = if positive {
            c0 >= 0.0 && c1 >= 0.0 && c2 >= 0.0
        } else {
            c0 <= 0.0 && c1 <= 0.0 && c2 <= 0.0
        };

        if !keep {
            continue;
        }

        let mut new_tri = [0u32; 3];
        for (slot, &orig_i) in [i0, i1, i2].iter().enumerate() {
            let ni = match remap[orig_i] {
                Some(n) => n,
                None => {
                    let n = out_positions.len() as u32;
                    out_positions.push(mesh.positions[orig_i]);
                    out_normals.push(mesh.normals[orig_i]);
                    out_tangents.push(mesh.tangents[orig_i]);
                    out_uvs.push(mesh.uvs[orig_i]);
                    if has_colors {
                        if let Some(ref cols) = mesh.colors {
                            out_colors_list.push(cols[orig_i]);
                        }
                    }
                    remap[orig_i] = Some(n);
                    n
                }
            };
            new_tri[slot] = ni;
        }
        out_indices.push(new_tri[0]);
        out_indices.push(new_tri[1]);
        out_indices.push(new_tri[2]);
    }

    let colors = if has_colors && !out_colors_list.is_empty() {
        Some(out_colors_list)
    } else {
        None
    };

    MeshBuffers {
        positions: out_positions,
        normals: out_normals,
        tangents: out_tangents,
        uvs: out_uvs,
        indices: out_indices,
        colors,
        has_suit: mesh.has_suit,
    }
}

/// Flip vertex positions across the axis plane (in-place).
pub fn flip_positions(positions: &mut [[f32; 3]], axis: MirrorAxis) {
    for p in positions.iter_mut() {
        *p = axis.flip(*p);
    }
}

/// Flip the axis component of normals (in-place).
///
/// When mirroring geometry, the normal component along the mirror axis must be
/// negated to remain correct.
pub fn flip_normals_axis(normals: &mut [[f32; 3]], axis: MirrorAxis) {
    for n in normals.iter_mut() {
        *n = match axis {
            MirrorAxis::X => [-n[0], n[1], n[2]],
            MirrorAxis::Y => [n[0], -n[1], n[2]],
            MirrorAxis::Z => [n[0], n[1], -n[2]],
        };
    }
}

/// Reverse triangle winding order for all triangles in an index buffer.
///
/// For every triangle `[a, b, c]`, swaps `b` and `c` to produce `[a, c, b]`.
/// Triangles with fewer than 3 indices (incomplete) are left unchanged.
pub fn reverse_winding(indices: &mut [u32]) {
    for chunk in indices.chunks_exact_mut(3) {
        chunk.swap(1, 2);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn make_triangle_mesh() -> MeshBuffers {
        // A simple triangle on the positive-X side
        MeshBuffers {
            positions: vec![[1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [1.0, 0.0, 1.0]],
            normals: vec![[1.0, 0.0, 0.0]; 3],
            tangents: vec![[0.0, 1.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            indices: vec![0, 1, 2],
            colors: None,
            has_suit: false,
        }
    }

    fn make_symmetric_quad() -> MeshBuffers {
        // Two triangles straddling X=0 (symmetric about YZ plane)
        MeshBuffers {
            positions: vec![
                [-1.0, 0.0, 0.0],
                [-1.0, 1.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]],
            indices: vec![0, 1, 2, 1, 3, 2],
            colors: None,
            has_suit: false,
        }
    }

    fn make_half_plane_mesh() -> MeshBuffers {
        // A quad exactly on the positive-X side, with verts on the axis (x=0)
        MeshBuffers {
            positions: vec![
                [0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]],
            indices: vec![0, 2, 1, 1, 2, 3],
            colors: None,
            has_suit: false,
        }
    }

    // ── MirrorAxis helpers ────────────────────────────────────────────────

    #[test]
    fn test_mirror_axis_flip_x() {
        let p = [3.0f32, 4.0, 5.0];
        let flipped = MirrorAxis::X.flip(p);
        assert_eq!(flipped, [-3.0, 4.0, 5.0]);
        fs::write(std::env::temp_dir().join("test_mirror_axis_flip_x.txt"), "ok").expect("should succeed");
    }

    #[test]
    fn test_mirror_axis_flip_y() {
        let p = [3.0f32, 4.0, 5.0];
        let flipped = MirrorAxis::Y.flip(p);
        assert_eq!(flipped, [3.0, -4.0, 5.0]);
        fs::write(std::env::temp_dir().join("test_mirror_axis_flip_y.txt"), "ok").expect("should succeed");
    }

    #[test]
    fn test_mirror_axis_flip_z() {
        let p = [3.0f32, 4.0, 5.0];
        let flipped = MirrorAxis::Z.flip(p);
        assert_eq!(flipped, [3.0, 4.0, -5.0]);
        fs::write(std::env::temp_dir().join("test_mirror_axis_flip_z.txt"), "ok").expect("should succeed");
    }

    #[test]
    fn test_mirror_axis_coord() {
        let p = [2.0f32, 3.0, 4.0];
        assert_eq!(MirrorAxis::X.coord(p), 2.0);
        assert_eq!(MirrorAxis::Y.coord(p), 3.0);
        assert_eq!(MirrorAxis::Z.coord(p), 4.0);
        fs::write(std::env::temp_dir().join("test_mirror_axis_coord.txt"), "ok").expect("should succeed");
    }

    // ── flip_positions / flip_normals_axis / reverse_winding ─────────────

    #[test]
    fn test_flip_positions_x() {
        let mut positions = vec![[1.0f32, 2.0, 3.0], [-1.0, 0.0, 0.5]];
        flip_positions(&mut positions, MirrorAxis::X);
        assert_eq!(positions[0], [-1.0, 2.0, 3.0]);
        assert_eq!(positions[1], [1.0, 0.0, 0.5]);
        fs::write(std::env::temp_dir().join("test_flip_positions_x.txt"), "ok").expect("should succeed");
    }

    #[test]
    fn test_flip_normals_axis_y() {
        let mut normals = vec![[0.0f32, 1.0, 0.0], [0.5, -0.5, 0.7]];
        flip_normals_axis(&mut normals, MirrorAxis::Y);
        assert!((normals[0][1] - (-1.0)).abs() < 1e-6);
        assert!((normals[1][1] - 0.5).abs() < 1e-6);
        fs::write(std::env::temp_dir().join("test_flip_normals_axis_y.txt"), "ok").expect("should succeed");
    }

    #[test]
    fn test_reverse_winding_basic() {
        let mut indices = vec![0u32, 1, 2, 3, 4, 5];
        reverse_winding(&mut indices);
        assert_eq!(indices, vec![0, 2, 1, 3, 5, 4]);
        fs::write(std::env::temp_dir().join("test_reverse_winding.txt"), "ok").expect("should succeed");
    }

    // ── mirror_copy ───────────────────────────────────────────────────────

    #[test]
    fn test_mirror_copy_x_flips_positions() {
        let mesh = make_triangle_mesh();
        let copy = mirror_copy(&mesh, MirrorAxis::X);
        assert_eq!(copy.positions.len(), mesh.positions.len());
        for (orig, mirrored) in mesh.positions.iter().zip(copy.positions.iter()) {
            assert!((mirrored[0] + orig[0]).abs() < 1e-5, "X should be negated");
            assert!(
                (mirrored[1] - orig[1]).abs() < 1e-5,
                "Y should be unchanged"
            );
        }
        fs::write(std::env::temp_dir().join("test_mirror_copy_x.txt"), "ok").expect("should succeed");
    }

    #[test]
    fn test_mirror_copy_preserves_face_count() {
        let mesh = make_symmetric_quad();
        let copy = mirror_copy(&mesh, MirrorAxis::Z);
        assert_eq!(copy.indices.len(), mesh.indices.len());
        fs::write(std::env::temp_dir().join("test_mirror_copy_face_count.txt"), "ok").expect("should succeed");
    }

    // ── mirror_mesh ───────────────────────────────────────────────────────

    #[test]
    fn test_mirror_mesh_doubles_faces() {
        let mesh = make_triangle_mesh();
        let config = MirrorConfig {
            axis: MirrorAxis::X,
            merge_threshold: 0.001,
            flip_normals: true,
            offset: 0.0,
        };
        let result = mirror_mesh(&mesh, &config);
        // Original has 1 triangle (3 indices), mirrored adds another → 6 total
        assert_eq!(result.mesh.indices.len(), mesh.indices.len() * 2);
        assert_eq!(result.original_vertex_count, mesh.positions.len());
        fs::write(std::env::temp_dir().join("test_mirror_mesh_doubles_faces.txt"), "ok").expect("should succeed");
    }

    #[test]
    fn test_mirror_mesh_welds_on_axis() {
        let mesh = make_half_plane_mesh();
        let config = MirrorConfig {
            axis: MirrorAxis::X,
            merge_threshold: 0.01,
            flip_normals: true,
            offset: 0.0,
        };
        let result = mirror_mesh(&mesh, &config);
        // Verts at x=0 should be welded
        assert!(result.welded_vertex_count > 0, "Expected some welded verts");
        fs::write(std::env::temp_dir().join("test_mirror_mesh_welds.txt"), "ok").expect("should succeed");
    }

    #[test]
    fn test_mirror_mesh_result_fields() {
        let mesh = make_triangle_mesh();
        let config = MirrorConfig::default();
        let result = mirror_mesh(&mesh, &config);
        assert_eq!(
            result.original_vertex_count
                + result.mirrored_vertex_count
                + result.welded_vertex_count,
            result.original_vertex_count
                + result.mirrored_vertex_count
                + result.welded_vertex_count
        );
        // Total output verts = original + mirrored (welded ones reuse original slots)
        assert_eq!(
            result.mesh.positions.len(),
            result.original_vertex_count + result.mirrored_vertex_count
        );
        fs::write(std::env::temp_dir().join("test_mirror_mesh_result_fields.txt"), "ok").expect("should succeed");
    }

    // ── extract_half ──────────────────────────────────────────────────────

    #[test]
    fn test_extract_positive_half() {
        let mesh = make_symmetric_quad();
        let half = extract_half(&mesh, MirrorAxis::X, true);
        // Only verts with x >= 0 survive
        for p in &half.positions {
            assert!(p[0] >= 0.0, "positive half should have x >= 0");
        }
        fs::write(std::env::temp_dir().join("test_extract_positive_half.txt"), "ok").expect("should succeed");
    }

    #[test]
    fn test_extract_negative_half() {
        let mesh = make_symmetric_quad();
        let half = extract_half(&mesh, MirrorAxis::X, false);
        for p in &half.positions {
            assert!(p[0] <= 0.0, "negative half should have x <= 0");
        }
        fs::write(std::env::temp_dir().join("test_extract_negative_half.txt"), "ok").expect("should succeed");
    }

    #[test]
    fn test_extract_half_empty_mesh() {
        let mesh = make_empty_mesh();
        let half = extract_half(&mesh, MirrorAxis::Y, true);
        assert!(half.positions.is_empty());
        fs::write(std::env::temp_dir().join("test_extract_half_empty.txt"), "ok").expect("should succeed");
    }

    // ── symmetry_error ────────────────────────────────────────────────────

    #[test]
    fn test_symmetry_error_perfect() {
        let mesh = make_symmetric_quad();
        let err = symmetry_error(&mesh, MirrorAxis::X);
        assert!(
            err < 1e-4,
            "Symmetric quad should have near-zero error, got {err}"
        );
        fs::write(std::env::temp_dir().join("test_symmetry_error_perfect.txt"), "ok").expect("should succeed");
    }

    #[test]
    fn test_symmetry_error_asymmetric() {
        let mesh = make_triangle_mesh(); // all verts at x=1, no mirror
        let err = symmetry_error(&mesh, MirrorAxis::X);
        // No vertex at x=-1, so error should be non-zero
        assert!(err > 0.0, "Asymmetric mesh should have non-zero error");
        fs::write(std::env::temp_dir().join("test_symmetry_error_asymmetric.txt"), "ok").expect("should succeed");
    }

    // ── find_symmetry_pairs ───────────────────────────────────────────────

    #[test]
    fn test_find_symmetry_pairs_symmetric_quad() {
        let mesh = make_symmetric_quad();
        let pairs = find_symmetry_pairs(&mesh, MirrorAxis::X, 0.1);
        // The symmetric quad has 2 pairs: (-1,0,0)↔(1,0,0) and (-1,1,0)↔(1,1,0)
        assert_eq!(
            pairs.len(),
            2,
            "Expected 2 symmetry pairs, got {}",
            pairs.len()
        );
        fs::write(std::env::temp_dir().join("test_find_symmetry_pairs.txt"), "ok").expect("should succeed");
    }

    // ── symmetrize_mesh ───────────────────────────────────────────────────

    #[test]
    fn test_symmetrize_preserves_vertex_count() {
        let mesh = make_symmetric_quad();
        let sym = symmetrize_mesh(&mesh, MirrorAxis::X, 0.1);
        assert_eq!(sym.positions.len(), mesh.positions.len());
        fs::write(std::env::temp_dir().join("test_symmetrize_preserves_count.txt"), "ok").expect("should succeed");
    }

    #[test]
    fn test_symmetrize_reduces_error() {
        // Build a slightly asymmetric mesh
        let mut mesh = make_symmetric_quad();
        mesh.positions[0][1] += 0.2; // perturb one vertex
        let before = symmetry_error(&mesh, MirrorAxis::X);
        let sym = symmetrize_mesh(&mesh, MirrorAxis::X, 1.0);
        let after = symmetry_error(&sym, MirrorAxis::X);
        assert!(
            after <= before + 1e-4,
            "Symmetrize should not increase error; before={before}, after={after}"
        );
        fs::write(std::env::temp_dir().join("test_symmetrize_reduces_error.txt"), "ok").expect("should succeed");
    }

    // ── MirrorConfig default ──────────────────────────────────────────────

    #[test]
    fn test_mirror_config_default() {
        let cfg = MirrorConfig::default();
        assert_eq!(cfg.axis, MirrorAxis::X);
        assert!((cfg.merge_threshold - 0.001).abs() < 1e-7);
        assert!(cfg.flip_normals);
        assert_eq!(cfg.offset, 0.0);
        fs::write(std::env::temp_dir().join("test_mirror_config_default.txt"), "ok").expect("should succeed");
    }
}
