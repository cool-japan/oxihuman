// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

/// A map where each key maps to a group of values (one-to-many).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GroupMap<V: Clone> {
    groups: HashMap<String, Vec<V>>,
}

#[allow(dead_code)]
impl<V: Clone + PartialEq> GroupMap<V> {
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &str, value: V) {
        self.groups
            .entry(key.to_string())
            .or_default()
            .push(value);
    }

    pub fn get(&self, key: &str) -> &[V] {
        self.groups.get(key).map_or(&[], |v| v.as_slice())
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.groups.contains_key(key)
    }

    pub fn contains_value(&self, key: &str, value: &V) -> bool {
        self.groups
            .get(key)
            .is_some_and(|v| v.contains(value))
    }

    pub fn remove_key(&mut self, key: &str) -> Option<Vec<V>> {
        self.groups.remove(key)
    }

    pub fn remove_value(&mut self, key: &str, value: &V) {
        if let Some(v) = self.groups.get_mut(key) {
            v.retain(|x| x != value);
            if v.is_empty() {
                self.groups.remove(key);
            }
        }
    }

    pub fn num_keys(&self) -> usize {
        self.groups.len()
    }

    pub fn total_values(&self) -> usize {
        self.groups.values().map(|v| v.len()).sum()
    }

    pub fn group_size(&self, key: &str) -> usize {
        self.groups.get(key).map_or(0, |v| v.len())
    }

    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    pub fn clear(&mut self) {
        self.groups.clear();
    }

    pub fn keys(&self) -> Vec<&str> {
        self.groups.keys().map(|k| k.as_str()).collect()
    }
}

impl<V: Clone + PartialEq> Default for GroupMap<V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m: GroupMap<i32> = GroupMap::new();
        assert!(m.is_empty());
    }

    #[test]
    fn test_insert_get() {
        let mut m = GroupMap::new();
        m.insert("colors", "red");
        m.insert("colors", "blue");
        assert_eq!(m.get("colors"), &["red", "blue"]);
    }

    #[test]
    fn test_contains_key() {
        let mut m = GroupMap::new();
        m.insert("k", 1);
        assert!(m.contains_key("k"));
        assert!(!m.contains_key("z"));
    }

    #[test]
    fn test_contains_value() {
        let mut m = GroupMap::new();
        m.insert("nums", 10);
        m.insert("nums", 20);
        assert!(m.contains_value("nums", &10));
        assert!(!m.contains_value("nums", &30));
    }

    #[test]
    fn test_remove_key() {
        let mut m = GroupMap::new();
        m.insert("k", 1);
        let removed = m.remove_key("k");
        assert_eq!(removed, Some(vec![1]));
        assert!(m.is_empty());
    }

    #[test]
    fn test_remove_value() {
        let mut m = GroupMap::new();
        m.insert("k", 1);
        m.insert("k", 2);
        m.remove_value("k", &1);
        assert_eq!(m.get("k"), &[2]);
    }

    #[test]
    fn test_remove_value_removes_empty_key() {
        let mut m = GroupMap::new();
        m.insert("k", 1);
        m.remove_value("k", &1);
        assert!(!m.contains_key("k"));
    }

    #[test]
    fn test_num_keys_total_values() {
        let mut m = GroupMap::new();
        m.insert("a", 1);
        m.insert("a", 2);
        m.insert("b", 3);
        assert_eq!(m.num_keys(), 2);
        assert_eq!(m.total_values(), 3);
    }

    #[test]
    fn test_group_size() {
        let mut m = GroupMap::new();
        m.insert("x", 1);
        m.insert("x", 2);
        m.insert("x", 3);
        assert_eq!(m.group_size("x"), 3);
        assert_eq!(m.group_size("y"), 0);
    }

    #[test]
    fn test_clear() {
        let mut m = GroupMap::new();
        m.insert("k", 1);
        m.clear();
        assert!(m.is_empty());
    }
}
