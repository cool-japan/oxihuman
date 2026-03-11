// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

/// A named bundle of binary data blobs, for grouping related assets.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DataBundle {
    name: String,
    entries: HashMap<String, Vec<u8>>,
}

#[allow(dead_code)]
impl DataBundle {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            entries: HashMap::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn insert(&mut self, key: &str, data: Vec<u8>) {
        self.entries.insert(key.to_string(), data);
    }

    pub fn get(&self, key: &str) -> Option<&[u8]> {
        self.entries.get(key).map(|v| v.as_slice())
    }

    pub fn remove(&mut self, key: &str) -> Option<Vec<u8>> {
        self.entries.remove(key)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    pub fn keys(&self) -> Vec<&str> {
        self.entries.keys().map(|k| k.as_str()).collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn total_bytes(&self) -> usize {
        self.entries.values().map(|v| v.len()).sum()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn merge(&mut self, other: &DataBundle) {
        for (k, v) in &other.entries {
            self.entries.insert(k.clone(), v.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let b = DataBundle::new("test");
        assert_eq!(b.name(), "test");
        assert!(b.is_empty());
    }

    #[test]
    fn test_insert_get() {
        let mut b = DataBundle::new("b");
        b.insert("k1", vec![1, 2, 3]);
        assert_eq!(b.get("k1"), Some(&[1u8, 2, 3][..]));
    }

    #[test]
    fn test_remove() {
        let mut b = DataBundle::new("b");
        b.insert("k1", vec![1]);
        let removed = b.remove("k1");
        assert_eq!(removed, Some(vec![1]));
        assert!(!b.contains("k1"));
    }

    #[test]
    fn test_contains() {
        let mut b = DataBundle::new("b");
        b.insert("key", vec![]);
        assert!(b.contains("key"));
        assert!(!b.contains("nope"));
    }

    #[test]
    fn test_len() {
        let mut b = DataBundle::new("b");
        b.insert("a", vec![1]);
        b.insert("b", vec![2]);
        assert_eq!(b.len(), 2);
    }

    #[test]
    fn test_total_bytes() {
        let mut b = DataBundle::new("b");
        b.insert("a", vec![1, 2]);
        b.insert("b", vec![3, 4, 5]);
        assert_eq!(b.total_bytes(), 5);
    }

    #[test]
    fn test_clear() {
        let mut b = DataBundle::new("b");
        b.insert("x", vec![1]);
        b.clear();
        assert!(b.is_empty());
    }

    #[test]
    fn test_merge() {
        let mut a = DataBundle::new("a");
        a.insert("k1", vec![1]);
        let mut b = DataBundle::new("b");
        b.insert("k2", vec![2]);
        a.merge(&b);
        assert!(a.contains("k1"));
        assert!(a.contains("k2"));
    }

    #[test]
    fn test_keys() {
        let mut b = DataBundle::new("b");
        b.insert("alpha", vec![]);
        b.insert("beta", vec![]);
        let keys = b.keys();
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn test_get_missing() {
        let b = DataBundle::new("b");
        assert!(b.get("missing").is_none());
    }
}
