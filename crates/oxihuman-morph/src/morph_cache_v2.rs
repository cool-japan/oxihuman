// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Morph result cache to avoid recomputing unchanged weights.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphCacheV2 {
    pub entries: Vec<(u64, f32)>,
}

#[allow(dead_code)]
pub fn new_morph_cache_v2() -> MorphCacheV2 {
    MorphCacheV2 { entries: Vec::new() }
}

#[allow(dead_code)]
pub fn mc2_insert(cache: &mut MorphCacheV2, key: u64, value: f32) {
    for entry in &mut cache.entries {
        if entry.0 == key {
            entry.1 = value;
            return;
        }
    }
    cache.entries.push((key, value));
}

#[allow(dead_code)]
pub fn mc2_get(cache: &MorphCacheV2, key: u64) -> Option<f32> {
    cache.entries.iter().find(|e| e.0 == key).map(|e| e.1)
}

#[allow(dead_code)]
pub fn mc2_invalidate(cache: &mut MorphCacheV2, key: u64) {
    cache.entries.retain(|e| e.0 != key);
}

#[allow(dead_code)]
pub fn mc2_clear(cache: &mut MorphCacheV2) {
    cache.entries.clear();
}

#[allow(dead_code)]
pub fn mc2_size(cache: &MorphCacheV2) -> usize {
    cache.entries.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut c = new_morph_cache_v2();
        mc2_insert(&mut c, 42, 0.75);
        assert!((mc2_get(&c, 42).unwrap() - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_get_missing_returns_none() {
        let c = new_morph_cache_v2();
        assert_eq!(mc2_get(&c, 99), None);
    }

    #[test]
    fn test_invalidate_removes_key() {
        let mut c = new_morph_cache_v2();
        mc2_insert(&mut c, 1, 0.5);
        mc2_invalidate(&mut c, 1);
        assert_eq!(mc2_get(&c, 1), None);
    }

    #[test]
    fn test_clear_empties_cache() {
        let mut c = new_morph_cache_v2();
        mc2_insert(&mut c, 1, 0.1);
        mc2_insert(&mut c, 2, 0.2);
        mc2_clear(&mut c);
        assert_eq!(mc2_size(&c), 0);
    }

    #[test]
    fn test_size_after_inserts() {
        let mut c = new_morph_cache_v2();
        mc2_insert(&mut c, 10, 1.0);
        mc2_insert(&mut c, 20, 2.0);
        assert_eq!(mc2_size(&c), 2);
    }

    #[test]
    fn test_overwrite_existing_key() {
        let mut c = new_morph_cache_v2();
        mc2_insert(&mut c, 5, 0.1);
        mc2_insert(&mut c, 5, 0.9);
        assert!((mc2_get(&c, 5).unwrap() - 0.9).abs() < 1e-6);
        assert_eq!(mc2_size(&c), 1);
    }

    #[test]
    fn test_invalidate_other_key_stays() {
        let mut c = new_morph_cache_v2();
        mc2_insert(&mut c, 1, 0.5);
        mc2_insert(&mut c, 2, 0.6);
        mc2_invalidate(&mut c, 1);
        assert!(mc2_get(&c, 2).is_some());
    }

    #[test]
    fn test_initial_size_zero() {
        let c = new_morph_cache_v2();
        assert_eq!(mc2_size(&c), 0);
    }
}
