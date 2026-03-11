// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Bloom filter v3 — probabilistic membership test using a bit-array.

const SEED_A: u64 = 0x9e37_79b9_7f4a_7c15;
const SEED_B: u64 = 0x6c62_272e_07bb_0142;

/// Bloom filter backed by a `Vec<u64>` bit-array.
pub struct BloomFilterV3 {
    bits: Vec<u64>,
    num_bits: usize,
    num_hashes: usize,
    count: usize,
}

impl BloomFilterV3 {
    /// Create a new filter with at least `capacity_bits` bits and `num_hashes` hash functions.
    pub fn new(capacity_bits: usize, num_hashes: usize) -> Self {
        let words = capacity_bits.div_ceil(64).max(1);
        BloomFilterV3 {
            bits: vec![0u64; words],
            num_bits: words * 64,
            num_hashes: num_hashes.max(1),
            count: 0,
        }
    }

    fn hash_pair(&self, item: u64) -> (u64, u64) {
        let h1 = item
            .wrapping_mul(SEED_A)
            .rotate_left(17)
            .wrapping_add(SEED_B);
        let h2 = h1.wrapping_mul(SEED_B).rotate_left(31).wrapping_add(SEED_A);
        (h1, h2)
    }

    fn bit_indices(&self, item: u64) -> impl Iterator<Item = usize> + '_ {
        let (h1, h2) = self.hash_pair(item);
        (0..self.num_hashes).map(move |i| {
            let combined = h1.wrapping_add(h2.wrapping_mul(i as u64));
            (combined as usize) % self.num_bits
        })
    }

    /// Insert an item (represented as a `u64` hash).
    pub fn insert(&mut self, item: u64) {
        for idx in self.bit_indices(item).collect::<Vec<_>>() {
            self.bits[idx / 64] |= 1u64 << (idx % 64);
        }
        self.count += 1;
    }

    /// Query membership — may return false positives but never false negatives.
    pub fn contains(&self, item: u64) -> bool {
        self.bit_indices(item)
            .all(|idx| self.bits[idx / 64] & (1u64 << (idx % 64)) != 0)
    }

    /// Approximate number of inserted items.
    pub fn len(&self) -> usize {
        self.count
    }

    /// True if no items have been inserted.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Estimated false-positive rate given current fill.
    pub fn estimated_fpr(&self) -> f64 {
        let n = self.count as f64;
        let m = self.num_bits as f64;
        let k = self.num_hashes as f64;
        (1.0 - (-k * n / m).exp()).powf(k)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_contains() {
        let mut bf = BloomFilterV3::new(1024, 3);
        bf.insert(42);
        assert!(bf.contains(42) /* inserted item must be found */);
    }

    #[test]
    fn test_not_inserted() {
        let bf = BloomFilterV3::new(1024, 3);
        /* item 99 was never inserted — should not be found (no false pos for empty filter) */
        assert!(!bf.contains(99));
    }

    #[test]
    fn test_len_tracks_inserts() {
        let mut bf = BloomFilterV3::new(512, 2);
        bf.insert(1);
        bf.insert(2);
        bf.insert(3);
        assert_eq!(bf.len(), 3 /* three inserts */);
    }

    #[test]
    fn test_is_empty() {
        let bf = BloomFilterV3::new(256, 2);
        assert!(bf.is_empty() /* fresh filter is empty */);
    }

    #[test]
    fn test_multiple_items() {
        let mut bf = BloomFilterV3::new(2048, 4);
        for i in 0u64..100 {
            bf.insert(i);
        }
        for i in 0u64..100 {
            assert!(bf.contains(i) /* all inserted items must be found */);
        }
    }

    #[test]
    fn test_estimated_fpr_zero_inserts() {
        let bf = BloomFilterV3::new(512, 3);
        let fpr = bf.estimated_fpr();
        assert!(fpr >= 0.0 /* fpr is non-negative */);
    }

    #[test]
    fn test_large_capacity() {
        let mut bf = BloomFilterV3::new(65536, 5);
        bf.insert(u64::MAX);
        assert!(bf.contains(u64::MAX) /* large hash must be found */);
    }

    #[test]
    fn test_hash_pair_distinct() {
        let bf = BloomFilterV3::new(128, 2);
        let (h1, h2) = bf.hash_pair(0);
        let (h3, h4) = bf.hash_pair(1);
        assert!(h1 != h3 || h2 != h4 /* different inputs produce different hashes */);
    }

    #[test]
    fn test_single_bit_filter() {
        /* edge case: minimum capacity */
        let mut bf = BloomFilterV3::new(1, 1);
        bf.insert(7);
        assert!(bf.len() == 1);
    }
}
