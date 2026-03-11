// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A mapping from string action names to callbacks represented as closures.
///
/// `ActionMap` stores named actions that can be invoked by name,
/// enabling decoupled input-to-action dispatch.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ActionMap {
    entries: Vec<ActionEntry>,
    enabled: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ActionEntry {
    pub name: String,
    pub category: String,
    pub priority: u32,
    pub enabled: bool,
}

impl Default for ActionMap {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl ActionMap {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            enabled: true,
        }
    }

    pub fn register(&mut self, name: &str, category: &str, priority: u32) -> bool {
        if self.entries.iter().any(|e| e.name == name) {
            return false;
        }
        self.entries.push(ActionEntry {
            name: name.to_string(),
            category: category.to_string(),
            priority,
            enabled: true,
        });
        true
    }

    pub fn unregister(&mut self, name: &str) -> bool {
        let before = self.entries.len();
        self.entries.retain(|e| e.name != name);
        self.entries.len() < before
    }

    pub fn get(&self, name: &str) -> Option<&ActionEntry> {
        self.entries.iter().find(|e| e.name == name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.entries.iter().any(|e| e.name == name)
    }

    pub fn set_enabled(&mut self, name: &str, enabled: bool) -> bool {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.name == name) {
            entry.enabled = enabled;
            true
        } else {
            false
        }
    }

    pub fn is_enabled(&self, name: &str) -> bool {
        self.entries
            .iter()
            .find(|e| e.name == name)
            .is_some_and(|e| e.enabled)
    }

    pub fn count(&self) -> usize {
        self.entries.len()
    }

    pub fn by_category(&self, category: &str) -> Vec<&ActionEntry> {
        self.entries
            .iter()
            .filter(|e| e.category == category)
            .collect()
    }

    pub fn by_priority(&self) -> Vec<&ActionEntry> {
        let mut sorted: Vec<&ActionEntry> = self.entries.iter().collect();
        sorted.sort_by(|a, b| b.priority.cmp(&a.priority));
        sorted
    }

    pub fn categories(&self) -> Vec<String> {
        let mut cats: Vec<String> = self.entries.iter().map(|e| e.category.clone()).collect();
        cats.sort();
        cats.dedup();
        cats
    }

    pub fn enabled_count(&self) -> usize {
        self.entries.iter().filter(|e| e.enabled).count()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn names(&self) -> Vec<&str> {
        self.entries.iter().map(|e| e.name.as_str()).collect()
    }

    pub fn set_global_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_global_enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let map = ActionMap::new();
        assert_eq!(map.count(), 0);
        assert!(map.is_global_enabled());
    }

    #[test]
    fn test_register_and_get() {
        let mut map = ActionMap::new();
        assert!(map.register("jump", "movement", 10));
        assert!(map.contains("jump"));
        let entry = map.get("jump").unwrap();
        assert_eq!(entry.category, "movement");
        assert_eq!(entry.priority, 10);
    }

    #[test]
    fn test_duplicate_register() {
        let mut map = ActionMap::new();
        assert!(map.register("fire", "combat", 5));
        assert!(!map.register("fire", "combat", 5));
    }

    #[test]
    fn test_unregister() {
        let mut map = ActionMap::new();
        map.register("run", "movement", 1);
        assert!(map.unregister("run"));
        assert!(!map.contains("run"));
        assert!(!map.unregister("run"));
    }

    #[test]
    fn test_enable_disable() {
        let mut map = ActionMap::new();
        map.register("crouch", "movement", 2);
        assert!(map.is_enabled("crouch"));
        map.set_enabled("crouch", false);
        assert!(!map.is_enabled("crouch"));
    }

    #[test]
    fn test_by_category() {
        let mut map = ActionMap::new();
        map.register("jump", "movement", 10);
        map.register("fire", "combat", 5);
        map.register("run", "movement", 8);
        assert_eq!(map.by_category("movement").len(), 2);
        assert_eq!(map.by_category("combat").len(), 1);
    }

    #[test]
    fn test_by_priority() {
        let mut map = ActionMap::new();
        map.register("a", "cat", 1);
        map.register("b", "cat", 10);
        map.register("c", "cat", 5);
        let sorted = map.by_priority();
        assert_eq!(sorted[0].name, "b");
        assert_eq!(sorted[2].name, "a");
    }

    #[test]
    fn test_categories() {
        let mut map = ActionMap::new();
        map.register("a", "x", 1);
        map.register("b", "y", 2);
        map.register("c", "x", 3);
        let cats = map.categories();
        assert_eq!(cats.len(), 2);
    }

    #[test]
    fn test_clear() {
        let mut map = ActionMap::new();
        map.register("a", "b", 1);
        map.clear();
        assert_eq!(map.count(), 0);
    }

    #[test]
    fn test_enabled_count() {
        let mut map = ActionMap::new();
        map.register("a", "c", 1);
        map.register("b", "c", 2);
        map.set_enabled("b", false);
        assert_eq!(map.enabled_count(), 1);
    }
}
