// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Fenwick tree (Binary Indexed Tree) for prefix sum queries.

/// A Fenwick tree (BIT) over i64 values.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FenwickTree {
    tree: Vec<i64>,
    n: usize,
}

impl FenwickTree {
    /// Create a Fenwick tree of size `n` (1-indexed internally).
    #[allow(dead_code)]
    pub fn new(n: usize) -> Self {
        Self {
            tree: vec![0; n + 1],
            n,
        }
    }

    /// Build from a slice of values.
    #[allow(dead_code)]
    pub fn build(data: &[i64]) -> Self {
        let mut ft = Self::new(data.len());
        for (i, &v) in data.iter().enumerate() {
            ft.update(i, v);
        }
        ft
    }

    /// Add `delta` to position `i` (0-indexed).
    #[allow(dead_code)]
    pub fn update(&mut self, i: usize, delta: i64) {
        let mut pos = (i + 1) as isize;
        while pos <= self.n as isize {
            self.tree[pos as usize] += delta;
            pos += pos & (-pos);
        }
    }

    /// Prefix sum of `[0, i]` (0-indexed).
    #[allow(dead_code)]
    pub fn prefix_sum(&self, i: usize) -> i64 {
        let mut pos = (i + 1) as isize;
        let mut s = 0i64;
        while pos > 0 {
            s += self.tree[pos as usize];
            pos -= pos & (-pos);
        }
        s
    }

    /// Range sum of `[l, r]` (both 0-indexed, inclusive).
    #[allow(dead_code)]
    pub fn range_sum(&self, l: usize, r: usize) -> i64 {
        if l > r {
            return 0;
        }
        let right = self.prefix_sum(r);
        let left = if l > 0 { self.prefix_sum(l - 1) } else { 0 };
        right - left
    }

    /// Total sum.
    #[allow(dead_code)]
    pub fn total(&self) -> i64 {
        if self.n == 0 {
            return 0;
        }
        self.prefix_sum(self.n - 1)
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

    /// Set the value at position `i` (0-indexed) to `val`.
    #[allow(dead_code)]
    pub fn set(&mut self, i: usize, val: i64) {
        let old = self.range_sum(i, i);
        self.update(i, val - old);
    }
}

/// Quick prefix sum query.
#[allow(dead_code)]
pub fn fenwick_prefix(ft: &FenwickTree, i: usize) -> i64 {
    ft.prefix_sum(i)
}

/// Quick range sum query.
#[allow(dead_code)]
pub fn fenwick_range(ft: &FenwickTree, l: usize, r: usize) -> i64 {
    ft.range_sum(l, r)
}

#[allow(dead_code)]
pub fn new_fenwick_tree(n: usize) -> FenwickTree {
    FenwickTree::new(n)
}

#[allow(dead_code)]
pub fn ft_update(tree: &mut FenwickTree, i: usize, delta: i64) {
    tree.update(i, delta);
}

#[allow(dead_code)]
pub fn ft_prefix_sum(tree: &FenwickTree, i: usize) -> i64 {
    tree.prefix_sum(i)
}

#[allow(dead_code)]
pub fn ft_range_sum(tree: &FenwickTree, l: usize, r: usize) -> i64 {
    tree.range_sum(l, r)
}

#[allow(dead_code)]
pub fn ft_size(tree: &FenwickTree) -> usize {
    tree.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_sum_correct() {
        let ft = FenwickTree::build(&[1, 2, 3, 4, 5]);
        assert_eq!(ft.total(), 15);
    }

    #[test]
    fn prefix_sum_correct() {
        let ft = FenwickTree::build(&[1, 2, 3, 4, 5]);
        assert_eq!(ft.prefix_sum(2), 6);
    }

    #[test]
    fn range_sum_correct() {
        let ft = FenwickTree::build(&[1, 2, 3, 4, 5]);
        assert_eq!(ft.range_sum(1, 3), 9);
    }

    #[test]
    fn update_changes_sum() {
        let mut ft = FenwickTree::build(&[1, 2, 3]);
        ft.update(1, 10);
        assert_eq!(ft.total(), 16);
    }

    #[test]
    fn set_replaces_value() {
        let mut ft = FenwickTree::build(&[1, 2, 3]);
        ft.set(1, 10);
        assert_eq!(ft.range_sum(1, 1), 10);
    }

    #[test]
    fn len_matches_input() {
        let ft = FenwickTree::build(&[1, 2, 3, 4]);
        assert_eq!(ft.len(), 4);
    }

    #[test]
    fn empty_tree_is_empty() {
        let ft = FenwickTree::new(0);
        assert!(ft.is_empty());
    }

    #[test]
    fn single_element() {
        let ft = FenwickTree::build(&[42]);
        assert_eq!(ft.prefix_sum(0), 42);
    }

    #[test]
    fn fenwick_prefix_helper() {
        let ft = FenwickTree::build(&[5, 5, 5]);
        assert_eq!(fenwick_prefix(&ft, 1), 10);
    }

    #[test]
    fn fenwick_range_helper() {
        let ft = FenwickTree::build(&[1, 1, 1, 1, 1]);
        assert_eq!(fenwick_range(&ft, 1, 3), 3);
    }
}
