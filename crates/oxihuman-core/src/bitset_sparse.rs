// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Sparse bitset backed by a HashSet for large bit indices.

#![allow(dead_code)]

use std::collections::HashSet;

/// A sparse bitset that supports large bit indices efficiently.
#[allow(dead_code)]
pub struct SparseBitset {
    pub bits: HashSet<u64>,
}

/// Create a new empty sparse bitset.
#[allow(dead_code)]
pub fn new_sparse_bitset() -> SparseBitset {
    SparseBitset {
        bits: HashSet::new(),
    }
}

/// Set bit at index `bit`.
#[allow(dead_code)]
pub fn bitset_set(bs: &mut SparseBitset, bit: u64) {
    bs.bits.insert(bit);
}

/// Clear bit at index `bit`.
#[allow(dead_code)]
pub fn bitset_clear(bs: &mut SparseBitset, bit: u64) {
    bs.bits.remove(&bit);
}

/// Return true if bit at index `bit` is set.
#[allow(dead_code)]
pub fn bitset_get(bs: &SparseBitset, bit: u64) -> bool {
    bs.bits.contains(&bit)
}

/// Return the number of set bits.
#[allow(dead_code)]
pub fn bitset_count(bs: &SparseBitset) -> usize {
    bs.bits.len()
}

/// Compute the union of two sparse bitsets.
#[allow(dead_code)]
pub fn bitset_union(a: &SparseBitset, b: &SparseBitset) -> SparseBitset {
    let bits = a.bits.union(&b.bits).copied().collect();
    SparseBitset { bits }
}

/// Compute the intersection of two sparse bitsets.
#[allow(dead_code)]
pub fn bitset_intersect(a: &SparseBitset, b: &SparseBitset) -> SparseBitset {
    let bits = a.bits.intersection(&b.bits).copied().collect();
    SparseBitset { bits }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_bitset_empty() {
        let bs = new_sparse_bitset();
        assert_eq!(bitset_count(&bs), 0);
    }

    #[test]
    fn set_and_get() {
        let mut bs = new_sparse_bitset();
        bitset_set(&mut bs, 42);
        assert!(bitset_get(&bs, 42));
    }

    #[test]
    fn unset_bit_returns_false() {
        let bs = new_sparse_bitset();
        assert!(!bitset_get(&bs, 100));
    }

    #[test]
    fn clear_bit() {
        let mut bs = new_sparse_bitset();
        bitset_set(&mut bs, 10);
        bitset_clear(&mut bs, 10);
        assert!(!bitset_get(&bs, 10));
    }

    #[test]
    fn count_after_sets() {
        let mut bs = new_sparse_bitset();
        bitset_set(&mut bs, 1);
        bitset_set(&mut bs, 1_000_000);
        bitset_set(&mut bs, u64::MAX);
        assert_eq!(bitset_count(&bs), 3);
    }

    #[test]
    fn large_index() {
        let mut bs = new_sparse_bitset();
        bitset_set(&mut bs, 10_000_000_000);
        assert!(bitset_get(&bs, 10_000_000_000));
    }

    #[test]
    fn union_combines_bits() {
        let mut a = new_sparse_bitset();
        let mut b = new_sparse_bitset();
        bitset_set(&mut a, 1);
        bitset_set(&mut b, 2);
        let u = bitset_union(&a, &b);
        assert!(bitset_get(&u, 1));
        assert!(bitset_get(&u, 2));
        assert_eq!(bitset_count(&u), 2);
    }

    #[test]
    fn intersect_keeps_common_bits() {
        let mut a = new_sparse_bitset();
        let mut b = new_sparse_bitset();
        bitset_set(&mut a, 5);
        bitset_set(&mut a, 10);
        bitset_set(&mut b, 10);
        bitset_set(&mut b, 20);
        let i = bitset_intersect(&a, &b);
        assert!(bitset_get(&i, 10));
        assert!(!bitset_get(&i, 5));
        assert!(!bitset_get(&i, 20));
    }

    #[test]
    fn intersect_empty_result() {
        let mut a = new_sparse_bitset();
        let mut b = new_sparse_bitset();
        bitset_set(&mut a, 1);
        bitset_set(&mut b, 2);
        let i = bitset_intersect(&a, &b);
        assert_eq!(bitset_count(&i), 0);
    }
}
