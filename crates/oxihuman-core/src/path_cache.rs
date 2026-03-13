// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

/// Cache for resolved filesystem-style paths.
#[allow(dead_code)]
pub struct PathCache {
    cache: HashMap<String, String>,
    hits: u64,
    misses: u64,
    max_size: usize,
}

#[allow(dead_code)]
impl PathCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            hits: 0,
            misses: 0,
            max_size: max_size.max(1),
        }
    }
    pub fn get(&mut self, key: &str) -> Option<&str> {
        if self.cache.contains_key(key) {
            self.hits += 1;
            self.cache.get(key).map(|s| s.as_str())
        } else {
            self.misses += 1;
            None
        }
    }
    pub fn insert(&mut self, key: &str, resolved: &str) {
        if self.cache.len() >= self.max_size && !self.cache.contains_key(key) {
            if let Some(k) = self.cache.keys().next().cloned() {
                self.cache.remove(&k);
            }
        }
        self.cache.insert(key.to_string(), resolved.to_string());
    }
    pub fn invalidate(&mut self, key: &str) -> bool {
        self.cache.remove(key).is_some()
    }
    pub fn contains(&self, key: &str) -> bool {
        self.cache.contains_key(key)
    }
    pub fn len(&self) -> usize {
        self.cache.len()
    }
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
    pub fn hits(&self) -> u64 {
        self.hits
    }
    pub fn misses(&self) -> u64 {
        self.misses
    }
    pub fn hit_rate(&self) -> f32 {
        let t = self.hits + self.misses;
        if t == 0 {
            0.0
        } else {
            self.hits as f32 / t as f32
        }
    }
    pub fn clear(&mut self) {
        self.cache.clear();
    }
    pub fn max_size(&self) -> usize {
        self.max_size
    }
}

#[allow(dead_code)]
pub fn new_path_cache(max_size: usize) -> PathCache {
    PathCache::new(max_size)
}
#[allow(dead_code)]
pub fn pc_get<'a>(c: &'a mut PathCache, key: &str) -> Option<&'a str> {
    c.get(key)
}
#[allow(dead_code)]
pub fn pc_insert(c: &mut PathCache, key: &str, resolved: &str) {
    c.insert(key, resolved);
}
#[allow(dead_code)]
pub fn pc_invalidate(c: &mut PathCache, key: &str) -> bool {
    c.invalidate(key)
}
#[allow(dead_code)]
pub fn pc_contains(c: &PathCache, key: &str) -> bool {
    c.contains(key)
}
#[allow(dead_code)]
pub fn pc_len(c: &PathCache) -> usize {
    c.len()
}
#[allow(dead_code)]
pub fn pc_is_empty(c: &PathCache) -> bool {
    c.is_empty()
}
#[allow(dead_code)]
pub fn pc_hit_rate(c: &PathCache) -> f32 {
    c.hit_rate()
}
#[allow(dead_code)]
pub fn pc_clear(c: &mut PathCache) {
    c.clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_insert_get() {
        let mut c = new_path_cache(16);
        pc_insert(&mut c, "/a", "/resolved/a");
        assert_eq!(pc_get(&mut c, "/a"), Some("/resolved/a"));
    }
    #[test]
    fn test_miss() {
        let mut c = new_path_cache(16);
        assert_eq!(pc_get(&mut c, "/nope"), None);
    }
    #[test]
    fn test_invalidate() {
        let mut c = new_path_cache(16);
        pc_insert(&mut c, "/k", "/v");
        assert!(pc_invalidate(&mut c, "/k"));
        assert!(!pc_contains(&c, "/k"));
    }
    #[test]
    fn test_hit_rate() {
        let mut c = new_path_cache(16);
        pc_insert(&mut c, "/k", "/v");
        pc_get(&mut c, "/k");
        pc_get(&mut c, "/missing");
        let r = pc_hit_rate(&c);
        assert!((0.0..=1.0).contains(&r));
    }
    #[test]
    fn test_len() {
        let mut c = new_path_cache(16);
        pc_insert(&mut c, "/a", "/ra");
        pc_insert(&mut c, "/b", "/rb");
        assert_eq!(pc_len(&c), 2);
    }
    #[test]
    fn test_max_size_respected() {
        let mut c = new_path_cache(2);
        pc_insert(&mut c, "/a", "/ra");
        pc_insert(&mut c, "/b", "/rb");
        pc_insert(&mut c, "/c", "/rc");
        assert!(pc_len(&c) <= 2);
    }
    #[test]
    fn test_clear() {
        let mut c = new_path_cache(16);
        pc_insert(&mut c, "/a", "/ra");
        pc_clear(&mut c);
        assert!(pc_is_empty(&c));
    }
    #[test]
    fn test_contains() {
        let mut c = new_path_cache(16);
        pc_insert(&mut c, "/x", "/rx");
        assert!(pc_contains(&c, "/x"));
        assert!(!pc_contains(&c, "/y"));
    }
    #[test]
    fn test_is_empty() {
        let c = new_path_cache(8);
        assert!(pc_is_empty(&c));
    }
    #[test]
    fn test_max_size_getter() {
        let c = new_path_cache(32);
        assert_eq!(c.max_size(), 32);
    }
}
