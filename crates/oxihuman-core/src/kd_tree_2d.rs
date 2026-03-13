// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! 2D k-d tree for nearest-neighbor queries.

/// A 2D point with an optional payload index.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct KdPoint2 {
    pub x: f32,
    pub y: f32,
    pub id: usize,
}

impl KdPoint2 {
    #[allow(dead_code)]
    pub fn new(x: f32, y: f32, id: usize) -> Self {
        Self { x, y, id }
    }

    fn dist_sq(self, other: [f32; 2]) -> f32 {
        let dx = self.x - other[0];
        let dy = self.y - other[1];
        dx * dx + dy * dy
    }
}

/// Internal node of a 2D k-d tree.
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum KdNode2 {
    Leaf(KdPoint2),
    Split {
        axis: usize,
        split_val: f32,
        left: Box<KdNode2>,
        right: Box<KdNode2>,
    },
}

/// A 2D k-d tree.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct KdTree2D {
    root: Option<Box<KdNode2>>,
    count: usize,
}

fn build_node(points: &mut [KdPoint2], depth: usize) -> Option<Box<KdNode2>> {
    if points.is_empty() {
        return None;
    }
    if points.len() == 1 {
        return Some(Box::new(KdNode2::Leaf(points[0])));
    }
    let axis = depth % 2;
    if axis == 0 {
        points.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
    } else {
        points.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal));
    }
    let mid = points.len() / 2;
    let split_val = if axis == 0 {
        points[mid].x
    } else {
        points[mid].y
    };
    let left = build_node(&mut points[..mid], depth + 1);
    let right = build_node(&mut points[mid..], depth + 1);
    let node = match (left, right) {
        (Some(l), Some(r)) => KdNode2::Split {
            axis,
            split_val,
            left: l,
            right: r,
        },
        (Some(l), None) => *l,
        (None, Some(r)) => *r,
        (None, None) => unreachable!(),
    };
    Some(Box::new(node))
}

fn nn_search(node: &KdNode2, query: [f32; 2], best: &mut Option<(f32, KdPoint2)>) {
    match node {
        KdNode2::Leaf(p) => {
            let d = p.dist_sq(query);
            if best.is_none_or(|(bd, _)| d < bd) {
                *best = Some((d, *p));
            }
        }
        KdNode2::Split {
            axis,
            split_val,
            left,
            right,
        } => {
            let qval = if *axis == 0 { query[0] } else { query[1] };
            let (near, far) = if qval < *split_val {
                (left.as_ref(), right.as_ref())
            } else {
                (right.as_ref(), left.as_ref())
            };
            nn_search(near, query, best);
            let plane_dist = (qval - split_val) * (qval - split_val);
            if best.is_none_or(|(bd, _)| plane_dist < bd) {
                nn_search(far, query, best);
            }
        }
    }
}

impl KdTree2D {
    /// Build a tree from a slice of points.
    #[allow(dead_code)]
    pub fn build(points: &[KdPoint2]) -> Self {
        let mut pts = points.to_vec();
        let n = pts.len();
        Self {
            root: build_node(&mut pts, 0),
            count: n,
        }
    }

    /// Nearest neighbor query. Returns the nearest point and its squared distance.
    #[allow(dead_code)]
    pub fn nearest(&self, query: [f32; 2]) -> Option<(KdPoint2, f32)> {
        let root = self.root.as_ref()?;
        let mut best = None;
        nn_search(root, query, &mut best);
        best.map(|(d, p)| (p, d))
    }

    /// Number of points.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns true if there are no points.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

/// Build a `KdTree2D` from raw (x, y) pairs.
#[allow(dead_code)]
pub fn kd2_build(xys: &[[f32; 2]]) -> KdTree2D {
    let pts: Vec<KdPoint2> = xys
        .iter()
        .enumerate()
        .map(|(i, &[x, y])| KdPoint2::new(x, y, i))
        .collect();
    KdTree2D::build(&pts)
}

/// Nearest-neighbor distance (squared) for a query.
#[allow(dead_code)]
pub fn kd2_nn_dist_sq(tree: &KdTree2D, query: [f32; 2]) -> Option<f32> {
    tree.nearest(query).map(|(_, d)| d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tree() -> KdTree2D {
        kd2_build(&[[0.0, 0.0], [3.0, 0.0], [1.0, 2.0], [5.0, 5.0]])
    }

    #[test]
    fn empty_tree_returns_none() {
        let t = KdTree2D::build(&[]);
        assert!(t.nearest([0.0, 0.0]).is_none());
    }

    #[test]
    fn single_point_is_nearest() {
        let t = kd2_build(&[[2.0, 3.0]]);
        let (p, _) = t.nearest([0.0, 0.0]).expect("should succeed");
        assert_eq!(p.id, 0);
    }

    #[test]
    fn nearest_to_origin() {
        let t = sample_tree();
        let (p, d) = t.nearest([0.0, 0.0]).expect("should succeed");
        assert_eq!(p.id, 0);
        assert!(d < 1e-5);
    }

    #[test]
    fn nearest_to_far_point() {
        let t = sample_tree();
        let (p, _) = t.nearest([5.0, 5.0]).expect("should succeed");
        assert_eq!(p.id, 3);
    }

    #[test]
    fn tree_len_matches_input() {
        let t = sample_tree();
        assert_eq!(t.len(), 4);
    }

    #[test]
    fn is_empty_false_for_nonempty() {
        let t = sample_tree();
        assert!(!t.is_empty());
    }

    #[test]
    fn is_empty_true_for_empty() {
        let t = KdTree2D::build(&[]);
        assert!(t.is_empty());
    }

    #[test]
    fn nn_dist_sq_is_zero_for_exact_match() {
        let t = kd2_build(&[[1.0, 1.0], [2.0, 2.0]]);
        let d = kd2_nn_dist_sq(&t, [1.0, 1.0]).expect("should succeed");
        assert!(d < 1e-6);
    }

    #[test]
    fn nearest_among_two_picks_closer() {
        let t = kd2_build(&[[0.0, 0.0], [10.0, 0.0]]);
        let (p, _) = t.nearest([3.0, 0.0]).expect("should succeed");
        assert_eq!(p.id, 0);
    }

    #[test]
    fn build_from_raw_xy_assigns_ids() {
        let t = kd2_build(&[[0.0, 0.0], [1.0, 1.0], [2.0, 2.0]]);
        assert_eq!(t.len(), 3);
    }
}
