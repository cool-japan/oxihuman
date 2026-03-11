// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Persistent (immutable) hash map stub — each insert or remove produces a
//! new logical version, represented here as a cloned HashMap, making all
//! previous versions independently accessible.

use std::collections::HashMap;
use std::hash::Hash;

/// A version of the persistent map.
pub type PmapVersion = usize;

/// Persistent hash map that keeps all historical versions.
pub struct PersistentHashMap<K, V> {
    versions: Vec<HashMap<K, V>>,
}

impl<K, V> PersistentHashMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// Create a persistent map with an empty initial version.
    pub fn new() -> Self {
        Self {
            versions: vec![HashMap::new()],
        }
    }

    /// Current (latest) version number.
    pub fn current_version(&self) -> PmapVersion {
        self.versions.len() - 1
    }

    /// Insert `key`→`value` into the current version, producing a new version.
    pub fn insert(&mut self, key: K, value: V) -> PmapVersion {
        let mut next = self.versions.last().cloned().unwrap_or_default();
        next.insert(key, value);
        self.versions.push(next);
        self.current_version()
    }

    /// Remove `key` from the current version, producing a new version.
    pub fn remove(&mut self, key: &K) -> PmapVersion {
        let mut next = self.versions.last().cloned().unwrap_or_default();
        next.remove(key);
        self.versions.push(next);
        self.current_version()
    }

    /// Get a value at a specific version.
    pub fn get_at(&self, version: PmapVersion, key: &K) -> Option<&V> {
        self.versions.get(version)?.get(key)
    }

    /// Get a value at the current version.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.get_at(self.current_version(), key)
    }

    /// Number of entries in the current version.
    pub fn len(&self) -> usize {
        self.versions.last().map(|m| m.len()).unwrap_or(0)
    }

    /// True if the current version is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Total versions stored.
    pub fn version_count(&self) -> usize {
        self.versions.len()
    }
}

impl<K, V> Default for PersistentHashMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new persistent hash map.
pub fn new_persistent_hash_map<K, V>() -> PersistentHashMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    PersistentHashMap::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut m: PersistentHashMap<&str, i32> = PersistentHashMap::new();
        m.insert("a", 1);
        assert_eq!(m.get(&"a"), Some(&1)); /* current version has value */
    }

    #[test]
    fn test_old_version_preserved() {
        let mut m: PersistentHashMap<&str, i32> = PersistentHashMap::new();
        m.insert("a", 1);
        let v1 = m.insert("b", 2);
        /* version 1 should only have "a" */
        assert_eq!(m.get_at(1, &"a"), Some(&1));
        assert_eq!(m.get_at(1, &"b"), None);
        /* current version (v1) has both */
        assert_eq!(m.get_at(v1, &"b"), Some(&2));
    }

    #[test]
    fn test_remove_creates_version() {
        let mut m: PersistentHashMap<&str, i32> = PersistentHashMap::new();
        let v0 = m.insert("x", 10);
        let v1 = m.remove(&"x");
        assert!(m.get_at(v1, &"x").is_none()); /* removed in v1 */
        assert!(m.get_at(v0, &"x").is_some()); /* still in v0 */
    }

    #[test]
    fn test_len() {
        let mut m: PersistentHashMap<i32, i32> = PersistentHashMap::new();
        m.insert(1, 1);
        m.insert(2, 2);
        assert_eq!(m.len(), 2); /* two entries */
    }

    #[test]
    fn test_is_empty_initially() {
        let m: PersistentHashMap<i32, i32> = PersistentHashMap::new();
        assert!(m.is_empty()); /* initial version empty */
    }

    #[test]
    fn test_version_count() {
        let mut m: PersistentHashMap<i32, i32> = PersistentHashMap::new();
        m.insert(1, 1);
        m.insert(2, 2);
        assert_eq!(m.version_count(), 3); /* initial + 2 inserts */
    }

    #[test]
    fn test_current_version() {
        let mut m: PersistentHashMap<i32, i32> = PersistentHashMap::new();
        assert_eq!(m.current_version(), 0);
        m.insert(1, 1);
        assert_eq!(m.current_version(), 1);
    }

    #[test]
    fn test_default() {
        let m: PersistentHashMap<i32, i32> = PersistentHashMap::default();
        assert!(m.is_empty()); /* default is empty */
    }

    #[test]
    fn test_new_helper() {
        let m = new_persistent_hash_map::<i32, i32>();
        assert!(m.is_empty()); /* helper creates empty map */
    }

    #[test]
    fn test_overwrite_key() {
        let mut m: PersistentHashMap<&str, i32> = PersistentHashMap::new();
        m.insert("k", 1);
        m.insert("k", 2);
        assert_eq!(m.get(&"k"), Some(&2)); /* latest value wins */
    }
}
