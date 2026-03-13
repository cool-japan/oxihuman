// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! BVH leaf-level utilities: per-triangle AABB building and leaf queries.

/// Axis-aligned bounding box for a leaf triangle.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct LeafAabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub face_index: usize,
}

/// Result of a leaf query.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LeafQueryResult {
    pub candidates: Vec<usize>,
}

/// Build per-face AABBs from triangle mesh.
#[allow(dead_code)]
pub fn build_leaf_aabbs(positions: &[[f32; 3]], indices: &[u32]) -> Vec<LeafAabb> {
    let n_faces = indices.len() / 3;
    let mut leaves = Vec::with_capacity(n_faces);
    for f in 0..n_faces {
        let i0 = indices[f * 3] as usize;
        let i1 = indices[f * 3 + 1] as usize;
        let i2 = indices[f * 3 + 2] as usize;
        if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
            continue;
        }
        let p0 = positions[i0];
        let p1 = positions[i1];
        let p2 = positions[i2];
        let min = [
            p0[0].min(p1[0]).min(p2[0]),
            p0[1].min(p1[1]).min(p2[1]),
            p0[2].min(p1[2]).min(p2[2]),
        ];
        let max = [
            p0[0].max(p1[0]).max(p2[0]),
            p0[1].max(p1[1]).max(p2[1]),
            p0[2].max(p1[2]).max(p2[2]),
        ];
        leaves.push(LeafAabb {
            min,
            max,
            face_index: f,
        });
    }
    leaves
}

/// Return all leaf indices whose AABB overlaps the query AABB.
#[allow(dead_code)]
pub fn query_leaves_aabb(leaves: &[LeafAabb], qmin: [f32; 3], qmax: [f32; 3]) -> LeafQueryResult {
    let candidates = leaves
        .iter()
        .filter(|l| {
            l.max[0] >= qmin[0]
                && l.min[0] <= qmax[0]
                && l.max[1] >= qmin[1]
                && l.min[1] <= qmax[1]
                && l.max[2] >= qmin[2]
                && l.min[2] <= qmax[2]
        })
        .map(|l| l.face_index)
        .collect();
    LeafQueryResult { candidates }
}

/// Surface area of a leaf AABB.
#[allow(dead_code)]
pub fn leaf_surface_area(leaf: &LeafAabb) -> f32 {
    let d = [
        leaf.max[0] - leaf.min[0],
        leaf.max[1] - leaf.min[1],
        leaf.max[2] - leaf.min[2],
    ];
    2.0 * (d[0] * d[1] + d[1] * d[2] + d[2] * d[0])
}

/// Volume of a leaf AABB.
#[allow(dead_code)]
pub fn leaf_volume(leaf: &LeafAabb) -> f32 {
    (leaf.max[0] - leaf.min[0]) * (leaf.max[1] - leaf.min[1]) * (leaf.max[2] - leaf.min[2])
}

/// Centroid of a leaf AABB.
#[allow(dead_code)]
pub fn leaf_centroid(leaf: &LeafAabb) -> [f32; 3] {
    [
        (leaf.min[0] + leaf.max[0]) * 0.5,
        (leaf.min[1] + leaf.max[1]) * 0.5,
        (leaf.min[2] + leaf.max[2]) * 0.5,
    ]
}

/// Number of leaves.
#[allow(dead_code)]
pub fn leaf_count(leaves: &[LeafAabb]) -> usize {
    leaves.len()
}

/// Average surface area across all leaves.
#[allow(dead_code)]
pub fn avg_leaf_surface_area(leaves: &[LeafAabb]) -> f32 {
    if leaves.is_empty() {
        return 0.0;
    }
    let sum: f32 = leaves.iter().map(leaf_surface_area).sum();
    sum / leaves.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_tri_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 1, 3, 2];
        (pos, idx)
    }

    #[test]
    fn leaf_count_two_tris() {
        let (pos, idx) = two_tri_mesh();
        let leaves = build_leaf_aabbs(&pos, &idx);
        assert_eq!(leaf_count(&leaves), 2);
    }

    #[test]
    fn leaf_min_max_sane() {
        let (pos, idx) = two_tri_mesh();
        let leaves = build_leaf_aabbs(&pos, &idx);
        assert!(!leaves.is_empty());
        let l = &leaves[0];
        assert!(l.min[0] <= l.max[0]);
        assert!(l.min[1] <= l.max[1]);
    }

    #[test]
    fn query_all_hits_with_large_aabb() {
        let (pos, idx) = two_tri_mesh();
        let leaves = build_leaf_aabbs(&pos, &idx);
        let res = query_leaves_aabb(&leaves, [-10.0; 3], [10.0; 3]);
        assert_eq!(res.candidates.len(), 2);
    }

    #[test]
    fn query_no_hits_outside() {
        let (pos, idx) = two_tri_mesh();
        let leaves = build_leaf_aabbs(&pos, &idx);
        let res = query_leaves_aabb(&leaves, [5.0; 3], [10.0; 3]);
        assert!(res.candidates.is_empty());
    }

    #[test]
    fn surface_area_nonneg() {
        let (pos, idx) = two_tri_mesh();
        let leaves = build_leaf_aabbs(&pos, &idx);
        for l in &leaves {
            assert!(leaf_surface_area(l) >= 0.0);
        }
    }

    #[test]
    fn volume_nonneg() {
        let (pos, idx) = two_tri_mesh();
        let leaves = build_leaf_aabbs(&pos, &idx);
        for l in &leaves {
            assert!(leaf_volume(l) >= 0.0);
        }
    }

    #[test]
    fn centroid_inside_aabb() {
        let (pos, idx) = two_tri_mesh();
        let leaves = build_leaf_aabbs(&pos, &idx);
        for l in &leaves {
            let c = leaf_centroid(l);
            assert!((0.0..=1.0).contains(&c[0]));
        }
    }

    #[test]
    fn avg_surface_area_positive() {
        let (pos, idx) = two_tri_mesh();
        let leaves = build_leaf_aabbs(&pos, &idx);
        assert!(avg_leaf_surface_area(&leaves) >= 0.0);
    }

    #[test]
    fn empty_leaves_avg_zero() {
        assert!((avg_leaf_surface_area(&[])).abs() < 1e-6);
    }

    #[test]
    fn face_index_correct() {
        let (pos, idx) = two_tri_mesh();
        let leaves = build_leaf_aabbs(&pos, &idx);
        assert_eq!(leaves[0].face_index, 0);
        assert_eq!(leaves[1].face_index, 1);
    }
}
