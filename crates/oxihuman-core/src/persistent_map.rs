// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

/// A persistent-style map that supports snapshotting.
#[allow(dead_code)]
pub struct PersistentMap {
    current: HashMap<String, String>,
    snapshots: Vec<HashMap<String, String>>,
    version: u64,
}

#[allow(dead_code)]
impl PersistentMap {
    pub fn new() -> Self {
        Self {
            current: HashMap::new(),
            snapshots: Vec::new(),
            version: 0,
        }
    }
    pub fn insert(&mut self, key: &str, value: &str) {
        self.current.insert(key.to_string(), value.to_string());
        self.version += 1;
    }
    pub fn get(&self, key: &str) -> Option<&str> {
        self.current.get(key).map(|s| s.as_str())
    }
    pub fn remove(&mut self, key: &str) -> bool {
        let removed = self.current.remove(key).is_some();
        if removed {
            self.version += 1;
        }
        removed
    }
    pub fn contains(&self, key: &str) -> bool {
        self.current.contains_key(key)
    }
    pub fn snapshot(&mut self) {
        self.snapshots.push(self.current.clone());
    }
    pub fn restore(&mut self, idx: usize) -> bool {
        if let Some(snap) = self.snapshots.get(idx) {
            self.current = snap.clone();
            self.version += 1;
            true
        } else {
            false
        }
    }
    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }
    pub fn version(&self) -> u64 {
        self.version
    }
    pub fn len(&self) -> usize {
        self.current.len()
    }
    pub fn is_empty(&self) -> bool {
        self.current.is_empty()
    }
    pub fn clear(&mut self) {
        self.current.clear();
        self.version += 1;
    }
    pub fn keys(&self) -> Vec<&str> {
        self.current.keys().map(|s| s.as_str()).collect()
    }
    pub fn diff_from_snapshot(&self, idx: usize) -> Vec<String> {
        if let Some(snap) = self.snapshots.get(idx) {
            self.current
                .keys()
                .filter(|k| snap.get(*k) != self.current.get(*k))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl Default for PersistentMap {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
pub fn new_persistent_map() -> PersistentMap {
    PersistentMap::new()
}
#[allow(dead_code)]
pub fn pm_insert(m: &mut PersistentMap, k: &str, v: &str) {
    m.insert(k, v);
}
#[allow(dead_code)]
pub fn pm_get<'a>(m: &'a PersistentMap, k: &str) -> Option<&'a str> {
    m.get(k)
}
#[allow(dead_code)]
pub fn pm_remove(m: &mut PersistentMap, k: &str) -> bool {
    m.remove(k)
}
#[allow(dead_code)]
pub fn pm_contains(m: &PersistentMap, k: &str) -> bool {
    m.contains(k)
}
#[allow(dead_code)]
pub fn pm_snapshot(m: &mut PersistentMap) {
    m.snapshot();
}
#[allow(dead_code)]
pub fn pm_restore(m: &mut PersistentMap, idx: usize) -> bool {
    m.restore(idx)
}
#[allow(dead_code)]
pub fn pm_snapshot_count(m: &PersistentMap) -> usize {
    m.snapshot_count()
}
#[allow(dead_code)]
pub fn pm_version(m: &PersistentMap) -> u64 {
    m.version()
}
#[allow(dead_code)]
pub fn pm_len(m: &PersistentMap) -> usize {
    m.len()
}
#[allow(dead_code)]
pub fn pm_is_empty(m: &PersistentMap) -> bool {
    m.is_empty()
}
#[allow(dead_code)]
pub fn pm_clear(m: &mut PersistentMap) {
    m.clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_insert_get() {
        let mut m = new_persistent_map();
        pm_insert(&mut m, "a", "1");
        assert_eq!(pm_get(&m, "a"), Some("1"));
    }
    #[test]
    fn test_remove() {
        let mut m = new_persistent_map();
        pm_insert(&mut m, "x", "v");
        assert!(pm_remove(&mut m, "x"));
        assert!(!pm_contains(&m, "x"));
    }
    #[test]
    fn test_snapshot_restore() {
        let mut m = new_persistent_map();
        pm_insert(&mut m, "k", "before");
        pm_snapshot(&mut m);
        pm_insert(&mut m, "k", "after");
        pm_restore(&mut m, 0);
        assert_eq!(pm_get(&m, "k"), Some("before"));
    }
    #[test]
    fn test_snapshot_count() {
        let mut m = new_persistent_map();
        pm_snapshot(&mut m);
        pm_snapshot(&mut m);
        assert_eq!(pm_snapshot_count(&m), 2);
    }
    #[test]
    fn test_version_increments() {
        let mut m = new_persistent_map();
        let v0 = pm_version(&m);
        pm_insert(&mut m, "a", "1");
        assert!(pm_version(&m) > v0);
    }
    #[test]
    fn test_len() {
        let mut m = new_persistent_map();
        pm_insert(&mut m, "a", "1");
        pm_insert(&mut m, "b", "2");
        assert_eq!(pm_len(&m), 2);
    }
    #[test]
    fn test_is_empty() {
        let m = new_persistent_map();
        assert!(pm_is_empty(&m));
    }
    #[test]
    fn test_clear() {
        let mut m = new_persistent_map();
        pm_insert(&mut m, "a", "1");
        pm_clear(&mut m);
        assert!(pm_is_empty(&m));
    }
    #[test]
    fn test_diff_from_snapshot() {
        let mut m = new_persistent_map();
        pm_insert(&mut m, "k", "old");
        pm_snapshot(&mut m);
        pm_insert(&mut m, "k", "new");
        let diff = m.diff_from_snapshot(0);
        assert!(diff.contains(&"k".to_string()));
    }
    #[test]
    fn test_restore_invalid_idx() {
        let mut m = new_persistent_map();
        assert!(!pm_restore(&mut m, 99));
    }
}
