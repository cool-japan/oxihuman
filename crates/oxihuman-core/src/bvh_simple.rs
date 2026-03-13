// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Simple bounding volume hierarchy (BVH) for 3D primitives.

/// Axis-aligned bounding box for BVH.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BvhAabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl BvhAabb {
    pub fn new(min: [f32; 3], max: [f32; 3]) -> Self {
        BvhAabb { min, max }
    }

    pub fn from_point(p: [f32; 3]) -> Self {
        BvhAabb { min: p, max: p }
    }

    pub fn expand(&self, other: &BvhAabb) -> BvhAabb {
        BvhAabb {
            min: [
                self.min[0].min(other.min[0]),
                self.min[1].min(other.min[1]),
                self.min[2].min(other.min[2]),
            ],
            max: [
                self.max[0].max(other.max[0]),
                self.max[1].max(other.max[1]),
                self.max[2].max(other.max[2]),
            ],
        }
    }

    pub fn intersects(&self, other: &BvhAabb) -> bool {
        (0..3).all(|i| self.min[i] <= other.max[i] && self.max[i] >= other.min[i])
    }

    pub fn contains_point(&self, p: &[f32; 3]) -> bool {
        (0..3).all(|i| p[i] >= self.min[i] && p[i] <= self.max[i])
    }

    pub fn surface_area(&self) -> f32 {
        let d = [
            self.max[0] - self.min[0],
            self.max[1] - self.min[1],
            self.max[2] - self.min[2],
        ];
        2.0 * (d[0] * d[1] + d[1] * d[2] + d[2] * d[0])
    }

    pub fn center(&self) -> [f32; 3] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }
}

/// A leaf primitive stored in the BVH.
#[derive(Debug, Clone)]
pub struct BvhPrimitive {
    pub aabb: BvhAabb,
    pub id: usize,
}

/// BVH node (internal or leaf).
pub enum BvhNode {
    Leaf {
        aabb: BvhAabb,
        primitives: Vec<usize>,
    },
    Internal {
        aabb: BvhAabb,
        left: Box<BvhNode>,
        right: Box<BvhNode>,
    },
}

impl BvhNode {
    pub fn aabb(&self) -> &BvhAabb {
        match self {
            BvhNode::Leaf { aabb, .. } => aabb,
            BvhNode::Internal { aabb, .. } => aabb,
        }
    }

    fn query_aabb(&self, query: &BvhAabb, result: &mut Vec<usize>) {
        if !self.aabb().intersects(query) {
            return;
        }
        match self {
            BvhNode::Leaf { primitives, .. } => result.extend_from_slice(primitives),
            BvhNode::Internal { left, right, .. } => {
                left.query_aabb(query, result);
                right.query_aabb(query, result);
            }
        }
    }

    fn query_point(&self, p: &[f32; 3], result: &mut Vec<usize>) {
        if !self.aabb().contains_point(p) {
            return;
        }
        match self {
            BvhNode::Leaf { primitives, .. } => result.extend_from_slice(primitives),
            BvhNode::Internal { left, right, .. } => {
                left.query_point(p, result);
                right.query_point(p, result);
            }
        }
    }

    fn depth(&self) -> usize {
        match self {
            BvhNode::Leaf { .. } => 1,
            BvhNode::Internal { left, right, .. } => 1 + left.depth().max(right.depth()),
        }
    }
}

