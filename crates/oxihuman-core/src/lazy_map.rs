// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Lazy map: values are computed on first access and cached.

use std::collections::HashMap;

/// Lazy map entry state.
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum LazyEntry<V> {
    Pending,
    Computed(V),
}

/// Lazy map that stores computed results.
#[derive(Debug)]
#[allow(dead_code)]
pub struct LazyMap<V> {
    entries: HashMap<String, LazyEntry<V>>,
    pending_count: usize,
    compute_count: u64,
}

/// Create a new LazyMap.
#[allow(dead_code)]
pub fn new_lazy_map<V>() -> LazyMap<V> {
    LazyMap {
        entries: HashMap::new(),
        pending_count: 0,
        compute_count: 0,
    }
}

/// Declare a key as pending (not yet computed).
#[allow(dead_code)]
pub fn lm_declare<V>(map: &mut LazyMap<V>, key: &str) {
    map.entries.insert(key.to_string(), LazyEntry::Pending);
    map.pending_count += 1;
}

/// Store a computed value for a key.
#[allow(dead_code)]
pub fn lm_set<V>(map: &mut LazyMap<V>, key: &str, value: V) {
    let was_pending = matches!(map.entries.get(key), Some(LazyEntry::Pending));
    map.entries
        .insert(key.to_string(), LazyEntry::Computed(value));
    if was_pending {
        map.pending_count = map.pending_count.saturating_sub(1);
    }
    map.compute_count += 1;
}

/// Get computed value; returns None if pending or missing.
#[allow(dead_code)]
pub fn lm_get<'a, V>(map: &'a LazyMap<V>, key: &str) -> Option<&'a V> {
    match map.entries.get(key) {
        Some(LazyEntry::Computed(v)) => Some(v),
        _ => None,
    }
}

/// Whether a key is pending.
#[allow(dead_code)]
pub fn lm_is_pending<V>(map: &LazyMap<V>, key: &str) -> bool {
    matches!(map.entries.get(key), Some(LazyEntry::Pending))
}

/// Whether a key is computed.
#[allow(dead_code)]
pub fn lm_is_computed<V>(map: &LazyMap<V>, key: &str) -> bool {
    matches!(map.entries.get(key), Some(LazyEntry::Computed(_)))
}

/// Number of pending keys.
#[allow(dead_code)]
pub fn lm_pending_count<V>(map: &LazyMap<V>) -> usize {
    map.pending_count
}

/// Total compute operations.
#[allow(dead_code)]
pub fn lm_compute_count<V>(map: &LazyMap<V>) -> u64 {
    map.compute_count
}

/// Remove a key.
#[allow(dead_code)]
pub fn lm_remove<V>(map: &mut LazyMap<V>, key: &str) -> bool {
    if let Some(entry) = map.entries.remove(key) {
        if matches!(entry, LazyEntry::Pending) {
            map.pending_count = map.pending_count.saturating_sub(1);
        }
        true
    } else {
        false
    }
}

/// Clear all entries.
#[allow(dead_code)]
pub fn lm_clear<V>(map: &mut LazyMap<V>) {
    map.entries.clear();
    map.pending_count = 0;
}

/// Total number of keys.
#[allow(dead_code)]
pub fn lm_len<V>(map: &LazyMap<V>) -> usize {
    map.entries.len()
}

/// Keys that are still pending.
#[allow(dead_code)]
pub fn lm_pending_keys<V>(map: &LazyMap<V>) -> Vec<String> {
    map.entries
        .iter()
        .filter_map(|(k, v)| {
            if matches!(v, LazyEntry::Pending) {
                Some(k.clone())
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_declare_pending() {
        let mut map: LazyMap<u32> = new_lazy_map();
        lm_declare(&mut map, "a");
        assert!(lm_is_pending(&map, "a"));
        assert!(!lm_is_computed(&map, "a"));
    }

    #[test]
    fn test_set_and_get() {
        let mut map: LazyMap<u32> = new_lazy_map();
        lm_set(&mut map, "x", 42);
        assert_eq!(lm_get(&map, "x"), Some(&42));
    }

    #[test]
    fn test_pending_becomes_computed() {
        let mut map: LazyMap<f32> = new_lazy_map();
        lm_declare(&mut map, "k");
        lm_set(&mut map, "k", std::f32::consts::PI);
        assert!(lm_is_computed(&map, "k"));
        assert_eq!(lm_pending_count(&map), 0);
    }

    #[test]
    fn test_missing_key() {
        let map: LazyMap<u32> = new_lazy_map();
        assert_eq!(lm_get(&map, "missing"), None);
        assert!(!lm_is_pending(&map, "missing"));
    }

    #[test]
    fn test_pending_count() {
        let mut map: LazyMap<u32> = new_lazy_map();
        lm_declare(&mut map, "a");
        lm_declare(&mut map, "b");
        assert_eq!(lm_pending_count(&map), 2);
        lm_set(&mut map, "a", 1);
        assert_eq!(lm_pending_count(&map), 1);
    }

    #[test]
    fn test_compute_count() {
        let mut map: LazyMap<u32> = new_lazy_map();
        lm_set(&mut map, "a", 1);
        lm_set(&mut map, "b", 2);
        assert_eq!(lm_compute_count(&map), 2);
    }

    #[test]
    fn test_remove() {
        let mut map: LazyMap<u32> = new_lazy_map();
        lm_set(&mut map, "z", 9);
        assert!(lm_remove(&mut map, "z"));
        assert_eq!(lm_len(&map), 0);
    }

    #[test]
    fn test_clear() {
        let mut map: LazyMap<u32> = new_lazy_map();
        lm_declare(&mut map, "a");
        lm_set(&mut map, "b", 2);
        lm_clear(&mut map);
        assert_eq!(lm_len(&map), 0);
        assert_eq!(lm_pending_count(&map), 0);
    }

    #[test]
    fn test_pending_keys() {
        let mut map: LazyMap<u32> = new_lazy_map();
        lm_declare(&mut map, "p1");
        lm_declare(&mut map, "p2");
        lm_set(&mut map, "c", 1);
        let pk = lm_pending_keys(&map);
        assert_eq!(pk.len(), 2);
    }
}
