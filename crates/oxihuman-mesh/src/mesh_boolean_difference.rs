// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Mesh boolean difference (CSG subtraction) — volumetric SDF-based implementation.

use crate::mesh::MeshBuffers;
use crate::mesh_sdf::{combined_aabb, compute_sdf_on_bounds, sdf_subtraction, sdf_to_mesh};

/// Result of a boolean difference (A minus B).
#[derive(Debug, Clone, Default)]
pub struct BooleanDifferenceResult {
    pub vertices: Vec<[f32; 3]>,
    pub triangles: Vec<[u32; 3]>,
    pub removed_triangle_count: usize,
}

/// Configuration for boolean difference.
#[derive(Debug, Clone)]
pub struct BooleanDifferenceConfig {
    pub tolerance: f32,
    pub flip_normals_on_b: bool,
}

impl Default for BooleanDifferenceConfig {
    fn default() -> Self {
        Self {
            tolerance: 1e-6,
            flip_normals_on_b: true,
        }
    }
}

/// Build a `MeshBuffers` from raw vertex and flat-triangle-index slices.
fn build_mesh_buffers(verts: &[[f32; 3]], tris: &[[u32; 3]]) -> MeshBuffers {
    let n = verts.len();
    MeshBuffers {
        positions: verts.to_vec(),
        indices: tris.iter().flat_map(|t| [t[0], t[1], t[2]]).collect(),
        normals: vec![[0.0f32, 1.0, 0.0]; n],
        tangents: vec![[1.0f32, 0.0, 0.0, 1.0]; n],
        uvs: vec![[0.0f32; 2]; n],
        colors: None,
        has_suit: false,
    }
}

/// Compute A minus B via volumetric SDF subtraction.
///
/// Falls back to returning A unchanged when B is empty (mathematically correct —
/// subtracting nothing leaves A intact).
pub fn mesh_boolean_difference(
    verts_a: &[[f32; 3]],
    tris_a: &[[u32; 3]],
    verts_b: &[[f32; 3]],
    tris_b: &[[u32; 3]],
    _cfg: &BooleanDifferenceConfig,
) -> BooleanDifferenceResult {
    // Subtracting empty mesh or from empty mesh is trivial.
    if verts_b.is_empty() || tris_b.is_empty() {
        return BooleanDifferenceResult {
            vertices: verts_a.to_vec(),
            triangles: tris_a.to_vec(),
            removed_triangle_count: 0,
        };
    }
    if verts_a.is_empty() || tris_a.is_empty() {
        return BooleanDifferenceResult::default();
    }

    let original_tri_count = tris_a.len();

    // Build shared-grid SDF for both meshes then subtract.
    let mesh_a = build_mesh_buffers(verts_a, tris_a);
    let mesh_b = build_mesh_buffers(verts_b, tris_b);

    let (mn, mx) = combined_aabb(&mesh_a, &mesh_b, 0.1);
    let res = 32usize;
    let sdf_a = compute_sdf_on_bounds(&mesh_a, mn, mx, res, true);
    let sdf_b = compute_sdf_on_bounds(&mesh_b, mn, mx, res, true);

    let sdf_result = sdf_subtraction(&sdf_a, &sdf_b).unwrap_or(sdf_a);
    let mesh_out = sdf_to_mesh(&sdf_result, 0.0);

    let triangles: Vec<[u32; 3]> = mesh_out
        .indices
        .chunks_exact(3)
        .map(|c| [c[0], c[1], c[2]])
        .collect();

    let result_tri_count = triangles.len();
    let removed_triangle_count = original_tri_count.saturating_sub(result_tri_count);

    BooleanDifferenceResult {
        vertices: mesh_out.positions,
        triangles,
        removed_triangle_count,
    }
}

/// Flip triangle winding order (used to invert subtracted mesh normals).
pub fn flip_triangle_winding(tris: &[[u32; 3]]) -> Vec<[u32; 3]> {
    /* Swap first and last vertex of each triangle */
    tris.iter().map(|t| [t[2], t[1], t[0]]).collect()
}

/// Estimate how many triangles would be removed by the subtraction.
pub fn estimate_removed_triangles(
    verts_a: &[[f32; 3]],
    tris_a: &[[u32; 3]],
    verts_b: &[[f32; 3]],
) -> usize {
    /* Stub: count triangles whose centroid falls inside B's bounding box */
    if verts_b.is_empty() {
        return 0;
    }
    let mut bmin = [f32::MAX; 3];
    let mut bmax = [f32::MIN; 3];
    for v in verts_b {
        for k in 0..3 {
            if v[k] < bmin[k] {
                bmin[k] = v[k];
            }
            if v[k] > bmax[k] {
                bmax[k] = v[k];
            }
        }
    }
    tris_a
        .iter()
        .filter(|tri| {
            let mut c = [0.0f32; 3];
            for idx in tri.iter() {
                let vi = *idx as usize;
                if vi < verts_a.len() {
                    for k in 0..3 {
                        c[k] += verts_a[vi][k];
                    }
                }
            }
            c.iter_mut().for_each(|x| *x /= 3.0);
            (0..3).all(|k| (bmin[k]..=bmax[k]).contains(&c[k]))
        })
        .count()
}