fn build_bvh(prims: &[BvhPrimitive], depth: usize) -> BvhNode {
    if prims.is_empty() {
        return BvhNode::Leaf {
            aabb: BvhAabb::new([0.0; 3], [0.0; 3]),
            primitives: vec![],
        };
    }
    if prims.len() <= 4 || depth >= 16 {
        let aabb = prims
            .iter()
            .fold(prims[0].aabb, |acc, p| acc.expand(&p.aabb));
        return BvhNode::Leaf {
            aabb,
            primitives: prims.iter().map(|p| p.id).collect(),
        };
    }
    /* Split along longest axis by centroid */
    let combined = prims
        .iter()
        .fold(prims[0].aabb, |acc, p| acc.expand(&p.aabb));
    let extents = [
        combined.max[0] - combined.min[0],
        combined.max[1] - combined.min[1],
        combined.max[2] - combined.min[2],
    ];
    let axis = extents
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0);
    let mut sorted: Vec<BvhPrimitive> = prims.to_vec();
    sorted.sort_by(|a, b| {
        a.aabb.center()[axis]
            .partial_cmp(&b.aabb.center()[axis])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mid = sorted.len() / 2;
    let left = build_bvh(&sorted[..mid], depth + 1);
    let right = build_bvh(&sorted[mid..], depth + 1);
    let aabb = left.aabb().expand(right.aabb());
    BvhNode::Internal {
        aabb,
        left: Box::new(left),
        right: Box::new(right),
    }
}

/// Simple BVH structure.
pub struct SimpleBvh {
    root: Option<BvhNode>,
    primitives: Vec<BvhPrimitive>,
}

impl SimpleBvh {
    /// Build BVH from a list of primitives.
    pub fn build(primitives: Vec<BvhPrimitive>) -> Self {
        let root = if primitives.is_empty() {
            None
        } else {
            Some(build_bvh(&primitives, 0))
        };
        SimpleBvh { root, primitives }
    }

    /// Query all primitive IDs overlapping the given AABB.
    pub fn query_aabb(&self, query: &BvhAabb) -> Vec<usize> {
        let mut candidates = Vec::new();
        if let Some(root) = &self.root {
            root.query_aabb(query, &mut candidates);
        }
        /* Filter candidates against per-primitive AABBs */
        candidates
            .into_iter()
            .filter(|&id| self.primitives[id].aabb.intersects(query))
            .collect()
    }

    /// Query all primitive IDs containing the given point.
    pub fn query_point(&self, p: &[f32; 3]) -> Vec<usize> {
        let mut result = Vec::new();
        if let Some(root) = &self.root {
            root.query_point(p, result.as_mut());
        }
        result
    }

    /// Return the tree depth.
    pub fn depth(&self) -> usize {
        self.root.as_ref().map(|r| r.depth()).unwrap_or(0)
    }

    /// Return the number of primitives.
    pub fn len(&self) -> usize {
        self.primitives.len()
    }

    /// True if empty.
    pub fn is_empty(&self) -> bool {
        self.primitives.is_empty()
    }

    /// Return the root AABB.
    pub fn root_aabb(&self) -> Option<BvhAabb> {
        self.root.as_ref().map(|r| *r.aabb())
    }
}

/// Build a BVH from a list of (id, min, max) triples.
pub fn new_bvh(data: &[(usize, [f32; 3], [f32; 3])]) -> SimpleBvh {
    let prims: Vec<BvhPrimitive> = data
        .iter()
        .map(|&(id, min, max)| BvhPrimitive {
            aabb: BvhAabb::new(min, max),
            id,
        })
        .collect();
    SimpleBvh::build(prims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_and_query() {
        let bvh = new_bvh(&[
            (0, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]),
            (1, [5.0, 5.0, 5.0], [6.0, 6.0, 6.0]),
        ]);
        let r = bvh.query_aabb(&BvhAabb::new([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]));
        assert!(r.contains(&0));
        assert!(!r.contains(&1));
    }

    #[test]
    fn test_empty_bvh() {
        let bvh = SimpleBvh::build(vec![]);
        assert!(bvh.is_empty());
        assert_eq!(bvh.depth(), 0);
    }

    #[test]
    fn test_len() {
        let bvh = new_bvh(&[
            (0, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]),
            (1, [2.0, 0.0, 0.0], [3.0, 1.0, 1.0]),
        ]);
        assert_eq!(bvh.len(), 2);
    }

    #[test]
    fn test_root_aabb() {
        let bvh = new_bvh(&[
            (0, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]),
            (1, [2.0, 0.0, 0.0], [3.0, 1.0, 1.0]),
        ]);
        let raabb = bvh.root_aabb().expect("should succeed");
        assert!(raabb.max[0] >= 3.0);
    }

    #[test]
    fn test_aabb_surface_area() {
        let a = BvhAabb::new([0.0, 0.0, 0.0], [2.0, 3.0, 4.0]);
        /* 2*(2*3 + 3*4 + 4*2) = 2*(6+12+8) = 52 */
        assert!((a.surface_area() - 52.0).abs() < 1e-4);
    }

    #[test]
    fn test_depth_single_leaf() {
        let bvh = new_bvh(&[(0, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0])]);
        assert_eq!(bvh.depth(), 1);
    }

    #[test]
    fn test_many_primitives() {
        let data: Vec<(usize, [f32; 3], [f32; 3])> = (0..20)
            .map(|i| {
                let f = i as f32;
                (i, [f, 0.0, 0.0], [f + 1.0, 1.0, 1.0])
            })
            .collect();
        let bvh = new_bvh(&data);
        assert_eq!(bvh.len(), 20);
        let r = bvh.query_aabb(&BvhAabb::new([9.0, 0.0, 0.0], [11.0, 1.0, 1.0]));
        assert!(r.len() >= 2);
    }

    #[test]
    fn test_aabb_contains_point() {
        let a = BvhAabb::new([0.0, 0.0, 0.0], [5.0, 5.0, 5.0]);
        assert!(a.contains_point(&[2.5, 2.5, 2.5]));
        assert!(!a.contains_point(&[6.0, 0.0, 0.0]));
    }
}
