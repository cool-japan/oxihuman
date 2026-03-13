// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Linked map: insertion-ordered key→value map using a Vec + HashMap.

use std::collections::HashMap;

/// Insertion-ordered map.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LinkedMap<V> {
    keys: Vec<String>,
    map: HashMap<String, V>,
}

/// Create a new LinkedMap.
#[allow(dead_code)]
pub fn new_linked_map<V>() -> LinkedMap<V> {
    LinkedMap {
        keys: Vec::new(),
        map: HashMap::new(),
    }
}

/// Insert or update a key; preserves insertion order for new keys.
#[allow(dead_code)]
pub fn lmap_insert<V>(lm: &mut LinkedMap<V>, key: &str, val: V) {
    if !lm.map.contains_key(key) {
        lm.keys.push(key.to_string());
    }
    lm.map.insert(key.to_string(), val);
}

/// Get a reference by key.
#[allow(dead_code)]
pub fn lmap_get<'a, V>(lm: &'a LinkedMap<V>, key: &str) -> Option<&'a V> {
    lm.map.get(key)
}

/// Get a mutable reference by key.
#[allow(dead_code)]
pub fn lmap_get_mut<'a, V>(lm: &'a mut LinkedMap<V>, key: &str) -> Option<&'a mut V> {
    lm.map.get_mut(key)
}

/// Remove a key; returns value if present.
#[allow(dead_code)]
pub fn lmap_remove<V>(lm: &mut LinkedMap<V>, key: &str) -> Option<V> {
    if let Some(val) = lm.map.remove(key) {
        lm.keys.retain(|k| k != key);
        Some(val)
    } else {
        None
    }
}

/// Whether key is present.
#[allow(dead_code)]
pub fn lmap_contains<V>(lm: &LinkedMap<V>, key: &str) -> bool {
    lm.map.contains_key(key)
}

/// Number of entries.
#[allow(dead_code)]
pub fn lmap_len<V>(lm: &LinkedMap<V>) -> usize {
    lm.map.len()
}

/// Whether empty.
#[allow(dead_code)]
pub fn lmap_is_empty<V>(lm: &LinkedMap<V>) -> bool {
    lm.map.is_empty()
}

/// Ordered keys.
#[allow(dead_code)]
pub fn lmap_keys<V>(lm: &LinkedMap<V>) -> &[String] {
    &lm.keys
}

/// Ordered values.
#[allow(dead_code)]
pub fn lmap_values<V>(lm: &LinkedMap<V>) -> Vec<&V> {
    lm.keys.iter().filter_map(|k| lm.map.get(k)).collect()
}

/// Clear all entries.
#[allow(dead_code)]
pub fn lmap_clear<V>(lm: &mut LinkedMap<V>) {
    lm.keys.clear();
    lm.map.clear();
}

/// Get entry by insertion position.
#[allow(dead_code)]
pub fn lmap_get_at<V>(lm: &LinkedMap<V>, pos: usize) -> Option<(&str, &V)> {
    lm.keys
        .get(pos)
        .and_then(|k| lm.map.get(k).map(|v| (k.as_str(), v)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_get() {
        let mut lm: LinkedMap<u32> = new_linked_map();
        lmap_insert(&mut lm, "a", 1);
        assert_eq!(lmap_get(&lm, "a"), Some(&1));
    }

    #[test]
    fn test_order_preserved() {
        let mut lm: LinkedMap<u32> = new_linked_map();
        lmap_insert(&mut lm, "c", 3);
        lmap_insert(&mut lm, "a", 1);
        lmap_insert(&mut lm, "b", 2);
        assert_eq!(lmap_keys(&lm), &["c", "a", "b"]);
    }

    #[test]
    fn test_update_no_reorder() {
        let mut lm: LinkedMap<u32> = new_linked_map();
        lmap_insert(&mut lm, "x", 1);
        lmap_insert(&mut lm, "y", 2);
        lmap_insert(&mut lm, "x", 99);
        assert_eq!(lmap_keys(&lm), &["x", "y"]);
        assert_eq!(lmap_get(&lm, "x"), Some(&99));
    }

    #[test]
    fn test_remove() {
        let mut lm: LinkedMap<u32> = new_linked_map();
        lmap_insert(&mut lm, "a", 1);
        lmap_insert(&mut lm, "b", 2);
        assert_eq!(lmap_remove(&mut lm, "a"), Some(1));
        assert_eq!(lmap_keys(&lm), &["b"]);
    }

    #[test]
    fn test_contains() {
        let mut lm: LinkedMap<u32> = new_linked_map();
        lmap_insert(&mut lm, "k", 0);
        assert!(lmap_contains(&lm, "k"));
        assert!(!lmap_contains(&lm, "z"));
    }

    #[test]
    fn test_len_and_empty() {
        let mut lm: LinkedMap<u32> = new_linked_map();
        assert!(lmap_is_empty(&lm));
        lmap_insert(&mut lm, "a", 1);
        assert_eq!(lmap_len(&lm), 1);
    }

    #[test]
    fn test_values_ordered() {
        let mut lm: LinkedMap<u32> = new_linked_map();
        lmap_insert(&mut lm, "a", 10);
        lmap_insert(&mut lm, "b", 20);
        assert_eq!(lmap_values(&lm), vec![&10, &20]);
    }

    #[test]
    fn test_get_at() {
        let mut lm: LinkedMap<u32> = new_linked_map();
        lmap_insert(&mut lm, "first", 1);
        lmap_insert(&mut lm, "second", 2);
        let (k, v) = lmap_get_at(&lm, 1).expect("should succeed");
        assert_eq!(k, "second");
        assert_eq!(*v, 2);
    }

    #[test]
    fn test_clear() {
        let mut lm: LinkedMap<u32> = new_linked_map();
        lmap_insert(&mut lm, "a", 1);
        lmap_clear(&mut lm);
        assert!(lmap_is_empty(&lm));
    }
}
