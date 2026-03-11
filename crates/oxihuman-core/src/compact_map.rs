// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A compact ordered map backed by a sorted Vec of key-value pairs.
//! Efficient for small collections where cache locality matters more than O(1) lookup.

/// A sorted-vec backed map with binary search lookup.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CompactMap<K, V> {
    entries: Vec<(K, V)>,
}

#[allow(dead_code)]
impl<K: Ord + Clone, V> CompactMap<K, V> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            entries: Vec::with_capacity(cap),
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        match self.entries.binary_search_by(|e| e.0.cmp(&key)) {
            Ok(idx) => {
                let old = std::mem::replace(&mut self.entries[idx].1, value);
                Some(old)
            }
            Err(idx) => {
                self.entries.insert(idx, (key, value));
                None
            }
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        match self.entries.binary_search_by(|e| e.0.cmp(key)) {
            Ok(idx) => Some(&self.entries[idx].1),
            Err(_) => None,
        }
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        match self.entries.binary_search_by(|e| e.0.cmp(key)) {
            Ok(idx) => Some(&mut self.entries[idx].1),
            Err(_) => None,
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        match self.entries.binary_search_by(|e| e.0.cmp(key)) {
            Ok(idx) => Some(self.entries.remove(idx).1),
            Err(_) => None,
        }
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.entries.binary_search_by(|e| e.0.cmp(key)).is_ok()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.entries.iter().map(|(k, _)| k)
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.entries.iter().map(|(_, v)| v)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.entries.iter().map(|(k, v)| (k, v))
    }

    /// Range query: returns entries where key is in [lo, hi].
    pub fn range(&self, lo: &K, hi: &K) -> Vec<(&K, &V)> {
        let start = match self.entries.binary_search_by(|e| e.0.cmp(lo)) {
            Ok(i) | Err(i) => i,
        };
        let end = match self.entries.binary_search_by(|e| e.0.cmp(hi)) {
            Ok(i) => i + 1,
            Err(i) => i,
        };
        self.entries[start..end]
            .iter()
            .map(|(k, v)| (k, v))
            .collect()
    }
}

impl<K: Ord + Clone, V> Default for CompactMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut m = CompactMap::new();
        m.insert(3, "c");
        m.insert(1, "a");
        m.insert(2, "b");
        assert_eq!(m.get(&1), Some(&"a"));
        assert_eq!(m.get(&2), Some(&"b"));
        assert_eq!(m.get(&3), Some(&"c"));
    }

    #[test]
    fn test_overwrite() {
        let mut m = CompactMap::new();
        assert!(m.insert(1, "old").is_none());
        assert_eq!(m.insert(1, "new"), Some("old"));
        assert_eq!(m.get(&1), Some(&"new"));
    }

    #[test]
    fn test_remove() {
        let mut m = CompactMap::new();
        m.insert(1, 10);
        assert_eq!(m.remove(&1), Some(10));
        assert!(m.is_empty());
    }

    #[test]
    fn test_contains_key() {
        let mut m = CompactMap::new();
        m.insert(5, 50);
        assert!(m.contains_key(&5));
        assert!(!m.contains_key(&6));
    }

    #[test]
    fn test_keys_sorted() {
        let mut m = CompactMap::new();
        m.insert(3, 'c');
        m.insert(1, 'a');
        m.insert(2, 'b');
        let keys: Vec<_> = m.keys().copied().collect();
        assert_eq!(keys, vec![1, 2, 3]);
    }

    #[test]
    fn test_range() {
        let mut m = CompactMap::new();
        for i in 0..10 {
            m.insert(i, i * 10);
        }
        let r: Vec<_> = m.range(&3, &6);
        assert_eq!(r.len(), 4);
        assert_eq!(*r[0].0, 3);
        assert_eq!(*r[3].0, 6);
    }

    #[test]
    fn test_get_mut() {
        let mut m = CompactMap::new();
        m.insert(1, 10);
        if let Some(v) = m.get_mut(&1) {
            *v = 20;
        }
        assert_eq!(m.get(&1), Some(&20));
    }

    #[test]
    fn test_clear() {
        let mut m = CompactMap::new();
        m.insert(1, 1);
        m.clear();
        assert!(m.is_empty());
    }

    #[test]
    fn test_with_capacity() {
        let m: CompactMap<i32, i32> = CompactMap::with_capacity(100);
        assert!(m.is_empty());
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut m: CompactMap<i32, i32> = CompactMap::new();
        assert_eq!(m.remove(&999), None);
    }
}
