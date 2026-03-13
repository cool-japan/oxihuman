// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Warm-starting cache — retains constraint impulses between time steps.

use std::collections::HashMap;

/// Key identifying a constraint between two bodies at a specific feature pair.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WarmStartKey {
    pub body_a: u32,
    pub body_b: u32,
    pub feature_a: u32,
    pub feature_b: u32,
}

impl WarmStartKey {
    pub fn new(body_a: u32, body_b: u32, feature_a: u32, feature_b: u32) -> Self {
        /* canonicalize: body_a <= body_b */
        if body_a <= body_b {
            WarmStartKey {
                body_a,
                body_b,
                feature_a,
                feature_b,
            }
        } else {
            WarmStartKey {
                body_a: body_b,
                body_b: body_a,
                feature_a: feature_b,
                feature_b: feature_a,
            }
        }
    }
}

/// Cached impulse for a constraint.
#[derive(Debug, Clone)]
pub struct CachedImpulse {
    pub normal_lambda: f64,
    pub tangent_lambda: [f64; 2],
    pub age: u32, /* frames since last used */
}

impl CachedImpulse {
    pub fn new(normal_lambda: f64, tangent_lambda: [f64; 2]) -> Self {
        CachedImpulse {
            normal_lambda,
            tangent_lambda,
            age: 0,
        }
    }
}

/// Warm-starting cache for constraint impulses.
pub struct WarmStartCache {
    cache: HashMap<WarmStartKey, CachedImpulse>,
    max_age: u32,
}

impl WarmStartCache {
    /// Create a new cache with a maximum age for pruning.
    pub fn new(max_age: u32) -> Self {
        WarmStartCache {
            cache: HashMap::new(),
            max_age,
        }
    }

    /// Store or update a cached impulse.
    pub fn store(&mut self, key: WarmStartKey, normal: f64, tangent: [f64; 2]) {
        self.cache.insert(key, CachedImpulse::new(normal, tangent));
    }

    /// Retrieve a cached impulse, if any.
    pub fn get(&self, key: &WarmStartKey) -> Option<&CachedImpulse> {
        self.cache.get(key)
    }

    /// Age all entries and prune stale ones.
    pub fn age_and_prune(&mut self) {
        self.cache.retain(|_, v| {
            v.age += 1;
            v.age <= self.max_age
        });
    }

    /// Number of cached entries.
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// True if cache is empty.
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Clear all cached impulses.
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Apply warm-start impulses from cache to a lambda array.
    pub fn apply_warm_start(
        &self,
        key: &WarmStartKey,
        lambda: &mut f64,
        tangent_lambda: &mut [f64; 2],
        scale: f64,
    ) {
        if let Some(cached) = self.cache.get(key) {
            *lambda = cached.normal_lambda * scale;
            tangent_lambda[0] = cached.tangent_lambda[0] * scale;
            tangent_lambda[1] = cached.tangent_lambda[1] * scale;
        }
    }
}

/// Compute warm-start effectiveness: fraction of cache entries actually used.
pub fn warm_start_hit_rate(total_constraints: usize, cache_hits: usize) -> f64 {
    if total_constraints == 0 {
        return 0.0;
    }
    cache_hits as f64 / total_constraints as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_get() {
        let mut cache = WarmStartCache::new(3);
        let key = WarmStartKey::new(0, 1, 0, 0);
        cache.store(key.clone(), 5.0, [1.0, 2.0]);
        let c = cache.get(&key);
        assert!(c.is_some() /* stored entry should be retrievable */);
        assert!((c.expect("should succeed").normal_lambda - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_get_missing() {
        let cache = WarmStartCache::new(3);
        let key = WarmStartKey::new(0, 1, 0, 0);
        assert!(cache.get(&key).is_none() /* missing key returns None */);
    }

    #[test]
    fn test_len() {
        let mut cache = WarmStartCache::new(3);
        cache.store(WarmStartKey::new(0, 1, 0, 0), 1.0, [0.0; 2]);
        cache.store(WarmStartKey::new(1, 2, 0, 0), 2.0, [0.0; 2]);
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_is_empty() {
        let cache = WarmStartCache::new(3);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_clear() {
        let mut cache = WarmStartCache::new(3);
        cache.store(WarmStartKey::new(0, 1, 0, 0), 1.0, [0.0; 2]);
        cache.clear();
        assert!(cache.is_empty() /* cleared cache is empty */);
    }

    #[test]
    fn test_age_and_prune() {
        let mut cache = WarmStartCache::new(2);
        cache.store(WarmStartKey::new(0, 1, 0, 0), 1.0, [0.0; 2]);
        cache.age_and_prune();
        cache.age_and_prune();
        cache.age_and_prune(); /* age = 3, max_age = 2: should be pruned */
        assert!(cache.is_empty() /* stale entry pruned */);
    }

    #[test]
    fn test_key_canonicalization() {
        let k1 = WarmStartKey::new(0, 1, 0, 0);
        let k2 = WarmStartKey::new(1, 0, 0, 0);
        assert_eq!(k1, k2 /* canonicalized keys are equal */);
    }

    #[test]
    fn test_apply_warm_start() {
        let mut cache = WarmStartCache::new(3);
        let key = WarmStartKey::new(0, 1, 0, 0);
        cache.store(key.clone(), 4.0, [2.0, 3.0]);
        let mut lambda = 0.0;
        let mut tl = [0.0f64; 2];
        cache.apply_warm_start(&key, &mut lambda, &mut tl, 0.5);
        assert!((lambda - 2.0).abs() < 1e-10 /* 4 * 0.5 = 2 */);
        assert!((tl[0] - 1.0).abs() < 1e-10 /* 2 * 0.5 = 1 */);
    }

    #[test]
    fn test_warm_start_hit_rate() {
        let rate = warm_start_hit_rate(10, 7);
        assert!((rate - 0.7).abs() < 1e-10 /* 7/10 = 0.7 */);
    }
}
