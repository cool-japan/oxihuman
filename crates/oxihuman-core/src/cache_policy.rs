// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Eviction policies for cache systems (LRU, FIFO, LFU scoring).

/// Eviction policy kind.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyKind {
    Lru,
    Fifo,
    Lfu,
}

/// An entry tracked by the cache policy.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PolicyEntry {
    pub key: String,
    pub insert_order: u64,
    pub last_access: u64,
    pub access_count: u64,
}

/// Cache eviction policy tracker.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CachePolicy {
    kind: PolicyKind,
    entries: Vec<PolicyEntry>,
    counter: u64,
}

#[allow(dead_code)]
impl CachePolicy {
    pub fn new(kind: PolicyKind) -> Self {
        Self {
            kind,
            entries: Vec::new(),
            counter: 0,
        }
    }

    pub fn insert(&mut self, key: &str) {
        self.counter += 1;
        if let Some(e) = self.entries.iter_mut().find(|e| e.key == key) {
            e.last_access = self.counter;
            e.access_count += 1;
        } else {
            self.entries.push(PolicyEntry {
                key: key.to_string(),
                insert_order: self.counter,
                last_access: self.counter,
                access_count: 1,
            });
        }
    }

    pub fn touch(&mut self, key: &str) {
        self.counter += 1;
        if let Some(e) = self.entries.iter_mut().find(|e| e.key == key) {
            e.last_access = self.counter;
            e.access_count += 1;
        }
    }

    pub fn evict_candidate(&self) -> Option<&str> {
        if self.entries.is_empty() {
            return None;
        }
        let best = match self.kind {
            PolicyKind::Lru => self.entries.iter().min_by_key(|e| e.last_access),
            PolicyKind::Fifo => self.entries.iter().min_by_key(|e| e.insert_order),
            PolicyKind::Lfu => self.entries.iter().min_by_key(|e| e.access_count),
        };
        best.map(|e| e.key.as_str())
    }

    pub fn remove(&mut self, key: &str) {
        self.entries.retain(|e| e.key != key);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn kind(&self) -> PolicyKind {
        self.kind
    }

    pub fn contains(&self, key: &str) -> bool {
        self.entries.iter().any(|e| e.key == key)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.counter = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_policy_empty() {
        let p = CachePolicy::new(PolicyKind::Lru);
        assert!(p.is_empty());
    }

    #[test]
    fn insert_adds_entry() {
        let mut p = CachePolicy::new(PolicyKind::Lru);
        p.insert("a");
        assert_eq!(p.len(), 1);
        assert!(p.contains("a"));
    }

    #[test]
    fn lru_evicts_least_recently_used() {
        let mut p = CachePolicy::new(PolicyKind::Lru);
        p.insert("a");
        p.insert("b");
        p.touch("a");
        assert_eq!(p.evict_candidate(), Some("b"));
    }

    #[test]
    fn fifo_evicts_oldest() {
        let mut p = CachePolicy::new(PolicyKind::Fifo);
        p.insert("first");
        p.insert("second");
        assert_eq!(p.evict_candidate(), Some("first"));
    }

    #[test]
    fn lfu_evicts_least_frequent() {
        let mut p = CachePolicy::new(PolicyKind::Lfu);
        p.insert("a");
        p.insert("b");
        p.touch("a");
        p.touch("a");
        assert_eq!(p.evict_candidate(), Some("b"));
    }

    #[test]
    fn remove_entry() {
        let mut p = CachePolicy::new(PolicyKind::Lru);
        p.insert("x");
        p.remove("x");
        assert!(p.is_empty());
    }

    #[test]
    fn clear_resets() {
        let mut p = CachePolicy::new(PolicyKind::Lru);
        p.insert("a");
        p.insert("b");
        p.clear();
        assert!(p.is_empty());
    }

    #[test]
    fn evict_candidate_empty() {
        let p = CachePolicy::new(PolicyKind::Lru);
        assert!(p.evict_candidate().is_none());
    }

    #[test]
    fn kind_returns_policy() {
        let p = CachePolicy::new(PolicyKind::Fifo);
        assert_eq!(p.kind(), PolicyKind::Fifo);
    }

    #[test]
    fn duplicate_insert_updates() {
        let mut p = CachePolicy::new(PolicyKind::Lfu);
        p.insert("a");
        p.insert("a");
        assert_eq!(p.len(), 1);
    }
}
