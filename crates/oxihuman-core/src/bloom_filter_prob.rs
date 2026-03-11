// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Probabilistic Bloom filter using FNV-1a and djb2 hash functions.

pub struct BloomFilterProb {
    pub bits: Vec<u64>,
    pub num_hashes: usize,
    pub capacity: usize,
}

fn fnv1a(data: &[u8]) -> u64 {
    let mut hash: u64 = 14695981039346656037;
    for &b in data {
        hash ^= b as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    hash
}

fn djb2(data: &[u8]) -> u64 {
    let mut hash: u64 = 5381;
    for &b in data {
        hash = hash
            .wrapping_shl(5)
            .wrapping_add(hash)
            .wrapping_add(b as u64);
    }
    hash
}

pub fn new_bloom_filter_prob(capacity: usize, num_hashes: usize) -> BloomFilterProb {
    let num_bits = capacity.max(64);
    let num_words = num_bits.div_ceil(64);
    BloomFilterProb {
        bits: vec![0u64; num_words],
        num_hashes,
        capacity,
    }
}

fn bit_count(f: &BloomFilterProb) -> usize {
    f.bits.len() * 64
}

fn set_bit(f: &mut BloomFilterProb, idx: usize) {
    let total = bit_count(f);
    let i = idx % total;
    f.bits[i / 64] |= 1u64 << (i % 64);
}

fn check_bit(f: &BloomFilterProb, idx: usize) -> bool {
    let total = bit_count(f);
    let i = idx % total;
    (f.bits[i / 64] >> (i % 64)) & 1 == 1
}

pub fn bloom_prob_insert(f: &mut BloomFilterProb, item: &str) {
    let data = item.as_bytes();
    let h1 = fnv1a(data);
    let h2 = djb2(data);
    for k in 0..f.num_hashes {
        let idx = h1.wrapping_add((k as u64).wrapping_mul(h2)) as usize;
        set_bit(f, idx);
    }
}

pub fn bloom_prob_may_contain(f: &BloomFilterProb, item: &str) -> bool {
    let data = item.as_bytes();
    let h1 = fnv1a(data);
    let h2 = djb2(data);
    for k in 0..f.num_hashes {
        let idx = h1.wrapping_add((k as u64).wrapping_mul(h2)) as usize;
        if !check_bit(f, idx) {
            return false;
        }
    }
    true
}

pub fn bloom_prob_bit_count(f: &BloomFilterProb) -> usize {
    bit_count(f)
}

pub fn bloom_prob_hash_count(f: &BloomFilterProb) -> usize {
    f.num_hashes
}

pub fn bloom_prob_estimated_fp_rate(f: &BloomFilterProb, inserted: usize) -> f64 {
    let m = bit_count(f) as f64;
    let k = f.num_hashes as f64;
    let n = inserted as f64;
    (1.0 - (-k * n / m).exp()).powf(k)
}

pub fn bloom_prob_reset(f: &mut BloomFilterProb) {
    for w in &mut f.bits {
        *w = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_bloom_filter_prob() {
        /* create a bloom filter with capacity 1000 and 3 hashes */
        let f = new_bloom_filter_prob(1000, 3);
        assert_eq!(f.num_hashes, 3);
        assert_eq!(f.capacity, 1000);
        assert!(!f.bits.is_empty());
    }

    #[test]
    fn test_bloom_prob_insert_and_may_contain() {
        /* insert items and check membership */
        let mut f = new_bloom_filter_prob(1000, 3);
        bloom_prob_insert(&mut f, "hello");
        bloom_prob_insert(&mut f, "world");
        assert!(bloom_prob_may_contain(&f, "hello"));
        assert!(bloom_prob_may_contain(&f, "world"));
    }

    #[test]
    fn test_bloom_prob_not_contained() {
        /* items not inserted should not be found (for small sets) */
        let mut f = new_bloom_filter_prob(10000, 5);
        bloom_prob_insert(&mut f, "apple");
        assert!(bloom_prob_may_contain(&f, "apple"));
        assert!(!bloom_prob_may_contain(
            &f,
            "xyz_definitely_not_there_99999"
        ));
    }

    #[test]
    fn test_bloom_prob_bit_count() {
        /* bit count should be >= capacity */
        let f = new_bloom_filter_prob(256, 4);
        assert!(bloom_prob_bit_count(&f) >= 256);
    }

    #[test]
    fn test_bloom_prob_hash_count() {
        /* hash count should match constructor arg */
        let f = new_bloom_filter_prob(512, 7);
        assert_eq!(bloom_prob_hash_count(&f), 7);
    }

    #[test]
    fn test_bloom_prob_fp_rate() {
        /* fp rate should be between 0 and 1 */
        let f = new_bloom_filter_prob(1000, 3);
        let rate = bloom_prob_estimated_fp_rate(&f, 100);
        assert!((0.0..=1.0).contains(&rate));
    }

    #[test]
    fn test_bloom_prob_reset() {
        /* after reset, previously inserted items should not be found */
        let mut f = new_bloom_filter_prob(1000, 3);
        bloom_prob_insert(&mut f, "hello");
        assert!(bloom_prob_may_contain(&f, "hello"));
        bloom_prob_reset(&mut f);
        assert!(!bloom_prob_may_contain(&f, "hello"));
    }

    #[test]
    fn test_bloom_prob_multiple_hashes() {
        /* test with different number of hash functions */
        let mut f = new_bloom_filter_prob(2000, 6);
        for i in 0..50u32 {
            bloom_prob_insert(&mut f, &format!("item_{i}"));
        }
        for i in 0..50u32 {
            assert!(bloom_prob_may_contain(&f, &format!("item_{i}")));
        }
    }
}
