// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A registry of named callbacks identified by string keys.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CallbackEntry {
    pub name: String,
    pub priority: i32,
    pub enabled: bool,
}

#[allow(dead_code)]
impl CallbackEntry {
    pub fn new(name: &str, priority: i32) -> Self {
        Self {
            name: name.to_string(),
            priority,
            enabled: true,
        }
    }
}

/// Manages a collection of named callback entries with priority ordering.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct CallbackRegistry {
    entries: Vec<CallbackEntry>,
}

#[allow(dead_code)]
impl CallbackRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, name: &str, priority: i32) -> usize {
        let id = self.entries.len();
        self.entries.push(CallbackEntry::new(name, priority));
        id
    }

    pub fn unregister(&mut self, name: &str) -> bool {
        let before = self.entries.len();
        self.entries.retain(|e| e.name != name);
        self.entries.len() < before
    }

    pub fn get(&self, name: &str) -> Option<&CallbackEntry> {
        self.entries.iter().find(|e| e.name == name)
    }

    pub fn set_enabled(&mut self, name: &str, enabled: bool) -> bool {
        if let Some(e) = self.entries.iter_mut().find(|e| e.name == name) {
            e.enabled = enabled;
            true
        } else {
            false
        }
    }

    pub fn enabled_entries(&self) -> Vec<&CallbackEntry> {
        let mut v: Vec<_> = self.entries.iter().filter(|e| e.enabled).collect();
        v.sort_by(|a, b| b.priority.cmp(&a.priority));
        v
    }

    pub fn count(&self) -> usize {
        self.entries.len()
    }

    pub fn enabled_count(&self) -> usize {
        self.entries.iter().filter(|e| e.enabled).count()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn contains(&self, name: &str) -> bool {
        self.entries.iter().any(|e| e.name == name)
    }

    pub fn names(&self) -> Vec<&str> {
        self.entries.iter().map(|e| e.name.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register() {
        let mut reg = CallbackRegistry::new();
        let id = reg.register("on_click", 0);
        assert_eq!(id, 0);
        assert_eq!(reg.count(), 1);
    }

    #[test]
    fn test_unregister() {
        let mut reg = CallbackRegistry::new();
        reg.register("a", 0);
        assert!(reg.unregister("a"));
        assert!(!reg.unregister("a"));
    }

    #[test]
    fn test_get() {
        let mut reg = CallbackRegistry::new();
        reg.register("ev", 5);
        let entry = reg.get("ev").expect("should succeed");
        assert_eq!(entry.priority, 5);
    }

    #[test]
    fn test_set_enabled() {
        let mut reg = CallbackRegistry::new();
        reg.register("x", 0);
        reg.set_enabled("x", false);
        assert_eq!(reg.enabled_count(), 0);
    }

    #[test]
    fn test_enabled_entries_sorted() {
        let mut reg = CallbackRegistry::new();
        reg.register("low", 1);
        reg.register("high", 10);
        let enabled = reg.enabled_entries();
        assert_eq!(enabled[0].name, "high");
        assert_eq!(enabled[1].name, "low");
    }

    #[test]
    fn test_clear() {
        let mut reg = CallbackRegistry::new();
        reg.register("a", 0);
        reg.register("b", 0);
        reg.clear();
        assert!(reg.count() == 0);
    }

    #[test]
    fn test_contains() {
        let mut reg = CallbackRegistry::new();
        reg.register("test", 0);
        assert!(reg.contains("test"));
        assert!(!reg.contains("nope"));
    }

    #[test]
    fn test_names() {
        let mut reg = CallbackRegistry::new();
        reg.register("alpha", 0);
        reg.register("beta", 0);
        let names = reg.names();
        assert!(names.contains(&"alpha"));
        assert!(names.contains(&"beta"));
    }

    #[test]
    fn test_default() {
        let reg = CallbackRegistry::default();
        assert!(reg.count() == 0);
    }

    #[test]
    fn test_set_enabled_missing() {
        let mut reg = CallbackRegistry::new();
        assert!(!reg.set_enabled("missing", true));
    }
}
