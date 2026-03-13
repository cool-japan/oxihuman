// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Mesh boolean difference (CSG subtraction) stub.

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

/// Compute A minus B (stub: returns A unchanged when meshes don't overlap).
pub fn mesh_boolean_difference(
    verts_a: &[[f32; 3]],
    tris_a: &[[u32; 3]],
    verts_b: &[[f32; 3]],
    _tris_b: &[[u32; 3]],
    _cfg: &BooleanDifferenceConfig,
) -> BooleanDifferenceResult {
    /* Stub: if B is empty or no AABB overlap, return A unchanged */
    if verts_b.is_empty() {
        return BooleanDifferenceResult {
            vertices: verts_a.to_vec(),
            triangles: tris_a.to_vec(),
            removed_triangle_count: 0,
        };
    }
    BooleanDifferenceResult {
        vertices: verts_a.to_vec(),
        triangles: tris_a.to_vec(),
        removed_triangle_count: 0,
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
}
