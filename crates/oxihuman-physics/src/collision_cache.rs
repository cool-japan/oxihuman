#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Collision contact cache for frame-to-frame persistence.

use std::collections::VecDeque;

/// A cached contact between two bodies.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CachedContact {
    pub body_a: u32,
    pub body_b: u32,
    pub normal: [f32; 3],
    pub penetration: f32,
    pub frame: u32,
}

/// A time-ordered cache of contacts, with hit/miss tracking.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct CollisionCache {
    contacts: VecDeque<CachedContact>,
    max_size: usize,
    hits: u64,
    misses: u64,
}

/// Create a new `CollisionCache` with given capacity.
#[allow(dead_code)]
pub fn new_collision_cache(capacity: usize) -> CollisionCache {
    CollisionCache { contacts: VecDeque::new(), max_size: capacity.max(1), hits: 0, misses: 0 }
}

/// Add a contact to the cache.
#[allow(dead_code)]
pub fn cache_contact(cache: &mut CollisionCache, contact: CachedContact) {
    if cache.contacts.len() >= cache.max_size {
        cache.contacts.pop_front();
    }
    cache.contacts.push_back(contact);
}

/// Get a cached contact between two bodies.
#[allow(dead_code)]
pub fn get_cached_contact(cache: &mut CollisionCache, body_a: u32, body_b: u32) -> Option<&CachedContact> {
    let found = cache.contacts.iter().any(|c| {
        (c.body_a == body_a && c.body_b == body_b) || (c.body_a == body_b && c.body_b == body_a)
    });
    if found {
        cache.hits += 1;
        cache.contacts.iter().find(|c| {
            (c.body_a == body_a && c.body_b == body_b)
            || (c.body_a == body_b && c.body_b == body_a)
        })
    } else {
        cache.misses += 1;
        None
    }
}

/// Return the number of cached contacts.
#[allow(dead_code)]
pub fn cache_size(cache: &CollisionCache) -> usize {
    cache.contacts.len()
}

/// Return the hit count.
#[allow(dead_code)]
pub fn cache_hit(cache: &CollisionCache) -> u64 {
    cache.hits
}

/// Return the miss count.
#[allow(dead_code)]
pub fn cache_miss(cache: &CollisionCache) -> u64 {
    cache.misses
}

/// Clear all cached contacts.
#[allow(dead_code)]
pub fn clear_collision_cache(cache: &mut CollisionCache) {
    cache.contacts.clear();
    cache.hits = 0;
    cache.misses = 0;
}

/// Evict the oldest contact.
#[allow(dead_code)]
pub fn cache_evict_oldest(cache: &mut CollisionCache) -> Option<CachedContact> {
    cache.contacts.pop_front()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_contact(a: u32, b: u32) -> CachedContact {
        CachedContact { body_a: a, body_b: b, normal: [0.0, 1.0, 0.0], penetration: 0.01, frame: 1 }
    }

    #[test]
    fn test_new_collision_cache() {
        let c = new_collision_cache(8);
        assert_eq!(cache_size(&c), 0);
    }

    #[test]
    fn test_cache_contact() {
        let mut c = new_collision_cache(4);
        cache_contact(&mut c, make_contact(0, 1));
        assert_eq!(cache_size(&c), 1);
    }

    #[test]
    fn test_get_cached_contact_hit() {
        let mut c = new_collision_cache(4);
        cache_contact(&mut c, make_contact(0, 1));
        assert!(get_cached_contact(&mut c, 0, 1).is_some());
        assert_eq!(cache_hit(&c), 1);
    }

    #[test]
    fn test_get_cached_contact_miss() {
        let mut c = new_collision_cache(4);
        assert!(get_cached_contact(&mut c, 0, 1).is_none());
        assert_eq!(cache_miss(&c), 1);
    }

    #[test]
    fn test_cache_eviction() {
        let mut c = new_collision_cache(2);
        cache_contact(&mut c, make_contact(0, 1));
        cache_contact(&mut c, make_contact(1, 2));
        cache_contact(&mut c, make_contact(2, 3)); // evicts (0,1)
        assert_eq!(cache_size(&c), 2);
    }

    #[test]
    fn test_clear_collision_cache() {
        let mut c = new_collision_cache(4);
        cache_contact(&mut c, make_contact(0, 1));
        clear_collision_cache(&mut c);
        assert_eq!(cache_size(&c), 0);
    }

    #[test]
    fn test_cache_evict_oldest() {
        let mut c = new_collision_cache(4);
        cache_contact(&mut c, make_contact(0, 1));
        cache_contact(&mut c, make_contact(2, 3));
        let evicted = cache_evict_oldest(&mut c);
        assert!(evicted.is_some());
        assert_eq!(evicted.expect("should succeed").body_a, 0);
    }

    #[test]
    fn test_reverse_lookup() {
        let mut c = new_collision_cache(4);
        cache_contact(&mut c, make_contact(1, 0));
        // Should find even with reversed order
        assert!(get_cached_contact(&mut c, 0, 1).is_some());
    }
}
