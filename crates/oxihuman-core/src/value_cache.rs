// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Value cache: memoizes computed f32 values by string key with invalidation.

use std::collections::HashMap;

/// A cached value entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ValueEntry {
    pub value: f32,
    pub version: u32,
    pub dirty: bool,
}

/// The value cache.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ValueCache {
    entries: HashMap<String, ValueEntry>,
    global_version: u32,
}

/// Create a new `ValueCache`.
#[allow(dead_code)]
pub fn new_value_cache() -> ValueCache {
    ValueCache::default()
}

/// Store a computed value.
#[allow(dead_code)]
pub fn vc_store(vc: &mut ValueCache, key: &str, value: f32) {
    vc.entries.insert(
        key.to_string(),
        ValueEntry {
            value,
            version: vc.global_version,
            dirty: false,
        },
    );
}

/// Retrieve a cached value if not dirty.
#[allow(dead_code)]
pub fn vc_get(vc: &ValueCache, key: &str) -> Option<f32> {
    vc.entries
        .get(key)
        .and_then(|e| if e.dirty { None } else { Some(e.value) })
}

/// Mark a key as dirty (needs recompute).
#[allow(dead_code)]
pub fn vc_invalidate(vc: &mut ValueCache, key: &str) {
    if let Some(e) = vc.entries.get_mut(key) {
        e.dirty = true;
    }
}

/// Invalidate all entries.
#[allow(dead_code)]
pub fn vc_invalidate_all(vc: &mut ValueCache) {
    vc.global_version += 1;
    for e in vc.entries.values_mut() {
        e.dirty = true;
    }
}

/// Whether a key is cached and clean.
#[allow(dead_code)]
pub fn vc_is_valid(vc: &ValueCache, key: &str) -> bool {
    vc.entries.get(key).is_some_and(|e| !e.dirty)
}

/// Number of cached entries.
#[allow(dead_code)]
pub fn vc_len(vc: &ValueCache) -> usize {
    vc.entries.len()
}

/// Count of dirty entries.
#[allow(dead_code)]
pub fn vc_dirty_count(vc: &ValueCache) -> usize {
    vc.entries.values().filter(|e| e.dirty).count()
}

/// Remove a key.
#[allow(dead_code)]
pub fn vc_remove(vc: &mut ValueCache, key: &str) {
    vc.entries.remove(key);
}

/// Clear all entries.
#[allow(dead_code)]
pub fn vc_clear(vc: &mut ValueCache) {
    vc.entries.clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_store_and_get() {
        let mut vc = new_value_cache();
        vc_store(&mut vc, "x", PI);
        assert!((vc_get(&vc, "x").expect("should succeed") - PI).abs() < 1e-5);
    }

    #[test]
    fn test_missing_returns_none() {
        let vc = new_value_cache();
        assert!(vc_get(&vc, "y").is_none());
    }

    #[test]
    fn test_invalidate() {
        let mut vc = new_value_cache();
        vc_store(&mut vc, "a", 1.0);
        vc_invalidate(&mut vc, "a");
        assert!(vc_get(&vc, "a").is_none());
    }

    #[test]
    fn test_invalidate_all() {
        let mut vc = new_value_cache();
        vc_store(&mut vc, "a", 1.0);
        vc_store(&mut vc, "b", 2.0);
        vc_invalidate_all(&mut vc);
        assert_eq!(vc_dirty_count(&vc), 2);
    }

    #[test]
    fn test_is_valid() {
        let mut vc = new_value_cache();
        vc_store(&mut vc, "v", 5.0);
        assert!(vc_is_valid(&vc, "v"));
        vc_invalidate(&mut vc, "v");
        assert!(!vc_is_valid(&vc, "v"));
    }

    #[test]
    fn test_remove() {
        let mut vc = new_value_cache();
        vc_store(&mut vc, "k", 7.0);
        vc_remove(&mut vc, "k");
        assert!(vc_get(&vc, "k").is_none());
    }

    #[test]
    fn test_len() {
        let mut vc = new_value_cache();
        vc_store(&mut vc, "a", 1.0);
        vc_store(&mut vc, "b", 2.0);
        assert_eq!(vc_len(&vc), 2);
    }

    #[test]
    fn test_clear() {
        let mut vc = new_value_cache();
        vc_store(&mut vc, "x", 9.0);
        vc_clear(&mut vc);
        assert_eq!(vc_len(&vc), 0);
    }

    #[test]
    fn test_reclean_after_restore() {
        let mut vc = new_value_cache();
        vc_store(&mut vc, "m", 1.0);
        vc_invalidate(&mut vc, "m");
        vc_store(&mut vc, "m", 2.0);
        assert!(vc_is_valid(&vc, "m"));
    }
}
