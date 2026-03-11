#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Impulse cache for warm-starting the impulse solver.

use std::collections::HashMap;

/// A cached impulse value.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CachedImpulse {
    pub body_pair: (u32, u32),
    pub impulse: [f32; 3],
}

/// A fixed-capacity impulse cache.
#[derive(Debug)]
#[allow(dead_code)]
pub struct ImpulseCache {
    entries: HashMap<(u32, u32), [f32; 3]>,
    capacity: usize,
}

/// Create a new `ImpulseCache` with the given capacity.
#[allow(dead_code)]
pub fn new_impulse_cache(capacity: usize) -> ImpulseCache {
    ImpulseCache { entries: HashMap::new(), capacity: capacity.max(1) }
}

/// Store an impulse for a body pair.
#[allow(dead_code)]
pub fn store_impulse(cache: &mut ImpulseCache, body_a: u32, body_b: u32, impulse: [f32; 3]) {
    let key = (body_a.min(body_b), body_a.max(body_b));
    if cache.entries.len() >= cache.capacity && !cache.entries.contains_key(&key) {
        // Evict first entry when at capacity
        if let Some(k) = cache.entries.keys().next().copied() {
            cache.entries.remove(&k);
        }
    }
    cache.entries.insert(key, impulse);
}

/// Retrieve a cached impulse. Returns `[0,0,0]` if not found.
#[allow(dead_code)]
pub fn retrieve_impulse(cache: &ImpulseCache, body_a: u32, body_b: u32) -> [f32; 3] {
    let key = (body_a.min(body_b), body_a.max(body_b));
    cache.entries.get(&key).copied().unwrap_or([0.0; 3])
}

/// Return the capacity of the cache.
#[allow(dead_code)]
pub fn cache_capacity(cache: &ImpulseCache) -> usize {
    cache.capacity
}

/// Return the number of cached entries.
#[allow(dead_code)]
pub fn cache_used(cache: &ImpulseCache) -> usize {
    cache.entries.len()
}

/// Clear all cached impulses.
#[allow(dead_code)]
pub fn clear_impulse_cache(cache: &mut ImpulseCache) {
    cache.entries.clear();
}

/// Return the impulse for a specific body pair (alias for `retrieve_impulse`).
#[allow(dead_code)]
pub fn impulse_at(cache: &ImpulseCache, body_a: u32, body_b: u32) -> [f32; 3] {
    retrieve_impulse(cache, body_a, body_b)
}

/// Apply warm-start: retrieve cached impulse for a body pair.
#[allow(dead_code)]
pub fn warm_start_impulse(cache: &ImpulseCache, body_a: u32, body_b: u32) -> CachedImpulse {
    let impulse = retrieve_impulse(cache, body_a, body_b);
    CachedImpulse { body_pair: (body_a, body_b), impulse }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_impulse_cache() {
        let c = new_impulse_cache(8);
        assert_eq!(cache_capacity(&c), 8);
        assert_eq!(cache_used(&c), 0);
    }

    #[test]
    fn test_store_and_retrieve() {
        let mut c = new_impulse_cache(8);
        store_impulse(&mut c, 0, 1, [1.0, 2.0, 3.0]);
        let v = retrieve_impulse(&c, 0, 1);
        assert!((v[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_retrieve_reversed() {
        let mut c = new_impulse_cache(8);
        store_impulse(&mut c, 1, 0, [4.0, 0.0, 0.0]);
        let v = retrieve_impulse(&c, 0, 1);
        assert!((v[0] - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_retrieve_miss_returns_zero() {
        let c = new_impulse_cache(8);
        let v = retrieve_impulse(&c, 0, 1);
        assert_eq!(v, [0.0; 3]);
    }

    #[test]
    fn test_cache_used() {
        let mut c = new_impulse_cache(8);
        store_impulse(&mut c, 0, 1, [0.0; 3]);
        store_impulse(&mut c, 1, 2, [0.0; 3]);
        assert_eq!(cache_used(&c), 2);
    }

    #[test]
    fn test_clear_impulse_cache() {
        let mut c = new_impulse_cache(8);
        store_impulse(&mut c, 0, 1, [1.0; 3]);
        clear_impulse_cache(&mut c);
        assert_eq!(cache_used(&c), 0);
    }

    #[test]
    fn test_impulse_at() {
        let mut c = new_impulse_cache(8);
        store_impulse(&mut c, 3, 4, [5.0, 0.0, 0.0]);
        let v = impulse_at(&c, 3, 4);
        assert!((v[0] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_warm_start_impulse() {
        let mut c = new_impulse_cache(8);
        store_impulse(&mut c, 0, 1, [1.0, 2.0, 3.0]);
        let ws = warm_start_impulse(&c, 0, 1);
        assert!((ws.impulse[0] - 1.0).abs() < 1e-6);
    }
}
