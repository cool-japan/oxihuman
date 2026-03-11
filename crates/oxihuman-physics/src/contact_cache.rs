// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Contact point cache for warm-starting solvers.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CachedContact {
    pub id_a: u32,
    pub id_b: u32,
    pub normal: [f32; 3],
    pub depth: f32,
    pub lambda: f32,
    pub age: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ContactCache {
    pub contacts: Vec<CachedContact>,
    pub max_age: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ContactCacheConfig {
    pub capacity: usize,
    pub max_age: u32,
}

#[allow(dead_code)]
pub fn default_contact_cache_config() -> ContactCacheConfig {
    ContactCacheConfig {
        capacity: 256,
        max_age: 4,
    }
}

#[allow(dead_code)]
pub fn new_contact_cache(config: &ContactCacheConfig) -> ContactCache {
    ContactCache {
        contacts: Vec::with_capacity(config.capacity),
        max_age: config.max_age,
    }
}

#[allow(dead_code)]
pub fn cc_insert(cache: &mut ContactCache, mut contact: CachedContact) {
    contact.age = 0;
    // Replace existing entry if present
    for c in &mut cache.contacts {
        if (c.id_a == contact.id_a && c.id_b == contact.id_b)
            || (c.id_a == contact.id_b && c.id_b == contact.id_a)
        {
            *c = contact;
            return;
        }
    }
    cache.contacts.push(contact);
}

#[allow(dead_code)]
pub fn cc_find(cache: &ContactCache, a: u32, b: u32) -> Option<&CachedContact> {
    cache
        .contacts
        .iter()
        .find(|c| (c.id_a == a && c.id_b == b) || (c.id_a == b && c.id_b == a))
}

#[allow(dead_code)]
pub fn cc_count(cache: &ContactCache) -> usize {
    cache.contacts.len()
}

#[allow(dead_code)]
pub fn cc_clear(cache: &mut ContactCache) {
    cache.contacts.clear();
}

#[allow(dead_code)]
pub fn cc_evict_old(cache: &mut ContactCache, current_frame: u32) {
    cache.contacts.retain(|c| {
        let age = current_frame.saturating_sub(c.age);
        age <= cache.max_age
    });
}

#[allow(dead_code)]
pub fn cc_has_contact(cache: &ContactCache, a: u32, b: u32) -> bool {
    cc_find(cache, a, b).is_some()
}

#[allow(dead_code)]
pub fn cc_warm_lambda(cache: &ContactCache, a: u32, b: u32) -> f32 {
    cc_find(cache, a, b).map(|c| c.lambda).unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_contact(a: u32, b: u32, lambda: f32) -> CachedContact {
        CachedContact {
            id_a: a,
            id_b: b,
            normal: [0.0, 1.0, 0.0],
            depth: 0.1,
            lambda,
            age: 0,
        }
    }

    #[test]
    fn test_default_config() {
        let cfg = default_contact_cache_config();
        assert_eq!(cfg.capacity, 256);
        assert_eq!(cfg.max_age, 4);
    }

    #[test]
    fn test_new_cache_empty() {
        let cfg = default_contact_cache_config();
        let cache = new_contact_cache(&cfg);
        assert_eq!(cc_count(&cache), 0);
    }

    #[test]
    fn test_cc_insert_and_find() {
        let cfg = default_contact_cache_config();
        let mut cache = new_contact_cache(&cfg);
        cc_insert(&mut cache, make_contact(1, 2, 0.5));
        assert!(cc_find(&cache, 1, 2).is_some());
    }

    #[test]
    fn test_cc_find_reversed() {
        let cfg = default_contact_cache_config();
        let mut cache = new_contact_cache(&cfg);
        cc_insert(&mut cache, make_contact(3, 4, 0.25));
        assert!(cc_find(&cache, 4, 3).is_some());
    }

    #[test]
    fn test_cc_warm_lambda() {
        let cfg = default_contact_cache_config();
        let mut cache = new_contact_cache(&cfg);
        cc_insert(&mut cache, make_contact(5, 6, 0.75));
        assert!((cc_warm_lambda(&cache, 5, 6) - 0.75).abs() < 1e-5);
    }

    #[test]
    fn test_cc_warm_lambda_missing() {
        let cfg = default_contact_cache_config();
        let cache = new_contact_cache(&cfg);
        assert_eq!(cc_warm_lambda(&cache, 9, 10), 0.0);
    }

    #[test]
    fn test_cc_clear() {
        let cfg = default_contact_cache_config();
        let mut cache = new_contact_cache(&cfg);
        cc_insert(&mut cache, make_contact(1, 2, 0.5));
        cc_clear(&mut cache);
        assert_eq!(cc_count(&cache), 0);
    }

    #[test]
    fn test_cc_has_contact() {
        let cfg = default_contact_cache_config();
        let mut cache = new_contact_cache(&cfg);
        cc_insert(&mut cache, make_contact(7, 8, 0.1));
        assert!(cc_has_contact(&cache, 7, 8));
        assert!(!cc_has_contact(&cache, 7, 9));
    }

    #[test]
    fn test_cc_evict_old() {
        let cfg = ContactCacheConfig {
            capacity: 64,
            max_age: 2,
        };
        let mut cache = new_contact_cache(&cfg);
        // Insert contact with age=0
        let c = CachedContact {
            id_a: 1,
            id_b: 2,
            normal: [0.0, 1.0, 0.0],
            depth: 0.1,
            lambda: 0.0,
            age: 0,
        };
        cc_insert(&mut cache, c);
        // Evict at frame 3 (age would be 3 > max_age 2)
        cc_evict_old(&mut cache, 3);
        assert_eq!(cc_count(&cache), 0);
    }
}
