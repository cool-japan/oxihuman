// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Binary indexed tree (Fenwick tree) for prefix-sum queries and point updates.

/// A Fenwick (binary indexed) tree for `i64` values.
#[derive(Debug, Clone)]
pub struct FenwickTreeV2 {
    tree: Vec<i64>,
    n: usize,
}

impl FenwickTreeV2 {
    /// Create a Fenwick tree of `n` zeros.
    pub fn new(n: usize) -> Self {
        FenwickTreeV2 {
            tree: vec![0i64; n + 1],
            n,
        }
    }

    /// Build from an existing slice.
    pub fn from_slice(data: &[i64]) -> Self {
        let mut ft = FenwickTreeV2::new(data.len());
        for (i, &v) in data.iter().enumerate() {
            ft.add(i, v);
        }
        ft
    }

    /// Add `delta` to position `i` (0-indexed).
    pub fn add(&mut self, i: usize, delta: i64) {
        let mut j = (i + 1) as isize;
        while j <= self.n as isize {
            self.tree[j as usize] += delta;
            j += j & -j;
        }
    }

    /// Prefix sum of `[0, i]` (0-indexed, inclusive).
    pub fn prefix_sum(&self, i: usize) -> i64 {
        let mut sum = 0i64;
        let mut j = (i + 1) as isize;
        while j > 0 {
            sum += self.tree[j as usize];
            j -= j & -j;
        }
        sum
    }

    /// Range sum of `[lo, hi]` (0-indexed, inclusive).
    pub fn range_sum(&self, lo: usize, hi: usize) -> i64 {
        if lo == 0 {
            self.prefix_sum(hi)
        } else {
            self.prefix_sum(hi) - self.prefix_sum(lo - 1)
        }
    }

    /// Capacity.
    pub fn len(&self) -> usize {
        self.n
    }

    /// True if no elements.
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    /// Point query (value at position `i`).
    pub fn point_query(&self, i: usize) -> i64 {
        self.range_sum(i, i)
    }
}

/// Create a new Fenwick tree with `n` elements.
pub fn new_fenwick_tree_v2(n: usize) -> FenwickTreeV2 {
    FenwickTreeV2::new(n)
}

/// Add delta at position.
pub fn ft2_add(ft: &mut FenwickTreeV2, i: usize, delta: i64) {
    ft.add(i, delta);
}

/// Prefix sum [0..=i].
pub fn ft2_prefix_sum(ft: &FenwickTreeV2, i: usize) -> i64 {
    ft.prefix_sum(i)
}

/// Range sum [lo..=hi].
pub fn ft2_range_sum(ft: &FenwickTreeV2, lo: usize, hi: usize) -> i64 {
    ft.range_sum(lo, hi)
}

/// Size.
pub fn ft2_len(ft: &FenwickTreeV2) -> usize {
    ft.len()
}

/// Point query.
pub fn ft2_point_query(ft: &FenwickTreeV2, i: usize) -> i64 {
    ft.point_query(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_prefix_sum() {
        let mut ft = new_fenwick_tree_v2(5);
        ft2_add(&mut ft, 0, 3);
        ft2_add(&mut ft, 2, 5);
        assert_eq!(ft2_prefix_sum(&ft, 2), 8 /* 3 + 0 + 5 */);
    }

    #[test]
    fn test_range_sum() {
        let ft = FenwickTreeV2::from_slice(&[1, 2, 3, 4, 5]);
        assert_eq!(ft2_range_sum(&ft, 1, 3), 9 /* 2+3+4 */);
    }

    #[test]
    fn test_total_sum() {
        let ft = FenwickTreeV2::from_slice(&[1, 2, 3, 4, 5]);
        assert_eq!(ft2_prefix_sum(&ft, 4), 15 /* total */);
    }

    #[test]
    fn test_point_query() {
        let ft = FenwickTreeV2::from_slice(&[10, 20, 30]);
        assert_eq!(ft2_point_query(&ft, 1), 20 /* second element */);
    }

    #[test]
    fn test_update_point() {
        let mut ft = FenwickTreeV2::from_slice(&[1, 2, 3]);
        ft2_add(&mut ft, 1, 10); /* add 10 to index 1 */
        assert_eq!(ft2_point_query(&ft, 1), 12 /* 2 + 10 */);
    }

    #[test]
    fn test_len() {
        let ft = new_fenwick_tree_v2(7);
        assert_eq!(ft2_len(&ft), 7 /* capacity 7 */);
    }

    #[test]
    fn test_zero_tree() {
        let ft = new_fenwick_tree_v2(10);
        assert_eq!(ft2_prefix_sum(&ft, 9), 0 /* all zeros */);
    }

    #[test]
    fn test_single_element() {
        let ft = FenwickTreeV2::from_slice(&[42]);
        assert_eq!(ft2_prefix_sum(&ft, 0), 42 /* single element */);
    }

    #[test]
    fn test_from_slice() {
        let ft = FenwickTreeV2::from_slice(&[5, 5, 5, 5]);
        assert_eq!(ft2_range_sum(&ft, 0, 3), 20 /* four fives */);
    }

    #[test]
    fn test_negative_delta() {
        let mut ft = FenwickTreeV2::from_slice(&[10, 10, 10]);
        ft2_add(&mut ft, 1, -5);
        assert_eq!(ft2_point_query(&ft, 1), 5 /* 10 - 5 */);
    }
}