/// Build a subtraction mask: true for each triangle in A that should be kept.
pub fn build_keep_mask(
    verts_a: &[[f32; 3]],
    tris_a: &[[u32; 3]],
    verts_b: &[[f32; 3]],
) -> Vec<bool> {
    /* Triangles outside B's bounding box are kept */
    if verts_b.is_empty() {
        return vec![true; tris_a.len()];
    }
    let mut bmin = [f32::MAX; 3];
    let mut bmax = [f32::MIN; 3];
    for v in verts_b {
        for k in 0..3 {
            if v[k] < bmin[k] {
                bmin[k] = v[k];
            }
            if v[k] > bmax[k] {
                bmax[k] = v[k];
            }
        }
    }
    tris_a
        .iter()
        .map(|tri| {
            let mut c = [0.0f32; 3];
            for idx in tri.iter() {
                let vi = *idx as usize;
                if vi < verts_a.len() {
                    for k in 0..3 {
                        c[k] += verts_a[vi][k];
                    }
                }
            }
            c.iter_mut().for_each(|x| *x /= 3.0);
            !(0..3).all(|k| (bmin[k]..=bmax[k]).contains(&c[k]))
        })
        .collect()
}

/// Filter triangles using a keep mask.
pub fn apply_keep_mask(tris: &[[u32; 3]], mask: &[bool]) -> Vec<[u32; 3]> {
    tris.iter()
        .zip(mask.iter())
        .filter_map(|(t, &keep)| if keep { Some(*t) } else { None })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difference_empty_b_returns_a() {
        let va = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let ta = vec![[0u32, 1, 2]];
        let cfg = BooleanDifferenceConfig::default();
        let result = mesh_boolean_difference(&va, &ta, &[], &[], &cfg);
        assert_eq!(result.vertices.len(), 3 /* A unchanged */);
        assert_eq!(result.triangles.len(), 1 /* A unchanged */);
    }

    #[test]
    fn test_flip_winding() {
        let tris = vec![[0u32, 1, 2]];
        let flipped = flip_triangle_winding(&tris);
        assert_eq!(flipped[0], [2, 1, 0] /* winding reversed */);
    }

    #[test]
    fn test_flip_winding_multiple() {
        let tris = vec![[0u32, 1, 2], [3, 4, 5]];
        let flipped = flip_triangle_winding(&tris);
        assert_eq!(flipped.len(), 2 /* same count */);
        assert_eq!(flipped[1], [5, 4, 3]);
    }

    #[test]
    fn test_estimate_removed_empty_b() {
        let va = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let ta = vec![[0u32, 1, 2]];
        assert_eq!(estimate_removed_triangles(&va, &ta, &[]), 0 /* no B */);
    }

    #[test]
    fn test_keep_mask_empty_b() {
        let va = vec![[0.0f32; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let ta = vec![[0u32, 1, 2]];
        let mask = build_keep_mask(&va, &ta, &[]);
        assert!(mask.iter().all(|&k| k) /* all kept when B is empty */);
    }

    #[test]
    fn test_apply_keep_mask_all_true() {
        let tris = vec![[0u32, 1, 2], [3, 4, 5]];
        let mask = vec![true, true];
        let result = apply_keep_mask(&tris, &mask);
        assert_eq!(result.len(), 2 /* all kept */);
    }

    #[test]
    fn test_apply_keep_mask_mixed() {
        let tris = vec![[0u32, 1, 2], [3, 4, 5]];
        let mask = vec![true, false];
        let result = apply_keep_mask(&tris, &mask);
        assert_eq!(result.len(), 1 /* second filtered */);
    }

    #[test]
    fn test_default_config_flip() {
        let cfg = BooleanDifferenceConfig::default();
        assert!(cfg.flip_normals_on_b /* should flip B normals by default */);
    }

    #[test]
    fn test_difference_result_removed_count() {
        let va = vec![[0.0f32; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let ta = vec![[0u32, 1, 2]];
        let cfg = BooleanDifferenceConfig::default();
        let result = mesh_boolean_difference(&va, &ta, &[], &[], &cfg);
        assert_eq!(result.removed_triangle_count, 0 /* none removed */);
    }

    /// Build a closed unit box half-size mesh via `shapes::box_mesh`.
    fn unit_box_half() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let mesh = crate::shapes::box_mesh([0.5, 0.5, 0.5]);
        let tris: Vec<[u32; 3]> = mesh
            .indices
            .chunks_exact(3)
            .map(|c| [c[0], c[1], c[2]])
            .collect();
        (mesh.positions, tris)
    }

    #[test]
    fn difference_a_minus_a_near_empty() {
        let cfg = BooleanDifferenceConfig::default();
        let (va, ta) = unit_box_half();
        let (vb, tb) = unit_box_half(); // identical geometry
        let result = mesh_boolean_difference(&va, &ta, &vb, &tb, &cfg);
        // Subtracting A from itself should leave near-empty (zero or very few
        // triangles, depending on numerical precision at the surface).
        assert!(
            result.triangles.len() <= 4,
            "A minus A should be near-empty, got {} triangles",
            result.triangles.len()
        );
    }
}
