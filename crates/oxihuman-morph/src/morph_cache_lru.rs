#![allow(dead_code)]
//! LRU cache for morph evaluation results.

use std::collections::HashMap;

/// A single cache entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// Cached key.
    pub key: String,
    /// Cached morph weights.
    pub weights: Vec<f32>,
    /// Access counter for LRU tracking.
    pub access_count: u64,
    /// Last access order (higher = more recent).
    pub last_access: u64,
}

/// An LRU cache for morph computations.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphCacheLru {
    /// Maximum number of entries.
    capacity: usize,
    /// Stored entries.
    entries: HashMap<String, CacheEntry>,
    /// Global access counter.
    counter: u64,
    /// Hit count.
    hits: u64,
    /// Miss count.
    misses: u64,
}

/// Create a new [`MorphCacheLru`] with the given capacity.
#[allow(dead_code)]
pub fn new_morph_cache_lru(capacity: usize) -> MorphCacheLru {
    MorphCacheLru {
        capacity: capacity.max(1),
        entries: HashMap::new(),
        counter: 0,
        hits: 0,
        misses: 0,
    }
}

/// Insert or update a cache entry.
#[allow(dead_code)]
pub fn cache_put(cache: &mut MorphCacheLru, key: &str, weights: Vec<f32>) {
    cache.counter += 1;
    if cache.entries.len() >= cache.capacity && !cache.entries.contains_key(key) {
        // Evict least recently used
        if let Some(lru_key) = cache
            .entries
            .iter()
            .min_by_key(|(_, e)| e.last_access)
            .map(|(k, _)| k.clone())
        {
            cache.entries.remove(&lru_key);
        }
    }
    cache.entries.insert(
        key.to_string(),
        CacheEntry {
            key: key.to_string(),
            weights,
            access_count: 1,
            last_access: cache.counter,
        },
    );
}

/// Get a cached entry, updating access stats.
#[allow(dead_code)]
pub fn cache_get(cache: &mut MorphCacheLru, key: &str) -> Option<Vec<f32>> {
    cache.counter += 1;
    if let Some(entry) = cache.entries.get_mut(key) {
        entry.access_count += 1;
        entry.last_access = cache.counter;
        cache.hits += 1;
        Some(entry.weights.clone())
    } else {
        cache.misses += 1;
        None
    }
}

/// Check if a key exists in the cache (without updating stats).
#[allow(dead_code)]
pub fn cache_contains(cache: &MorphCacheLru, key: &str) -> bool {
    cache.entries.contains_key(key)
}

/// Evict a specific key. Returns true if found and removed.
#[allow(dead_code)]
pub fn cache_evict(cache: &mut MorphCacheLru, key: &str) -> bool {
    cache.entries.remove(key).is_some()
}

/// Return the number of entries in the cache.
#[allow(dead_code)]
pub fn cache_size(cache: &MorphCacheLru) -> usize {
    cache.entries.len()
}

/// Return the capacity of the cache.
#[allow(dead_code)]
pub fn cache_capacity(cache: &MorphCacheLru) -> usize {
    cache.capacity
}

/// Return the cache hit rate as a fraction [0, 1].
#[allow(dead_code)]
pub fn cache_hit_rate(cache: &MorphCacheLru) -> f32 {
    let total = cache.hits + cache.misses;
    if total == 0 {
        return 0.0;
    }
    cache.hits as f32 / total as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_morph_cache_lru() {
        let c = new_morph_cache_lru(10);
        assert_eq!(cache_capacity(&c), 10);
        assert_eq!(cache_size(&c), 0);
    }

    #[test]
    fn test_cache_put_get() {
        let mut c = new_morph_cache_lru(5);
        cache_put(&mut c, "smile", vec![0.5, 0.8]);
        let result = cache_get(&mut c, "smile");
        assert!(result.is_some());
        assert!((result.unwrap()[0] - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_cache_miss() {
        let mut c = new_morph_cache_lru(5);
        assert!(cache_get(&mut c, "nope").is_none());
    }

    #[test]
    fn test_cache_contains() {
        let mut c = new_morph_cache_lru(5);
        cache_put(&mut c, "a", vec![1.0]);
        assert!(cache_contains(&c, "a"));
        assert!(!cache_contains(&c, "b"));
    }

    #[test]
    fn test_cache_evict() {
        let mut c = new_morph_cache_lru(5);
        cache_put(&mut c, "a", vec![1.0]);
        assert!(cache_evict(&mut c, "a"));
        assert!(!cache_contains(&c, "a"));
    }

    #[test]
    fn test_cache_evict_missing() {
        let mut c = new_morph_cache_lru(5);
        assert!(!cache_evict(&mut c, "nope"));
    }

    #[test]
    fn test_cache_lru_eviction() {
        let mut c = new_morph_cache_lru(2);
        cache_put(&mut c, "a", vec![1.0]);
        cache_put(&mut c, "b", vec![2.0]);
        cache_put(&mut c, "c", vec![3.0]); // should evict "a"
        assert!(!cache_contains(&c, "a"));
        assert!(cache_contains(&c, "b"));
        assert!(cache_contains(&c, "c"));
    }

    #[test]
    fn test_cache_hit_rate_initial() {
        let c = new_morph_cache_lru(5);
        assert!((cache_hit_rate(&c) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_cache_hit_rate_after_ops() {
        let mut c = new_morph_cache_lru(5);
        cache_put(&mut c, "a", vec![1.0]);
        let _ = cache_get(&mut c, "a"); // hit
        let _ = cache_get(&mut c, "b"); // miss
        assert!((cache_hit_rate(&c) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_cache_size() {
        let mut c = new_morph_cache_lru(10);
        cache_put(&mut c, "a", vec![1.0]);
        cache_put(&mut c, "b", vec![2.0]);
        assert_eq!(cache_size(&c), 2);
    }
}
