// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! k-d tree for nearest neighbor search in 2D and 3D.

/// A 3D point with an associated ID.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KdPoint3 {
    pub pos: [f32; 3],
    pub id: usize,
}

impl KdPoint3 {
    pub fn new(x: f32, y: f32, z: f32, id: usize) -> Self {
        KdPoint3 { pos: [x, y, z], id }
    }

    fn dist_sq(&self, other: &[f32; 3]) -> f32 {
        (0..3).map(|i| (self.pos[i] - other[i]).powi(2)).sum()
    }
}

/// A k-d tree node.
#[derive(Debug)]
struct KdNode {
    point: KdPoint3,
    left: Option<Box<KdNode>>,
    right: Option<Box<KdNode>>,
    axis: usize,
}

/// k-d tree for 3D nearest neighbor queries.
#[derive(Default)]
pub struct KdTree3 {
    root: Option<Box<KdNode>>,
    count: usize,
}

fn build(points: &mut [KdPoint3], depth: usize) -> Option<Box<KdNode>> {
    if points.is_empty() {
        return None;
    }
    let axis = depth % 3;
    points.sort_by(|a, b| {
        a.pos[axis]
            .partial_cmp(&b.pos[axis])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mid = points.len() / 2;
    Some(Box::new(KdNode {
        point: points[mid],
        axis,
        left: build(&mut points[..mid], depth + 1),
        right: build(&mut points[mid + 1..], depth + 1),
    }))
}

fn nn_search<'a>(node: &'a KdNode, query: &[f32; 3], best: &mut Option<(f32, &'a KdPoint3)>) {
    let d = node.point.dist_sq(query);
    if best.is_none_or(|(bd, _)| d < bd) {
        *best = Some((d, &node.point));
    }
    let axis = node.axis;
    let diff = query[axis] - node.point.pos[axis];
    let (near, far) = if diff <= 0.0 {
        (node.left.as_deref(), node.right.as_deref())
    } else {
        (node.right.as_deref(), node.left.as_deref())
    };
    if let Some(n) = near {
        nn_search(n, query, best);
    }
    if let Some(f) = far {
        let best_d = best.map(|(bd, _)| bd).unwrap_or(f32::INFINITY);
        if diff * diff <= best_d {
            nn_search(f, query, best);
        }
    }
}

fn range_search(node: &KdNode, query: &[f32; 3], r_sq: f32, result: &mut Vec<KdPoint3>) {
    if node.point.dist_sq(query) <= r_sq {
        result.push(node.point);
    }
    let axis = node.axis;
    let diff = query[axis] - node.point.pos[axis];
    if diff - r_sq.sqrt() <= 0.0 {
        if let Some(n) = &node.left {
            range_search(n, query, r_sq, result);
        }
    }
    if diff + r_sq.sqrt() >= 0.0 {
        if let Some(n) = &node.right {
            range_search(n, query, r_sq, result);
        }
    }
}

impl KdTree3 {
    /// Build a k-d tree from a list of points.
    pub fn build(mut points: Vec<KdPoint3>) -> Self {
        let count = points.len();
        let root = build(&mut points, 0);
        KdTree3 { root, count }
    }

    /// Nearest neighbor query. Returns (point, distance).
    pub fn nearest(&self, query: &[f32; 3]) -> Option<(KdPoint3, f32)> {
        let root = self.root.as_deref()?;
        let mut best = None;
        nn_search(root, query, &mut best);
        best.map(|(d, p)| (*p, d.sqrt()))
    }

    /// Range query: all points within radius `r`.
    pub fn range_query(&self, query: &[f32; 3], r: f32) -> Vec<KdPoint3> {
        let mut result = Vec::new();
        if let Some(root) = &self.root {
            range_search(root, query, r * r, &mut result);
        }
        result
    }

    /// Number of points in the tree.
    pub fn len(&self) -> usize {
        self.count
    }

    /// True if empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

/// Build a k-d tree from xyz arrays.
pub fn new_kd_tree(positions: &[[f32; 3]]) -> KdTree3 {
    let points: Vec<KdPoint3> = positions
        .iter()
        .enumerate()
        .map(|(i, p)| KdPoint3 { pos: *p, id: i })
        .collect();
    KdTree3::build(points)
}

/// Build a 2D k-d tree (z = 0).
pub fn new_kd_tree_2d(positions: &[[f32; 2]]) -> KdTree3 {
    let points: Vec<KdPoint3> = positions
        .iter()
        .enumerate()
        .map(|(i, p)| KdPoint3 {
            pos: [p[0], p[1], 0.0],
            id: i,
        })
        .collect();
    KdTree3::build(points)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nearest_basic() {
        let pts = vec![
            KdPoint3::new(0.0, 0.0, 0.0, 0),
            KdPoint3::new(1.0, 0.0, 0.0, 1),
            KdPoint3::new(5.0, 5.0, 5.0, 2),
        ];
        let tree = KdTree3::build(pts);
        let (p, d) = tree.nearest(&[0.1, 0.0, 0.0]).unwrap();
        assert_eq!(p.id, 0);
        assert!(d < 0.2);
    }

    #[test]
    fn test_nearest_single_point() {
        let pts = vec![KdPoint3::new(3.0, 4.0, 0.0, 0)];
        let tree = KdTree3::build(pts);
        let (p, d) = tree.nearest(&[3.0, 4.0, 0.0]).unwrap();
        assert_eq!(p.id, 0);
        assert!(d < 1e-5);
    }

    #[test]
    fn test_empty_tree() {
        let tree = KdTree3::build(vec![]);
        assert!(tree.nearest(&[0.0, 0.0, 0.0]).is_none());
        assert!(tree.is_empty());
    }

    #[test]
    fn test_range_query() {
        let pts: Vec<KdPoint3> = (0..10)
            .map(|i| KdPoint3::new(i as f32, 0.0, 0.0, i))
            .collect();
        let tree = KdTree3::build(pts);
        let found = tree.range_query(&[5.0, 0.0, 0.0], 2.5);
        /* Should include 3,4,5,6,7 */
        assert!(found.len() >= 4);
    }

    #[test]
    fn test_new_kd_tree() {
        let pos = vec![[0.0f32, 1.0, 2.0], [3.0, 4.0, 5.0]];
        let tree = new_kd_tree(&pos);
        assert_eq!(tree.len(), 2);
    }

    #[test]
    fn test_new_kd_tree_2d() {
        let pos = vec![[0.0f32, 0.0], [1.0, 1.0], [2.0, 2.0]];
        let tree = new_kd_tree_2d(&pos);
        let (p, _) = tree.nearest(&[0.9, 0.9, 0.0]).unwrap();
        assert_eq!(p.id, 1);
    }

    #[test]
    fn test_len() {
        let pts: Vec<KdPoint3> = (0..5)
            .map(|i| KdPoint3::new(i as f32, 0.0, 0.0, i))
            .collect();
        let tree = KdTree3::build(pts);
        assert_eq!(tree.len(), 5);
    }

    #[test]
    fn test_many_points_nearest() {
        let pts: Vec<KdPoint3> = (0..100)
            .map(|i| KdPoint3::new(i as f32, 0.0, 0.0, i))
            .collect();
        let tree = KdTree3::build(pts);
        let (p, d) = tree.nearest(&[49.5, 0.0, 0.0]).unwrap();
        assert!(p.id == 49 || p.id == 50);
        assert!(d < 1.0);
    }
}
