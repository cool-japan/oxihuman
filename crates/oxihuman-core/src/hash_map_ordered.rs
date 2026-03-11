#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Insertion-ordered hash map.

use std::collections::HashMap;

/// A hash map that preserves insertion order.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OrderedHashMap<V> {
    map: HashMap<String, V>,
    keys: Vec<String>,
}

#[allow(dead_code)]
pub fn new_ordered_map<V>() -> OrderedHashMap<V> {
    OrderedHashMap {
        map: HashMap::new(),
        keys: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn insert_ordered<V>(m: &mut OrderedHashMap<V>, key: &str, value: V) {
    if !m.map.contains_key(key) {
        m.keys.push(key.to_string());
    }
    m.map.insert(key.to_string(), value);
}

#[allow(dead_code)]
pub fn get_ordered<'a, V>(m: &'a OrderedHashMap<V>, key: &str) -> Option<&'a V> {
    m.map.get(key)
}

#[allow(dead_code)]
pub fn remove_ordered<V>(m: &mut OrderedHashMap<V>, key: &str) -> Option<V> {
    if let Some(v) = m.map.remove(key) {
        m.keys.retain(|k| k != key);
        Some(v)
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn ordered_keys<V>(m: &OrderedHashMap<V>) -> &[String] {
    &m.keys
}

#[allow(dead_code)]
pub fn ordered_len<V>(m: &OrderedHashMap<V>) -> usize {
    m.map.len()
}

#[allow(dead_code)]
pub fn ordered_iter_keys<V>(m: &OrderedHashMap<V>) -> impl Iterator<Item = &str> {
    m.keys.iter().map(|s| s.as_str())
}

#[allow(dead_code)]
pub fn ordered_contains<V>(m: &OrderedHashMap<V>, key: &str) -> bool {
    m.map.contains_key(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_is_empty() {
        let m: OrderedHashMap<i32> = new_ordered_map();
        assert_eq!(ordered_len(&m), 0);
    }

    #[test]
    fn test_insert_and_get() {
        let mut m = new_ordered_map();
        insert_ordered(&mut m, "a", 1);
        assert_eq!(get_ordered(&m, "a"), Some(&1));
    }

    #[test]
    fn test_order_preserved() {
        let mut m = new_ordered_map();
        insert_ordered(&mut m, "b", 2);
        insert_ordered(&mut m, "a", 1);
        insert_ordered(&mut m, "c", 3);
        let keys: Vec<&str> = ordered_iter_keys(&m).collect();
        assert_eq!(keys, vec!["b", "a", "c"]);
    }

    #[test]
    fn test_remove() {
        let mut m = new_ordered_map();
        insert_ordered(&mut m, "x", 10);
        let v = remove_ordered(&mut m, "x");
        assert_eq!(v, Some(10));
        assert_eq!(ordered_len(&m), 0);
    }

    #[test]
    fn test_remove_missing() {
        let mut m: OrderedHashMap<i32> = new_ordered_map();
        assert_eq!(remove_ordered(&mut m, "z"), None);
    }

    #[test]
    fn test_contains() {
        let mut m = new_ordered_map();
        insert_ordered(&mut m, "k", 5);
        assert!(ordered_contains(&m, "k"));
        assert!(!ordered_contains(&m, "missing"));
    }

    #[test]
    fn test_keys_slice() {
        let mut m = new_ordered_map();
        insert_ordered(&mut m, "one", 1);
        insert_ordered(&mut m, "two", 2);
        assert_eq!(ordered_keys(&m).len(), 2);
    }

    #[test]
    fn test_overwrite_preserves_order() {
        let mut m = new_ordered_map();
        insert_ordered(&mut m, "a", 1);
        insert_ordered(&mut m, "b", 2);
        insert_ordered(&mut m, "a", 99);
        assert_eq!(get_ordered(&m, "a"), Some(&99));
        let keys: Vec<&str> = ordered_iter_keys(&m).collect();
        assert_eq!(keys, vec!["a", "b"]);
    }

    #[test]
    fn test_len_after_ops() {
        let mut m = new_ordered_map();
        insert_ordered(&mut m, "a", 1);
        insert_ordered(&mut m, "b", 2);
        remove_ordered(&mut m, "a");
        assert_eq!(ordered_len(&m), 1);
    }

    #[test]
    fn test_get_missing() {
        let m: OrderedHashMap<i32> = new_ordered_map();
        assert_eq!(get_ordered(&m, "nope"), None);
    }
}
