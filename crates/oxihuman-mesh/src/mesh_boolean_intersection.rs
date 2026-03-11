// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Mesh boolean intersection (CSG) stub.

/// Result of a mesh boolean intersection.
#[derive(Debug, Clone, Default)]
pub struct BooleanIntersectionResult {
    pub vertices: Vec<[f32; 3]>,
    pub triangles: Vec<[u32; 3]>,
}

/// Config for boolean intersection.
#[derive(Debug, Clone)]
pub struct BooleanIntersectionConfig {
    pub tolerance: f32,
    pub max_iterations: usize,
}

impl Default for BooleanIntersectionConfig {
    fn default() -> Self {
        Self {
            tolerance: 1e-6,
            max_iterations: 64,
        }
    }
}

/// Compute the intersection of two meshes (stub returns empty if no AABB overlap).
pub fn mesh_boolean_intersection(
    verts_a: &[[f32; 3]],
    tris_a: &[[u32; 3]],
    verts_b: &[[f32; 3]],
    tris_b: &[[u32; 3]],
    _cfg: &BooleanIntersectionConfig,
) -> BooleanIntersectionResult {
    /* Stub: return empty result if bounding boxes don't overlap */
    if !aabbs_overlap(verts_a, verts_b) || tris_a.is_empty() || tris_b.is_empty() {
        return BooleanIntersectionResult::default();
    }
    /* Placeholder: return mesh_a as a stand-in for the intersection region */
    BooleanIntersectionResult {
        vertices: verts_a.to_vec(),
        triangles: tris_a.to_vec(),
    }
}

/// Check if two vertex sets have overlapping axis-aligned bounding boxes.
pub fn aabbs_overlap(va: &[[f32; 3]], vb: &[[f32; 3]]) -> bool {
    /* Early out on empty input */
    if va.is_empty() || vb.is_empty() {
        return false;
    }
    let mut amin = [f32::MAX; 3];
    let mut amax = [f32::MIN; 3];
    for v in va {
        for k in 0..3 {
            if v[k] < amin[k] {
                amin[k] = v[k];
            }
            if v[k] > amax[k] {
                amax[k] = v[k];
            }
        }
    }
    let mut bmin = [f32::MAX; 3];
    let mut bmax = [f32::MIN; 3];
    for v in vb {
        for k in 0..3 {
            if v[k] < bmin[k] {
                bmin[k] = v[k];
            }
            if v[k] > bmax[k] {
                bmax[k] = v[k];
            }
        }
    }
    (0..3).all(|k| amin[k] <= bmax[k] && bmin[k] <= amax[k])
}

/// Classify which triangles of mesh A are inside mesh B's bounding box.
#[allow(clippy::needless_range_loop)]
pub fn triangles_inside_aabb(
    verts: &[[f32; 3]],
    tris: &[[u32; 3]],
    aabb_min: [f32; 3],
    aabb_max: [f32; 3],
) -> Vec<[u32; 3]> {
    /* Keep triangles whose centroid lies within the AABB */
    let mut result = Vec::new();
    for tri in tris {
        let mut centroid = [0.0f32; 3];
        for i in 0..3 {
            let vi = tri[i] as usize;
            if vi >= verts.len() {
                continue;
            }
            for k in 0..3 {
                centroid[k] += verts[vi][k];
            }
        }
        for k in 0..3 {
            centroid[k] /= 3.0;
        }
        let inside = (0..3).all(|k| (aabb_min[k]..=aabb_max[k]).contains(&centroid[k]));
        if inside {
            result.push(*tri);
        }
    }
    result
}

/// Count shared edges between two triangle meshes (stub).
pub fn count_shared_edge_candidates(tris_a: &[[u32; 3]], tris_b: &[[u32; 3]]) -> usize {
    /* Stub: estimate shared-edge count proportional to min triangle count */
    tris_a.len().min(tris_b.len())
}

