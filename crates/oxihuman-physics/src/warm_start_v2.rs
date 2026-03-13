// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Impulse warm-starting for iterative solvers.

/// A cached impulse for warm starting.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WarmImpulse {
    pub id_a: usize,
    pub id_b: usize,
    pub normal_impulse: f32,
    pub tangent_impulse: [f32; 2],
    pub age: usize,
}

impl WarmImpulse {
    #[allow(dead_code)]
    pub fn new(id_a: usize, id_b: usize) -> Self {
        Self {
            id_a,
            id_b,
            normal_impulse: 0.0,
            tangent_impulse: [0.0; 2],
            age: 0,
        }
    }

    /// Key for lookup.
    fn key(&self) -> (usize, usize) {
        if self.id_a <= self.id_b {
            (self.id_a, self.id_b)
        } else {
            (self.id_b, self.id_a)
        }
    }
}

/// Warm-start cache.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct WarmStartCache {
    pub entries: Vec<WarmImpulse>,
    pub max_age: usize,
}

impl WarmStartCache {
    #[allow(dead_code)]
    pub fn new(max_age: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_age,
        }
    }

    /// Find a cached impulse for a pair.
    #[allow(dead_code)]
    pub fn find(&self, id_a: usize, id_b: usize) -> Option<&WarmImpulse> {
        let key = if id_a <= id_b {
            (id_a, id_b)
        } else {
            (id_b, id_a)
        };
        self.entries.iter().find(|e| e.key() == key)
    }

    /// Find mutable cached impulse.
    #[allow(dead_code)]
    pub fn find_mut(&mut self, id_a: usize, id_b: usize) -> Option<&mut WarmImpulse> {
        let key = if id_a <= id_b {
            (id_a, id_b)
        } else {
            (id_b, id_a)
        };
        self.entries.iter_mut().find(|e| e.key() == key)
    }

    /// Store or update a warm impulse.
    #[allow(dead_code)]
    pub fn store(&mut self, imp: WarmImpulse) {
        let key = imp.key();
        if let Some(e) = self.entries.iter_mut().find(|e| e.key() == key) {
            e.normal_impulse = imp.normal_impulse;
            e.tangent_impulse = imp.tangent_impulse;
            e.age = 0;
        } else {
            self.entries.push(imp);
        }
    }

    /// Age all entries and remove stale ones.
    #[allow(dead_code)]
    pub fn advance(&mut self) {
        for e in &mut self.entries {
            e.age += 1;
        }
        self.entries.retain(|e| e.age <= self.max_age);
    }

    /// Clear all cached impulses.
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Number of cached impulses.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Apply warm-start impulses to velocities.
#[allow(dead_code)]
pub fn apply_warm_start(
    vel_a: &mut [f32; 3],
    vel_b: &mut [f32; 3],
    normal: [f32; 3],
    warm: &WarmImpulse,
    inv_mass_a: f32,
    inv_mass_b: f32,
    scale: f32,
) {
    let jn = warm.normal_impulse * scale;
    for k in 0..3 {
        vel_a[k] -= inv_mass_a * jn * normal[k];
        vel_b[k] += inv_mass_b * jn * normal[k];
    }
}

/// Blend a new impulse with the warm-start value.
#[allow(dead_code)]
pub fn blend_impulse(warm: f32, current: f32, alpha: f32) -> f32 {
    warm * (1.0 - alpha) + current * alpha
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_and_find() {
        let mut cache = WarmStartCache::new(5);
        let mut imp = WarmImpulse::new(0, 1);
        imp.normal_impulse = 3.0;
        cache.store(imp);
        let found = cache.find(0, 1).expect("should succeed");
        assert!((found.normal_impulse - 3.0).abs() < 1e-5);
    }

    #[test]
    fn find_missing_is_none() {
        let cache = WarmStartCache::new(5);
        assert!(cache.find(0, 1).is_none());
    }

    #[test]
    fn advance_removes_old_entries() {
        let mut cache = WarmStartCache::new(2);
        cache.store(WarmImpulse::new(0, 1));
        for _ in 0..4 {
            cache.advance();
        }
        assert!(cache.is_empty());
    }

    #[test]
    fn clear_empties_cache() {
        let mut cache = WarmStartCache::new(5);
        cache.store(WarmImpulse::new(0, 1));
        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn store_same_key_updates() {
        let mut cache = WarmStartCache::new(5);
        let mut imp1 = WarmImpulse::new(0, 1);
        imp1.normal_impulse = 1.0;
        cache.store(imp1);
        let mut imp2 = WarmImpulse::new(0, 1);
        imp2.normal_impulse = 5.0;
        cache.store(imp2);
        assert_eq!(cache.len(), 1);
        assert!((cache.find(0, 1).expect("should succeed").normal_impulse - 5.0).abs() < 1e-5);
    }

    #[test]
    fn apply_warm_start_changes_velocity() {
        let mut va = [0.0f32; 3];
        let mut vb = [0.0f32; 3];
        let imp = WarmImpulse {
            id_a: 0,
            id_b: 1,
            normal_impulse: 2.0,
            tangent_impulse: [0.0; 2],
            age: 0,
        };
        apply_warm_start(&mut va, &mut vb, [0.0, 1.0, 0.0], &imp, 1.0, 1.0, 1.0);
        assert!(va[1] < 0.0 || vb[1] > 0.0);
    }

    #[test]
    fn blend_impulse_lerp() {
        let b = blend_impulse(0.0, 10.0, 0.5);
        assert!((b - 5.0).abs() < 1e-5);
    }

    #[test]
    fn len_tracks_entries() {
        let mut cache = WarmStartCache::new(5);
        cache.store(WarmImpulse::new(0, 1));
        cache.store(WarmImpulse::new(1, 2));
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn find_mut_works() {
        let mut cache = WarmStartCache::new(5);
        cache.store(WarmImpulse::new(0, 1));
        let e = cache.find_mut(0, 1).expect("should succeed");
        e.normal_impulse = 42.0;
        assert!((cache.find(0, 1).expect("should succeed").normal_impulse - 42.0).abs() < 1e-5);
    }

    #[test]
    fn symmetry_of_key() {
        let mut cache = WarmStartCache::new(5);
        cache.store(WarmImpulse::new(3, 1));
        assert!(cache.find(1, 3).is_some());
    }
}
