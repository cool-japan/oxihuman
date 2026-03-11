// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A probabilistic set membership test using a bloom-filter style bit array.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BloomSet {
    bits: Vec<u64>,
    num_bits: usize,
    num_hashes: usize,
    count: usize,
}

#[allow(dead_code)]
impl BloomSet {
    pub fn new(num_bits: usize, num_hashes: usize) -> Self {
        let words = num_bits.max(64).div_ceil(64);
        Self {
            bits: vec![0u64; words],
            num_bits: words * 64,
            num_hashes: num_hashes.max(1),
            count: 0,
        }
    }

    fn hash_indices(&self, key: &str) -> Vec<usize> {
        let mut h1: u64 = 0;
        let mut h2: u64 = 0;
        for (i, b) in key.bytes().enumerate() {
            h1 = h1.wrapping_mul(31).wrapping_add(b as u64);
            h2 = h2
                .wrapping_mul(37)
                .wrapping_add((b as u64).wrapping_add(i as u64));
        }
        (0..self.num_hashes)
            .map(|i| {
                let h = h1.wrapping_add((i as u64).wrapping_mul(h2));
                (h as usize) % self.num_bits
            })
            .collect()
    }

    pub fn insert(&mut self, key: &str) {
        for idx in self.hash_indices(key) {
            let word = idx / 64;
            let bit = idx % 64;
            self.bits[word] |= 1u64 << bit;
        }
        self.count += 1;
    }

    pub fn might_contain(&self, key: &str) -> bool {
        self.hash_indices(key).iter().all(|&idx| {
            let word = idx / 64;
            let bit = idx % 64;
            (self.bits[word] & (1u64 << bit)) != 0
        })
    }

    pub fn insert_count(&self) -> usize {
        self.count
    }

    pub fn num_bits(&self) -> usize {
        self.num_bits
    }

    pub fn num_hashes(&self) -> usize {
        self.num_hashes
    }

    pub fn bits_set(&self) -> usize {
        self.bits.iter().map(|w| w.count_ones() as usize).sum()
    }

    pub fn fill_ratio(&self) -> f64 {
        if self.num_bits == 0 {
            return 0.0;
        }
        self.bits_set() as f64 / self.num_bits as f64
    }

    pub fn clear(&mut self) {
        for w in &mut self.bits {
            *w = 0;
        }
        self.count = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn estimated_false_positive_rate(&self) -> f64 {
        let m = self.num_bits as f64;
        let k = self.num_hashes as f64;
        let n = self.count as f64;
        (1.0 - (-k * n / m).exp()).powf(k)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let bs = BloomSet::new(256, 3);
        assert!(bs.is_empty());
        assert_eq!(bs.num_hashes(), 3);
    }

    #[test]
    fn test_insert_and_query() {
        let mut bs = BloomSet::new(1024, 3);
        bs.insert("hello");
        assert!(bs.might_contain("hello"));
    }

    #[test]
    fn test_likely_absent() {
        let bs = BloomSet::new(1024, 3);
        assert!(!bs.might_contain("never_inserted"));
    }

    #[test]
    fn test_insert_count() {
        let mut bs = BloomSet::new(1024, 3);
        bs.insert("a");
        bs.insert("b");
        assert_eq!(bs.insert_count(), 2);
    }

    #[test]
    fn test_bits_set() {
        let mut bs = BloomSet::new(1024, 3);
        assert_eq!(bs.bits_set(), 0);
        bs.insert("key");
        assert!(bs.bits_set() > 0);
    }

    #[test]
    fn test_fill_ratio() {
        let bs = BloomSet::new(1024, 3);
        assert!((bs.fill_ratio()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_clear() {
        let mut bs = BloomSet::new(256, 2);
        bs.insert("x");
        bs.clear();
        assert!(bs.is_empty());
        assert_eq!(bs.bits_set(), 0);
    }

    #[test]
    fn test_false_positive_rate_zero() {
        let bs = BloomSet::new(1024, 3);
        assert!((bs.estimated_false_positive_rate()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_multiple_inserts() {
        let mut bs = BloomSet::new(2048, 4);
        for i in 0..50 {
            bs.insert(&format!("key_{i}"));
        }
        for i in 0..50 {
            assert!(bs.might_contain(&format!("key_{i}")));
        }
    }

    #[test]
    fn test_num_bits_rounded() {
        let bs = BloomSet::new(100, 2);
        assert!(bs.num_bits() >= 100);
        assert_eq!(bs.num_bits() % 64, 0);
    }
}
