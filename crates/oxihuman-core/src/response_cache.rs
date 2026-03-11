// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! HTTP-style response cache — stores and retrieves keyed response entries.

/// A cached HTTP-style response entry.
#[derive(Clone, Debug)]
pub struct CachedResponse {
    pub status: u16,
    pub body: String,
    pub content_type: String,
    pub max_age_secs: u64,
    pub cached_at: u64,
}

/// Cache configuration.
#[derive(Clone, Debug)]
pub struct ResponseCacheConfig {
    pub max_entries: usize,
    pub default_max_age_secs: u64,
}

impl Default for ResponseCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 512,
            default_max_age_secs: 300,
        }
    }
}

/// An LRU-style response cache (simplified: evicts oldest entry).
pub struct ResponseCache {
    pub config: ResponseCacheConfig,
    entries: Vec<(String, CachedResponse)>,
}

/// Creates a new response cache.
pub fn new_response_cache(config: ResponseCacheConfig) -> ResponseCache {
    ResponseCache {
        config,
        entries: Vec::new(),
    }
}

/// Stores a response in the cache.
pub fn cache_store(cache: &mut ResponseCache, key: &str, response: CachedResponse) {
    cache.entries.retain(|(k, _)| k != key);
    if cache.entries.len() >= cache.config.max_entries {
        cache.entries.remove(0); /* evict oldest */
    }
    cache.entries.push((key.into(), response));
}

/// Retrieves a cached response by key if it has not expired.
pub fn cache_get<'a>(cache: &'a ResponseCache, key: &str, now: u64) -> Option<&'a CachedResponse> {
    cache
        .entries
        .iter()
        .find(|(k, r)| k == key && now.saturating_sub(r.cached_at) < r.max_age_secs)
        .map(|(_, r)| r)
}

/// Invalidates a cached entry by key.
pub fn cache_invalidate(cache: &mut ResponseCache, key: &str) -> bool {
    let before = cache.entries.len();
    cache.entries.retain(|(k, _)| k != key);
    cache.entries.len() < before
}

/// Purges all expired entries at the given timestamp.
pub fn purge_expired_responses(cache: &mut ResponseCache, now: u64) -> usize {
    let before = cache.entries.len();
    cache
        .entries
        .retain(|(_, r)| now.saturating_sub(r.cached_at) < r.max_age_secs);
    before.saturating_sub(cache.entries.len())
}

/// Returns the number of entries currently in the cache.
pub fn cache_size(cache: &ResponseCache) -> usize {
    cache.entries.len()
}

impl ResponseCache {
    /// Creates a new cache with default config.
    pub fn new(config: ResponseCacheConfig) -> Self {
        new_response_cache(config)
    }
}

fn make_response(status: u16, body: &str, max_age: u64, cached_at: u64) -> CachedResponse {
    CachedResponse {
        status,
        body: body.into(),
        content_type: "text/plain".into(),
        max_age_secs: max_age,
        cached_at,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cache() -> ResponseCache {
        new_response_cache(ResponseCacheConfig::default())
    }

    #[test]
    fn test_store_and_retrieve() {
        let mut c = make_cache();
        cache_store(&mut c, "/api", make_response(200, "ok", 300, 0));
        let r = cache_get(&c, "/api", 100);
        assert!(r.is_some());
    }

    #[test]
    fn test_expired_entry_returns_none() {
        let mut c = make_cache();
        cache_store(&mut c, "/api", make_response(200, "ok", 60, 0));
        /* now = 100, max_age=60 => expired */
        assert!(cache_get(&c, "/api", 100).is_none());
    }

    #[test]
    fn test_invalidate_removes_entry() {
        let mut c = make_cache();
        cache_store(&mut c, "/api", make_response(200, "ok", 300, 0));
        assert!(cache_invalidate(&mut c, "/api"));
        assert_eq!(cache_size(&c), 0);
    }

    #[test]
    fn test_invalidate_nonexistent_returns_false() {
        let mut c = make_cache();
        assert!(!cache_invalidate(&mut c, "/missing"));
    }

    #[test]
    fn test_purge_expired_removes_old() {
        let mut c = make_cache();
        cache_store(&mut c, "/old", make_response(200, "old", 10, 0));
        cache_store(&mut c, "/new", make_response(200, "new", 1000, 0));
        let removed = purge_expired_responses(&mut c, 100);
        assert_eq!(removed, 1);
        assert_eq!(cache_size(&c), 1);
    }

    #[test]
    fn test_overwrite_existing_key() {
        let mut c = make_cache();
        cache_store(&mut c, "/k", make_response(200, "v1", 300, 0));
        cache_store(&mut c, "/k", make_response(200, "v2", 300, 0));
        assert_eq!(cache_size(&c), 1);
        assert_eq!(cache_get(&c, "/k", 0).unwrap().body, "v2");
    }

    #[test]
    fn test_eviction_when_at_capacity() {
        let mut c = new_response_cache(ResponseCacheConfig {
            max_entries: 2,
            default_max_age_secs: 300,
        });
        cache_store(&mut c, "/a", make_response(200, "a", 300, 0));
        cache_store(&mut c, "/b", make_response(200, "b", 300, 0));
        cache_store(&mut c, "/c", make_response(200, "c", 300, 0)); /* evicts /a */
        assert!(cache_get(&c, "/a", 0).is_none());
        assert_eq!(cache_size(&c), 2);
    }

    #[test]
    fn test_cache_size_tracks_correctly() {
        let mut c = make_cache();
        assert_eq!(cache_size(&c), 0);
        cache_store(&mut c, "/x", make_response(200, "x", 60, 0));
        assert_eq!(cache_size(&c), 1);
    }

    #[test]
    fn test_response_status_preserved() {
        let mut c = make_cache();
        cache_store(&mut c, "/e", make_response(404, "not found", 300, 0));
        assert_eq!(cache_get(&c, "/e", 0).unwrap().status, 404);
    }
}
