// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Segment tree for range sum / range min / range max queries.

/// Operation supported by the segment tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegTreeOp {
    Sum,
    Min,
    Max,
}

/// A generic segment tree over `i64` values.
#[derive(Debug, Clone)]
pub struct SegmentTreeV3 {
    n: usize,
    tree: Vec<i64>,
    op: SegTreeOp,
}

impl SegmentTreeV3 {
    /// Build a segment tree from a slice of values.
    pub fn build(values: &[i64], op: SegTreeOp) -> Self {
        let n = values.len();
        let mut tree = vec![identity(op); 2 * n];
        tree[n..].copy_from_slice(values);
        for i in (1..n).rev() {
            tree[i] = combine(tree[2 * i], tree[2 * i + 1], op);
        }
        SegmentTreeV3 { n, tree, op }
    }

    /// Point update: set index `i` to `value`.
    pub fn update(&mut self, i: usize, value: i64) {
        let mut pos = i + self.n;
        self.tree[pos] = value;
        pos /= 2;
        while pos >= 1 {
            self.tree[pos] = combine(self.tree[2 * pos], self.tree[2 * pos + 1], self.op);
            if pos == 1 { break; }
            pos /= 2;
        }
    }

    /// Range query over `[lo, hi)`.
    pub fn query(&self, lo: usize, hi: usize) -> i64 {
        let mut result = identity(self.op);
        let mut l = lo + self.n;
        let mut r = hi + self.n;
        while l < r {
            if l & 1 == 1 {
                result = combine(result, self.tree[l], self.op);
                l += 1;
            }
            if r & 1 == 1 {
                r -= 1;
                result = combine(result, self.tree[r], self.op);
            }
            l /= 2;
            r /= 2;
        }
        result
    }

    /// Number of leaf elements.
    pub fn len(&self) -> usize {
        self.n
    }

    /// True if the tree has no elements.
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }
}

fn identity(op: SegTreeOp) -> i64 {
    match op {
        SegTreeOp::Sum => 0,
        SegTreeOp::Min => i64::MAX,
        SegTreeOp::Max => i64::MIN,
    }
}

fn combine(a: i64, b: i64, op: SegTreeOp) -> i64 {
    match op {
        SegTreeOp::Sum => a + b,
        SegTreeOp::Min => a.min(b),
        SegTreeOp::Max => a.max(b),
    }
}

/// Build a sum segment tree.
pub fn new_seg_tree_sum(values: &[i64]) -> SegmentTreeV3 {
    SegmentTreeV3::build(values, SegTreeOp::Sum)
}

/// Build a min segment tree.
pub fn new_seg_tree_min(values: &[i64]) -> SegmentTreeV3 {
    SegmentTreeV3::build(values, SegTreeOp::Min)
}

/// Build a max segment tree.
pub fn new_seg_tree_max(values: &[i64]) -> SegmentTreeV3 {
    SegmentTreeV3::build(values, SegTreeOp::Max)
}

/// Point update.
pub fn st3_update(tree: &mut SegmentTreeV3, i: usize, value: i64) {
    tree.update(i, value);
}

/// Range query.
pub fn st3_query(tree: &SegmentTreeV3, lo: usize, hi: usize) -> i64 {
    tree.query(lo, hi)
}

/// Element count.
pub fn st3_len(tree: &SegmentTreeV3) -> usize {
    tree.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum_query() {
        let t = new_seg_tree_sum(&[1, 2, 3, 4, 5]);
        assert_eq!(st3_query(&t, 0, 5), 15 /* total sum */);
    }

    #[test]
    fn test_sum_partial() {
        let t = new_seg_tree_sum(&[1, 2, 3, 4, 5]);
        assert_eq!(st3_query(&t, 1, 4), 9 /* 2+3+4 */);
    }

    #[test]
    fn test_min_query() {
        let t = new_seg_tree_min(&[5, 3, 8, 1, 7]);
        assert_eq!(st3_query(&t, 0, 5), 1 /* global min */);
    }

    #[test]
    fn test_max_query() {
        let t = new_seg_tree_max(&[5, 3, 8, 1, 7]);
        assert_eq!(st3_query(&t, 0, 5), 8 /* global max */);
    }

    #[test]
    fn test_update() {
        let mut t = new_seg_tree_sum(&[1, 2, 3]);
        st3_update(&mut t, 1, 10);
        assert_eq!(st3_query(&t, 0, 3), 14 /* 1 + 10 + 3 */);
    }

    #[test]
    fn test_len() {
        let t = new_seg_tree_sum(&[1, 2, 3, 4]);
        assert_eq!(st3_len(&t), 4 /* four elements */);
    }

    #[test]
    fn test_single_element() {
        let t = new_seg_tree_sum(&[42]);
        assert_eq!(st3_query(&t, 0, 1), 42 /* single element */);
    }

    #[test]
    fn test_min_range() {
        let t = new_seg_tree_min(&[10, 3, 7, 2, 9]);
        assert_eq!(st3_query(&t, 2, 5), 2 /* min of [7,2,9] */);
    }

    #[test]
    fn test_max_range() {
        let t = new_seg_tree_max(&[1, 5, 2, 8, 3]);
        assert_eq!(st3_query(&t, 1, 4), 8 /* max of [5,2,8] */);
    }

    #[test]
    fn test_update_min() {
        let mut t = new_seg_tree_min(&[5, 10, 15]);
        st3_update(&mut t, 0, 1);
        assert_eq!(st3_query(&t, 0, 3), 1 /* updated min */);
    }
}
