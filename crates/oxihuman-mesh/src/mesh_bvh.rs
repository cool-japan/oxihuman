// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Bounding Volume Hierarchy (BVH) for ray/point queries over triangle meshes.

#![allow(dead_code)]

/// Axis-aligned bounding box.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct BvhAabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl BvhAabb {
    /// Create an empty AABB (inverted extremes).
    #[allow(dead_code)]
    pub fn empty() -> Self {
        Self {
            min: [f32::MAX; 3],
            max: [f32::MIN; 3],
        }
    }

    /// Expand to include `pt`.
    #[allow(dead_code)]
    #[allow(clippy::needless_range_loop)]
    pub fn expand(&mut self, pt: [f32; 3]) {
        for i in 0..3 {
            self.min[i] = self.min[i].min(pt[i]);
            self.max[i] = self.max[i].max(pt[i]);
        }
    }

    /// Return the union of two AABBs.
    #[allow(dead_code)]
    pub fn union(a: &Self, b: &Self) -> Self {
        Self {
            min: [
                a.min[0].min(b.min[0]),
                a.min[1].min(b.min[1]),
                a.min[2].min(b.min[2]),
            ],
            max: [
                a.max[0].max(b.max[0]),
                a.max[1].max(b.max[1]),
                a.max[2].max(b.max[2]),
            ],
        }
    }

    /// Surface area of the AABB.
    #[allow(dead_code)]
    pub fn surface_area(&self) -> f32 {
        let dx = (self.max[0] - self.min[0]).max(0.0);
        let dy = (self.max[1] - self.min[1]).max(0.0);
        let dz = (self.max[2] - self.min[2]).max(0.0);
        2.0 * (dx * dy + dy * dz + dz * dx)
    }

    /// Centroid of the AABB.
    #[allow(dead_code)]
    pub fn centroid(&self) -> [f32; 3] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }

    /// Test if a point is inside the AABB.
    #[allow(dead_code)]
    pub fn contains_point(&self, pt: [f32; 3]) -> bool {
        (0..3usize).all(|i| pt[i] >= self.min[i] && pt[i] <= self.max[i])
    }

    /// Test ray AABB intersection (slab method). Returns min t or None.
    #[allow(dead_code)]
    #[allow(clippy::needless_range_loop)]
    pub fn ray_intersect(&self, origin: [f32; 3], inv_dir: [f32; 3]) -> Option<f32> {
        let mut t_min = f32::NEG_INFINITY;
        let mut t_max = f32::INFINITY;
        for i in 0..3 {
            let t0 = (self.min[i] - origin[i]) * inv_dir[i];
            let t1 = (self.max[i] - origin[i]) * inv_dir[i];
            let (lo, hi) = if t0 < t1 { (t0, t1) } else { (t1, t0) };
            t_min = t_min.max(lo);
            t_max = t_max.min(hi);
        }
        if t_max >= t_min && t_max >= 0.0 {
            Some(t_min)
        } else {
            None
        }
    }
}

/// A BVH node — either a leaf (holds triangle indices) or an internal node.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum BvhNode {
    /// Leaf node.
    Leaf {
        aabb: BvhAabb,
        triangles: Vec<usize>,
    },
    /// Internal node.
    Internal {
        aabb: BvhAabb,
        left: Box<BvhNode>,
        right: Box<BvhNode>,
    },
}

impl BvhNode {
    /// Return the AABB of this node.
    #[allow(dead_code)]
    pub fn aabb(&self) -> &BvhAabb {
        match self {
            Self::Leaf { aabb, .. } => aabb,
            Self::Internal { aabb, .. } => aabb,
        }
    }

    /// Count total nodes in the tree.
    #[allow(dead_code)]
    pub fn node_count(&self) -> usize {
        match self {
            Self::Leaf { .. } => 1,
            Self::Internal { left, right, .. } => 1 + left.node_count() + right.node_count(),
        }
    }

