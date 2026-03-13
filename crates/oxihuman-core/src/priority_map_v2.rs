// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Priority-keyed hash map: each key carries a numeric priority.
//! Higher priority values are considered more important.

use std::collections::HashMap;

/// An entry in the priority map.
#[derive(Debug, Clone)]
pub struct PriorityEntry<V> {
    pub value: V,
    pub priority: i64,
}

/// A map where every key-value pair has an associated priority.
#[derive(Debug, Clone, Default)]
pub struct PriorityMapV2<K, V> {
    inner: HashMap<K, PriorityEntry<V>>,
}

impl<K: std::hash::Hash + Eq + Clone, V: Clone> PriorityMapV2<K, V> {
    /// Create a new empty priority map.
    pub fn new() -> Self {
        PriorityMapV2 { inner: HashMap::new() }
    }

    /// Insert or update a key with a value and priority.
    pub fn insert(&mut self, key: K, value: V, priority: i64) {
        self.inner.insert(key, PriorityEntry { value, priority });
    }

    /// Get the value associated with `key`, if present.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.inner.get(key).map(|e| &e.value)
    }

    /// Get the priority of `key`, if present.
    pub fn priority_of(&self, key: &K) -> Option<i64> {
        self.inner.get(key).map(|e| e.priority)
    }

    /// Remove a key and return its entry, if present.
    pub fn remove(&mut self, key: &K) -> Option<PriorityEntry<V>> {
        self.inner.remove(key)
    }

    /// Return the key with the highest priority, if any.
    pub fn highest_priority_key(&self) -> Option<&K> {
        self.inner
            .iter()
            .max_by_key(|(_, e)| e.priority)
            .map(|(k, _)| k)
    }

    /// Return the key with the lowest priority, if any.
    pub fn lowest_priority_key(&self) -> Option<&K> {
        self.inner
            .iter()
            .min_by_key(|(_, e)| e.priority)
            .map(|(k, _)| k)
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// True if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Drain all entries into a sorted Vec (highest priority first).
    pub fn drain_sorted(&mut self) -> Vec<(K, V, i64)> {
        let mut out: Vec<(K, V, i64)> = self
            .inner
            .drain()
            .map(|(k, e)| (k, e.value, e.priority))
            .collect();
        out.sort_by(|a, b| b.2.cmp(&a.2));
        out
    }
}

/// Create a new priority map.
pub fn new_priority_map_v2<K: std::hash::Hash + Eq + Clone, V: Clone>() -> PriorityMapV2<K, V> {
    PriorityMapV2::new()
}

/// Insert into a priority map.
pub fn pm_insert<K: std::hash::Hash + Eq + Clone, V: Clone>(
    map: &mut PriorityMapV2<K, V>,
    key: K,
    value: V,
    priority: i64,
) {
    map.insert(key, value, priority);
}

/// Get from a priority map.
pub fn pm_get<'a, K: std::hash::Hash + Eq + Clone, V: Clone>(
    map: &'a PriorityMapV2<K, V>,
    key: &K,
) -> Option<&'a V> {
    map.get(key)
}

/// Get the size of a priority map.
pub fn pm_len<K: std::hash::Hash + Eq + Clone, V: Clone>(map: &PriorityMapV2<K, V>) -> usize {
    map.len()
}

/// Remove from a priority map.
pub fn pm_remove<K: std::hash::Hash + Eq + Clone, V: Clone>(
    map: &mut PriorityMapV2<K, V>,
    key: &K,
) -> Option<PriorityEntry<V>> {
    map.remove(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut m = new_priority_map_v2::<&str, i32>();
        pm_insert(&mut m, "a", 10, 5);
        assert_eq!(pm_get(&m, &"a"), Some(&10) /* inserted value */);
    }

    #[test]
    fn test_priority_of() {
        let mut m = new_priority_map_v2::<&str, i32>();
        m.insert("x", 99, 42);
        assert_eq!(m.priority_of(&"x"), Some(42) /* priority stored */);
    }

    #[test]
    fn test_highest_priority() {
        let mut m = new_priority_map_v2::<&str, i32>();
        m.insert("low", 1, 1);
        m.insert("high", 2, 100);
        m.insert("mid", 3, 50);
        assert_eq!(m.highest_priority_key(), Some(&"high") /* max priority */);
    }

    #[test]
    fn test_lowest_priority() {
        let mut m = new_priority_map_v2::<&str, i32>();
        m.insert("low", 1, 1);
        m.insert("high", 2, 100);
        assert_eq!(m.lowest_priority_key(), Some(&"low") /* min priority */);
    }

    #[test]
    fn test_remove() {
        let mut m = new_priority_map_v2::<&str, i32>();
        m.insert("a", 7, 10);
        let e = pm_remove(&mut m, &"a").expect("should succeed");
        assert_eq!(e.value, 7 /* removed value */);
        assert!(m.is_empty());
    }

    #[test]
    fn test_len() {
        let mut m = new_priority_map_v2::<i32, &str>();
        m.insert(1, "one", 1);
        m.insert(2, "two", 2);
        assert_eq!(pm_len(&m), 2 /* two entries */);
    }

    #[test]
    fn test_drain_sorted() {
        let mut m = new_priority_map_v2::<&str, i32>();
        m.insert("c", 3, 30);
        m.insert("a", 1, 10);
        m.insert("b", 2, 20);
        let sorted = m.drain_sorted();
        assert_eq!(sorted[0].2, 30 /* highest first */);
        assert_eq!(sorted[2].2, 10 /* lowest last */);
    }

    #[test]
    fn test_empty_map() {
        let m = new_priority_map_v2::<&str, i32>();
        assert!(m.is_empty() /* starts empty */);
        assert_eq!(m.highest_priority_key(), None);
    }

    #[test]
    fn test_overwrite() {
        let mut m = new_priority_map_v2::<&str, i32>();
        m.insert("k", 1, 5);
        m.insert("k", 2, 50);
        assert_eq!(m.get(&"k"), Some(&2) /* overwritten */);
    }

    #[test]
    fn test_missing_key() {
        let m = new_priority_map_v2::<&str, i32>();
        assert_eq!(pm_get(&m, &"missing"), None /* not found */);
    }
}
