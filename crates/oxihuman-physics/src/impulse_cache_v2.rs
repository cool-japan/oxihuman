// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Caches impulse values from the previous solver iteration for warm-starting.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ImpulseCacheV2 {
    entries: Vec<CacheEntry>,
    capacity: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct CacheEntry {
    pub key: u64,
    pub normal_impulse: f32,
    pub tangent_impulse: [f32; 2],
    pub age: u32,
}

#[allow(dead_code)]
impl CacheEntry {
    pub fn new(key: u64) -> Self {
        Self {
            key,
            normal_impulse: 0.0,
            tangent_impulse: [0.0; 2],
            age: 0,
        }
    }

    pub fn total_impulse(&self) -> f32 {
        let tn = self.tangent_impulse[0] * self.tangent_impulse[0]
            + self.tangent_impulse[1] * self.tangent_impulse[1];
        (self.normal_impulse * self.normal_impulse + tn).sqrt()
    }
}

#[allow(dead_code)]
impl ImpulseCacheV2 {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: Vec::new(),
            capacity,
        }
    }

    pub fn lookup(&self, key: u64) -> Option<&CacheEntry> {
        self.entries.iter().find(|e| e.key == key)
    }

    pub fn store(&mut self, entry: CacheEntry) {
        if let Some(existing) = self.entries.iter_mut().find(|e| e.key == entry.key) {
            *existing = entry;
        } else {
            if self.entries.len() >= self.capacity {
                // Evict oldest
                if let Some(idx) = self
                    .entries
                    .iter()
                    .enumerate()
                    .max_by_key(|(_, e)| e.age)
                    .map(|(i, _)| i)
                {
                    self.entries.swap_remove(idx);
                }
            }
            self.entries.push(entry);
        }
    }

    pub fn age_all(&mut self) {
        for e in &mut self.entries {
            e.age += 1;
        }
    }

    pub fn evict_old(&mut self, max_age: u32) {
        self.entries.retain(|e| e.age <= max_age);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn warm_start_factor(age: u32) -> f32 {
        match age {
            0 => 1.0,
            1 => 0.8,
            2 => 0.5,
            _ => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let cache = ImpulseCacheV2::new(64);
        assert!(cache.is_empty());
        assert_eq!(cache.capacity(), 64);
    }

    #[test]
    fn test_store_lookup() {
        let mut cache = ImpulseCacheV2::new(64);
        let mut e = CacheEntry::new(42);
        e.normal_impulse = 5.0;
        cache.store(e);
        let found = cache.lookup(42).expect("should succeed");
        assert!((found.normal_impulse - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_overwrite() {
        let mut cache = ImpulseCacheV2::new(64);
        let mut e1 = CacheEntry::new(1);
        e1.normal_impulse = 1.0;
        cache.store(e1);
        let mut e2 = CacheEntry::new(1);
        e2.normal_impulse = 2.0;
        cache.store(e2);
        assert_eq!(cache.len(), 1);
        assert!((cache.lookup(1).expect("should succeed").normal_impulse - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_age_all() {
        let mut cache = ImpulseCacheV2::new(64);
        cache.store(CacheEntry::new(1));
        cache.age_all();
        assert_eq!(cache.lookup(1).expect("should succeed").age, 1);
    }

    #[test]
    fn test_evict_old() {
        let mut cache = ImpulseCacheV2::new(64);
        cache.store(CacheEntry::new(1));
        cache.age_all();
        cache.age_all();
        cache.age_all();
        cache.evict_old(2);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_capacity_eviction() {
        let mut cache = ImpulseCacheV2::new(2);
        cache.store(CacheEntry::new(1));
        cache.store(CacheEntry::new(2));
        cache.age_all();
        cache.store(CacheEntry::new(3));
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_clear() {
        let mut cache = ImpulseCacheV2::new(64);
        cache.store(CacheEntry::new(1));
        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_total_impulse() {
        let mut e = CacheEntry::new(0);
        e.normal_impulse = 3.0;
        e.tangent_impulse = [4.0, 0.0];
        assert!((e.total_impulse() - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_warm_start_factor() {
        assert!((ImpulseCacheV2::warm_start_factor(0) - 1.0).abs() < 1e-6);
        assert!((ImpulseCacheV2::warm_start_factor(1) - 0.8).abs() < 1e-6);
        assert!((ImpulseCacheV2::warm_start_factor(5)).abs() < 1e-6);
    }

    #[test]
    fn test_lookup_missing() {
        let cache = ImpulseCacheV2::new(64);
        assert!(cache.lookup(999).is_none());
    }
}
