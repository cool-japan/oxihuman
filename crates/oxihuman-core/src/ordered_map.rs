// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

/// An order-preserving map (insertion order).
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct OrderedMap {
    pub keys: Vec<String>,
    pub values: Vec<String>,
}

/// Create a new empty `OrderedMap`.
#[allow(dead_code)]
pub fn new_ordered_map() -> OrderedMap {
    OrderedMap::default()
}

/// Insert or update a key-value pair (preserves insertion order for new keys).
#[allow(dead_code)]
pub fn omap_insert(map: &mut OrderedMap, key: &str, val: &str) {
    if let Some(pos) = map.keys.iter().position(|k| k == key) {
        map.values[pos] = val.to_string();
    } else {
        map.keys.push(key.to_string());
        map.values.push(val.to_string());
    }
}

/// Get a value by key.
#[allow(dead_code)]
pub fn omap_get<'a>(map: &'a OrderedMap, key: &str) -> Option<&'a str> {
    map.keys
        .iter()
        .position(|k| k == key)
        .map(|i| map.values[i].as_str())
}

/// Remove a key-value pair. Returns `true` if removed, `false` if not found.
#[allow(dead_code)]
pub fn omap_remove(map: &mut OrderedMap, key: &str) -> bool {
    if let Some(pos) = map.keys.iter().position(|k| k == key) {
        map.keys.remove(pos);
        map.values.remove(pos);
        true
    } else {
        false
    }
}

/// Get all keys in insertion order.
#[allow(dead_code)]
pub fn omap_keys(map: &OrderedMap) -> Vec<&str> {
    map.keys.iter().map(|s| s.as_str()).collect()
}

/// Get the number of entries.
#[allow(dead_code)]
pub fn omap_len(map: &OrderedMap) -> usize {
    map.keys.len()
}

/// Get the index of a key in insertion order (0-based).
#[allow(dead_code)]
pub fn omap_index_of(map: &OrderedMap, key: &str) -> Option<usize> {
    map.keys.iter().position(|k| k == key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut m = new_ordered_map();
        omap_insert(&mut m, "color", "red");
        assert_eq!(omap_get(&m, "color"), Some("red"));
    }

    #[test]
    fn insertion_order_preserved() {
        let mut m = new_ordered_map();
        omap_insert(&mut m, "a", "1");
        omap_insert(&mut m, "b", "2");
        omap_insert(&mut m, "c", "3");
        let keys = omap_keys(&m);
        assert_eq!(keys, vec!["a", "b", "c"]);
    }

    #[test]
    fn update_does_not_change_order() {
        let mut m = new_ordered_map();
        omap_insert(&mut m, "x", "old");
        omap_insert(&mut m, "y", "y_val");
        omap_insert(&mut m, "x", "new");
        assert_eq!(omap_index_of(&m, "x"), Some(0));
        assert_eq!(omap_get(&m, "x"), Some("new"));
    }

    #[test]
    fn remove_key() {
        let mut m = new_ordered_map();
        omap_insert(&mut m, "k", "v");
        assert!(omap_remove(&mut m, "k"));
        assert_eq!(omap_len(&m), 0);
    }

    #[test]
    fn remove_missing_key_false() {
        let mut m = new_ordered_map();
        assert!(!omap_remove(&mut m, "nope"));
    }

    #[test]
    fn omap_len_correct() {
        let mut m = new_ordered_map();
        assert_eq!(omap_len(&m), 0);
        omap_insert(&mut m, "a", "1");
        omap_insert(&mut m, "b", "2");
        assert_eq!(omap_len(&m), 2);
    }

    #[test]
    fn get_missing_key_none() {
        let m = new_ordered_map();
        assert_eq!(omap_get(&m, "missing"), None);
    }

    #[test]
    fn index_of_correct() {
        let mut m = new_ordered_map();
        omap_insert(&mut m, "first", "a");
        omap_insert(&mut m, "second", "b");
        assert_eq!(omap_index_of(&m, "second"), Some(1));
    }

    #[test]
    fn remove_middle_preserves_order() {
        let mut m = new_ordered_map();
        omap_insert(&mut m, "a", "1");
        omap_insert(&mut m, "b", "2");
        omap_insert(&mut m, "c", "3");
        omap_remove(&mut m, "b");
        let keys = omap_keys(&m);
        assert_eq!(keys, vec!["a", "c"]);
    }

    #[test]
    fn omap_keys_empty() {
        let m = new_ordered_map();
        assert!(omap_keys(&m).is_empty());
    }
}
