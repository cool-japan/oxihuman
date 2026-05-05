// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

/// A table that maps string keys to handler IDs for dispatch.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DispatchTable {
    handlers: HashMap<String, HandlerEntry>,
    next_id: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HandlerEntry {
    pub id: u64,
    pub name: String,
    pub priority: i32,
    pub enabled: bool,
}

impl Default for DispatchTable {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl DispatchTable {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn register(&mut self, key: &str, priority: i32) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.handlers.insert(
            key.to_string(),
            HandlerEntry {
                id,
                name: key.to_string(),
                priority,
                enabled: true,
            },
        );
        id
    }

    pub fn unregister(&mut self, key: &str) -> bool {
        self.handlers.remove(key).is_some()
    }

    pub fn lookup(&self, key: &str) -> Option<u64> {
        self.handlers.get(key).filter(|h| h.enabled).map(|h| h.id)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.handlers.contains_key(key)
    }

    pub fn set_enabled(&mut self, key: &str, enabled: bool) -> bool {
        if let Some(h) = self.handlers.get_mut(key) {
            h.enabled = enabled;
            true
        } else {
            false
        }
    }

    pub fn is_enabled(&self, key: &str) -> bool {
        self.handlers.get(key).is_some_and(|h| h.enabled)
    }

    pub fn count(&self) -> usize {
        self.handlers.len()
    }

    pub fn enabled_count(&self) -> usize {
        self.handlers.values().filter(|h| h.enabled).count()
    }

    pub fn keys(&self) -> Vec<&str> {
        self.handlers.keys().map(|k| k.as_str()).collect()
    }

    pub fn by_priority(&self) -> Vec<&HandlerEntry> {
        let mut entries: Vec<&HandlerEntry> = self.handlers.values().collect();
        entries.sort_by_key(|b| std::cmp::Reverse(b.priority));
        entries
    }

    pub fn clear(&mut self) {
        self.handlers.clear();
    }

    pub fn get_entry(&self, key: &str) -> Option<&HandlerEntry> {
        self.handlers.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let dt = DispatchTable::new();
        assert_eq!(dt.count(), 0);
    }

    #[test]
    fn test_register_and_lookup() {
        let mut dt = DispatchTable::new();
        let id = dt.register("click", 0);
        assert_eq!(dt.lookup("click"), Some(id));
    }

    #[test]
    fn test_unregister() {
        let mut dt = DispatchTable::new();
        dt.register("click", 0);
        assert!(dt.unregister("click"));
        assert!(!dt.contains("click"));
    }

    #[test]
    fn test_disable() {
        let mut dt = DispatchTable::new();
        dt.register("click", 0);
        dt.set_enabled("click", false);
        assert!(dt.lookup("click").is_none());
        assert!(!dt.is_enabled("click"));
    }

    #[test]
    fn test_contains() {
        let mut dt = DispatchTable::new();
        dt.register("a", 0);
        assert!(dt.contains("a"));
        assert!(!dt.contains("b"));
    }

    #[test]
    fn test_by_priority() {
        let mut dt = DispatchTable::new();
        dt.register("low", 1);
        dt.register("high", 10);
        dt.register("mid", 5);
        let sorted = dt.by_priority();
        assert_eq!(sorted[0].name, "high");
    }

    #[test]
    fn test_enabled_count() {
        let mut dt = DispatchTable::new();
        dt.register("a", 0);
        dt.register("b", 0);
        dt.set_enabled("b", false);
        assert_eq!(dt.enabled_count(), 1);
    }

    #[test]
    fn test_clear() {
        let mut dt = DispatchTable::new();
        dt.register("a", 0);
        dt.clear();
        assert_eq!(dt.count(), 0);
    }

    #[test]
    fn test_unique_ids() {
        let mut dt = DispatchTable::new();
        let id1 = dt.register("a", 0);
        let id2 = dt.register("b", 0);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_get_entry() {
        let mut dt = DispatchTable::new();
        dt.register("x", 42);
        let entry = dt.get_entry("x").expect("should succeed");
        assert_eq!(entry.priority, 42);
    }
}
