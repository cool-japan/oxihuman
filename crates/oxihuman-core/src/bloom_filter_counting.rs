// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Counting bloom filter supporting increment and decrement operations.

/// Counting bloom filter with 4-bit counters per cell.
pub struct CountingBloomFilter {
    counters: Vec<u8>,
    width: usize,
    hash_count: usize,
    items: usize,
}

impl CountingBloomFilter {
    /// Create a new counting bloom filter with `width` counter cells and `hash_count` hash functions.
    pub fn new(width: usize, hash_count: usize) -> Self {
        CountingBloomFilter {
            counters: vec![0u8; width],
            width,
            hash_count: hash_count.max(1),
            items: 0,
        }
    }

    fn hashes(&self, item: &[u8]) -> Vec<usize> {
        let mut result = Vec::with_capacity(self.hash_count);
        let mut h: u64 = 14695981039346656037;
        for &b in item {
            h ^= b as u64;
            h = h.wrapping_mul(1099511628211);
        }
        for i in 0..self.hash_count {
            let idx = (h.wrapping_add(i as u64).wrapping_mul(2654435761)) as usize % self.width;
            result.push(idx);
        }
        result
    }

    /// Insert an element (increments its counters).
    pub fn insert(&mut self, item: &[u8]) {
        for idx in self.hashes(item) {
            self.counters[idx] = self.counters[idx].saturating_add(1);
        }
        self.items += 1;
    }

    /// Remove an element (decrements its counters). No-op if counter would go negative.
    pub fn remove(&mut self, item: &[u8]) {
        let hashes = self.hashes(item);
        /* Only decrement if all counters > 0 (item was probably inserted) */
        if hashes.iter().all(|&i| self.counters[i] > 0) {
            for idx in hashes {
                self.counters[idx] -= 1;
            }
            if self.items > 0 {
                self.items -= 1;
            }
        }
    }

    /// Query whether an element is possibly in the filter.
    pub fn contains(&self, item: &[u8]) -> bool {
        self.hashes(item).iter().all(|&i| self.counters[i] > 0)
    }

    /// Return an estimated count for the item (minimum counter value).
    pub fn estimate_count(&self, item: &[u8]) -> u8 {
        self.hashes(item)
            .iter()
            .map(|&i| self.counters[i])
            .min()
            .unwrap_or(0)
    }

    /// Number of items inserted (net after removals).
    pub fn len(&self) -> usize {
        self.items
    }

    /// True if empty.
    pub fn is_empty(&self) -> bool {
        self.items == 0
    }

    /// Clear all counters.
    pub fn clear(&mut self) {
        self.counters.fill(0);
        self.items = 0;
    }

    /// Return total counter sum (useful for diagnostics).
    pub fn total_count(&self) -> u64 {
        self.counters.iter().map(|&c| c as u64).sum()
    }

    /// Return the number of non-zero cells.
    pub fn occupied_cells(&self) -> usize {
        self.counters.iter().filter(|&&c| c > 0).count()
    }

    /// Return load factor (fraction of non-zero cells).
    pub fn load_factor(&self) -> f64 {
        self.occupied_cells() as f64 / self.width as f64
    }

    /// Return the number of hash functions used.
    pub fn hash_count(&self) -> usize {
        self.hash_count
    }

    /// Return the filter width.
    pub fn width(&self) -> usize {
        self.width
    }
}

/// Create a new counting bloom filter with reasonable defaults.
pub fn new_counting_bloom_filter(capacity: usize) -> CountingBloomFilter {
    let width = (capacity * 10).max(64);
    CountingBloomFilter::new(width, 3)
}

/// Insert a string key.
pub fn cbf_insert_str(cbf: &mut CountingBloomFilter, key: &str) {
    cbf.insert(key.as_bytes());
}

/// Remove a string key.
pub fn cbf_remove_str(cbf: &mut CountingBloomFilter, key: &str) {
    cbf.remove(key.as_bytes());
}

/// Check membership for a string key.
pub fn cbf_contains_str(cbf: &CountingBloomFilter, key: &str) -> bool {
    cbf.contains(key.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_contains() {
        let mut cbf = new_counting_bloom_filter(100);
        cbf_insert_str(&mut cbf, "hello");
        assert!(cbf_contains_str(&cbf, "hello"));
    }

    #[test]
    fn test_remove() {
        let mut cbf = new_counting_bloom_filter(100);
        cbf_insert_str(&mut cbf, "world");
        assert!(cbf_contains_str(&cbf, "world"));
        cbf_remove_str(&mut cbf, "world");
        assert!(!cbf_contains_str(&cbf, "world"));
    }

    #[test]
    fn test_len_and_empty() {
        let mut cbf = new_counting_bloom_filter(50);
        assert!(cbf.is_empty());
        cbf_insert_str(&mut cbf, "a");
        assert_eq!(cbf.len(), 1);
        assert!(!cbf.is_empty());
    }

    #[test]
    fn test_clear() {
        let mut cbf = new_counting_bloom_filter(50);
        cbf_insert_str(&mut cbf, "a");
        cbf_insert_str(&mut cbf, "b");
        cbf.clear();
        assert!(cbf.is_empty());
        assert_eq!(cbf.total_count(), 0);
    }

    #[test]
    fn test_estimate_count() {
        let mut cbf = CountingBloomFilter::new(256, 3);
        cbf.insert(b"key");
        cbf.insert(b"key");
        let est = cbf.estimate_count(b"key");
        assert!(est >= 1);
    }

    #[test]
    fn test_width_and_hash_count() {
        let cbf = CountingBloomFilter::new(128, 4);
        assert_eq!(cbf.width(), 128);
        assert_eq!(cbf.hash_count(), 4);
    }

    #[test]
    fn test_load_factor() {
        let mut cbf = new_counting_bloom_filter(100);
        cbf_insert_str(&mut cbf, "test");
        assert!(cbf.load_factor() > 0.0);
        assert!(cbf.load_factor() <= 1.0);
    }

    #[test]
    fn test_no_false_negative() {
        /* Items definitely inserted must be found */
        let mut cbf = new_counting_bloom_filter(200);
        let keys = ["alpha", "beta", "gamma", "delta", "epsilon"];
        for k in &keys {
            cbf_insert_str(&mut cbf, k);
        }
        for k in &keys {
            assert!(cbf_contains_str(&cbf, k), "false negative for {k}");
        }
    }
}
