// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Simple octree for 3D spatial queries.

/// Axis-aligned bounding box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OctAabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl OctAabb {
    pub fn new(min: [f32; 3], max: [f32; 3]) -> Self {
        OctAabb { min, max }
    }

    pub fn center(&self) -> [f32; 3] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }

    pub fn contains_point(&self, p: &[f32; 3]) -> bool {
        (0..3).all(|i| p[i] >= self.min[i] && p[i] <= self.max[i])
    }

    pub fn intersects(&self, other: &OctAabb) -> bool {
        (0..3).all(|i| self.min[i] <= other.max[i] && self.max[i] >= other.min[i])
    }

    fn octant(&self, p: &[f32; 3]) -> usize {
        let c = self.center();
        let x = if p[0] >= c[0] { 1 } else { 0 };
        let y = if p[1] >= c[1] { 2 } else { 0 };
        let z = if p[2] >= c[2] { 4 } else { 0 };
        x | y | z
    }

    fn child_aabb(&self, octant: usize) -> OctAabb {
        let c = self.center();
        let min_x = if octant & 1 != 0 { c[0] } else { self.min[0] };
        let max_x = if octant & 1 != 0 { self.max[0] } else { c[0] };
        let min_y = if octant & 2 != 0 { c[1] } else { self.min[1] };
        let max_y = if octant & 2 != 0 { self.max[1] } else { c[1] };
        let min_z = if octant & 4 != 0 { c[2] } else { self.min[2] };
        let max_z = if octant & 4 != 0 { self.max[2] } else { c[2] };
        OctAabb {
            min: [min_x, min_y, min_z],
            max: [max_x, max_y, max_z],
        }
    }
}

const MAX_POINTS_PER_LEAF: usize = 8;
const MAX_DEPTH: usize = 8;

/// A simple octree node.
pub enum SimpleOctreeNode {
    Leaf {
        points: Vec<(usize, [f32; 3])>,
        bounds: OctAabb,
    },
    Internal {
        bounds: OctAabb,
        children: Box<[Option<SimpleOctreeNode>; 8]>,
    },
}

impl SimpleOctreeNode {
    fn new_leaf(bounds: OctAabb) -> Self {
        SimpleOctreeNode::Leaf {
            points: Vec::new(),
            bounds,
        }
    }

    fn bounds(&self) -> &OctAabb {
        match self {
            SimpleOctreeNode::Leaf { bounds, .. } => bounds,
            SimpleOctreeNode::Internal { bounds, .. } => bounds,
        }
    }

    fn insert(&mut self, id: usize, p: [f32; 3], depth: usize) {
        match self {
            SimpleOctreeNode::Leaf { points, bounds } => {
                points.push((id, p));
                if points.len() > MAX_POINTS_PER_LEAF && depth < MAX_DEPTH {
                    let old_pts = std::mem::take(points);
                    let old_bounds = *bounds;
                    let mut children: [Option<SimpleOctreeNode>; 8] = Default::default();
                    #[allow(clippy::needless_range_loop)]
                    for oct in 0..8 {
                        children[oct] =
                            Some(SimpleOctreeNode::new_leaf(old_bounds.child_aabb(oct)));
                    }
                    let mut new_self = SimpleOctreeNode::Internal {
                        bounds: old_bounds,
                        children: Box::new(children),
                    };
                    for (oid, op) in old_pts {
                        new_self.insert(oid, op, depth + 1);
                    }
                    *self = new_self;
                }
            }
            SimpleOctreeNode::Internal { bounds, children } => {
                let oct = bounds.octant(&p);
                if let Some(child) = &mut children[oct] {
                    child.insert(id, p, depth + 1);
                }
            }
        }
    }

    fn query_aabb(&self, query: &OctAabb, result: &mut Vec<usize>) {
        if !self.bounds().intersects(query) {
            return;
        }
        match self {
            SimpleOctreeNode::Leaf { points, .. } => {
                for (id, p) in points {
                    if query.contains_point(p) {
                        result.push(*id);
                    }
                }
            }
            SimpleOctreeNode::Internal { children, .. } => {
                for child in children.iter().flatten() {
                    child.query_aabb(query, result);
                }
            }
        }
    }

    fn count(&self) -> usize {
        match self {
            SimpleOctreeNode::Leaf { points, .. } => points.len(),
            SimpleOctreeNode::Internal { children, .. } => {
                children.iter().flatten().map(|c| c.count()).sum()
            }
        }
    }
}

/// Simple octree.
pub struct SimpleOctree {
    root: Option<SimpleOctreeNode>,
    bounds: OctAabb,
    count: usize,
}

impl SimpleOctree {
    /// Create a new octree with the given bounds.
    pub fn new(bounds: OctAabb) -> Self {
        SimpleOctree {
            root: Some(SimpleOctreeNode::new_leaf(bounds)),
            bounds,
            count: 0,
        }
    }

    /// Insert a point with ID.
    pub fn insert(&mut self, id: usize, p: [f32; 3]) {
        if !self.bounds.contains_point(&p) {
            return;
        }
        if let Some(root) = &mut self.root {
            root.insert(id, p, 0);
            self.count += 1;
        }
    }

