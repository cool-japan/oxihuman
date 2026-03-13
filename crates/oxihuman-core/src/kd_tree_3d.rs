// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! 3D k-d tree for nearest-neighbor queries.

/// A 3D point with an optional payload index.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct KdPoint3 {
    pub coords: [f32; 3],
    pub id: usize,
}

impl KdPoint3 {
    #[allow(dead_code)]
    pub fn new(x: f32, y: f32, z: f32, id: usize) -> Self {
        Self {
            coords: [x, y, z],
            id,
        }
    }

    fn dist_sq(self, q: [f32; 3]) -> f32 {
        (0..3).map(|i| (self.coords[i] - q[i]).powi(2)).sum()
    }
}

#[derive(Debug, Clone)]
enum KdNode3 {
    Leaf(KdPoint3),
    Split {
        axis: usize,
        split_val: f32,
        left: Box<KdNode3>,
        right: Box<KdNode3>,
    },
}

fn build3(points: &mut [KdPoint3], depth: usize) -> Option<Box<KdNode3>> {
    if points.is_empty() {
        return None;
    }
    if points.len() == 1 {
        return Some(Box::new(KdNode3::Leaf(points[0])));
    }
    let axis = depth % 3;
    points.sort_by(|a, b| {
        a.coords[axis]
            .partial_cmp(&b.coords[axis])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mid = points.len() / 2;
    let split_val = points[mid].coords[axis];
    let left = build3(&mut points[..mid], depth + 1);
    let right = build3(&mut points[mid..], depth + 1);
    let node = match (left, right) {
        (Some(l), Some(r)) => KdNode3::Split {
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

fn nn3(node: &KdNode3, q: [f32; 3], best: &mut Option<(f32, KdPoint3)>) {
    match node {
        KdNode3::Leaf(p) => {
            let d = p.dist_sq(q);
            if best.is_none_or(|(bd, _)| d < bd) {
                *best = Some((d, *p));
            }
        }
        KdNode3::Split {
            axis,
            split_val,
            left,
            right,
        } => {
            let qv = q[*axis];
            let (near, far) = if qv < *split_val {
                (left.as_ref(), right.as_ref())
            } else {
                (right.as_ref(), left.as_ref())
            };
            nn3(near, q, best);
            let pd = (qv - split_val) * (qv - split_val);
            if best.is_none_or(|(bd, _)| pd < bd) {
                nn3(far, q, best);
            }
        }
    }
}

/// A 3D k-d tree.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct KdTree3D {
    root: Option<Box<KdNode3>>,
    count: usize,
}

impl KdTree3D {
    /// Build a tree from a slice of points.
    #[allow(dead_code)]
    pub fn build(points: &[KdPoint3]) -> Self {
        let mut pts = points.to_vec();
        let n = pts.len();
        Self {
            root: build3(&mut pts, 0),
            count: n,
        }
    }

    /// Nearest neighbor query.
    #[allow(dead_code)]
    pub fn nearest(&self, q: [f32; 3]) -> Option<(KdPoint3, f32)> {
        let root = self.root.as_ref()?;
        let mut best = None;
        nn3(root, q, &mut best);
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

/// Build a `KdTree3D` from raw xyz triples.
#[allow(dead_code)]
pub fn kd3_build(xyzs: &[[f32; 3]]) -> KdTree3D {
    let pts: Vec<KdPoint3> = xyzs
        .iter()
        .enumerate()
        .map(|(i, &c)| KdPoint3 { coords: c, id: i })
        .collect();
    KdTree3D::build(&pts)
}

/// Nearest-neighbor ID for a query (None if empty).
#[allow(dead_code)]
pub fn kd3_nearest_id(tree: &KdTree3D, q: [f32; 3]) -> Option<usize> {
    tree.nearest(q).map(|(p, _)| p.id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tree() -> KdTree3D {
        kd3_build(&[
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ])
    }

    #[test]
    fn empty_tree_returns_none() {
        let t = KdTree3D::build(&[]);
        assert!(t.nearest([0.0, 0.0, 0.0]).is_none());
    }

    #[test]
    fn single_point_nearest() {
        let t = kd3_build(&[[2.0, 3.0, 4.0]]);
        let (p, _) = t.nearest([0.0, 0.0, 0.0]).expect("should succeed");
        assert_eq!(p.id, 0);
    }

    #[test]
    fn nearest_to_origin() {
        let t = sample_tree();
        let (p, d) = t.nearest([0.0, 0.0, 0.0]).expect("should succeed");
        assert_eq!(p.id, 0);
        assert!(d < 1e-6);
    }

    #[test]
    fn nearest_to_unit_x() {
        let t = sample_tree();
        let id = kd3_nearest_id(&t, [0.9, 0.0, 0.0]).expect("should succeed");
        assert_eq!(id, 1);
    }

    #[test]
    fn len_matches_input() {
        let t = sample_tree();
        assert_eq!(t.len(), 4);
    }

    #[test]
    fn is_empty_false() {
        let t = sample_tree();
        assert!(!t.is_empty());
    }

    #[test]
    fn is_empty_true() {
        let t = KdTree3D::build(&[]);
        assert!(t.is_empty());
    }

    #[test]
    fn nn_exact_match_zero_dist() {
        let t = kd3_build(&[[1.0, 2.0, 3.0]]);
        let (_, d) = t.nearest([1.0, 2.0, 3.0]).expect("should succeed");
        assert!(d < 1e-6);
    }

    #[test]
    fn nn_axis_z() {
        let t = kd3_build(&[[0.0, 0.0, 0.0], [0.0, 0.0, 5.0]]);
        let id = kd3_nearest_id(&t, [0.0, 0.0, 4.0]).expect("should succeed");
        assert_eq!(id, 1);
    }

    #[test]
    fn build_assigns_sequential_ids() {
        let t = kd3_build(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        assert_eq!(t.len(), 3);
    }
}
