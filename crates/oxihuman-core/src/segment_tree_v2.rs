// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Segment tree for range min/max/sum queries over i64 values.

/// The operation supported by the segment tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SegOp {
    Sum,
    Min,
    Max,
}

/// A segment tree supporting point updates and range queries.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SegTreeV2 {
    n: usize,
    tree: Vec<i64>,
    op: SegOp,
}

impl SegTreeV2 {
    fn identity(&self) -> i64 {
        match self.op {
            SegOp::Sum => 0,
            SegOp::Min => i64::MAX,
            SegOp::Max => i64::MIN,
        }
    }

    fn combine(&self, a: i64, b: i64) -> i64 {
        match self.op {
            SegOp::Sum => a + b,
            SegOp::Min => a.min(b),
            SegOp::Max => a.max(b),
        }
    }

    /// Build from a slice.
    #[allow(dead_code)]
    pub fn build(data: &[i64], op: SegOp) -> Self {
        let n = data.len();
        let mut tree = vec![0i64; 2 * n];
        for (i, &v) in data.iter().enumerate() {
            tree[n + i] = v;
        }
        let mut st = Self { n, tree, op };
        for i in (1..n).rev() {
            st.tree[i] = st.combine(st.tree[2 * i], st.tree[2 * i + 1]);
        }
        st
    }

    /// Point update at index `i`.
    #[allow(dead_code)]
    pub fn update(&mut self, mut i: usize, val: i64) {
        if i >= self.n {
            return;
        }
        i += self.n;
        self.tree[i] = val;
        let mut pos = i >> 1;
        while pos >= 1 {
            self.tree[pos] = self.combine(self.tree[2 * pos], self.tree[2 * pos + 1]);
            if pos == 1 {
                break;
            }
            pos >>= 1;
        }
    }

    /// Range query over `[l, r)`.
    #[allow(dead_code)]
    pub fn query(&self, mut l: usize, mut r: usize) -> i64 {
        if l >= r || self.n == 0 {
            return self.identity();
        }
        let mut res = self.identity();
        l += self.n;
        r += self.n;
        while l < r {
            if l & 1 == 1 {
                res = self.combine(res, self.tree[l]);
                l += 1;
            }
            if r & 1 == 1 {
                r -= 1;
                res = self.combine(res, self.tree[r]);
            }
            l >>= 1;
            r >>= 1;
        }
        res
    }

    /// Number of elements.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.n
    }

    /// Returns true if empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }
}

/// Build a sum segment tree.
#[allow(dead_code)]
pub fn seg2_sum(data: &[i64]) -> SegTreeV2 {
    SegTreeV2::build(data, SegOp::Sum)
}

/// Build a min segment tree.
#[allow(dead_code)]
pub fn seg2_min(data: &[i64]) -> SegTreeV2 {
    SegTreeV2::build(data, SegOp::Min)
}

/// Build a max segment tree.
#[allow(dead_code)]
pub fn seg2_max(data: &[i64]) -> SegTreeV2 {
    SegTreeV2::build(data, SegOp::Max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sum_query_full_range() {
        let t = seg2_sum(&[1, 2, 3, 4, 5]);
        assert_eq!(t.query(0, 5), 15);
    }

    #[test]
    fn sum_query_partial() {
        let t = seg2_sum(&[1, 2, 3, 4, 5]);
        assert_eq!(t.query(1, 4), 9);
    }

    #[test]
    fn min_query() {
        let t = seg2_min(&[5, 3, 8, 1, 7]);
        assert_eq!(t.query(0, 5), 1);
    }

    #[test]
    fn max_query() {
        let t = seg2_max(&[5, 3, 8, 1, 7]);
        assert_eq!(t.query(0, 5), 8);
    }

    #[test]
    fn update_and_requery() {
        let mut t = seg2_sum(&[1, 2, 3]);
        t.update(1, 10);
        assert_eq!(t.query(0, 3), 14);
    }

    #[test]
    fn len_correct() {
        let t = seg2_sum(&[1, 2, 3, 4]);
        assert_eq!(t.len(), 4);
    }

    #[test]
    fn empty_query_returns_identity() {
        let t = seg2_sum(&[1, 2, 3]);
        assert_eq!(t.query(2, 2), 0);
    }

    #[test]
    fn single_element() {
        let t = seg2_min(&[42]);
        assert_eq!(t.query(0, 1), 42);
    }

    #[test]
    fn is_empty_false() {
        let t = seg2_sum(&[1]);
        assert!(!t.is_empty());
    }

    #[test]
    fn max_update_changes_result() {
        let mut t = seg2_max(&[1, 2, 3]);
        t.update(0, 100);
        assert_eq!(t.query(0, 3), 100);
    }
}
