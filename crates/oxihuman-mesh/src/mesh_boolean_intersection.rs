// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
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

use crate::mesh::MeshBuffers;
use crate::mesh_sdf::{combined_aabb, compute_sdf_on_bounds, sdf_intersection, sdf_to_mesh};

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

/// Compute the intersection of two meshes via volumetric SDF.
///
/// Returns an empty result when either mesh is empty or AABBs don't overlap.
pub fn mesh_boolean_intersection(
    verts_a: &[[f32; 3]],
    tris_a: &[[u32; 3]],
    verts_b: &[[f32; 3]],
    tris_b: &[[u32; 3]],
    _cfg: &BooleanIntersectionConfig,
) -> BooleanIntersectionResult {
    // If either mesh is empty or the AABBs don't overlap there is no
    // intersection — return an empty result immediately.
    if verts_a.is_empty()
        || tris_a.is_empty()
        || verts_b.is_empty()
        || tris_b.is_empty()
        || !aabbs_overlap(verts_a, verts_b)
    {
        return BooleanIntersectionResult::default();
    }

    let mesh_a = build_mesh_buffers(verts_a, tris_a);
    let mesh_b = build_mesh_buffers(verts_b, tris_b);

    let (mn, mx) = combined_aabb(&mesh_a, &mesh_b, 0.1);
    let res = 32usize;
    let sdf_a = compute_sdf_on_bounds(&mesh_a, mn, mx, res, true);
    let sdf_b = compute_sdf_on_bounds(&mesh_b, mn, mx, res, true);

    let sdf_result = sdf_intersection(&sdf_a, &sdf_b).unwrap_or(sdf_a);
    let mesh_out = sdf_to_mesh(&sdf_result, 0.0);

    let triangles: Vec<[u32; 3]> = mesh_out
        .indices
        .chunks_exact(3)
        .map(|c| [c[0], c[1], c[2]])
        .collect();

    BooleanIntersectionResult {
        vertices: mesh_out.positions,
        triangles,
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

    /// Build a closed box mesh centred at `offset` with half-size 0.5.
    fn box_at(offset: [f32; 3]) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let mesh = crate::shapes::box_mesh([0.5, 0.5, 0.5]);
        let verts: Vec<[f32; 3]> = mesh
            .positions
            .iter()
            .map(|p| [p[0] + offset[0], p[1] + offset[1], p[2] + offset[2]])
            .collect();
        let tris: Vec<[u32; 3]> = mesh
            .indices
            .chunks_exact(3)
            .map(|c| [c[0], c[1], c[2]])
            .collect();
        (verts, tris)
    }

    #[test]
    fn intersection_of_overlapping_boxes_is_non_empty() {
        let cfg = BooleanIntersectionConfig::default();
        let (va, ta) = box_at([0.0, 0.0, 0.0]);
        let (vb, tb) = box_at([0.25, 0.25, 0.25]); // partial overlap
        let result = mesh_boolean_intersection(&va, &ta, &vb, &tb, &cfg);
        assert!(
            !result.vertices.is_empty(),
            "overlapping boxes must have a non-empty intersection"
        );
    }

    #[test]
    fn intersection_of_non_overlapping_boxes_is_empty() {
        let cfg = BooleanIntersectionConfig::default();
        let (va, ta) = box_at([0.0, 0.0, 0.0]);
        let (vb, tb) = box_at([5.0, 5.0, 5.0]); // clearly disjoint
        let result = mesh_boolean_intersection(&va, &ta, &vb, &tb, &cfg);
        assert_eq!(
            result.vertices.len(),
            0,
            "disjoint boxes must have an empty intersection"
        );
    }
}
