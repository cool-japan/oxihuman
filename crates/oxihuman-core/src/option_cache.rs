// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

/// Cache that stores optional values (present or absent).
#[allow(dead_code)]
pub struct OptionCache {
    present: HashMap<String, String>,
    absent: std::collections::HashSet<String>,
    hits: u64,
    misses: u64,
}

#[allow(dead_code)]
impl OptionCache {
    pub fn new() -> Self {
        Self {
            present: HashMap::new(),
            absent: std::collections::HashSet::new(),
            hits: 0,
            misses: 0,
        }
    }
    pub fn set_some(&mut self, key: &str, value: &str) {
        self.absent.remove(key);
        self.present.insert(key.to_string(), value.to_string());
    }
    pub fn set_none(&mut self, key: &str) {
        self.present.remove(key);
        self.absent.insert(key.to_string());
    }
    pub fn get(&mut self, key: &str) -> Option<Option<&str>> {
        if let Some(v) = self.present.get(key) {
            self.hits += 1;
            Some(Some(v.as_str()))
        } else if self.absent.contains(key) {
            self.hits += 1;
            Some(None)
        } else {
            self.misses += 1;
            None
        }
    }
    pub fn has_key(&self, key: &str) -> bool {
        self.present.contains_key(key) || self.absent.contains(key)
    }
    pub fn remove(&mut self, key: &str) -> bool {
        self.present.remove(key).is_some() || self.absent.remove(key)
    }
    pub fn len(&self) -> usize {
        self.present.len() + self.absent.len()
    }
    pub fn is_empty(&self) -> bool {
        self.present.is_empty() && self.absent.is_empty()
    }
    pub fn hits(&self) -> u64 {
        self.hits
    }
    pub fn misses(&self) -> u64 {
        self.misses
    }
    pub fn hit_rate(&self) -> f32 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f32 / total as f32
        }
    }
    pub fn clear(&mut self) {
        self.present.clear();
        self.absent.clear();
    }
    pub fn present_count(&self) -> usize {
        self.present.len()
    }
    pub fn absent_count(&self) -> usize {
        self.absent.len()
    }
}

impl Default for OptionCache {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
pub fn new_option_cache() -> OptionCache {
    OptionCache::new()
}
#[allow(dead_code)]
pub fn oc_set_some(c: &mut OptionCache, key: &str, value: &str) {
    c.set_some(key, value);
}
#[allow(dead_code)]
pub fn oc_set_none(c: &mut OptionCache, key: &str) {
    c.set_none(key);
}
#[allow(dead_code)]
pub fn oc_get<'a>(c: &'a mut OptionCache, key: &str) -> Option<Option<&'a str>> {
    c.get(key)
}
#[allow(dead_code)]
pub fn oc_has_key(c: &OptionCache, key: &str) -> bool {
    c.has_key(key)
}
#[allow(dead_code)]
pub fn oc_remove(c: &mut OptionCache, key: &str) -> bool {
    c.remove(key)
}
#[allow(dead_code)]
pub fn oc_len(c: &OptionCache) -> usize {
    c.len()
}
#[allow(dead_code)]
pub fn oc_is_empty(c: &OptionCache) -> bool {
    c.is_empty()
}
#[allow(dead_code)]
pub fn oc_hit_rate(c: &OptionCache) -> f32 {
    c.hit_rate()
}
#[allow(dead_code)]
pub fn oc_clear(c: &mut OptionCache) {
    c.clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_set_some_get() {
        let mut c = new_option_cache();
        oc_set_some(&mut c, "k", "v");
        assert_eq!(oc_get(&mut c, "k"), Some(Some("v")));
    }
    #[test]
    fn test_set_none_get() {
        let mut c = new_option_cache();
        oc_set_none(&mut c, "k");
        assert_eq!(oc_get(&mut c, "k"), Some(None));
    }
    #[test]
    fn test_miss() {
        let mut c = new_option_cache();
        assert_eq!(oc_get(&mut c, "missing"), None);
    }
    #[test]
    fn test_has_key() {
        let mut c = new_option_cache();
        oc_set_some(&mut c, "a", "1");
        assert!(oc_has_key(&c, "a"));
        assert!(!oc_has_key(&c, "b"));
    }
    #[test]
    fn test_remove() {
        let mut c = new_option_cache();
        oc_set_some(&mut c, "x", "y");
        assert!(oc_remove(&mut c, "x"));
        assert!(!oc_has_key(&c, "x"));
    }
    #[test]
    fn test_hit_rate() {
        let mut c = new_option_cache();
        oc_set_some(&mut c, "k", "v");
        oc_get(&mut c, "k");
        oc_get(&mut c, "missing");
        let r = oc_hit_rate(&c);
        assert!((0.0..=1.0).contains(&r));
    }
    #[test]
    fn test_len() {
        let mut c = new_option_cache();
        oc_set_some(&mut c, "a", "1");
        oc_set_none(&mut c, "b");
        assert_eq!(oc_len(&c), 2);
    }
    #[test]
    fn test_clear() {
        let mut c = new_option_cache();
        oc_set_some(&mut c, "a", "1");
        oc_clear(&mut c);
        assert!(oc_is_empty(&c));
    }
    #[test]
    fn test_present_absent_counts() {
        let mut c = new_option_cache();
        oc_set_some(&mut c, "a", "1");
        oc_set_none(&mut c, "b");
        assert_eq!(c.present_count(), 1);
        assert_eq!(c.absent_count(), 1);
    }
    #[test]
    fn test_overwrite_some_to_none() {
        let mut c = new_option_cache();
        oc_set_some(&mut c, "k", "v");
        oc_set_none(&mut c, "k");
        assert_eq!(c.present_count(), 0);
        assert_eq!(c.absent_count(), 1);
    }
}
