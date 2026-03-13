// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

/// A simple Bloom filter using multiple FNV-style hash functions.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BloomFilter {
    pub bits: Vec<u64>,
    pub n_hash: usize,
    pub capacity_bits: usize,
}

/// Create a new Bloom filter with the given bit capacity and number of hash functions.
#[allow(dead_code)]
pub fn new_bloom_filter(capacity_bits: usize, n_hash: usize) -> BloomFilter {
    let n_words = capacity_bits.div_ceil(64);
    BloomFilter {
        bits: vec![0u64; n_words],
        n_hash: n_hash.max(1),
        capacity_bits: n_words * 64,
    }
}

fn fnv_hash(item: &[u8], seed: u64) -> u64 {
    let mut h = seed ^ 14695981039346656037u64;
    for &b in item {
        h ^= b as u64;
        h = h.wrapping_mul(1099511628211u64);
    }
    h
}

fn bit_index(bf: &BloomFilter, item: &[u8], hash_i: usize) -> usize {
    let h = fnv_hash(item, hash_i as u64 * 2654435761u64);
    (h as usize) % bf.capacity_bits
}

/// Insert an item into the Bloom filter.
#[allow(dead_code)]
pub fn bloom_insert(bf: &mut BloomFilter, item: &[u8]) {
    for i in 0..bf.n_hash {
        let idx = bit_index(bf, item, i);
        bf.bits[idx / 64] |= 1u64 << (idx % 64);
    }
}

/// Check whether an item is (probably) in the Bloom filter.
/// False positives are possible; false negatives are not.
#[allow(dead_code)]
pub fn bloom_contains(bf: &BloomFilter, item: &[u8]) -> bool {
    for i in 0..bf.n_hash {
        let idx = bit_index(bf, item, i);
        if bf.bits[idx / 64] & (1u64 << (idx % 64)) == 0 {
            return false;
        }
    }
    true
}

/// Return the fraction of bits that are set (fill ratio).
#[allow(dead_code)]
pub fn bloom_fill_ratio(bf: &BloomFilter) -> f64 {
    if bf.capacity_bits == 0 {
        return 0.0;
    }
    let set_bits: u32 = bf.bits.iter().map(|w| w.count_ones()).sum();
    set_bits as f64 / bf.capacity_bits as f64
}

/// Reset the Bloom filter (clear all bits).
#[allow(dead_code)]
pub fn bloom_reset(bf: &mut BloomFilter) {
    for w in &mut bf.bits {
        *w = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserted_item_found() {
        let mut bf = new_bloom_filter(1024, 3);
        bloom_insert(&mut bf, b"hello");
        assert!(bloom_contains(&bf, b"hello"));
    }

    #[test]
    fn not_inserted_item_not_found() {
        let bf = new_bloom_filter(1024, 3);
        assert!(!bloom_contains(&bf, b"world"));
    }

    #[test]
    fn multiple_items_all_found() {
        let mut bf = new_bloom_filter(2048, 4);
        for i in 0u32..20 {
            bloom_insert(&mut bf, &i.to_le_bytes());
        }
        for i in 0u32..20 {
            assert!(bloom_contains(&bf, &i.to_le_bytes()));
        }
    }

    #[test]
    fn fill_ratio_increases_after_insert() {
        let mut bf = new_bloom_filter(1024, 3);
        let before = bloom_fill_ratio(&bf);
        bloom_insert(&mut bf, b"test");
        let after = bloom_fill_ratio(&bf);
        assert!(after > before);
    }

    #[test]
    fn fill_ratio_range() {
        let bf = new_bloom_filter(1024, 3);
        let r = bloom_fill_ratio(&bf);
        assert!((0.0..=1.0).contains(&r));
    }

    #[test]
    fn reset_clears_bits() {
        let mut bf = new_bloom_filter(1024, 3);
        bloom_insert(&mut bf, b"data");
        bloom_reset(&mut bf);
        assert!(!bloom_contains(&bf, b"data"));
    }

    #[test]
    fn empty_item_insertable() {
        let mut bf = new_bloom_filter(512, 2);
        bloom_insert(&mut bf, b"");
        assert!(bloom_contains(&bf, b""));
    }

    #[test]
    fn single_hash_works() {
        let mut bf = new_bloom_filter(512, 1);
        bloom_insert(&mut bf, b"unique");
        assert!(bloom_contains(&bf, b"unique"));
    }

    #[test]
    fn fill_ratio_zero_initially() {
        let bf = new_bloom_filter(1024, 3);
        assert_eq!(bloom_fill_ratio(&bf), 0.0);
    }

    #[test]
    fn large_filter_capacity() {
        let bf = new_bloom_filter(65536, 5);
        assert!(bf.capacity_bits >= 65536);
    }
}
