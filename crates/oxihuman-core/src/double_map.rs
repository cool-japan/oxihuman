// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

/// A bidirectional map between two string key spaces.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DoubleMap {
    forward: HashMap<String, String>,
    backward: HashMap<String, String>,
}

impl Default for DoubleMap {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl DoubleMap {
    pub fn new() -> Self {
        Self {
            forward: HashMap::new(),
            backward: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &str, value: &str) -> bool {
        if self.forward.contains_key(key) || self.backward.contains_key(value) {
            return false;
        }
        self.forward.insert(key.to_string(), value.to_string());
        self.backward.insert(value.to_string(), key.to_string());
        true
    }

    pub fn get_forward(&self, key: &str) -> Option<&str> {
        self.forward.get(key).map(|v| v.as_str())
    }

    pub fn get_backward(&self, value: &str) -> Option<&str> {
        self.backward.get(value).map(|v| v.as_str())
    }

    pub fn remove_by_key(&mut self, key: &str) -> bool {
        if let Some(value) = self.forward.remove(key) {
            self.backward.remove(&value);
            true
        } else {
            false
        }
    }

    pub fn remove_by_value(&mut self, value: &str) -> bool {
        if let Some(key) = self.backward.remove(value) {
            self.forward.remove(&key);
            true
        } else {
            false
        }
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.forward.contains_key(key)
    }

    pub fn contains_value(&self, value: &str) -> bool {
        self.backward.contains_key(value)
    }

    pub fn count(&self) -> usize {
        self.forward.len()
    }

    pub fn is_empty(&self) -> bool {
        self.forward.is_empty()
    }

    pub fn keys(&self) -> Vec<&str> {
        self.forward.keys().map(|k| k.as_str()).collect()
    }

    pub fn values(&self) -> Vec<&str> {
        self.backward.keys().map(|k| k.as_str()).collect()
    }

    pub fn clear(&mut self) {
        self.forward.clear();
        self.backward.clear();
    }

    pub fn to_pairs(&self) -> Vec<(&str, &str)> {
        self.forward
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let dm = DoubleMap::new();
        assert!(dm.is_empty());
    }

    #[test]
    fn test_insert_and_get() {
        let mut dm = DoubleMap::new();
        assert!(dm.insert("a", "1"));
        assert_eq!(dm.get_forward("a"), Some("1"));
        assert_eq!(dm.get_backward("1"), Some("a"));
    }

    #[test]
    fn test_duplicate_key() {
        let mut dm = DoubleMap::new();
        dm.insert("a", "1");
        assert!(!dm.insert("a", "2"));
    }

    #[test]
    fn test_duplicate_value() {
        let mut dm = DoubleMap::new();
        dm.insert("a", "1");
        assert!(!dm.insert("b", "1"));
    }

    #[test]
    fn test_remove_by_key() {
        let mut dm = DoubleMap::new();
        dm.insert("a", "1");
        assert!(dm.remove_by_key("a"));
        assert!(!dm.contains_key("a"));
        assert!(!dm.contains_value("1"));
    }

    #[test]
    fn test_remove_by_value() {
        let mut dm = DoubleMap::new();
        dm.insert("a", "1");
        assert!(dm.remove_by_value("1"));
        assert!(dm.is_empty());
    }

    #[test]
    fn test_contains() {
        let mut dm = DoubleMap::new();
        dm.insert("x", "y");
        assert!(dm.contains_key("x"));
        assert!(dm.contains_value("y"));
        assert!(!dm.contains_key("y"));
    }

    #[test]
    fn test_count() {
        let mut dm = DoubleMap::new();
        dm.insert("a", "1");
        dm.insert("b", "2");
        assert_eq!(dm.count(), 2);
    }

    #[test]
    fn test_clear() {
        let mut dm = DoubleMap::new();
        dm.insert("a", "1");
        dm.clear();
        assert!(dm.is_empty());
    }

    #[test]
    fn test_to_pairs() {
        let mut dm = DoubleMap::new();
        dm.insert("a", "1");
        let pairs = dm.to_pairs();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], ("a", "1"));
    }
}
