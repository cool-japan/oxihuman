// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Manual reference-counted resource table.

use std::collections::HashMap;

/// Entry in the ref-counted table.
#[derive(Debug, Clone)]
pub struct RefEntry<V> {
    pub value: V,
    pub ref_count: u32,
}

/// Table that tracks reference counts for shared resources.
pub struct RefCounted<V> {
    entries: HashMap<String, RefEntry<V>>,
}

#[allow(dead_code)]
impl<V: Clone> RefCounted<V> {
    pub fn new() -> Self {
        RefCounted {
            entries: HashMap::new(),
        }
    }

    /// Insert with initial ref-count of 1. Returns false if already exists.
    pub fn insert(&mut self, key: &str, value: V) -> bool {
        if self.entries.contains_key(key) {
            return false;
        }
        self.entries.insert(
            key.to_string(),
            RefEntry {
                value,
                ref_count: 1,
            },
        );
        true
    }

    /// Increment ref-count. Returns new count, or None if not found.
    pub fn acquire(&mut self, key: &str) -> Option<u32> {
        let e = self.entries.get_mut(key)?;
        e.ref_count += 1;
        Some(e.ref_count)
    }

    /// Decrement ref-count. Removes entry when count reaches 0.
    /// Returns remaining count, or None if not found.
    pub fn release(&mut self, key: &str) -> Option<u32> {
        let count = {
            let e = self.entries.get_mut(key)?;
            e.ref_count = e.ref_count.saturating_sub(1);
            e.ref_count
        };
        if count == 0 {
            self.entries.remove(key);
        }
        Some(count)
    }

    pub fn get(&self, key: &str) -> Option<&V> {
        self.entries.get(key).map(|e| &e.value)
    }

    pub fn ref_count(&self, key: &str) -> Option<u32> {
        self.entries.get(key).map(|e| e.ref_count)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn total_refs(&self) -> u64 {
        self.entries.values().map(|e| e.ref_count as u64).sum()
    }

    pub fn keys(&self) -> Vec<&str> {
        self.entries.keys().map(|k| k.as_str()).collect()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl<V: Clone> Default for RefCounted<V> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_ref_counted<V: Clone>() -> RefCounted<V> {
    RefCounted::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut rc: RefCounted<i32> = new_ref_counted();
        assert!(rc.insert("res", 42));
        assert_eq!(*rc.get("res").unwrap(), 42);
    }

    #[test]
    fn duplicate_insert_fails() {
        let mut rc: RefCounted<i32> = new_ref_counted();
        rc.insert("k", 1);
        assert!(!rc.insert("k", 2));
    }

    #[test]
    fn acquire_increments() {
        let mut rc: RefCounted<i32> = new_ref_counted();
        rc.insert("k", 1);
        assert_eq!(rc.acquire("k"), Some(2));
        assert_eq!(rc.ref_count("k"), Some(2));
    }

    #[test]
    fn release_decrements() {
        let mut rc: RefCounted<i32> = new_ref_counted();
        rc.insert("k", 1);
        rc.acquire("k");
        assert_eq!(rc.release("k"), Some(1));
    }

    #[test]
    fn release_to_zero_removes() {
        let mut rc: RefCounted<i32> = new_ref_counted();
        rc.insert("k", 1);
        rc.release("k");
        assert!(!rc.contains("k"));
    }

    #[test]
    fn total_refs() {
        let mut rc: RefCounted<i32> = new_ref_counted();
        rc.insert("a", 1);
        rc.insert("b", 2);
        rc.acquire("a");
        assert_eq!(rc.total_refs(), 3);
    }

    #[test]
    fn contains_check() {
        let mut rc: RefCounted<i32> = new_ref_counted();
        rc.insert("x", 0);
        assert!(rc.contains("x"));
        assert!(!rc.contains("y"));
    }

    #[test]
    fn clear_empties() {
        let mut rc: RefCounted<i32> = new_ref_counted();
        rc.insert("a", 1);
        rc.clear();
        assert!(rc.is_empty());
    }

    #[test]
    fn release_missing_is_none() {
        let mut rc: RefCounted<i32> = new_ref_counted();
        assert!(rc.release("ghost").is_none());
    }
}
