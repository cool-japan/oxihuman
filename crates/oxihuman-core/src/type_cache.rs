// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Type-keyed cache: stores one boxed value per type name string.

use std::collections::HashMap;

/// A single entry in the type cache.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TypeCacheEntry {
    pub type_name: String,
    pub size_bytes: usize,
    pub version: u32,
}

/// Type cache keyed by type name string.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct TypeCache {
    entries: HashMap<String, TypeCacheEntry>,
    data: HashMap<String, Vec<u8>>,
}

/// Create a new `TypeCache`.
#[allow(dead_code)]
pub fn new_type_cache() -> TypeCache {
    TypeCache::default()
}

/// Store raw bytes for a type name.
#[allow(dead_code)]
pub fn tc_store(tc: &mut TypeCache, type_name: &str, data: Vec<u8>) {
    let version = tc
        .entries
        .get(type_name)
        .map(|e| e.version + 1)
        .unwrap_or(0);
    let size = data.len();
    tc.entries.insert(
        type_name.to_string(),
        TypeCacheEntry {
            type_name: type_name.to_string(),
            size_bytes: size,
            version,
        },
    );
    tc.data.insert(type_name.to_string(), data);
}

/// Retrieve raw bytes for a type name.
#[allow(dead_code)]
pub fn tc_get<'a>(tc: &'a TypeCache, type_name: &str) -> Option<&'a [u8]> {
    tc.data.get(type_name).map(|v| v.as_slice())
}

/// Check if a type is cached.
#[allow(dead_code)]
pub fn tc_contains(tc: &TypeCache, type_name: &str) -> bool {
    tc.entries.contains_key(type_name)
}

/// Remove a cached type.
#[allow(dead_code)]
pub fn tc_remove(tc: &mut TypeCache, type_name: &str) {
    tc.entries.remove(type_name);
    tc.data.remove(type_name);
}

/// Number of cached types.
#[allow(dead_code)]
pub fn tc_len(tc: &TypeCache) -> usize {
    tc.entries.len()
}

/// Whether the cache is empty.
#[allow(dead_code)]
pub fn tc_is_empty(tc: &TypeCache) -> bool {
    tc.entries.is_empty()
}

/// Version number for a cached type.
#[allow(dead_code)]
pub fn tc_version(tc: &TypeCache, type_name: &str) -> Option<u32> {
    tc.entries.get(type_name).map(|e| e.version)
}

/// Total bytes cached.
#[allow(dead_code)]
pub fn tc_total_bytes(tc: &TypeCache) -> usize {
    tc.entries.values().map(|e| e.size_bytes).sum()
}

/// Clear all cached entries.
#[allow(dead_code)]
pub fn tc_clear(tc: &mut TypeCache) {
    tc.entries.clear();
    tc.data.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let tc = new_type_cache();
        assert!(tc_is_empty(&tc));
    }

    #[test]
    fn test_store_and_get() {
        let mut tc = new_type_cache();
        tc_store(&mut tc, "MyStruct", vec![1, 2, 3]);
        assert_eq!(tc_get(&tc, "MyStruct"), Some([1u8, 2, 3].as_slice()));
    }

    #[test]
    fn test_contains() {
        let mut tc = new_type_cache();
        tc_store(&mut tc, "Foo", vec![]);
        assert!(tc_contains(&tc, "Foo"));
        assert!(!tc_contains(&tc, "Bar"));
    }

    #[test]
    fn test_remove() {
        let mut tc = new_type_cache();
        tc_store(&mut tc, "T", vec![9]);
        tc_remove(&mut tc, "T");
        assert!(!tc_contains(&tc, "T"));
    }

    #[test]
    fn test_version_increments() {
        let mut tc = new_type_cache();
        tc_store(&mut tc, "X", vec![1]);
        assert_eq!(tc_version(&tc, "X"), Some(0));
        tc_store(&mut tc, "X", vec![2]);
        assert_eq!(tc_version(&tc, "X"), Some(1));
    }

    #[test]
    fn test_total_bytes() {
        let mut tc = new_type_cache();
        tc_store(&mut tc, "A", vec![0; 10]);
        tc_store(&mut tc, "B", vec![0; 20]);
        assert_eq!(tc_total_bytes(&tc), 30);
    }

    #[test]
    fn test_len() {
        let mut tc = new_type_cache();
        tc_store(&mut tc, "A", vec![]);
        tc_store(&mut tc, "B", vec![]);
        assert_eq!(tc_len(&tc), 2);
    }

    #[test]
    fn test_clear() {
        let mut tc = new_type_cache();
        tc_store(&mut tc, "X", vec![1, 2]);
        tc_clear(&mut tc);
        assert!(tc_is_empty(&tc));
    }

    #[test]
    fn test_missing_returns_none() {
        let tc = new_type_cache();
        assert!(tc_get(&tc, "Missing").is_none());
        assert!(tc_version(&tc, "Missing").is_none());
    }
}
