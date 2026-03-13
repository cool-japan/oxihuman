// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A compact hash map using open addressing with linear probing.
/// Stores `u64` keys and `u64` values in parallel arrays.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CompactHash {
    keys: Vec<u64>,
    values: Vec<u64>,
    occupied: Vec<bool>,
    count: usize,
}

const EMPTY_KEY: u64 = u64::MAX;

#[allow(dead_code)]
impl CompactHash {
    pub fn new(capacity: usize) -> Self {
        let cap = capacity.max(8).next_power_of_two();
        Self {
            keys: vec![EMPTY_KEY; cap],
            values: vec![0; cap],
            occupied: vec![false; cap],
            count: 0,
        }
    }

    fn hash_index(&self, key: u64) -> usize {
        let h = key.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        (h as usize) & (self.keys.len() - 1)
    }

    pub fn insert(&mut self, key: u64, value: u64) -> bool {
        if self.count * 4 >= self.keys.len() * 3 {
            self.grow();
        }
        let mask = self.keys.len() - 1;
        let mut idx = self.hash_index(key);
        loop {
            if !self.occupied[idx] {
                self.keys[idx] = key;
                self.values[idx] = value;
                self.occupied[idx] = true;
                self.count += 1;
                return true;
            }
            if self.keys[idx] == key {
                self.values[idx] = value;
                return false;
            }
            idx = (idx + 1) & mask;
        }
    }

    pub fn get(&self, key: u64) -> Option<u64> {
        let mask = self.keys.len() - 1;
        let mut idx = self.hash_index(key);
        for _ in 0..self.keys.len() {
            if !self.occupied[idx] {
                return None;
            }
            if self.keys[idx] == key {
                return Some(self.values[idx]);
            }
            idx = (idx + 1) & mask;
        }
        None
    }

    pub fn contains(&self, key: u64) -> bool {
        self.get(key).is_some()
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn capacity(&self) -> usize {
        self.keys.len()
    }

    pub fn load_factor(&self) -> f64 {
        self.count as f64 / self.keys.len() as f64
    }

    fn grow(&mut self) {
        let old_keys = std::mem::take(&mut self.keys);
        let old_values = std::mem::take(&mut self.values);
        let old_occupied = std::mem::take(&mut self.occupied);
        let new_cap = old_keys.len() * 2;
        self.keys = vec![EMPTY_KEY; new_cap];
        self.values = vec![0; new_cap];
        self.occupied = vec![false; new_cap];
        self.count = 0;
        #[allow(clippy::needless_range_loop)]
        for i in 0..old_keys.len() {
            if old_occupied[i] {
                self.insert(old_keys[i], old_values[i]);
            }
        }
    }

    pub fn clear(&mut self) {
        self.keys.fill(EMPTY_KEY);
        self.values.fill(0);
        self.occupied.fill(false);
        self.count = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn keys(&self) -> Vec<u64> {
        self.keys
            .iter()
            .zip(self.occupied.iter())
            .filter(|(_, &occ)| occ)
            .map(|(&k, _)| k)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let ch = CompactHash::new(16);
        assert!(ch.is_empty());
        assert_eq!(ch.capacity(), 16);
    }

    #[test]
    fn test_insert_and_get() {
        let mut ch = CompactHash::new(16);
        assert!(ch.insert(42, 100));
        assert_eq!(ch.get(42), Some(100));
    }

    #[test]
    fn test_update_existing() {
        let mut ch = CompactHash::new(16);
        ch.insert(1, 10);
        assert!(!ch.insert(1, 20));
        assert_eq!(ch.get(1), Some(20));
        assert_eq!(ch.count(), 1);
    }

    #[test]
    fn test_contains() {
        let mut ch = CompactHash::new(16);
        ch.insert(5, 50);
        assert!(ch.contains(5));
        assert!(!ch.contains(6));
    }

    #[test]
    fn test_grow() {
        let mut ch = CompactHash::new(8);
        for i in 0..20 {
            ch.insert(i, i * 10);
        }
        assert_eq!(ch.count(), 20);
        for i in 0..20 {
            assert_eq!(ch.get(i), Some(i * 10));
        }
    }

    #[test]
    fn test_clear() {
        let mut ch = CompactHash::new(16);
        ch.insert(1, 2);
        ch.clear();
        assert!(ch.is_empty());
        assert!(ch.get(1).is_none());
    }

    #[test]
    fn test_load_factor() {
        let mut ch = CompactHash::new(8);
        ch.insert(1, 1);
        ch.insert(2, 2);
        let lf = ch.load_factor();
        assert!(lf > 0.0 && lf < 1.0);
    }

    #[test]
    fn test_keys() {
        let mut ch = CompactHash::new(16);
        ch.insert(10, 1);
        ch.insert(20, 2);
        let mut keys = ch.keys();
        keys.sort();
        assert_eq!(keys, vec![10, 20]);
    }

    #[test]
    fn test_missing_key() {
        let ch = CompactHash::new(16);
        assert!(ch.get(999).is_none());
    }

    #[test]
    fn test_count() {
        let mut ch = CompactHash::new(16);
        ch.insert(1, 1);
        ch.insert(2, 2);
        ch.insert(3, 3);
        assert_eq!(ch.count(), 3);
    }
}
