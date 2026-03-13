// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Layered configuration: base layer overridden by user layer.

use std::collections::HashMap;

/// A single configuration layer mapping string keys to string values.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ConfigLayer {
    entries: HashMap<String, String>,
    name: String,
}

#[allow(dead_code)]
impl ConfigLayer {
    pub fn new(name: &str) -> Self {
        Self {
            entries: HashMap::new(),
            name: name.to_string(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.entries.insert(key.to_string(), value.to_string());
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(|v| v.as_str())
    }

    pub fn remove(&mut self, key: &str) -> bool {
        self.entries.remove(key).is_some()
    }

    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    pub fn count(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn keys(&self) -> Vec<&str> {
        self.entries.keys().map(|k| k.as_str()).collect()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

/// Stack of config layers resolved top-down (last layer wins).
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct LayeredConfig {
    layers: Vec<ConfigLayer>,
}

#[allow(dead_code)]
impl LayeredConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_layer(&mut self, layer: ConfigLayer) {
        self.layers.push(layer);
    }

    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Resolve key: last layer that has it wins.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.layers.iter().rev().find_map(|l| l.get(key))
    }

    pub fn get_or<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.get(key).unwrap_or(default)
    }

    pub fn has_key(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    /// All unique keys across all layers.
    pub fn all_keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self
            .layers
            .iter()
            .flat_map(|l| l.keys().into_iter().map(|s| s.to_string()))
            .collect();
        keys.sort();
        keys.dedup();
        keys
    }

    pub fn pop_layer(&mut self) -> Option<ConfigLayer> {
        self.layers.pop()
    }

    pub fn clear(&mut self) {
        self.layers.clear();
    }

    /// Flatten all layers into a single resolved map.
    pub fn flatten(&self) -> HashMap<String, String> {
        let mut out = HashMap::new();
        for layer in &self.layers {
            for key in layer.keys() {
                if let Some(v) = layer.get(key) {
                    out.insert(key.to_string(), v.to_string());
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_set_get() {
        let mut l = ConfigLayer::new("base");
        l.set("color", "red");
        assert_eq!(l.get("color"), Some("red"));
    }

    #[test]
    fn layer_name() {
        let l = ConfigLayer::new("user");
        assert_eq!(l.name(), "user");
    }

    #[test]
    fn layer_remove() {
        let mut l = ConfigLayer::new("test");
        l.set("k", "v");
        assert!(l.remove("k"));
        assert!(!l.contains("k"));
    }

    #[test]
    fn layered_resolution_last_wins() {
        let mut base = ConfigLayer::new("base");
        base.set("size", "10");
        let mut user = ConfigLayer::new("user");
        user.set("size", "20");

        let mut lc = LayeredConfig::new();
        lc.push_layer(base);
        lc.push_layer(user);
        assert_eq!(lc.get("size"), Some("20"));
    }

    #[test]
    fn layered_fallback_to_base() {
        let mut base = ConfigLayer::new("base");
        base.set("mode", "auto");
        let user = ConfigLayer::new("user");

        let mut lc = LayeredConfig::new();
        lc.push_layer(base);
        lc.push_layer(user);
        assert_eq!(lc.get("mode"), Some("auto"));
    }

    #[test]
    fn get_or_default() {
        let lc = LayeredConfig::new();
        assert_eq!(lc.get_or("missing", "default"), "default");
    }

    #[test]
    fn all_keys_deduped() {
        let mut base = ConfigLayer::new("base");
        base.set("a", "1");
        base.set("b", "2");
        let mut user = ConfigLayer::new("user");
        user.set("b", "3");
        user.set("c", "4");

        let mut lc = LayeredConfig::new();
        lc.push_layer(base);
        lc.push_layer(user);
        let keys = lc.all_keys();
        assert_eq!(keys, vec!["a", "b", "c"]);
    }

    #[test]
    fn flatten_resolves_correctly() {
        let mut base = ConfigLayer::new("base");
        base.set("x", "1");
        let mut user = ConfigLayer::new("user");
        user.set("x", "2");
        user.set("y", "3");

        let mut lc = LayeredConfig::new();
        lc.push_layer(base);
        lc.push_layer(user);
        let flat = lc.flatten();
        assert_eq!(flat.get("x").expect("should succeed"), "2");
        assert_eq!(flat.get("y").expect("should succeed"), "3");
    }

    #[test]
    fn pop_layer() {
        let mut lc = LayeredConfig::new();
        lc.push_layer(ConfigLayer::new("a"));
        lc.push_layer(ConfigLayer::new("b"));
        let popped = lc.pop_layer().expect("should succeed");
        assert_eq!(popped.name(), "b");
        assert_eq!(lc.layer_count(), 1);
    }

    #[test]
    fn clear_all_layers() {
        let mut lc = LayeredConfig::new();
        lc.push_layer(ConfigLayer::new("x"));
        lc.clear();
        assert_eq!(lc.layer_count(), 0);
    }
}
