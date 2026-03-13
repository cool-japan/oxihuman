// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Query result cache with TTL and hit-rate tracking.

use std::collections::HashMap;

/// A cached query result.
#[derive(Debug, Clone)]
pub struct QueryEntry<V> {
    pub value: V,
    pub ttl: u32,
    pub hits: u32,
}

/// Cache mapping query keys to results.
pub struct QueryCache<V> {
    entries: HashMap<String, QueryEntry<V>>,
    total_hits: u64,
    total_misses: u64,
    default_ttl: u32,
}

#[allow(dead_code)]
impl<V: Clone> QueryCache<V> {
    pub fn new(default_ttl: u32) -> Self {
        QueryCache {
            entries: HashMap::new(),
            total_hits: 0,
            total_misses: 0,
            default_ttl,
        }
    }

    pub fn insert(&mut self, key: &str, value: V) {
        let ttl = self.default_ttl;
        self.entries.insert(
            key.to_string(),
            QueryEntry {
                value,
                ttl,
                hits: 0,
            },
        );
    }

    pub fn insert_with_ttl(&mut self, key: &str, value: V, ttl: u32) {
        self.entries.insert(
            key.to_string(),
            QueryEntry {
                value,
                ttl,
                hits: 0,
            },
        );
    }

    pub fn get(&mut self, key: &str) -> Option<&V> {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.hits += 1;
            self.total_hits += 1;
            Some(&entry.value)
        } else {
            self.total_misses += 1;
            None
        }
    }

    pub fn peek(&self, key: &str) -> Option<&V> {
        self.entries.get(key).map(|e| &e.value)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    pub fn remove(&mut self, key: &str) -> bool {
        self.entries.remove(key).is_some()
    }

    pub fn tick(&mut self) {
        self.entries.retain(|_, e| {
            if e.ttl == 0 {
                false
            } else {
                e.ttl -= 1;
                true
            }
        });
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.total_hits + self.total_misses;
        if total == 0 {
            0.0
        } else {
            self.total_hits as f64 / total as f64
        }
    }

    pub fn total_hits(&self) -> u64 {
        self.total_hits
    }

    pub fn total_misses(&self) -> u64 {
        self.total_misses
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn keys(&self) -> Vec<&str> {
        self.entries.keys().map(|k| k.as_str()).collect()
    }
}

pub fn new_query_cache<V: Clone>(default_ttl: u32) -> QueryCache<V> {
    QueryCache::new(default_ttl)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut c: QueryCache<i32> = new_query_cache(10);
        c.insert("k", 42);
        assert_eq!(*c.get("k").expect("should succeed"), 42);
    }

    #[test]
    fn miss_increments_counter() {
        let mut c: QueryCache<i32> = new_query_cache(10);
        assert!(c.get("none").is_none());
        assert_eq!(c.total_misses(), 1);
    }

    #[test]
    fn hit_rate_calculation() {
        let mut c: QueryCache<i32> = new_query_cache(10);
        c.insert("k", 1);
        c.get("k");
        c.get("k");
        c.get("missing");
        let hr = c.hit_rate();
        assert!((hr - 2.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn ttl_expiry() {
        let mut c: QueryCache<i32> = new_query_cache(1);
        c.insert("k", 1);
        c.tick();
        assert!(c.contains("k"));
        c.tick();
        assert!(!c.contains("k"));
    }

    #[test]
    fn contains_and_remove() {
        let mut c: QueryCache<i32> = new_query_cache(5);
        c.insert("x", 7);
        assert!(c.contains("x"));
        c.remove("x");
        assert!(!c.contains("x"));
    }

    #[test]
    fn clear() {
        let mut c: QueryCache<i32> = new_query_cache(5);
        c.insert("a", 1);
        c.insert("b", 2);
        c.clear();
        assert!(c.is_empty());
    }

    #[test]
    fn peek_no_hit_count() {
        let mut c: QueryCache<i32> = new_query_cache(5);
        c.insert("k", 1);
        c.peek("k");
        assert_eq!(c.total_hits(), 0);
    }

    #[test]
    fn insert_with_custom_ttl() {
        let mut c: QueryCache<i32> = new_query_cache(100);
        c.insert_with_ttl("k", 5, 1);
        c.tick();
        c.tick();
        assert!(!c.contains("k"));
    }

    #[test]
    fn len_tracking() {
        let mut c: QueryCache<i32> = new_query_cache(5);
        assert_eq!(c.len(), 0);
        c.insert("a", 1);
        c.insert("b", 2);
        assert_eq!(c.len(), 2);
    }
}
