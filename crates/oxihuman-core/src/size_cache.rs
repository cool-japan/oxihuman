// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A cache that tracks byte-size usage and evicts LRU entries when over budget.

use std::collections::HashMap;

/// An entry in the size cache.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SizeCacheEntry {
    pub key: String,
    pub size_bytes: usize,
    pub access_count: u64,
    pub last_access: u64,
}

/// Cache with a byte-size budget that evicts LRU entries.
#[allow(dead_code)]
pub struct SizeCache {
    entries: HashMap<String, SizeCacheEntry>,
    budget_bytes: usize,
    used_bytes: usize,
    clock: u64,
    evictions: u64,
}

#[allow(dead_code)]
impl SizeCache {
    pub fn new(budget_bytes: usize) -> Self {
        Self {
            entries: HashMap::new(),
            budget_bytes,
            used_bytes: 0,
            clock: 0,
            evictions: 0,
        }
    }

    pub fn insert(&mut self, key: &str, size_bytes: usize) {
        // Remove existing to reclaim space.
        if let Some(old) = self.entries.remove(key) {
            self.used_bytes = self.used_bytes.saturating_sub(old.size_bytes);
        }
        // Evict LRU until there is room.
        while !self.entries.is_empty() && self.used_bytes + size_bytes > self.budget_bytes {
            self.evict_lru();
        }
        self.clock += 1;
        self.used_bytes += size_bytes;
        self.entries.insert(
            key.to_string(),
            SizeCacheEntry {
                key: key.to_string(),
                size_bytes,
                access_count: 1,
                last_access: self.clock,
            },
        );
    }

    pub fn get(&mut self, key: &str) -> Option<&SizeCacheEntry> {
        self.clock += 1;
        let t = self.clock;
        if let Some(e) = self.entries.get_mut(key) {
            e.access_count += 1;
            e.last_access = t;
            Some(e)
        } else {
            None
        }
    }

    pub fn remove(&mut self, key: &str) -> bool {
        if let Some(e) = self.entries.remove(key) {
            self.used_bytes = self.used_bytes.saturating_sub(e.size_bytes);
            true
        } else {
            false
        }
    }

    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    pub fn used_bytes(&self) -> usize {
        self.used_bytes
    }

    pub fn budget_bytes(&self) -> usize {
        self.budget_bytes
    }

    pub fn count(&self) -> usize {
        self.entries.len()
    }

    pub fn evictions(&self) -> u64 {
        self.evictions
    }

    pub fn is_over_budget(&self) -> bool {
        self.used_bytes > self.budget_bytes
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.used_bytes = 0;
    }

    fn evict_lru(&mut self) {
        let key = self
            .entries
            .values()
            .min_by_key(|e| e.last_access)
            .map(|e| e.key.clone());
        if let Some(k) = key {
            if let Some(e) = self.entries.remove(&k) {
                self.used_bytes = self.used_bytes.saturating_sub(e.size_bytes);
                self.evictions += 1;
            }
        }
    }
}

impl Default for SizeCache {
    fn default() -> Self {
        Self::new(1024 * 1024)
    }
}

pub fn new_size_cache(budget_bytes: usize) -> SizeCache {
    SizeCache::new(budget_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut c = new_size_cache(1000);
        c.insert("a", 100);
        assert!(c.contains("a"));
        assert_eq!(c.used_bytes(), 100);
    }

    #[test]
    fn remove_entry() {
        let mut c = new_size_cache(1000);
        c.insert("x", 50);
        assert!(c.remove("x"));
        assert!(!c.contains("x"));
        assert_eq!(c.used_bytes(), 0);
    }

    #[test]
    fn eviction_on_overflow() {
        let mut c = new_size_cache(100);
        c.insert("a", 60);
        c.insert("b", 60); // should evict "a"
        assert!(c.evictions() > 0);
        assert!(c.used_bytes() <= 100);
    }

    #[test]
    fn get_updates_access() {
        let mut c = new_size_cache(1000);
        c.insert("a", 10);
        c.get("a");
        assert_eq!(c.get("a").unwrap().access_count, 3);
    }

    #[test]
    fn clear_resets_used() {
        let mut c = new_size_cache(1000);
        c.insert("a", 200);
        c.clear();
        assert_eq!(c.used_bytes(), 0);
        assert_eq!(c.count(), 0);
    }

    #[test]
    fn over_budget_flag() {
        let mut c = new_size_cache(10);
        // Force insert beyond budget by inserting matching single items.
        c.insert("big", 5);
        assert!(!c.is_over_budget());
    }

    #[test]
    fn duplicate_insert_reclaims() {
        let mut c = new_size_cache(1000);
        c.insert("k", 100);
        c.insert("k", 50);
        assert_eq!(c.used_bytes(), 50);
        assert_eq!(c.count(), 1);
    }

    #[test]
    fn budget_bytes() {
        let c = new_size_cache(512);
        assert_eq!(c.budget_bytes(), 512);
    }

    #[test]
    fn evictions_counter() {
        let mut c = new_size_cache(50);
        c.insert("a", 40);
        c.insert("b", 40);
        assert!(c.evictions() >= 1);
    }

    #[test]
    fn get_missing_returns_none() {
        let mut c = new_size_cache(100);
        assert!(c.get("missing").is_none());
    }
}
