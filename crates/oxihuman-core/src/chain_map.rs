// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

/// A chain of maps where lookup falls through parent layers.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChainMap {
    layers: Vec<HashMap<String, String>>,
}

impl Default for ChainMap {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl ChainMap {
    pub fn new() -> Self {
        Self {
            layers: vec![HashMap::new()],
        }
    }

    pub fn push_layer(&mut self) {
        self.layers.push(HashMap::new());
    }

    pub fn pop_layer(&mut self) -> bool {
        if self.layers.len() > 1 {
            self.layers.pop();
            true
        } else {
            false
        }
    }

    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    pub fn set(&mut self, key: &str, value: &str) {
        if let Some(top) = self.layers.last_mut() {
            top.insert(key.to_string(), value.to_string());
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        for layer in self.layers.iter().rev() {
            if let Some(v) = layer.get(key) {
                return Some(v.as_str());
            }
        }
        None
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    pub fn remove_from_top(&mut self, key: &str) -> bool {
        self.layers
            .last_mut()
            .is_some_and(|top| top.remove(key).is_some())
    }

    pub fn top_layer_count(&self) -> usize {
        self.layers.last().map_or(0, |l| l.len())
    }

    pub fn total_entries(&self) -> usize {
        self.layers.iter().map(|l| l.len()).sum()
    }

    pub fn all_keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.layers.iter().flat_map(|l| l.keys().cloned()).collect();
        keys.sort();
        keys.dedup();
        keys
    }

    pub fn flatten(&self) -> HashMap<String, String> {
        let mut result = HashMap::new();
        for layer in &self.layers {
            for (k, v) in layer {
                result.insert(k.clone(), v.clone());
            }
        }
        result
    }

    pub fn clear_top(&mut self) {
        if let Some(top) = self.layers.last_mut() {
            top.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let cm = ChainMap::new();
        assert_eq!(cm.layer_count(), 1);
    }

    #[test]
    fn test_set_and_get() {
        let mut cm = ChainMap::new();
        cm.set("k", "v");
        assert_eq!(cm.get("k"), Some("v"));
    }

    #[test]
    fn test_layer_override() {
        let mut cm = ChainMap::new();
        cm.set("k", "base");
        cm.push_layer();
        cm.set("k", "override");
        assert_eq!(cm.get("k"), Some("override"));
    }

    #[test]
    fn test_fallthrough() {
        let mut cm = ChainMap::new();
        cm.set("k", "base");
        cm.push_layer();
        assert_eq!(cm.get("k"), Some("base"));
    }

    #[test]
    fn test_pop_layer() {
        let mut cm = ChainMap::new();
        cm.set("k", "base");
        cm.push_layer();
        cm.set("k", "top");
        assert!(cm.pop_layer());
        assert_eq!(cm.get("k"), Some("base"));
    }

    #[test]
    fn test_cannot_pop_last() {
        let mut cm = ChainMap::new();
        assert!(!cm.pop_layer());
    }

    #[test]
    fn test_contains_key() {
        let mut cm = ChainMap::new();
        assert!(!cm.contains_key("x"));
        cm.set("x", "y");
        assert!(cm.contains_key("x"));
    }

    #[test]
    fn test_all_keys() {
        let mut cm = ChainMap::new();
        cm.set("b", "1");
        cm.push_layer();
        cm.set("a", "2");
        let keys = cm.all_keys();
        assert_eq!(keys, vec!["a", "b"]);
    }

    #[test]
    fn test_flatten() {
        let mut cm = ChainMap::new();
        cm.set("a", "1");
        cm.push_layer();
        cm.set("a", "2");
        cm.set("b", "3");
        let flat = cm.flatten();
        assert_eq!(flat["a"], "2");
        assert_eq!(flat["b"], "3");
    }

    #[test]
    fn test_remove_from_top() {
        let mut cm = ChainMap::new();
        cm.set("a", "1");
        cm.push_layer();
        cm.set("a", "2");
        assert!(cm.remove_from_top("a"));
        assert_eq!(cm.get("a"), Some("1"));
    }
}
