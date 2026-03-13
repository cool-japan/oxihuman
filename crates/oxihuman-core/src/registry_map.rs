// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Named registry map with category support.

use std::collections::HashMap;

/// A registered item with metadata.
#[derive(Debug, Clone)]
pub struct RegistryItem<V> {
    pub value: V,
    pub category: String,
    pub description: String,
}

/// Registry mapping string keys to typed items with categories.
pub struct RegistryMap<V> {
    items: HashMap<String, RegistryItem<V>>,
}

#[allow(dead_code)]
impl<V: Clone> RegistryMap<V> {
    pub fn new() -> Self {
        RegistryMap {
            items: HashMap::new(),
        }
    }

    pub fn register(&mut self, key: &str, value: V, category: &str, description: &str) -> bool {
        if self.items.contains_key(key) {
            return false;
        }
        self.items.insert(
            key.to_string(),
            RegistryItem {
                value,
                category: category.to_string(),
                description: description.to_string(),
            },
        );
        true
    }

    pub fn get(&self, key: &str) -> Option<&V> {
        self.items.get(key).map(|i| &i.value)
    }

    pub fn unregister(&mut self, key: &str) -> bool {
        self.items.remove(key).is_some()
    }

    pub fn contains(&self, key: &str) -> bool {
        self.items.contains_key(key)
    }

    pub fn by_category(&self, cat: &str) -> Vec<(&str, &V)> {
        self.items
            .iter()
            .filter(|(_, i)| i.category == cat)
            .map(|(k, i)| (k.as_str(), &i.value))
            .collect()
    }

    pub fn categories(&self) -> Vec<String> {
        let mut cats: Vec<String> = self.items.values().map(|i| i.category.clone()).collect();
        cats.sort();
        cats.dedup();
        cats
    }

    pub fn description(&self, key: &str) -> Option<&str> {
        self.items.get(key).map(|i| i.description.as_str())
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn keys(&self) -> Vec<&str> {
        self.items.keys().map(|k| k.as_str()).collect()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn count_in_category(&self, cat: &str) -> usize {
        self.items.values().filter(|i| i.category == cat).count()
    }
}

impl<V: Clone> Default for RegistryMap<V> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_registry_map<V: Clone>() -> RegistryMap<V> {
    RegistryMap::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_get() {
        let mut m: RegistryMap<i32> = new_registry_map();
        assert!(m.register("a", 1, "cat1", "desc a"));
        assert_eq!(*m.get("a").expect("should succeed"), 1);
    }

    #[test]
    fn duplicate_register_fails() {
        let mut m: RegistryMap<i32> = new_registry_map();
        m.register("a", 1, "c", "d");
        assert!(!m.register("a", 2, "c", "d"));
    }

    #[test]
    fn unregister() {
        let mut m: RegistryMap<i32> = new_registry_map();
        m.register("x", 5, "c", "d");
        assert!(m.unregister("x"));
        assert!(m.get("x").is_none());
    }

    #[test]
    fn by_category() {
        let mut m: RegistryMap<i32> = new_registry_map();
        m.register("a", 1, "shapes", "");
        m.register("b", 2, "shapes", "");
        m.register("c", 3, "colors", "");
        assert_eq!(m.by_category("shapes").len(), 2);
    }

    #[test]
    fn categories() {
        let mut m: RegistryMap<i32> = new_registry_map();
        m.register("a", 1, "alpha", "");
        m.register("b", 2, "beta", "");
        m.register("c", 3, "alpha", "");
        let cats = m.categories();
        assert_eq!(cats.len(), 2);
    }

    #[test]
    fn description_lookup() {
        let mut m: RegistryMap<i32> = new_registry_map();
        m.register("k", 1, "c", "my description");
        assert_eq!(m.description("k"), Some("my description"));
    }

    #[test]
    fn count_in_category() {
        let mut m: RegistryMap<i32> = new_registry_map();
        m.register("a", 1, "x", "");
        m.register("b", 2, "x", "");
        m.register("c", 3, "y", "");
        assert_eq!(m.count_in_category("x"), 2);
    }

    #[test]
    fn clear_empties() {
        let mut m: RegistryMap<i32> = new_registry_map();
        m.register("a", 1, "c", "");
        m.clear();
        assert!(m.is_empty());
    }

    #[test]
    fn len_tracking() {
        let mut m: RegistryMap<i32> = new_registry_map();
        m.register("a", 1, "c", "");
        m.register("b", 2, "c", "");
        assert_eq!(m.len(), 2);
    }
}
