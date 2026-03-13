// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Key-value access tracking map that records read/write counts per key.

use std::collections::HashMap;

/// Tracks per-key access counts (reads and writes).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AccessMap {
    reads: HashMap<String, u64>,
    writes: HashMap<String, u64>,
}

#[allow(dead_code)]
impl AccessMap {
    pub fn new() -> Self {
        Self {
            reads: HashMap::new(),
            writes: HashMap::new(),
        }
    }

    pub fn record_read(&mut self, key: &str) {
        *self.reads.entry(key.to_string()).or_insert(0) += 1;
    }

    pub fn record_write(&mut self, key: &str) {
        *self.writes.entry(key.to_string()).or_insert(0) += 1;
    }

    pub fn read_count(&self, key: &str) -> u64 {
        self.reads.get(key).copied().unwrap_or(0)
    }

    pub fn write_count(&self, key: &str) -> u64 {
        self.writes.get(key).copied().unwrap_or(0)
    }

    pub fn total_accesses(&self, key: &str) -> u64 {
        self.read_count(key) + self.write_count(key)
    }

    pub fn tracked_key_count(&self) -> usize {
        let mut keys: std::collections::HashSet<&String> = self.reads.keys().collect();
        keys.extend(self.writes.keys());
        keys.len()
    }

    pub fn most_read(&self) -> Option<(String, u64)> {
        self.reads
            .iter()
            .max_by_key(|(_, &v)| v)
            .map(|(k, &v)| (k.clone(), v))
    }

    pub fn most_written(&self) -> Option<(String, u64)> {
        self.writes
            .iter()
            .max_by_key(|(_, &v)| v)
            .map(|(k, &v)| (k.clone(), v))
    }

    pub fn clear(&mut self) {
        self.reads.clear();
        self.writes.clear();
    }

    pub fn has_key(&self, key: &str) -> bool {
        self.reads.contains_key(key) || self.writes.contains_key(key)
    }

    pub fn all_keys(&self) -> Vec<String> {
        let mut keys: std::collections::HashSet<String> = self.reads.keys().cloned().collect();
        keys.extend(self.writes.keys().cloned());
        let mut v: Vec<String> = keys.into_iter().collect();
        v.sort();
        v
    }
}

impl Default for AccessMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_map_is_empty() {
        let m = AccessMap::new();
        assert_eq!(m.tracked_key_count(), 0);
    }

    #[test]
    fn record_read_increments() {
        let mut m = AccessMap::new();
        m.record_read("a");
        m.record_read("a");
        assert_eq!(m.read_count("a"), 2);
    }

    #[test]
    fn record_write_increments() {
        let mut m = AccessMap::new();
        m.record_write("b");
        assert_eq!(m.write_count("b"), 1);
    }

    #[test]
    fn total_accesses_sums_reads_writes() {
        let mut m = AccessMap::new();
        m.record_read("x");
        m.record_write("x");
        m.record_write("x");
        assert_eq!(m.total_accesses("x"), 3);
    }

    #[test]
    fn missing_key_returns_zero() {
        let m = AccessMap::new();
        assert_eq!(m.read_count("missing"), 0);
        assert_eq!(m.write_count("missing"), 0);
    }

    #[test]
    fn most_read_returns_highest() {
        let mut m = AccessMap::new();
        m.record_read("a");
        m.record_read("b");
        m.record_read("b");
        let (k, c) = m.most_read().expect("should succeed");
        assert_eq!(k, "b");
        assert_eq!(c, 2);
    }

    #[test]
    fn clear_resets_all() {
        let mut m = AccessMap::new();
        m.record_read("a");
        m.record_write("b");
        m.clear();
        assert_eq!(m.tracked_key_count(), 0);
    }

    #[test]
    fn has_key_checks_both() {
        let mut m = AccessMap::new();
        m.record_write("w");
        assert!(m.has_key("w"));
        assert!(!m.has_key("missing"));
    }

    #[test]
    fn all_keys_sorted() {
        let mut m = AccessMap::new();
        m.record_read("c");
        m.record_write("a");
        m.record_read("b");
        assert_eq!(
            m.all_keys(),
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn default_is_empty() {
        let m = AccessMap::default();
        assert_eq!(m.tracked_key_count(), 0);
    }
}
