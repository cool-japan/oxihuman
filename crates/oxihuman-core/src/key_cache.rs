// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Key cache: string-keyed LRU-style cache with TTL (time-to-live) in frames.

use std::collections::HashMap;

/// A single cache entry with value and expiry frame.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct KeyCacheEntry<V> {
    pub value: V,
    pub expiry: u64,
    pub hits: u32,
}

/// String-keyed cache.
#[derive(Debug)]
#[allow(dead_code)]
pub struct KeyCache<V> {
    entries: HashMap<String, KeyCacheEntry<V>>,
    frame: u64,
    capacity: usize,
}

/// Create a new KeyCache with given capacity.
#[allow(dead_code)]
pub fn new_key_cache<V>(capacity: usize) -> KeyCache<V> {
    KeyCache {
        entries: HashMap::new(),
        frame: 0,
        capacity,
    }
}

/// Insert a key with TTL in frames.
#[allow(dead_code)]
pub fn kc_insert<V>(cache: &mut KeyCache<V>, key: &str, value: V, ttl: u64) {
    if cache.entries.len() >= cache.capacity && !cache.entries.contains_key(key) {
        // evict oldest expiry
        if let Some(oldest) = cache
            .entries
            .iter()
            .min_by_key(|(_, e)| e.expiry)
            .map(|(k, _)| k.clone())
        {
            cache.entries.remove(&oldest);
        }
    }
    cache.entries.insert(
        key.to_string(),
        KeyCacheEntry {
            value,
            expiry: cache.frame + ttl,
            hits: 0,
        },
    );
}

/// Get a reference; returns None if expired or missing.
#[allow(dead_code)]
pub fn kc_get<'a, V>(cache: &'a mut KeyCache<V>, key: &str) -> Option<&'a V> {
    let frame = cache.frame;
    if let Some(e) = cache.entries.get_mut(key) {
        if e.expiry >= frame {
            e.hits += 1;
            return Some(&e.value);
        }
    }
    None
}

/// Advance frame counter; evicts expired entries.
#[allow(dead_code)]
pub fn kc_advance(cache: &mut KeyCache<impl std::fmt::Debug>, frames: u64) {
    cache.frame += frames;
    let frame = cache.frame;
    cache.entries.retain(|_, e| e.expiry >= frame);
}

/// Remove a key explicitly.
#[allow(dead_code)]
pub fn kc_remove<V>(cache: &mut KeyCache<V>, key: &str) -> bool {
    cache.entries.remove(key).is_some()
}

/// Whether key is present and not expired.
#[allow(dead_code)]
pub fn kc_contains<V>(cache: &mut KeyCache<V>, key: &str) -> bool {
    kc_get(cache, key).is_some()
}

/// Number of cached entries.
#[allow(dead_code)]
pub fn kc_len<V>(cache: &KeyCache<V>) -> usize {
    cache.entries.len()
}

/// Current frame.
#[allow(dead_code)]
pub fn kc_frame<V>(cache: &KeyCache<V>) -> u64 {
    cache.frame
}

/// Hit count for a key.
#[allow(dead_code)]
pub fn kc_hits<V>(cache: &KeyCache<V>, key: &str) -> u32 {
    cache.entries.get(key).map(|e| e.hits).unwrap_or(0)
}

/// Clear all entries.
#[allow(dead_code)]
pub fn kc_clear<V>(cache: &mut KeyCache<V>) {
    cache.entries.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_get() {
        let mut c: KeyCache<u32> = new_key_cache(10);
        kc_insert(&mut c, "a", 42, 5);
        assert_eq!(kc_get(&mut c, "a"), Some(&42));
    }

    #[test]
    fn test_expired() {
        let mut c: KeyCache<u32> = new_key_cache(10);
        kc_insert(&mut c, "x", 1, 2);
        kc_advance(&mut c, 3);
        assert_eq!(kc_get(&mut c, "x"), None);
    }

    #[test]
    fn test_remove() {
        let mut c: KeyCache<u32> = new_key_cache(10);
        kc_insert(&mut c, "k", 5, 100);
        assert!(kc_remove(&mut c, "k"));
        assert_eq!(kc_get(&mut c, "k"), None);
    }

    #[test]
    fn test_contains() {
        let mut c: KeyCache<i32> = new_key_cache(10);
        kc_insert(&mut c, "z", -1, 10);
        assert!(kc_contains(&mut c, "z"));
    }

    #[test]
    fn test_capacity_eviction() {
        let mut c: KeyCache<u32> = new_key_cache(2);
        kc_insert(&mut c, "a", 1, 100);
        kc_insert(&mut c, "b", 2, 50);
        kc_insert(&mut c, "c", 3, 200);
        assert_eq!(kc_len(&c), 2);
    }

    #[test]
    fn test_advance_evicts() {
        let mut c: KeyCache<u32> = new_key_cache(10);
        kc_insert(&mut c, "a", 1, 1);
        kc_advance(&mut c, 2);
        assert_eq!(kc_len(&c), 0);
    }

    #[test]
    fn test_hits() {
        let mut c: KeyCache<u32> = new_key_cache(10);
        kc_insert(&mut c, "h", 7, 10);
        kc_get(&mut c, "h");
        kc_get(&mut c, "h");
        assert_eq!(kc_hits(&c, "h"), 2);
    }

    #[test]
    fn test_clear() {
        let mut c: KeyCache<u32> = new_key_cache(10);
        kc_insert(&mut c, "a", 1, 10);
        kc_clear(&mut c);
        assert_eq!(kc_len(&c), 0);
    }

    #[test]
    fn test_frame_advances() {
        let mut c: KeyCache<u32> = new_key_cache(10);
        kc_advance(&mut c, 5);
        assert_eq!(kc_frame(&c), 5);
    }
}
