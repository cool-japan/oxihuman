// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

use std::collections::HashMap;
use std::hash::Hash;

/// A multi-map: one key can have multiple values.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct MultiMap<K: Eq + Hash + Clone, V: Clone> {
    pub map: HashMap<K, Vec<V>>,
}

/// Create a new empty `MultiMap<String, String>`.
#[allow(dead_code)]
pub fn new_multimap() -> MultiMap<String, String> {
    MultiMap {
        map: HashMap::new(),
    }
}

/// Insert a key-value pair into the multi-map.
#[allow(dead_code)]
pub fn mm_insert(mm: &mut MultiMap<String, String>, key: &str, val: &str) {
    mm.map
        .entry(key.to_string())
        .or_default()
        .push(val.to_string());
}

/// Get all values for a key.
#[allow(dead_code)]
pub fn mm_get<'a>(mm: &'a MultiMap<String, String>, key: &str) -> Vec<&'a str> {
    mm.map
        .get(key)
        .map(|v| v.iter().map(|s| s.as_str()).collect())
        .unwrap_or_default()
}

/// Remove all values for a key.
#[allow(dead_code)]
pub fn mm_remove_key(mm: &mut MultiMap<String, String>, key: &str) {
    mm.map.remove(key);
}

/// Get the number of distinct keys.
#[allow(dead_code)]
pub fn mm_key_count(mm: &MultiMap<String, String>) -> usize {
    mm.map.len()
}

/// Get the total number of values across all keys.
#[allow(dead_code)]
pub fn mm_total_values(mm: &MultiMap<String, String>) -> usize {
    mm.map.values().map(|v| v.len()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut mm = new_multimap();
        mm_insert(&mut mm, "key", "val1");
        mm_insert(&mut mm, "key", "val2");
        let vals = mm_get(&mm, "key");
        assert_eq!(vals.len(), 2);
    }

    #[test]
    fn get_missing_key_empty() {
        let mm = new_multimap();
        assert!(mm_get(&mm, "missing").is_empty());
    }

    #[test]
    fn key_count() {
        let mut mm = new_multimap();
        mm_insert(&mut mm, "a", "1");
        mm_insert(&mut mm, "b", "2");
        mm_insert(&mut mm, "a", "3");
        assert_eq!(mm_key_count(&mm), 2);
    }

    #[test]
    fn total_values() {
        let mut mm = new_multimap();
        mm_insert(&mut mm, "a", "1");
        mm_insert(&mut mm, "a", "2");
        mm_insert(&mut mm, "b", "3");
        assert_eq!(mm_total_values(&mm), 3);
    }

    #[test]
    fn remove_key() {
        let mut mm = new_multimap();
        mm_insert(&mut mm, "x", "val");
        mm_remove_key(&mut mm, "x");
        assert_eq!(mm_key_count(&mm), 0);
    }

    #[test]
    fn remove_nonexistent_key_is_ok() {
        let mut mm = new_multimap();
        mm_remove_key(&mut mm, "nonexistent");
        assert_eq!(mm_key_count(&mm), 0);
    }

    #[test]
    fn values_order_preserved() {
        let mut mm = new_multimap();
        mm_insert(&mut mm, "k", "first");
        mm_insert(&mut mm, "k", "second");
        mm_insert(&mut mm, "k", "third");
        let vals = mm_get(&mm, "k");
        assert_eq!(vals[0], "first");
        assert_eq!(vals[2], "third");
    }

    #[test]
    fn multiple_keys_independent() {
        let mut mm = new_multimap();
        mm_insert(&mut mm, "a", "alpha");
        mm_insert(&mut mm, "b", "beta");
        assert_eq!(mm_get(&mm, "a"), vec!["alpha"]);
        assert_eq!(mm_get(&mm, "b"), vec!["beta"]);
    }

    #[test]
    fn empty_multimap() {
        let mm = new_multimap();
        assert_eq!(mm_key_count(&mm), 0);
        assert_eq!(mm_total_values(&mm), 0);
    }

    #[test]
    fn total_values_after_remove() {
        let mut mm = new_multimap();
        mm_insert(&mut mm, "a", "1");
        mm_insert(&mut mm, "a", "2");
        mm_insert(&mut mm, "b", "3");
        mm_remove_key(&mut mm, "a");
        assert_eq!(mm_total_values(&mm), 1);
    }
}