    /// Query all point IDs within an AABB.
    pub fn query_aabb(&self, query: &OctAabb) -> Vec<usize> {
        let mut result = Vec::new();
        if let Some(root) = &self.root {
            root.query_aabb(query, &mut result);
        }
        result
    }

    /// Query all point IDs within a sphere (returns AABB candidates; for exact sphere tests use SimpleOctree3).
    pub fn query_sphere(&self, center: &[f32; 3], r: f32) -> Vec<usize> {
        let aabb = OctAabb {
            min: [center[0] - r, center[1] - r, center[2] - r],
            max: [center[0] + r, center[1] + r, center[2] + r],
        };
        self.query_aabb(&aabb)
    }

    /// Return total number of points inserted.
    pub fn len(&self) -> usize {
        self.count
    }

    /// True if empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

/// Better simple octree with point storage for sphere queries.
pub struct SimpleOctree3 {
    bounds: OctAabb,
    root: Option<SimpleOctreeNode>,
    points: Vec<[f32; 3]>,
}

impl SimpleOctree3 {
    pub fn new(bounds: OctAabb) -> Self {
        SimpleOctree3 {
            root: Some(SimpleOctreeNode::new_leaf(bounds)),
            bounds,
            points: Vec::new(),
        }
    }

    pub fn insert(&mut self, p: [f32; 3]) -> usize {
        let id = self.points.len();
        self.points.push(p);
        if self.bounds.contains_point(&p) {
            if let Some(root) = &mut self.root {
                root.insert(id, p, 0);
            }
        }
        id
    }

    pub fn query_aabb(&self, query: &OctAabb) -> Vec<usize> {
        let mut result = Vec::new();
        if let Some(root) = &self.root {
            root.query_aabb(query, &mut result);
        }
        result
    }

    pub fn query_sphere(&self, center: &[f32; 3], r: f32) -> Vec<usize> {
        let aabb = OctAabb {
            min: [center[0] - r, center[1] - r, center[2] - r],
            max: [center[0] + r, center[1] + r, center[2] + r],
        };
        let r_sq = r * r;
        let candidates = self.query_aabb(&aabb);
        candidates
            .into_iter()
            .filter(|&id| {
                let p = &self.points[id];
                (0..3).map(|i| (p[i] - center[i]).powi(2)).sum::<f32>() <= r_sq
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    pub fn clear(&mut self) {
        self.root = Some(SimpleOctreeNode::new_leaf(self.bounds));
        self.points.clear();
    }

    pub fn bounds(&self) -> &OctAabb {
        &self.bounds
    }
}

/// Create a new simple octree with the given world size.
pub fn new_simple_octree(half_size: f32) -> SimpleOctree3 {
    let h = half_size.abs().max(1.0);
    SimpleOctree3::new(OctAabb {
        min: [-h, -h, -h],
        max: [h, h, h],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_query_aabb() {
        let mut tree = new_simple_octree(100.0);
        let id = tree.insert([1.0, 2.0, 3.0]);
        let found = tree.query_aabb(&OctAabb::new([0.0, 0.0, 0.0], [5.0, 5.0, 5.0]));
        assert!(found.contains(&id));
    }

    #[test]
    fn test_query_sphere() {
        let mut tree = new_simple_octree(100.0);
        let id = tree.insert([0.0, 0.0, 0.0]);
        tree.insert([50.0, 50.0, 50.0]);
        let found = tree.query_sphere(&[0.0, 0.0, 0.0], 1.0);
        assert!(found.contains(&id));
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn test_empty() {
        let tree = new_simple_octree(10.0);
        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
    }

    #[test]
    fn test_len() {
        let mut tree = new_simple_octree(100.0);
        tree.insert([1.0, 0.0, 0.0]);
        tree.insert([2.0, 0.0, 0.0]);
        assert_eq!(tree.len(), 2);
    }

    #[test]
    fn test_clear() {
        let mut tree = new_simple_octree(100.0);
        tree.insert([1.0, 1.0, 1.0]);
        tree.clear();
        assert!(tree.is_empty());
    }

    #[test]
    fn test_out_of_bounds_ignored() {
        let mut tree = new_simple_octree(10.0);
        tree.insert([1000.0, 0.0, 0.0]);
        /* ID is assigned but it won't be found in AABB query */
        let found = tree.query_aabb(&OctAabb::new([-10.0, -10.0, -10.0], [10.0, 10.0, 10.0]));
        assert!(found.is_empty());
    }

    #[test]
    fn test_many_points() {
        let mut tree = new_simple_octree(100.0);
        for i in 0..50 {
            tree.insert([i as f32, 0.0, 0.0]);
        }
        assert_eq!(tree.len(), 50);
        let found = tree.query_sphere(&[25.0, 0.0, 0.0], 3.0);
        assert!(found.len() >= 5);
    }

    #[test]
    fn test_aabb_intersects() {
        let a = OctAabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let b = OctAabb::new([0.5, 0.5, 0.5], [2.0, 2.0, 2.0]);
        let c = OctAabb::new([5.0, 5.0, 5.0], [6.0, 6.0, 6.0]);
        assert!(a.intersects(&b));
        assert!(!a.intersects(&c));
    }
}
