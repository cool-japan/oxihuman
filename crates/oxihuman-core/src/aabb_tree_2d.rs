// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! 2D AABB tree for overlap and point-containment queries.

/// A 2D axis-aligned bounding box.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub struct Aabb2D {
    pub min: [f32; 2],
    pub max: [f32; 2],
}

impl Aabb2D {
    #[allow(dead_code)]
    pub fn new(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self {
            min: [min_x, min_y],
            max: [max_x, max_y],
        }
    }

    #[allow(dead_code)]
    pub fn contains_point(&self, p: [f32; 2]) -> bool {
        (self.min[0]..=self.max[0]).contains(&p[0]) && (self.min[1]..=self.max[1]).contains(&p[1])
    }

    #[allow(dead_code)]
    pub fn overlaps(&self, other: &Aabb2D) -> bool {
        self.min[0] <= other.max[0]
            && self.max[0] >= other.min[0]
            && self.min[1] <= other.max[1]
            && self.max[1] >= other.min[1]
    }

    fn merge(&self, other: &Aabb2D) -> Aabb2D {
        Aabb2D::new(
            self.min[0].min(other.min[0]),
            self.min[1].min(other.min[1]),
            self.max[0].max(other.max[0]),
            self.max[1].max(other.max[1]),
        )
    }

    #[allow(dead_code)]
    pub fn area(&self) -> f32 {
        (self.max[0] - self.min[0]).max(0.0) * (self.max[1] - self.min[1]).max(0.0)
    }
}

/// An entry in the AABB tree.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AabbEntry {
    pub bounds: Aabb2D,
    pub id: usize,
}

/// Internal node of the 2D AABB tree.
#[derive(Debug, Clone)]
enum AabbNode {
    Leaf(AabbEntry),
    Branch {
        bounds: Aabb2D,
        left: Box<AabbNode>,
        right: Box<AabbNode>,
    },
}

impl AabbNode {
    fn bounds(&self) -> &Aabb2D {
        match self {
            AabbNode::Leaf(e) => &e.bounds,
            AabbNode::Branch { bounds, .. } => bounds,
        }
    }

    fn query_overlap(&self, q: &Aabb2D, out: &mut Vec<usize>) {
        if !self.bounds().overlaps(q) {
            return;
        }
        match self {
            AabbNode::Leaf(e) => {
                if e.bounds.overlaps(q) {
                    out.push(e.id);
                }
            }
            AabbNode::Branch { left, right, .. } => {
                left.query_overlap(q, out);
                right.query_overlap(q, out);
            }
        }
    }

    fn query_point(&self, p: [f32; 2], out: &mut Vec<usize>) {
        if !self.bounds().contains_point(p) {
            return;
        }
        match self {
            AabbNode::Leaf(e) => {
                if e.bounds.contains_point(p) {
                    out.push(e.id);
                }
            }
            AabbNode::Branch { left, right, .. } => {
                left.query_point(p, out);
                right.query_point(p, out);
            }
        }
    }
}

fn build_aabb(entries: &mut [AabbEntry], axis: usize) -> AabbNode {
    if entries.len() == 1 {
        return AabbNode::Leaf(entries[0].clone());
    }
    entries.sort_by(|a, b| {
        let ca = (a.bounds.min[axis] + a.bounds.max[axis]) * 0.5;
        let cb = (b.bounds.min[axis] + b.bounds.max[axis]) * 0.5;
        ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
    });
    let mid = entries.len() / 2;
    let left = build_aabb(&mut entries[..mid], 1 - axis);
    let right = build_aabb(&mut entries[mid..], 1 - axis);
    let bounds = left.bounds().merge(right.bounds());
    AabbNode::Branch {
        bounds,
        left: Box::new(left),
        right: Box::new(right),
    }
}

/// A 2D AABB tree.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct AabbTree2D {
    root: Option<AabbNode>,
    count: usize,
}

impl AabbTree2D {
    /// Create an empty tree.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Build from a list of entries.
    #[allow(dead_code)]
    pub fn build(entries: Vec<AabbEntry>) -> Self {
        if entries.is_empty() {
            return Self {
                root: None,
                count: 0,
            };
        }
        let n = entries.len();
        let mut v = entries;
        let root = Some(build_aabb(&mut v, 0));
        Self { root, count: n }
    }

    /// Query entries overlapping `q`.
    #[allow(dead_code)]
    pub fn query_overlap(&self, q: &Aabb2D) -> Vec<usize> {
        let mut out = Vec::new();
        if let Some(r) = &self.root {
            r.query_overlap(q, &mut out);
        }
        out
    }

    /// Query entries containing point `p`.
    #[allow(dead_code)]
    pub fn query_point(&self, p: [f32; 2]) -> Vec<usize> {
        let mut out = Vec::new();
        if let Some(r) = &self.root {
            r.query_point(p, &mut out);
        }
        out
    }

    /// Number of entries.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns true if empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

/// Helper to create an `AabbEntry`.
#[allow(dead_code)]
pub fn aabb2d_entry(id: usize, min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> AabbEntry {
    AabbEntry {
        bounds: Aabb2D::new(min_x, min_y, max_x, max_y),
        id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tree() -> AabbTree2D {
        AabbTree2D::build(vec![
            aabb2d_entry(0, 0.0, 0.0, 1.0, 1.0),
            aabb2d_entry(1, 2.0, 2.0, 4.0, 4.0),
            aabb2d_entry(2, 0.5, 0.5, 1.5, 1.5),
            aabb2d_entry(3, 5.0, 5.0, 6.0, 6.0),
        ])
    }

    #[test]
    fn empty_tree_is_empty() {
        let t = AabbTree2D::new();
        assert!(t.is_empty());
    }

    #[test]
    fn len_matches_input() {
        let t = sample_tree();
        assert_eq!(t.len(), 4);
    }

    #[test]
    fn query_point_inside() {
        let t = sample_tree();
        let r = t.query_point([0.5, 0.5]);
        assert!(r.contains(&0));
    }

    #[test]
    fn query_point_outside() {
        let t = sample_tree();
        let r = t.query_point([10.0, 10.0]);
        assert!(r.is_empty());
    }

    #[test]
    fn query_overlap_finds_overlapping() {
        let t = sample_tree();
        let q = Aabb2D::new(0.9, 0.9, 2.1, 2.1);
        let r = t.query_overlap(&q);
        assert!(r.contains(&2));
        assert!(r.contains(&1));
    }

    #[test]
    fn query_overlap_no_overlap() {
        let t = sample_tree();
        let q = Aabb2D::new(20.0, 20.0, 21.0, 21.0);
        assert!(t.query_overlap(&q).is_empty());
    }

    #[test]
    fn aabb_contains_corner() {
        let b = Aabb2D::new(0.0, 0.0, 1.0, 1.0);
        assert!(b.contains_point([0.0, 0.0]));
        assert!(b.contains_point([1.0, 1.0]));
    }

    #[test]
    fn aabb_area_correct() {
        let b = Aabb2D::new(0.0, 0.0, 3.0, 4.0);
        assert!((b.area() - 12.0).abs() < 1e-5);
    }

    #[test]
    fn build_empty_is_empty() {
        let t = AabbTree2D::build(vec![]);
        assert!(t.is_empty());
    }

    #[test]
    fn overlap_self_is_true() {
        let b = Aabb2D::new(1.0, 1.0, 2.0, 2.0);
        assert!(b.overlaps(&b));
    }
}
