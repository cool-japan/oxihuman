// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A simple cuckoo-hash-inspired map with two hash tables.
/// Uses deterministic hashing for reproducibility.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CuckooMap {
    table_a: Vec<Option<(u64, i64)>>,
    table_b: Vec<Option<(u64, i64)>>,
    count: usize,
}

#[allow(dead_code)]
fn hash_a(key: u64, cap: usize) -> usize {
    (key.wrapping_mul(2654435761) >> 16) as usize % cap
}

#[allow(dead_code)]
fn hash_b(key: u64, cap: usize) -> usize {
    (key.wrapping_mul(40503) ^ (key >> 8)) as usize % cap
}

#[allow(dead_code)]
impl CuckooMap {
    pub fn new(capacity: usize) -> Self {
        let cap = capacity.max(4);
        Self {
            table_a: vec![None; cap],
            table_b: vec![None; cap],
            count: 0,
        }
    }

    pub fn get(&self, key: u64) -> Option<i64> {
        let cap = self.table_a.len();
        let ia = hash_a(key, cap);
        if let Some((k, v)) = self.table_a[ia] {
            if k == key {
                return Some(v);
            }
        }
        let ib = hash_b(key, cap);
        if let Some((k, v)) = self.table_b[ib] {
            if k == key {
                return Some(v);
            }
        }
        None
    }

    pub fn insert(&mut self, key: u64, value: i64) -> bool {
        let cap = self.table_a.len();
        // Check if already present
        let ia = hash_a(key, cap);
        if self.table_a[ia].is_some_and(|(k, _)| k == key) {
            self.table_a[ia] = Some((key, value));
            return true;
        }
        let ib = hash_b(key, cap);
        if self.table_b[ib].is_some_and(|(k, _)| k == key) {
            self.table_b[ib] = Some((key, value));
            return true;
        }
        // Insert into table_a if empty
        if self.table_a[ia].is_none() {
            self.table_a[ia] = Some((key, value));
            self.count += 1;
            return true;
        }
        // Insert into table_b if empty
        if self.table_b[ib].is_none() {
            self.table_b[ib] = Some((key, value));
            self.count += 1;
            return true;
        }
        // Evict from table_a and relocate
        let Some(evicted) = self.table_a[ia].take() else { return false };
        self.table_a[ia] = Some((key, value));
        let eb = hash_b(evicted.0, cap);
        if self.table_b[eb].is_none() {
            self.table_b[eb] = Some(evicted);
            self.count += 1;
            return true;
        }
        // Give up for simplicity (real impl would retry or grow)
        self.table_b[eb] = Some(evicted);
        false
    }

    pub fn remove(&mut self, key: u64) -> Option<i64> {
        let cap = self.table_a.len();
        let ia = hash_a(key, cap);
        if self.table_a[ia].is_some_and(|(k, _)| k == key) {
            let v = self.table_a[ia].take().map(|(_, v)| v).unwrap_or_default();
            self.count -= 1;
            return Some(v);
        }
        let ib = hash_b(key, cap);
        if self.table_b[ib].is_some_and(|(k, _)| k == key) {
            let v = self.table_b[ib].take().map(|(_, v)| v).unwrap_or_default();
            self.count -= 1;
            return Some(v);
        }
        None
    }

    pub fn contains_key(&self, key: u64) -> bool {
        self.get(key).is_some()
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn capacity(&self) -> usize {
        self.table_a.len() * 2
    }

    pub fn clear(&mut self) {
        self.table_a.iter_mut().for_each(|s| *s = None);
        self.table_b.iter_mut().for_each(|s| *s = None);
        self.count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let m = CuckooMap::new(8);
        assert!(m.is_empty());
    }

    #[test]
    fn test_insert_get() {
        let mut m = CuckooMap::new(16);
        m.insert(42, 100);
        assert_eq!(m.get(42), Some(100));
    }

    #[test]
    fn test_insert_overwrite() {
        let mut m = CuckooMap::new(16);
        m.insert(1, 10);
        m.insert(1, 20);
        assert_eq!(m.get(1), Some(20));
    }

    #[test]
    fn test_remove() {
        let mut m = CuckooMap::new(16);
        m.insert(5, 50);
        assert_eq!(m.remove(5), Some(50));
        assert!(m.get(5).is_none());
    }

    #[test]
    fn test_contains_key() {
        let mut m = CuckooMap::new(16);
        m.insert(7, 70);
        assert!(m.contains_key(7));
        assert!(!m.contains_key(8));
    }

    #[test]
    fn test_len() {
        let mut m = CuckooMap::new(16);
        m.insert(1, 1);
        m.insert(2, 2);
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn test_clear() {
        let mut m = CuckooMap::new(16);
        m.insert(1, 1);
        m.clear();
        assert!(m.is_empty());
    }

    #[test]
    fn test_get_missing() {
        let m = CuckooMap::new(16);
        assert!(m.get(999).is_none());
    }

    #[test]
    fn test_remove_missing() {
        let mut m = CuckooMap::new(16);
        assert!(m.remove(999).is_none());
    }

    #[test]
    fn test_capacity() {
        let m = CuckooMap::new(8);
        assert!(m.capacity() >= 8);
    }
}
