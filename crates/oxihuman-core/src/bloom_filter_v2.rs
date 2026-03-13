// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Bloom filter v2 with configurable hash count (FNV variants).

#![allow(dead_code)]

/// Bloom filter v2 with multiple FNV-based hash functions.
#[allow(dead_code)]
pub struct BloomFilterV2 {
    bits: Vec<u64>,
    num_bits: usize,
    num_hashes: usize,
    count: usize,
}

/// FNV-1a 64-bit hash.
#[allow(dead_code)]
pub fn fnv1a_64(data: &[u8]) -> u64 {
    const FNV_PRIME: u64 = 0x00000100_000001B3;
    const FNV_OFFSET: u64 = 0xcbf29ce4_84222325;
    let mut hash = FNV_OFFSET;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// FNV-1 64-bit hash (multiply first).
#[allow(dead_code)]
pub fn fnv1_64(data: &[u8]) -> u64 {
    const FNV_PRIME: u64 = 0x00000100_000001B3;
    const FNV_OFFSET: u64 = 0xcbf29ce4_84222325;
    let mut hash = FNV_OFFSET;
    for &byte in data {
        hash = hash.wrapping_mul(FNV_PRIME);
        hash ^= byte as u64;
    }
    hash
}

/// Double-hashing scheme: h_i(x) = (h1 + i * h2) % m
#[allow(dead_code)]
fn bit_positions(data: &[u8], num_bits: usize, num_hashes: usize) -> Vec<usize> {
    let h1 = fnv1a_64(data);
    let h2 = fnv1_64(data) | 1;
    (0..num_hashes)
        .map(|i| (h1.wrapping_add((i as u64).wrapping_mul(h2)) as usize) % num_bits)
        .collect()
}

impl BloomFilterV2 {
    /// Create a new bloom filter with given bit count and hash count.
    #[allow(dead_code)]
    pub fn new(num_bits: usize, num_hashes: usize) -> Self {
        let words = num_bits.div_ceil(64);
        Self {
            bits: vec![0u64; words],
            num_bits,
            num_hashes,
            count: 0,
        }
    }

    /// Create with estimated capacity and false-positive rate.
    #[allow(dead_code)]
    pub fn with_capacity(capacity: usize, fp_rate: f64) -> Self {
        let ln2 = 2.0_f64.ln();
        let num_bits = (-(capacity as f64) * fp_rate.ln() / (ln2 * ln2)).ceil() as usize;
        let num_bits = num_bits.max(64);
        let num_hashes = ((num_bits as f64 / capacity as f64) * ln2).ceil() as usize;
        let num_hashes = num_hashes.clamp(1, 32);
        Self::new(num_bits, num_hashes)
    }

    #[allow(dead_code)]
    pub fn insert(&mut self, data: &[u8]) {
        for pos in bit_positions(data, self.num_bits, self.num_hashes) {
            self.bits[pos / 64] |= 1u64 << (pos % 64);
        }
        self.count += 1;
    }

    #[allow(dead_code)]
    pub fn contains(&self, data: &[u8]) -> bool {
        bit_positions(data, self.num_bits, self.num_hashes)
            .iter()
            .all(|&pos| self.bits[pos / 64] & (1u64 << (pos % 64)) != 0)
    }

    #[allow(dead_code)]
    pub fn count(&self) -> usize {
        self.count
    }

    #[allow(dead_code)]
    pub fn num_bits(&self) -> usize {
        self.num_bits
    }

    #[allow(dead_code)]
    pub fn num_hashes(&self) -> usize {
        self.num_hashes
    }

    /// Estimated false positive rate given current element count.
    #[allow(dead_code)]
    pub fn estimated_fp_rate(&self) -> f64 {
        let k = self.num_hashes as f64;
        let m = self.num_bits as f64;
        let n = self.count as f64;
        (1.0 - (-k * n / m).exp()).powf(k)
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        for word in &mut self.bits {
            *word = 0;
        }
        self.count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_contains() {
        let mut bf = BloomFilterV2::new(1024, 4);
        bf.insert(b"hello");
        assert!(bf.contains(b"hello"));
    }

    #[test]
    fn test_not_inserted() {
        let bf = BloomFilterV2::new(1024, 4);
        assert!(!bf.contains(b"missing"));
    }

    #[test]
    fn test_multiple_inserts() {
        let mut bf = BloomFilterV2::new(4096, 6);
        let items: &[&[u8]] = &[b"apple", b"banana", b"cherry"];
        for item in items {
            bf.insert(item);
        }
        for item in items {
            assert!(bf.contains(item));
        }
    }

    #[test]
    fn test_count() {
        let mut bf = BloomFilterV2::new(1024, 3);
        bf.insert(b"a");
        bf.insert(b"b");
        assert_eq!(bf.count(), 2);
    }

    #[test]
    fn test_clear() {
        let mut bf = BloomFilterV2::new(512, 3);
        bf.insert(b"x");
        bf.clear();
        assert_eq!(bf.count(), 0);
        assert!(!bf.contains(b"x"));
    }

    #[test]
    fn test_with_capacity() {
        let bf = BloomFilterV2::with_capacity(1000, 0.01);
        assert!(bf.num_bits() >= 64);
        assert!(bf.num_hashes() >= 1);
    }

    #[test]
    fn test_fnv1a_deterministic() {
        assert_eq!(fnv1a_64(b"test"), fnv1a_64(b"test"));
    }

    #[test]
    fn test_fnv1_vs_fnv1a_differ() {
        assert_ne!(fnv1a_64(b"hello"), fnv1_64(b"hello"));
    }

    #[test]
    fn test_fp_rate_increases_with_insertions() {
        let mut bf = BloomFilterV2::new(256, 3);
        let r0 = bf.estimated_fp_rate();
        for i in 0u8..50 {
            bf.insert(&[i]);
        }
        let r1 = bf.estimated_fp_rate();
        assert!(r1 >= r0);
    }

    #[test]
    fn test_insert_empty_slice() {
        let mut bf = BloomFilterV2::new(128, 2);
        bf.insert(b"");
        assert!(bf.contains(b""));
    }
}
