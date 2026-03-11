// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::collections::VecDeque;

/// FIFO (first-in, first-out) cache with fixed capacity.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FifoCache<T> {
    items: VecDeque<(String, T)>,
    capacity: usize,
}

#[allow(dead_code)]
impl<T: Clone> FifoCache<T> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0);
        Self {
            items: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn insert(&mut self, key: &str, value: T) {
        // Remove existing entry with same key
        self.items.retain(|(k, _)| k != key);
        if self.items.len() >= self.capacity {
            self.items.pop_front();
        }
        self.items.push_back((key.to_string(), value));
    }

    pub fn get(&self, key: &str) -> Option<&T> {
        self.items
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.items.iter().any(|(k, _)| k == key)
    }

    pub fn remove(&mut self, key: &str) -> Option<T> {
        let pos = self.items.iter().position(|(k, _)| k == key)?;
        self.items.remove(pos).map(|(_, v)| v)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn is_full(&self) -> bool {
        self.items.len() >= self.capacity
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn oldest_key(&self) -> Option<&str> {
        self.items.front().map(|(k, _)| k.as_str())
    }

    pub fn newest_key(&self) -> Option<&str> {
        self.items.back().map(|(k, _)| k.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let c: FifoCache<i32> = FifoCache::new(5);
        assert!(c.is_empty());
        assert_eq!(c.capacity(), 5);
    }

    #[test]
    fn test_insert_get() {
        let mut c = FifoCache::new(3);
        c.insert("a", 1);
        assert_eq!(c.get("a"), Some(&1));
    }

    #[test]
    fn test_eviction() {
        let mut c = FifoCache::new(2);
        c.insert("a", 1);
        c.insert("b", 2);
        c.insert("c", 3);
        assert!(!c.contains("a"));
        assert!(c.contains("b"));
        assert!(c.contains("c"));
    }

    #[test]
    fn test_remove() {
        let mut c = FifoCache::new(3);
        c.insert("x", 10);
        assert_eq!(c.remove("x"), Some(10));
        assert!(!c.contains("x"));
    }

    #[test]
    fn test_is_full() {
        let mut c = FifoCache::new(2);
        c.insert("a", 1);
        assert!(!c.is_full());
        c.insert("b", 2);
        assert!(c.is_full());
    }

    #[test]
    fn test_clear() {
        let mut c = FifoCache::new(3);
        c.insert("a", 1);
        c.clear();
        assert!(c.is_empty());
    }

    #[test]
    fn test_oldest_newest() {
        let mut c = FifoCache::new(5);
        c.insert("first", 1);
        c.insert("second", 2);
        assert_eq!(c.oldest_key(), Some("first"));
        assert_eq!(c.newest_key(), Some("second"));
    }

    #[test]
    fn test_duplicate_key() {
        let mut c = FifoCache::new(3);
        c.insert("k", 1);
        c.insert("k", 2);
        assert_eq!(c.get("k"), Some(&2));
        assert_eq!(c.len(), 1);
    }

    #[test]
    fn test_remove_missing() {
        let mut c: FifoCache<i32> = FifoCache::new(3);
        assert!(c.remove("nope").is_none());
    }

    #[test]
    fn test_len() {
        let mut c = FifoCache::new(5);
        c.insert("a", 1);
        c.insert("b", 2);
        assert_eq!(c.len(), 2);
    }
}
