// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A map that tracks one selected/active key among its entries.

use std::collections::HashMap;

/// A map with a notion of an "active" or selected entry.
#[allow(dead_code)]
pub struct SelectorMap<V> {
    entries: HashMap<String, V>,
    selected: Option<String>,
    change_count: u64,
}

#[allow(dead_code)]
impl<V: Clone> SelectorMap<V> {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            selected: None,
            change_count: 0,
        }
    }

    pub fn insert(&mut self, key: &str, value: V) {
        self.entries.insert(key.to_string(), value);
        self.change_count += 1;
    }

    pub fn remove(&mut self, key: &str) -> bool {
        if self.entries.remove(key).is_some() {
            if self.selected.as_deref() == Some(key) {
                self.selected = None;
            }
            self.change_count += 1;
            true
        } else {
            false
        }
    }

    pub fn select(&mut self, key: &str) -> bool {
        if self.entries.contains_key(key) {
            self.selected = Some(key.to_string());
            self.change_count += 1;
            true
        } else {
            false
        }
    }

    pub fn deselect(&mut self) {
        self.selected = None;
    }

    pub fn selected_key(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    pub fn selected_value(&self) -> Option<&V> {
        self.selected.as_ref().and_then(|k| self.entries.get(k))
    }

    pub fn get(&self, key: &str) -> Option<&V> {
        self.entries.get(key)
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

    pub fn keys(&self) -> Vec<&str> {
        let mut v: Vec<&str> = self.entries.keys().map(|s| s.as_str()).collect();
        v.sort_unstable();
        v
    }

    pub fn change_count(&self) -> u64 {
        self.change_count
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.selected = None;
        self.change_count += 1;
    }
}

impl<V: Clone> Default for SelectorMap<V> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_selector_map<V: Clone>() -> SelectorMap<V> {
    SelectorMap::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut m: SelectorMap<i32> = new_selector_map();
        m.insert("a", 1);
        assert_eq!(m.get("a"), Some(&1));
    }

    #[test]
    fn select_returns_value() {
        let mut m: SelectorMap<i32> = new_selector_map();
        m.insert("x", 42);
        assert!(m.select("x"));
        assert_eq!(m.selected_value(), Some(&42));
    }

    #[test]
    fn select_nonexistent_fails() {
        let mut m: SelectorMap<i32> = new_selector_map();
        assert!(!m.select("missing"));
        assert!(m.selected_key().is_none());
    }

    #[test]
    fn deselect_clears() {
        let mut m: SelectorMap<i32> = new_selector_map();
        m.insert("a", 1);
        m.select("a");
        m.deselect();
        assert!(m.selected_key().is_none());
    }

    #[test]
    fn remove_clears_selection() {
        let mut m: SelectorMap<i32> = new_selector_map();
        m.insert("a", 5);
        m.select("a");
        m.remove("a");
        assert!(m.selected_key().is_none());
    }

    #[test]
    fn len_and_is_empty() {
        let mut m: SelectorMap<u8> = new_selector_map();
        assert!(m.is_empty());
        m.insert("k", 1);
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn change_count_increments() {
        let mut m: SelectorMap<i32> = new_selector_map();
        let before = m.change_count();
        m.insert("a", 1);
        assert!(m.change_count() > before);
    }

    #[test]
    fn clear_removes_all() {
        let mut m: SelectorMap<i32> = new_selector_map();
        m.insert("a", 1);
        m.select("a");
        m.clear();
        assert!(m.is_empty());
        assert!(m.selected_key().is_none());
    }

    #[test]
    fn keys_sorted() {
        let mut m: SelectorMap<i32> = new_selector_map();
        m.insert("b", 2);
        m.insert("a", 1);
        assert_eq!(m.keys(), vec!["a", "b"]);
    }

    #[test]
    fn contains_check() {
        let mut m: SelectorMap<i32> = new_selector_map();
        m.insert("z", 9);
        assert!(m.contains("z"));
        assert!(!m.contains("nope"));
    }
}