    /// Count leaf nodes.
    #[allow(dead_code)]
    pub fn leaf_count(&self) -> usize {
        match self {
            Self::Leaf { .. } => 1,
            Self::Internal { left, right, .. } => left.leaf_count() + right.leaf_count(),
        }
    }
}

/// Compute the AABB for a single triangle.
fn triangle_aabb(positions: &[[f32; 3]], ia: usize, ib: usize, ic: usize) -> BvhAabb {
    let mut aabb = BvhAabb::empty();
    aabb.expand(positions[ia]);
    aabb.expand(positions[ib]);
    aabb.expand(positions[ic]);
    aabb
}

/// Build a BVH from a triangle soup. Uses midpoint splitting on the longest axis.
#[allow(dead_code)]
pub fn build_bvh(positions: &[[f32; 3]], indices: &[u32], leaf_size: usize) -> Option<BvhNode> {
    let tri_count = indices.len() / 3;
    if tri_count == 0 {
        return None;
    }
    let tri_list: Vec<usize> = (0..tri_count).collect();
    Some(build_recursive(
        positions,
        indices,
        &tri_list,
        leaf_size.max(1),
    ))
}

fn build_recursive(
    positions: &[[f32; 3]],
    indices: &[u32],
    tris: &[usize],
    leaf_size: usize,
) -> BvhNode {
    // Compute overall AABB
    let mut aabb = BvhAabb::empty();
    for &t in tris {
        let ia = indices[t * 3] as usize;
        let ib = indices[t * 3 + 1] as usize;
        let ic = indices[t * 3 + 2] as usize;
        if ia < positions.len() && ib < positions.len() && ic < positions.len() {
            aabb.expand(positions[ia]);
            aabb.expand(positions[ib]);
            aabb.expand(positions[ic]);
        }
    }

    if tris.len() <= leaf_size {
        return BvhNode::Leaf {
            aabb,
            triangles: tris.to_vec(),
        };
    }

    // Choose longest axis to split
    let dx = aabb.max[0] - aabb.min[0];
    let dy = aabb.max[1] - aabb.min[1];
    let dz = aabb.max[2] - aabb.min[2];
    let axis = if dx >= dy && dx >= dz {
        0
    } else if dy >= dz {
        1
    } else {
        2
    };
    let mid = (aabb.min[axis] + aabb.max[axis]) * 0.5;

    let mut left_tris: Vec<usize> = Vec::new();
    let mut right_tris: Vec<usize> = Vec::new();
    for &t in tris {
        let ia = indices[t * 3] as usize;
        let ib = indices[t * 3 + 1] as usize;
        let ic = indices[t * 3 + 2] as usize;
        let tri_mid = if ia < positions.len() && ib < positions.len() && ic < positions.len() {
            (positions[ia][axis] + positions[ib][axis] + positions[ic][axis]) / 3.0
        } else {
            mid
        };
        if tri_mid < mid {
            left_tris.push(t);
        } else {
            right_tris.push(t);
        }
    }

    // Prevent infinite recursion if all triangles end up on one side
    if left_tris.is_empty() || right_tris.is_empty() {
        return BvhNode::Leaf {
            aabb,
            triangles: tris.to_vec(),
        };
    }

    let left = build_recursive(positions, indices, &left_tris, leaf_size);
    let right = build_recursive(positions, indices, &right_tris, leaf_size);
    BvhNode::Internal {
        aabb,
        left: Box::new(left),
        right: Box::new(right),
    }
}

/// Count total triangles in a BVH (sums leaf triangle counts).
#[allow(dead_code)]
pub fn bvh_triangle_count(node: &BvhNode) -> usize {
    match node {
        BvhNode::Leaf { triangles, .. } => triangles.len(),
        BvhNode::Internal { left, right, .. } => {
            bvh_triangle_count(left) + bvh_triangle_count(right)
        }
    }
}

