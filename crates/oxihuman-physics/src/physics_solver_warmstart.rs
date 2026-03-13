// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Solver warm-starting: caches impulses from previous frame to accelerate convergence.

use std::collections::HashMap;

/// A cached impulse for a contact pair.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct CachedImpulse {
    pub normal_impulse: f32,
    pub tangent_impulse_1: f32,
    pub tangent_impulse_2: f32,
    pub age: u32,
}

#[allow(dead_code)]
impl CachedImpulse {
    pub fn new(normal: f32, t1: f32, t2: f32) -> Self {
        Self { normal_impulse: normal, tangent_impulse_1: t1, tangent_impulse_2: t2, age: 0 }
    }

    pub fn zero() -> Self {
        Self { normal_impulse: 0.0, tangent_impulse_1: 0.0, tangent_impulse_2: 0.0, age: 0 }
    }

    pub fn total_magnitude(&self) -> f32 {
        (self.normal_impulse * self.normal_impulse
            + self.tangent_impulse_1 * self.tangent_impulse_1
            + self.tangent_impulse_2 * self.tangent_impulse_2).sqrt()
    }

    pub fn scale(&self, factor: f32) -> CachedImpulse {
        CachedImpulse {
            normal_impulse: self.normal_impulse * factor,
            tangent_impulse_1: self.tangent_impulse_1 * factor,
            tangent_impulse_2: self.tangent_impulse_2 * factor,
            age: self.age,
        }
    }
}

/// Warm-start cache that persists impulses between frames.
#[allow(dead_code)]
#[derive(Debug)]
pub struct WarmStartCache {
    cache: HashMap<(u32, u32), CachedImpulse>,
    warmstart_factor: f32,
    max_age: u32,
}

#[allow(dead_code)]
fn pair_key(a: u32, b: u32) -> (u32, u32) {
    (a.min(b), a.max(b))
}

#[allow(dead_code)]
impl WarmStartCache {
    pub fn new(warmstart_factor: f32) -> Self {
        Self {
            cache: HashMap::new(),
            warmstart_factor: warmstart_factor.clamp(0.0, 1.0),
            max_age: 3,
        }
    }

    pub fn with_max_age(mut self, age: u32) -> Self {
        self.max_age = age;
        self
    }

    pub fn store(&mut self, body_a: u32, body_b: u32, impulse: CachedImpulse) {
        let key = pair_key(body_a, body_b);
        self.cache.insert(key, CachedImpulse { age: 0, ..impulse });
    }

    pub fn lookup(&self, body_a: u32, body_b: u32) -> CachedImpulse {
        let key = pair_key(body_a, body_b);
        self.cache.get(&key)
            .map(|c| c.scale(self.warmstart_factor))
            .unwrap_or_else(CachedImpulse::zero)
    }

    /// Age all entries and remove stale ones.
    pub fn advance_frame(&mut self) {
        let max_age = self.max_age;
        self.cache.values_mut().for_each(|c| c.age += 1);
        self.cache.retain(|_, c| c.age <= max_age);
    }

    pub fn entry_count(&self) -> usize {
        self.cache.len()
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }

    pub fn warmstart_factor(&self) -> f32 {
        self.warmstart_factor
    }

    pub fn set_warmstart_factor(&mut self, f: f32) {
        self.warmstart_factor = f.clamp(0.0, 1.0);
    }

    /// Total cached impulse energy.
    pub fn total_cached_energy(&self) -> f32 {
        self.cache.values().map(|c| c.total_magnitude()).sum()
    }

    pub fn has_entry(&self, body_a: u32, body_b: u32) -> bool {
        self.cache.contains_key(&pair_key(body_a, body_b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_lookup() {
        let mut c = WarmStartCache::new(1.0);
        c.store(0, 1, CachedImpulse::new(10.0, 0.0, 0.0));
        let r = c.lookup(0, 1);
        assert!((r.normal_impulse - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_warmstart_factor() {
        let mut c = WarmStartCache::new(0.5);
        c.store(0, 1, CachedImpulse::new(10.0, 2.0, 0.0));
        let r = c.lookup(0, 1);
        assert!((r.normal_impulse - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_missing_returns_zero() {
        let c = WarmStartCache::new(1.0);
        let r = c.lookup(99, 100);
        assert!((r.normal_impulse).abs() < 0.001);
    }

    #[test]
    fn test_advance_frame_evicts() {
        let mut c = WarmStartCache::new(1.0).with_max_age(1);
        c.store(0, 1, CachedImpulse::new(10.0, 0.0, 0.0));
        c.advance_frame();
        assert!(c.has_entry(0, 1));
        c.advance_frame();
        assert!(!c.has_entry(0, 1));
    }

    #[test]
    fn test_entry_count() {
        let mut c = WarmStartCache::new(1.0);
        c.store(0, 1, CachedImpulse::new(1.0, 0.0, 0.0));
        c.store(2, 3, CachedImpulse::new(2.0, 0.0, 0.0));
        assert_eq!(c.entry_count(), 2);
    }

    #[test]
    fn test_clear() {
        let mut c = WarmStartCache::new(1.0);
        c.store(0, 1, CachedImpulse::new(1.0, 0.0, 0.0));
        c.clear();
        assert_eq!(c.entry_count(), 0);
    }

    #[test]
    fn test_pair_ordering() {
        let mut c = WarmStartCache::new(1.0);
        c.store(5, 2, CachedImpulse::new(7.0, 0.0, 0.0));
        let r = c.lookup(2, 5);
        assert!((r.normal_impulse - 7.0).abs() < 0.01);
    }

    #[test]
    fn test_total_cached_energy() {
        let mut c = WarmStartCache::new(1.0);
        c.store(0, 1, CachedImpulse::new(3.0, 4.0, 0.0));
        assert!((c.total_cached_energy() - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_scale() {
        let imp = CachedImpulse::new(10.0, 5.0, 0.0);
        let scaled = imp.scale(0.5);
        assert!((scaled.normal_impulse - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_set_factor() {
        let mut c = WarmStartCache::new(0.8);
        c.set_warmstart_factor(0.3);
        assert!((c.warmstart_factor() - 0.3).abs() < 0.01);
    }
}
