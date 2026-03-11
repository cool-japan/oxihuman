// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

/// A map that counts occurrences of string keys.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CounterMap {
    counts: HashMap<String, u64>,
}

#[allow(dead_code)]
impl CounterMap {
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
        }
    }

    pub fn increment(&mut self, key: &str) -> u64 {
        let entry = self.counts.entry(key.to_string()).or_insert(0);
        *entry += 1;
        *entry
    }

    pub fn decrement(&mut self, key: &str) -> u64 {
        let entry = self.counts.entry(key.to_string()).or_insert(0);
        *entry = entry.saturating_sub(1);
        *entry
    }

    pub fn get(&self, key: &str) -> u64 {
        self.counts.get(key).copied().unwrap_or(0)
    }

    pub fn set(&mut self, key: &str, count: u64) {
        self.counts.insert(key.to_string(), count);
    }

    pub fn total(&self) -> u64 {
        self.counts.values().sum()
    }

    pub fn len(&self) -> usize {
        self.counts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.counts.is_empty()
    }

    pub fn max_key(&self) -> Option<&str> {
        self.counts
            .iter()
            .max_by_key(|(_, &v)| v)
            .map(|(k, _)| k.as_str())
    }

    pub fn min_key(&self) -> Option<&str> {
        self.counts
            .iter()
            .min_by_key(|(_, &v)| v)
            .map(|(k, _)| k.as_str())
    }

    pub fn remove(&mut self, key: &str) -> Option<u64> {
        self.counts.remove(key)
    }

    pub fn clear(&mut self) {
        self.counts.clear();
    }

    pub fn keys(&self) -> Vec<&str> {
        self.counts.keys().map(|k| k.as_str()).collect()
    }
}

impl Default for CounterMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let cm = CounterMap::new();
        assert!(cm.is_empty());
    }

    #[test]
    fn test_increment() {
        let mut cm = CounterMap::new();
        assert_eq!(cm.increment("a"), 1);
        assert_eq!(cm.increment("a"), 2);
    }

    #[test]
    fn test_decrement() {
        let mut cm = CounterMap::new();
        cm.set("a", 5);
        assert_eq!(cm.decrement("a"), 4);
    }

    #[test]
    fn test_decrement_floor() {
        let mut cm = CounterMap::new();
        assert_eq!(cm.decrement("x"), 0);
    }

    #[test]
    fn test_get_missing() {
        let cm = CounterMap::new();
        assert_eq!(cm.get("nope"), 0);
    }

    #[test]
    fn test_total() {
        let mut cm = CounterMap::new();
        cm.set("a", 10);
        cm.set("b", 20);
        assert_eq!(cm.total(), 30);
    }

    #[test]
    fn test_max_key() {
        let mut cm = CounterMap::new();
        cm.set("a", 1);
        cm.set("b", 5);
        cm.set("c", 3);
        assert_eq!(cm.max_key(), Some("b"));
    }

    #[test]
    fn test_min_key() {
        let mut cm = CounterMap::new();
        cm.set("a", 10);
        cm.set("b", 2);
        assert_eq!(cm.min_key(), Some("b"));
    }

    #[test]
    fn test_remove() {
        let mut cm = CounterMap::new();
        cm.set("k", 5);
        assert_eq!(cm.remove("k"), Some(5));
        assert!(cm.is_empty());
    }

    #[test]
    fn test_clear() {
        let mut cm = CounterMap::new();
        cm.increment("a");
        cm.increment("b");
        cm.clear();
        assert!(cm.is_empty());
    }
}