/// Compute the triangle AABB for a given triangle index.
#[allow(dead_code)]
pub fn triangle_aabb_for(positions: &[[f32; 3]], indices: &[u32], tri: usize) -> BvhAabb {
    let ia = indices[tri * 3] as usize;
    let ib = indices[tri * 3 + 1] as usize;
    let ic = indices[tri * 3 + 2] as usize;
    if ia < positions.len() && ib < positions.len() && ic < positions.len() {
        triangle_aabb(positions, ia, ib, ic)
    } else {
        BvhAabb::empty()
    }
}

/// Serialise BVH stats as JSON.
#[allow(dead_code)]
pub fn bvh_to_json(node: &BvhNode) -> String {
    format!(
        "{{\"nodes\":{},\"leaves\":{},\"triangles\":{}}}",
        node.node_count(),
        node.leaf_count(),
        bvh_triangle_count(node)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_tri_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [-1.0f32, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 1.0, 0.0],
            [4.0, 0.0, 0.0],
        ];
        let idx = vec![0u32, 1, 2, 3, 4, 5];
        (pos, idx)
    }

    #[test]
    fn test_build_bvh_some() {
        let (pos, idx) = two_tri_mesh();
        assert!(build_bvh(&pos, &idx, 1).is_some());
    }

    #[test]
    fn test_build_bvh_empty() {
        let pos: Vec<[f32; 3]> = vec![];
        let idx: Vec<u32> = vec![];
        assert!(build_bvh(&pos, &idx, 1).is_none());
    }

    #[test]
    fn test_node_count_at_least_one() {
        let (pos, idx) = two_tri_mesh();
        let bvh = build_bvh(&pos, &idx, 1).expect("should succeed");
        assert!(bvh.node_count() >= 1);
    }

    #[test]
    fn test_triangle_count_matches() {
        let (pos, idx) = two_tri_mesh();
        let bvh = build_bvh(&pos, &idx, 1).expect("should succeed");
        assert_eq!(bvh_triangle_count(&bvh), 2);
    }

    #[test]
    fn test_aabb_surface_area_positive() {
        let mut aabb = BvhAabb::empty();
        aabb.expand([0.0, 0.0, 0.0]);
        aabb.expand([1.0, 1.0, 1.0]);
        assert!(aabb.surface_area() > 0.0);
    }

    #[test]
    fn test_aabb_contains_point() {
        let mut aabb = BvhAabb::empty();
        aabb.expand([0.0, 0.0, 0.0]);
        aabb.expand([2.0, 2.0, 2.0]);
        assert!(aabb.contains_point([1.0, 1.0, 1.0]));
        assert!(!aabb.contains_point([3.0, 0.0, 0.0]));
    }

    #[test]
    fn test_aabb_centroid() {
        let mut aabb = BvhAabb::empty();
        aabb.expand([0.0, 0.0, 0.0]);
        aabb.expand([2.0, 2.0, 2.0]);
        let c = aabb.centroid();
        assert!((c[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_ray_hit_aabb() {
        let mut aabb = BvhAabb::empty();
        aabb.expand([-1.0, -1.0, -1.0]);
        aabb.expand([1.0, 1.0, 1.0]);
        let origin = [0.0, 0.0, -5.0];
        let inv_dir = [f32::INFINITY, f32::INFINITY, 1.0];
        let hit = aabb.ray_intersect(origin, inv_dir);
        assert!(hit.is_some());
    }

    #[test]
    fn test_bvh_to_json() {
        let (pos, idx) = two_tri_mesh();
        let bvh = build_bvh(&pos, &idx, 1).expect("should succeed");
        let json = bvh_to_json(&bvh);
        assert!(json.contains("triangles"));
    }

    #[test]
    fn test_leaf_count_ge_one() {
        let (pos, idx) = two_tri_mesh();
        let bvh = build_bvh(&pos, &idx, 1).expect("should succeed");
        assert!(bvh.leaf_count() >= 1);
    }
}
