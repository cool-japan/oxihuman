// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Map with TTL expiry — entries expire after a configurable number of ticks.

use std::collections::HashMap;

/// An entry in the expiry map.
#[derive(Debug, Clone)]
struct ExpiryEntry<V> {
    value: V,
    expires_at: u64,
}

/// A map where entries expire after a TTL (measured in ticks).
#[derive(Debug, Clone, Default)]
pub struct ExpiryMap<K, V> {
    entries: HashMap<K, ExpiryEntry<V>>,
    now: u64,
}

impl<K: std::hash::Hash + Eq + Clone, V: Clone> ExpiryMap<K, V> {
    /// Create a new expiry map.
    pub fn new() -> Self {
        ExpiryMap { entries: HashMap::new(), now: 0 }
    }

    /// Advance the internal clock by `ticks` and evict expired entries.
    pub fn tick(&mut self, ticks: u64) {
        self.now += ticks;
        self.entries.retain(|_, e| e.expires_at > self.now);
    }

    /// Insert a key-value pair that expires after `ttl` ticks from now.
    pub fn insert(&mut self, key: K, value: V, ttl: u64) {
        self.entries.insert(key, ExpiryEntry { value, expires_at: self.now + ttl });
    }

    /// Get a value, returning `None` if expired or absent.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.entries.get(key).and_then(|e| {
            if e.expires_at > self.now { Some(&e.value) } else { None }
        })
    }

    /// Remove a key explicitly.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.entries.remove(key).map(|e| e.value)
    }

    /// Number of currently live (not-yet-expired) entries.
    pub fn len(&self) -> usize {
        self.entries.values().filter(|e| e.expires_at > self.now).count()
    }

    /// True if there are no live entries.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Current tick counter.
    pub fn now(&self) -> u64 {
        self.now
    }

    /// Force eviction of all expired entries.
    pub fn evict_expired(&mut self) {
        self.entries.retain(|_, e| e.expires_at > self.now);
    }
}

/// Create a new expiry map.
pub fn new_expiry_map<K: std::hash::Hash + Eq + Clone, V: Clone>() -> ExpiryMap<K, V> {
    ExpiryMap::new()
}

/// Insert with TTL.
pub fn em_insert<K: std::hash::Hash + Eq + Clone, V: Clone>(
    map: &mut ExpiryMap<K, V>,
    key: K,
    value: V,
    ttl: u64,
) {
    map.insert(key, value, ttl);
}

/// Get a live value.
pub fn em_get<'a, K: std::hash::Hash + Eq + Clone, V: Clone>(
    map: &'a ExpiryMap<K, V>,
    key: &K,
) -> Option<&'a V> {
    map.get(key)
}

/// Advance the clock.
pub fn em_tick<K: std::hash::Hash + Eq + Clone, V: Clone>(map: &mut ExpiryMap<K, V>, ticks: u64) {
    map.tick(ticks);
}

/// Live entry count.
pub fn em_len<K: std::hash::Hash + Eq + Clone, V: Clone>(map: &ExpiryMap<K, V>) -> usize {
    map.len()
}

/// Remove a key.
pub fn em_remove<K: std::hash::Hash + Eq + Clone, V: Clone>(
    map: &mut ExpiryMap<K, V>,
    key: &K,
) -> Option<V> {
    map.remove(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_insert_get() {
        let mut m = new_expiry_map::<&str, i32>();
        em_insert(&mut m, "k", 99, 10);
        assert_eq!(em_get(&m, &"k"), Some(&99) /* live entry */);
    }

    #[test]
    fn test_expiry_after_tick() {
        let mut m = new_expiry_map::<&str, i32>();
        em_insert(&mut m, "k", 1, 5);
        em_tick(&mut m, 5);
        assert_eq!(em_get(&m, &"k"), None /* expired */);
    }

    #[test]
    fn test_not_expired_before_ttl() {
        let mut m = new_expiry_map::<&str, i32>();
        em_insert(&mut m, "k", 1, 10);
        em_tick(&mut m, 4);
        assert_eq!(em_get(&m, &"k"), Some(&1) /* still alive */);
    }

    #[test]
    fn test_remove() {
        let mut m = new_expiry_map::<&str, i32>();
        em_insert(&mut m, "k", 7, 100);
        assert_eq!(em_remove(&mut m, &"k"), Some(7) /* removed */);
    }

    #[test]
    fn test_len_counts_live() {
        let mut m = new_expiry_map::<i32, i32>();
        em_insert(&mut m, 1, 10, 5);
        em_insert(&mut m, 2, 20, 100);
        em_tick(&mut m, 5);
        assert_eq!(em_len(&m), 1 /* only key 2 alive */);
    }

    #[test]
    fn test_is_empty_after_expiry() {
        let mut m = new_expiry_map::<&str, i32>();
        em_insert(&mut m, "x", 0, 1);
        em_tick(&mut m, 2);
        assert!(m.is_empty() /* all expired */);
    }

    #[test]
    fn test_now() {
        let mut m = new_expiry_map::<&str, i32>();
        em_tick(&mut m, 7);
        assert_eq!(m.now(), 7 /* clock advanced */);
    }

    #[test]
    fn test_evict_expired() {
        let mut m = new_expiry_map::<&str, i32>();
        em_insert(&mut m, "old", 1, 1);
        m.now += 5; /* manual clock advance without eviction */
        m.evict_expired();
        assert_eq!(em_len(&m), 0 /* evicted */);
    }

    #[test]
    fn test_multiple_entries() {
        let mut m = new_expiry_map::<i32, &str>();
        for i in 0..5 {
            em_insert(&mut m, i, "v", 100);
        }
        assert_eq!(em_len(&m), 5 /* five entries */);
    }

    #[test]
    fn test_missing_key() {
        let m = new_expiry_map::<&str, i32>();
        assert_eq!(em_get(&m, &"ghost"), None /* not present */);
    }
}
