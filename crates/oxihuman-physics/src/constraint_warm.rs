#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Warm-starting support for constraint solvers.

use std::collections::HashMap;

/// Cached warm-start lambda (impulse) for a constraint.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct WarmStart {
    pub lambda: f32,
    pub constraint_id: u64,
}

/// A cache of warm-start lambdas keyed by constraint ID.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct WarmCache {
    entries: HashMap<u64, f32>,
    hits: u64,
    misses: u64,
}

/// Create a new empty `WarmCache`.
#[allow(dead_code)]
pub fn new_warm_cache() -> WarmCache {
    WarmCache::default()
}

/// Store a lambda for a constraint.
#[allow(dead_code)]
pub fn store_lambda(cache: &mut WarmCache, constraint_id: u64, lambda: f32) {
    cache.entries.insert(constraint_id, lambda);
}

/// Retrieve a lambda for a constraint. Returns 0.0 if not cached.
#[allow(dead_code)]
pub fn retrieve_lambda(cache: &mut WarmCache, constraint_id: u64) -> f32 {
    if let Some(&v) = cache.entries.get(&constraint_id) {
        cache.hits += 1;
        v
    } else {
        cache.misses += 1;
        0.0
    }
}

/// Apply warm starting to a constraint: retrieve cached lambda.
#[allow(dead_code)]
pub fn warm_start_constraint(cache: &mut WarmCache, constraint_id: u64) -> WarmStart {
    let lambda = retrieve_lambda(cache, constraint_id);
    WarmStart { lambda, constraint_id }
}

/// Return the number of cached entries.
#[allow(dead_code)]
pub fn cache_size(cache: &WarmCache) -> usize {
    cache.entries.len()
}

/// Clear all cached lambdas.
#[allow(dead_code)]
pub fn clear_warm_cache(cache: &mut WarmCache) {
    cache.entries.clear();
    cache.hits = 0;
    cache.misses = 0;
}

/// Return the total number of warm-start retrievals.
#[allow(dead_code)]
pub fn warm_start_count(cache: &WarmCache) -> u64 {
    cache.hits + cache.misses
}

/// Return the cache hit rate (0.0 if no lookups).
#[allow(dead_code)]
pub fn cache_hit_rate(cache: &WarmCache) -> f32 {
    let total = cache.hits + cache.misses;
    if total == 0 { 0.0 } else { cache.hits as f32 / total as f32 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_warm_cache() {
        let c = new_warm_cache();
        assert_eq!(cache_size(&c), 0);
    }

    #[test]
    fn test_store_and_retrieve() {
        let mut c = new_warm_cache();
        store_lambda(&mut c, 1, 0.5);
        let v = retrieve_lambda(&mut c, 1);
        assert!((v - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_retrieve_miss() {
        let mut c = new_warm_cache();
        let v = retrieve_lambda(&mut c, 99);
        assert_eq!(v, 0.0);
    }

    #[test]
    fn test_warm_start_constraint() {
        let mut c = new_warm_cache();
        store_lambda(&mut c, 5, 1.23);
        let ws = warm_start_constraint(&mut c, 5);
        assert!((ws.lambda - 1.23).abs() < 1e-5);
        assert_eq!(ws.constraint_id, 5);
    }

    #[test]
    fn test_cache_size() {
        let mut c = new_warm_cache();
        store_lambda(&mut c, 1, 0.1);
        store_lambda(&mut c, 2, 0.2);
        assert_eq!(cache_size(&c), 2);
    }

    #[test]
    fn test_clear_warm_cache() {
        let mut c = new_warm_cache();
        store_lambda(&mut c, 1, 0.5);
        clear_warm_cache(&mut c);
        assert_eq!(cache_size(&c), 0);
    }

    #[test]
    fn test_warm_start_count() {
        let mut c = new_warm_cache();
        store_lambda(&mut c, 1, 1.0);
        retrieve_lambda(&mut c, 1);
        retrieve_lambda(&mut c, 2);
        assert_eq!(warm_start_count(&c), 2);
    }

    #[test]
    fn test_cache_hit_rate() {
        let mut c = new_warm_cache();
        store_lambda(&mut c, 1, 1.0);
        retrieve_lambda(&mut c, 1); // hit
        retrieve_lambda(&mut c, 2); // miss
        let rate = cache_hit_rate(&c);
        assert!((rate - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_cache_hit_rate_empty() {
        let c = new_warm_cache();
        assert_eq!(cache_hit_rate(&c), 0.0);
    }
}
