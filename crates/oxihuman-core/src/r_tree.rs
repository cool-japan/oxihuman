// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! R-tree (bounding box hierarchy) for 2D rectangle queries.

/// A 2D axis-aligned bounding rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[allow(dead_code)]
pub struct Rect2 {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl Rect2 {
    #[allow(dead_code)]
    pub fn new(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    #[allow(dead_code)]
    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        (self.min_x..=self.max_x).contains(&x) && (self.min_y..=self.max_y).contains(&y)
    }

    #[allow(dead_code)]
    pub fn overlaps(&self, other: &Rect2) -> bool {
        self.min_x <= other.max_x
            && self.max_x >= other.min_x
            && self.min_y <= other.max_y
            && self.max_y >= other.min_y
    }

    fn area(&self) -> f32 {
        (self.max_x - self.min_x).max(0.0) * (self.max_y - self.min_y).max(0.0)
    }

    fn merge(&self, other: &Rect2) -> Rect2 {
        Rect2::new(
            self.min_x.min(other.min_x),
            self.min_y.min(other.min_y),
            self.max_x.max(other.max_x),
            self.max_y.max(other.max_y),
        )
    }
}

/// An entry stored in the R-tree.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct RTreeEntry {
    pub bounds: Rect2,
    pub id: usize,
}

/// A node in the R-tree (linear R-tree, bulk-loaded).
#[derive(Debug, Clone)]
enum RNode {
    Leaf { entry: RTreeEntry },
    Internal { bounds: Rect2, children: Vec<RNode> },
}

impl RNode {
    fn bounds(&self) -> &Rect2 {
        match self {
            RNode::Leaf { entry } => &entry.bounds,
            RNode::Internal { bounds, .. } => bounds,
        }
    }

    fn query_overlap(&self, query: &Rect2, result: &mut Vec<usize>) {
        if !self.bounds().overlaps(query) {
            return;
        }
        match self {
            RNode::Leaf { entry } => {
                if entry.bounds.overlaps(query) {
                    result.push(entry.id);
                }
            }
            RNode::Internal { children, .. } => {
                for child in children {
                    child.query_overlap(query, result);
                }
            }
        }
    }

    fn query_point(&self, x: f32, y: f32, result: &mut Vec<usize>) {
        if !self.bounds().contains_point(x, y) {
            return;
        }
        match self {
            RNode::Leaf { entry } => {
                if entry.bounds.contains_point(x, y) {
                    result.push(entry.id);
                }
            }
            RNode::Internal { children, .. } => {
                for child in children {
                    child.query_point(x, y, result);
                }
            }
        }
    }
}

/// An R-tree for 2D rectangles.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct RTree2D {
    entries: Vec<RTreeEntry>,
    root: Option<RNode>,
}

fn build_node_rtree(entries: Vec<RTreeEntry>, max_children: usize) -> RNode {
    if entries.len() <= max_children {
        if entries.len() == 1 {
            return RNode::Leaf {
                entry: entries.into_iter().next().unwrap_or_default(),
            };
        }
        let bounds = entries
            .iter()
            .fold(entries[0].bounds, |acc, e| acc.merge(&e.bounds));
        let children = entries
            .into_iter()
            .map(|e| RNode::Leaf { entry: e })
            .collect();
        return RNode::Internal { bounds, children };
    }
    // Partition by x-center
    let mut sorted = entries;
    sorted.sort_by(|a, b| {
        let ca = (a.bounds.min_x + a.bounds.max_x) * 0.5;
        let cb = (b.bounds.min_x + b.bounds.max_x) * 0.5;
        ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
    });
    let chunk_size = sorted.len().div_ceil(max_children);
    let mut children = Vec::new();
    for chunk in sorted.chunks(chunk_size.max(1)) {
        children.push(build_node_rtree(chunk.to_vec(), max_children));
    }
    let bounds = children
        .iter()
        .fold(*children[0].bounds(), |acc, c| acc.merge(c.bounds()));
    RNode::Internal { bounds, children }
}

impl RTree2D {
    /// Create a new empty R-tree.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Build from a slice of entries.
    #[allow(dead_code)]
    pub fn build(entries: Vec<RTreeEntry>) -> Self {
        if entries.is_empty() {
            return Self {
                entries: vec![],
                root: None,
            };
        }
        let root = Some(build_node_rtree(entries.clone(), 4));
        Self { entries, root }
    }

    /// Query entries overlapping the given rectangle.
    #[allow(dead_code)]
    pub fn query_overlap(&self, query: &Rect2) -> Vec<usize> {
        let mut result = Vec::new();
        if let Some(root) = &self.root {
            root.query_overlap(query, &mut result);
        }
        result
    }

    /// Query entries containing the given point.
    #[allow(dead_code)]
    pub fn query_point(&self, x: f32, y: f32) -> Vec<usize> {
        let mut result = Vec::new();
        if let Some(root) = &self.root {
            root.query_point(x, y, &mut result);
        }
        result
    }

    /// Number of entries.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Helper: create an `RTreeEntry`.
#[allow(dead_code)]
pub fn rtree_entry(id: usize, min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> RTreeEntry {
    RTreeEntry {
        bounds: Rect2::new(min_x, min_y, max_x, max_y),
        id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tree() -> RTree2D {
        let entries = vec![
            rtree_entry(0, 0.0, 0.0, 1.0, 1.0),
            rtree_entry(1, 2.0, 2.0, 3.0, 3.0),
            rtree_entry(2, 0.5, 0.5, 1.5, 1.5),
            rtree_entry(3, 10.0, 10.0, 11.0, 11.0),
        ];
        RTree2D::build(entries)
    }

    #[test]
    fn empty_tree_queries_return_empty() {
        let t = RTree2D::new();
        assert!(t
            .query_overlap(&Rect2::new(0.0, 0.0, 10.0, 10.0))
            .is_empty());
    }

    #[test]
    fn query_point_inside_entry() {
        let t = sample_tree();
        let r = t.query_point(0.5, 0.5);
        assert!(r.contains(&0));
    }

    #[test]
    fn query_point_outside_all() {
        let t = sample_tree();
        let r = t.query_point(5.0, 5.0);
        assert!(r.is_empty());
    }

    #[test]
    fn query_overlap_finds_overlapping() {
        let t = sample_tree();
        let r = t.query_overlap(&Rect2::new(0.8, 0.8, 2.2, 2.2));
        assert!(r.contains(&2));
        assert!(r.contains(&1));
    }

    #[test]
    fn query_overlap_miss() {
        let t = sample_tree();
        let r = t.query_overlap(&Rect2::new(20.0, 20.0, 21.0, 21.0));
        assert!(r.is_empty());
    }

    #[test]
    fn len_correct() {
        let t = sample_tree();
        assert_eq!(t.len(), 4);
    }

    #[test]
    fn is_empty_true() {
        let t = RTree2D::new();
        assert!(t.is_empty());
    }

    #[test]
    fn is_empty_false() {
        let t = sample_tree();
        assert!(!t.is_empty());
    }

    #[test]
    fn rect_overlaps_self() {
        let r = Rect2::new(0.0, 0.0, 1.0, 1.0);
        assert!(r.overlaps(&r));
    }

    #[test]
    fn rect_area_correct() {
        let r = Rect2::new(0.0, 0.0, 2.0, 3.0);
        assert!((r.area() - 6.0).abs() < 1e-5);
    }
}
