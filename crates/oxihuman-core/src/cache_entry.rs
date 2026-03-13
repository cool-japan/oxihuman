// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A single cache entry with key, value, TTL and access tracking.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CacheEntryItem {
    pub key: String,
    pub value: Vec<u8>,
    pub ttl_ms: u64,
    pub created_at: u64,
    pub last_access: u64,
    pub access_count: u64,
    pub dirty: bool,
}

/// A cache store backed by a Vec of entries.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CacheStore {
    entries: Vec<CacheEntryItem>,
    capacity: usize,
    current_time: u64,
}

#[allow(dead_code)]
impl CacheStore {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: Vec::new(),
            capacity: capacity.max(1),
            current_time: 0,
        }
    }

    pub fn advance_time(&mut self, ms: u64) {
        self.current_time += ms;
    }

    pub fn put(&mut self, key: &str, value: Vec<u8>, ttl_ms: u64) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.key == key) {
            entry.value = value;
            entry.ttl_ms = ttl_ms;
            entry.created_at = self.current_time;
            entry.last_access = self.current_time;
            entry.dirty = true;
            return;
        }
        if self.entries.len() >= self.capacity {
            self.evict_oldest();
        }
        self.entries.push(CacheEntryItem {
            key: key.to_string(),
            value,
            ttl_ms,
            created_at: self.current_time,
            last_access: self.current_time,
            access_count: 0,
            dirty: false,
        });
    }

    pub fn get(&mut self, key: &str) -> Option<&[u8]> {
        let now = self.current_time;
        if let Some(entry) = self.entries.iter_mut().find(|e| e.key == key) {
            if entry.ttl_ms > 0 && now - entry.created_at > entry.ttl_ms {
                return None;
            }
            entry.last_access = now;
            entry.access_count += 1;
            Some(entry.value.as_slice())
        } else {
            None
        }
    }

    pub fn contains(&self, key: &str) -> bool {
        self.entries.iter().any(|e| e.key == key)
    }

    pub fn remove(&mut self, key: &str) -> bool {
        let before = self.entries.len();
        self.entries.retain(|e| e.key != key);
        self.entries.len() < before
    }

    fn evict_oldest(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        let mut oldest_idx = 0;
        let mut oldest_access = u64::MAX;
        #[allow(clippy::needless_range_loop)]
        for i in 0..self.entries.len() {
            if self.entries[i].last_access < oldest_access {
                oldest_access = self.entries[i].last_access;
                oldest_idx = i;
            }
        }
        self.entries.remove(oldest_idx);
    }

    pub fn evict_expired(&mut self) {
        let now = self.current_time;
        self.entries
            .retain(|e| e.ttl_ms == 0 || now - e.created_at <= e.ttl_ms);
    }

    pub fn count(&self) -> usize {
        self.entries.len()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn total_bytes(&self) -> usize {
        self.entries.iter().map(|e| e.value.len()).sum()
    }

    pub fn most_accessed(&self) -> Option<&str> {
        self.entries
            .iter()
            .max_by_key(|e| e.access_count)
            .map(|e| e.key.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let cs = CacheStore::new(10);
        assert_eq!(cs.count(), 0);
        assert_eq!(cs.capacity(), 10);
    }

    #[test]
    fn test_put_and_get() {
        let mut cs = CacheStore::new(10);
        cs.put("k", vec![1, 2, 3], 0);
        assert_eq!(cs.get("k"), Some([1u8, 2, 3].as_slice()));
    }

    #[test]
    fn test_contains() {
        let mut cs = CacheStore::new(10);
        cs.put("a", vec![1], 0);
        assert!(cs.contains("a"));
        assert!(!cs.contains("b"));
    }

    #[test]
    fn test_remove() {
        let mut cs = CacheStore::new(10);
        cs.put("a", vec![1], 0);
        assert!(cs.remove("a"));
        assert!(!cs.contains("a"));
    }

    #[test]
    fn test_evict_on_capacity() {
        let mut cs = CacheStore::new(2);
        cs.put("a", vec![1], 0);
        cs.put("b", vec![2], 0);
        cs.put("c", vec![3], 0);
        assert_eq!(cs.count(), 2);
        assert!(cs.contains("c"));
    }

    #[test]
    fn test_ttl_expiry() {
        let mut cs = CacheStore::new(10);
        cs.put("a", vec![1], 100);
        cs.advance_time(200);
        assert!(cs.get("a").is_none());
    }

    #[test]
    fn test_evict_expired() {
        let mut cs = CacheStore::new(10);
        cs.put("a", vec![1], 50);
        cs.put("b", vec![2], 0);
        cs.advance_time(100);
        cs.evict_expired();
        assert_eq!(cs.count(), 1);
    }

    #[test]
    fn test_total_bytes() {
        let mut cs = CacheStore::new(10);
        cs.put("a", vec![1, 2, 3], 0);
        cs.put("b", vec![4, 5], 0);
        assert_eq!(cs.total_bytes(), 5);
    }

    #[test]
    fn test_most_accessed() {
        let mut cs = CacheStore::new(10);
        cs.put("a", vec![1], 0);
        cs.put("b", vec![2], 0);
        cs.get("b");
        cs.get("b");
        cs.get("a");
        assert_eq!(cs.most_accessed(), Some("b"));
    }

    #[test]
    fn test_clear() {
        let mut cs = CacheStore::new(10);
        cs.put("a", vec![1], 0);
        cs.clear();
        assert_eq!(cs.count(), 0);
    }
}
