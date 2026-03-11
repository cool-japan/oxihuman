// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Range-sum segment tree over f32 values.

/// A segment tree that supports point-update and range-sum queries.
#[allow(dead_code)]
pub struct SegmentTree {
    n: usize,
    data: Vec<f32>,
}

#[allow(dead_code)]
impl SegmentTree {
    /// Build from a slice of f32 values.
    pub fn build(values: &[f32]) -> Self {
        let n = values.len();
        let mut data = vec![0.0_f32; 2 * n];
        for (i, &v) in values.iter().enumerate() {
            data[n + i] = v;
        }
        #[allow(clippy::needless_range_loop)]
        for i in (1..n).rev() {
            data[i] = data[2 * i] + data[2 * i + 1];
        }
        Self { n, data }
    }

    /// Update position `pos` to `value`.
    pub fn update(&mut self, mut pos: usize, value: f32) {
        pos += self.n;
        self.data[pos] = value;
        let mut i = pos >> 1;
        while i >= 1 {
            self.data[i] = self.data[2 * i] + self.data[2 * i + 1];
            i >>= 1;
        }
    }

    /// Sum over [l, r) (exclusive right).
    pub fn query(&self, mut l: usize, mut r: usize) -> f32 {
        let mut sum = 0.0_f32;
        l += self.n;
        r += self.n;
        while l < r {
            if l & 1 != 0 {
                sum += self.data[l];
                l += 1;
            }
            if r & 1 != 0 {
                r -= 1;
                sum += self.data[r];
            }
            l >>= 1;
            r >>= 1;
        }
        sum
    }

    /// Query single element.
    pub fn get(&self, pos: usize) -> f32 {
        self.data[self.n + pos]
    }

    /// Total sum of all elements.
    pub fn total(&self) -> f32 {
        if self.n == 0 {
            0.0
        } else {
            self.data[1]
        }
    }

    pub fn len(&self) -> usize {
        self.n
    }

    pub fn is_empty(&self) -> bool {
        self.n == 0
    }
}

pub fn build_segment_tree(values: &[f32]) -> SegmentTree {
    SegmentTree::build(values)
}

pub fn seg_query(tree: &SegmentTree, l: usize, r: usize) -> f32 {
    tree.query(l, r)
}

pub fn seg_update(tree: &mut SegmentTree, pos: usize, val: f32) {
    tree.update(pos, val);
}

pub fn seg_total(tree: &SegmentTree) -> f32 {
    tree.total()
}

pub fn seg_get(tree: &SegmentTree, pos: usize) -> f32 {
    tree.get(pos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_sum() {
        let t = build_segment_tree(&[1.0, 2.0, 3.0, 4.0]);
        assert!((seg_total(&t) - 10.0).abs() < 1e-6);
    }

    #[test]
    fn range_query() {
        let t = build_segment_tree(&[1.0, 2.0, 3.0, 4.0]);
        assert!((seg_query(&t, 1, 3) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn point_update() {
        let mut t = build_segment_tree(&[1.0, 2.0, 3.0]);
        seg_update(&mut t, 1, 10.0);
        assert!((seg_get(&t, 1) - 10.0).abs() < 1e-6);
        assert!((seg_total(&t) - 14.0).abs() < 1e-6);
    }

    #[test]
    fn empty_tree() {
        let t = build_segment_tree(&[]);
        assert!(t.is_empty());
        assert!((seg_total(&t)).abs() < 1e-6);
    }

    #[test]
    fn single_element() {
        let t = build_segment_tree(&[42.0]);
        assert!((seg_total(&t) - 42.0).abs() < 1e-6);
        assert!((seg_query(&t, 0, 1) - 42.0).abs() < 1e-6);
    }

    #[test]
    fn full_range_equals_total() {
        let vals = [1.0, 5.0, 2.0, 8.0];
        let t = build_segment_tree(&vals);
        assert!((seg_query(&t, 0, 4) - seg_total(&t)).abs() < 1e-6);
    }

    #[test]
    fn update_first_element() {
        let mut t = build_segment_tree(&[0.0, 3.0, 5.0]);
        seg_update(&mut t, 0, 7.0);
        assert!((seg_total(&t) - 15.0).abs() < 1e-6);
    }

    #[test]
    fn len_matches_input() {
        let t = build_segment_tree(&[1.0, 2.0, 3.0]);
        assert_eq!(t.len(), 3);
    }

    #[test]
    fn get_returns_leaf() {
        let t = build_segment_tree(&[9.0, 4.0, 7.0]);
        assert!((seg_get(&t, 2) - 7.0).abs() < 1e-6);
    }

    #[test]
    fn repeated_updates_consistent() {
        let mut t = build_segment_tree(&[1.0, 1.0, 1.0, 1.0]);
        seg_update(&mut t, 0, 10.0);
        seg_update(&mut t, 3, 10.0);
        assert!((seg_total(&t) - 22.0).abs() < 1e-6);
    }
}
