// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Memoized result cache (key → value with hit/miss counters).

#![allow(dead_code)]

use std::collections::HashMap;

/// Configuration for the result cache.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ResultCacheConfig {
    pub max_entries: usize,
}

/// Cache statistics.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
}

/// A memoized result cache from String → f32.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ResultCache {
    config: ResultCacheConfig,
    entries: HashMap<String, f32>,
    stats: CacheStats,
}

/// Create a new result cache.
#[allow(dead_code)]
pub fn new_result_cache(config: ResultCacheConfig) -> ResultCache {
    ResultCache {
        config,
        entries: HashMap::new(),
        stats: CacheStats { hits: 0, misses: 0, evictions: 0 },
    }
}

/// Get a cached value, updating hit/miss counters.
#[allow(dead_code)]
pub fn cache_get(cache: &mut ResultCache, key: &str) -> Option<f32> {
    if let Some(&v) = cache.entries.get(key) {
        cache.stats.hits += 1;
        Some(v)
    } else {
        cache.stats.misses += 1;
        None
    }
}

/// Insert a key-value pair. Evicts an arbitrary entry if at capacity.
#[allow(dead_code)]
pub fn cache_insert(cache: &mut ResultCache, key: String, value: f32) {
    if cache.entries.len() >= cache.config.max_entries && !cache.entries.contains_key(&key) {
        // Evict an arbitrary entry
        if let Some(k) = cache.entries.keys().next().cloned() {
            cache.entries.remove(&k);
            cache.stats.evictions += 1;
        }
    }
    cache.entries.insert(key, value);
}

/// Remove a key from the cache.
#[allow(dead_code)]
pub fn cache_remove(cache: &mut ResultCache, key: &str) -> Option<f32> {
    cache.entries.remove(key)
}

/// Return cache statistics.
#[allow(dead_code)]
pub fn cache_stats(cache: &ResultCache) -> CacheStats {
    cache.stats.clone()
}

/// Return the cache hit rate (hits / (hits + misses)), or 0 if no lookups.
#[allow(dead_code)]
pub fn cache_hit_rate(cache: &ResultCache) -> f32 {
    let total = cache.stats.hits + cache.stats.misses;
    if total == 0 {
        0.0
    } else {
        cache.stats.hits as f32 / total as f32
    }
}

/// Return the number of entries.
#[allow(dead_code)]
pub fn cache_len(cache: &ResultCache) -> usize {
    cache.entries.len()
}

/// Clear all entries and reset stats.
#[allow(dead_code)]
pub fn cache_clear(cache: &mut ResultCache) {
    cache.entries.clear();
    cache.stats = CacheStats { hits: 0, misses: 0, evictions: 0 };
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cache(max: usize) -> ResultCache {
        new_result_cache(ResultCacheConfig { max_entries: max })
    }

    #[test]
    fn test_new_cache_empty() {
        let cache = make_cache(10);
        assert_eq!(cache_len(&cache), 0);
    }

    #[test]
    fn test_insert_and_get() {
        let mut cache = make_cache(10);
        cache_insert(&mut cache, "pi".to_string(), std::f32::consts::PI);
        let val = cache_get(&mut cache, "pi");
        assert!(val.is_some());
        assert!((val.expect("should succeed") - std::f32::consts::PI).abs() < 1e-6);
    }

    #[test]
    fn test_miss_increments_counter() {
        let mut cache = make_cache(10);
        cache_get(&mut cache, "missing");
        assert_eq!(cache_stats(&cache).misses, 1);
    }

    #[test]
    fn test_hit_increments_counter() {
        let mut cache = make_cache(10);
        cache_insert(&mut cache, "x".to_string(), 1.0);
        cache_get(&mut cache, "x");
        assert_eq!(cache_stats(&cache).hits, 1);
    }

    #[test]
    fn test_hit_rate() {
        let mut cache = make_cache(10);
        cache_insert(&mut cache, "a".to_string(), 1.0);
        cache_get(&mut cache, "a");   // hit
        cache_get(&mut cache, "b");   // miss
        let rate = cache_hit_rate(&cache);
        assert!((rate - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_remove() {
        let mut cache = make_cache(10);
        cache_insert(&mut cache, "r".to_string(), 99.0);
        let old = cache_remove(&mut cache, "r");
        assert_eq!(old, Some(99.0));
        assert_eq!(cache_len(&cache), 0);
    }

    #[test]
    fn test_clear() {
        let mut cache = make_cache(10);
        cache_insert(&mut cache, "k".to_string(), 1.0);
        cache_clear(&mut cache);
        assert_eq!(cache_len(&cache), 0);
        assert_eq!(cache_stats(&cache).hits, 0);
    }

    #[test]
    fn test_eviction_on_full() {
        let mut cache = make_cache(2);
        cache_insert(&mut cache, "a".to_string(), 1.0);
        cache_insert(&mut cache, "b".to_string(), 2.0);
        cache_insert(&mut cache, "c".to_string(), 3.0); // triggers eviction
        assert_eq!(cache_stats(&cache).evictions, 1);
        assert_eq!(cache_len(&cache), 2);
    }

    #[test]
    fn test_hit_rate_zero_on_no_lookups() {
        let cache = make_cache(10);
        assert_eq!(cache_hit_rate(&cache), 0.0);
    }
}