/// Compute the volume of the intersection bounding box.
pub fn intersection_aabb_volume(va: &[[f32; 3]], vb: &[[f32; 3]]) -> f32 {
    /* Returns 0 if no overlap */
    if !aabbs_overlap(va, vb) {
        return 0.0;
    }
    let mut amin = [f32::MAX; 3];
    let mut amax = [f32::MIN; 3];
    for v in va {
        for k in 0..3 {
            if v[k] < amin[k] {
                amin[k] = v[k];
            }
            if v[k] > amax[k] {
                amax[k] = v[k];
            }
        }
    }
    let mut bmin = [f32::MAX; 3];
    let mut bmax = [f32::MIN; 3];
    for v in vb {
        for k in 0..3 {
            if v[k] < bmin[k] {
                bmin[k] = v[k];
            }
            if v[k] > bmax[k] {
                bmax[k] = v[k];
            }
        }
    }
    let mut vol = 1.0f32;
    for k in 0..3 {
        let lo = amin[k].max(bmin[k]);
        let hi = amax[k].min(bmax[k]);
        vol *= (hi - lo).max(0.0);
    }
    vol
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = BooleanIntersectionConfig::default();
        assert!(cfg.tolerance > 0.0 /* tolerance positive */);
    }

    #[test]
    fn test_intersection_empty_returns_empty() {
        let cfg = BooleanIntersectionConfig::default();
        let result = mesh_boolean_intersection(&[], &[], &[], &[], &cfg);
        assert!(result.vertices.is_empty() /* empty intersection */);
    }

    #[test]
    fn test_aabbs_overlap_true() {
        let va = vec![[0.0f32, 0.0, 0.0], [2.0, 2.0, 2.0]];
        let vb = vec![[1.0f32, 1.0, 1.0], [3.0, 3.0, 3.0]];
        assert!(aabbs_overlap(&va, &vb) /* overlapping */);
    }

    #[test]
    fn test_aabbs_no_overlap() {
        let va = vec![[0.0f32, 0.0, 0.0], [1.0, 1.0, 1.0]];
        let vb = vec![[5.0f32, 5.0, 5.0], [6.0, 6.0, 6.0]];
        assert!(!aabbs_overlap(&va, &vb) /* no overlap */);
    }

    #[test]
    fn test_aabbs_empty_input() {
        assert!(!aabbs_overlap(&[], &[[0.0f32, 0.0, 0.0]]) /* empty A */);
    }

    #[test]
    fn test_triangles_inside_aabb() {
        let verts = vec![[0.5f32, 0.5, 0.5], [0.6, 0.5, 0.5], [0.5, 0.6, 0.5]];
        let tris = vec![[0u32, 1, 2]];
        let inside = triangles_inside_aabb(&verts, &tris, [0.0; 3], [1.0; 3]);
        assert_eq!(inside.len(), 1 /* triangle centroid inside AABB */);
    }

    #[test]
    fn test_triangles_outside_aabb() {
        let verts = vec![[5.0f32, 5.0, 5.0], [6.0, 5.0, 5.0], [5.0, 6.0, 5.0]];
        let tris = vec![[0u32, 1, 2]];
        let inside = triangles_inside_aabb(&verts, &tris, [0.0; 3], [1.0; 3]);
        assert_eq!(inside.len(), 0 /* triangle outside AABB */);
    }

    #[test]
    fn test_count_shared_edge_candidates() {
        let ta = vec![[0u32, 1, 2], [1, 2, 3]];
        let tb = vec![[0u32, 1, 2]];
        assert_eq!(
            count_shared_edge_candidates(&ta, &tb),
            1 /* min(2,1) */
        );
    }

    #[test]
    fn test_intersection_aabb_volume() {
        let va = vec![[0.0f32, 0.0, 0.0], [2.0, 2.0, 2.0]];
        let vb = vec![[1.0f32, 1.0, 1.0], [3.0, 3.0, 3.0]];
        let vol = intersection_aabb_volume(&va, &vb);
        assert!((vol - 1.0).abs() < 1e-4 /* 1x1x1 overlap box */);
    }

    #[test]
    fn test_intersection_aabb_volume_no_overlap() {
        let va = vec![[0.0f32, 0.0, 0.0], [1.0, 1.0, 1.0]];
        let vb = vec![[2.0f32, 2.0, 2.0], [3.0, 3.0, 3.0]];
        assert_eq!(
            intersection_aabb_volume(&va, &vb),
            0.0 /* no overlap */
        );
    }
}
