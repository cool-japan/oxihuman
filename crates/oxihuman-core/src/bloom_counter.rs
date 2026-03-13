// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A counting bloom filter for approximate frequency tracking.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BloomCounter {
    counters: Vec<u8>,
    num_hashes: usize,
}

#[allow(dead_code)]
impl BloomCounter {
    pub fn new(size: usize, num_hashes: usize) -> Self {
        Self {
            counters: vec![0u8; size],
            num_hashes,
        }
    }

    fn hash_indices(&self, key: &str) -> Vec<usize> {
        let len = self.counters.len();
        if len == 0 {
            return Vec::new();
        }
        let mut indices = Vec::with_capacity(self.num_hashes);
        let bytes = key.as_bytes();
        for i in 0..self.num_hashes {
            let mut h: u64 = 0xcbf2_9ce4_8422_2325;
            for &b in bytes {
                h ^= b as u64;
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
            h = h.wrapping_add((i as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15));
            indices.push((h as usize) % len);
        }
        indices
    }

    pub fn insert(&mut self, key: &str) {
        for idx in self.hash_indices(key) {
            self.counters[idx] = self.counters[idx].saturating_add(1);
        }
    }

    pub fn estimate_count(&self, key: &str) -> u8 {
        self.hash_indices(key)
            .into_iter()
            .map(|idx| self.counters[idx])
            .min()
            .unwrap_or(0)
    }

    pub fn remove(&mut self, key: &str) {
        for idx in self.hash_indices(key) {
            self.counters[idx] = self.counters[idx].saturating_sub(1);
        }
    }

    pub fn might_contain(&self, key: &str) -> bool {
        self.estimate_count(key) > 0
    }

    pub fn clear(&mut self) {
        self.counters.fill(0);
    }

    pub fn size(&self) -> usize {
        self.counters.len()
    }

    pub fn num_hashes(&self) -> usize {
        self.num_hashes
    }

    pub fn is_empty(&self) -> bool {
        self.counters.iter().all(|&c| c == 0)
    }

    pub fn total_count(&self) -> u64 {
        self.counters.iter().map(|&c| c as u64).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let bc = BloomCounter::new(64, 3);
        assert_eq!(bc.size(), 64);
        assert_eq!(bc.num_hashes(), 3);
        assert!(bc.is_empty());
    }

    #[test]
    fn test_insert_and_query() {
        let mut bc = BloomCounter::new(256, 3);
        bc.insert("apple");
        assert!(bc.might_contain("apple"));
        assert!(bc.estimate_count("apple") >= 1);
    }

    #[test]
    fn test_multiple_inserts() {
        let mut bc = BloomCounter::new(256, 3);
        bc.insert("key");
        bc.insert("key");
        bc.insert("key");
        assert!(bc.estimate_count("key") >= 3);
    }

    #[test]
    fn test_remove() {
        let mut bc = BloomCounter::new(256, 3);
        bc.insert("item");
        bc.insert("item");
        bc.remove("item");
        assert!(bc.estimate_count("item") >= 1);
    }

    #[test]
    fn test_clear() {
        let mut bc = BloomCounter::new(64, 2);
        bc.insert("test");
        bc.clear();
        assert!(bc.is_empty());
        assert!(!bc.might_contain("test"));
    }

    #[test]
    fn test_total_count() {
        let mut bc = BloomCounter::new(256, 2);
        bc.insert("a");
        assert!(bc.total_count() >= 2);
    }

    #[test]
    fn test_unknown_key() {
        let bc = BloomCounter::new(256, 3);
        assert_eq!(bc.estimate_count("never_inserted"), 0);
    }

    #[test]
    fn test_different_keys() {
        let mut bc = BloomCounter::new(256, 3);
        bc.insert("alpha");
        bc.insert("beta");
        assert!(bc.might_contain("alpha"));
        assert!(bc.might_contain("beta"));
    }

    #[test]
    fn test_remove_to_zero() {
        let mut bc = BloomCounter::new(256, 3);
        bc.insert("once");
        bc.remove("once");
        assert_eq!(bc.estimate_count("once"), 0);
    }

    #[test]
    fn test_saturating_add() {
        let mut bc = BloomCounter::new(1, 1);
        for _ in 0..300 {
            bc.insert("x");
        }
        assert_eq!(bc.counters[0], 255);
    }
}
