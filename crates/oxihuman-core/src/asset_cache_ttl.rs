// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Reference-counted asset cache with LRU eviction policy.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub max_entries: usize,
    pub max_bytes: usize,
    pub ttl_seconds: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TtlCacheEntry {
    pub key: String,
    pub size_bytes: usize,
    pub last_access: u64,
    pub access_count: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TtlAssetCache {
    pub config: CacheConfig,
    pub entries: Vec<TtlCacheEntry>,
    pub total_bytes: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hit_count: u64,
    pub miss_count: u64,
    pub eviction_count: u64,
    pub total_bytes: usize,
}

#[allow(dead_code)]
pub fn default_cache_config() -> CacheConfig {
    CacheConfig {
        max_entries: 256,
        max_bytes: 64 * 1024 * 1024,
        ttl_seconds: 300,
    }
}

#[allow(dead_code)]
pub fn new_asset_cache(cfg: CacheConfig) -> TtlAssetCache {
    TtlAssetCache {
        config: cfg,
        entries: Vec::new(),
        total_bytes: 0,
    }
}

#[allow(dead_code)]
pub fn ttl_cache_insert(cache: &mut TtlAssetCache, key: &str, size: usize, now: u64) {
    // Replace if key exists
    if let Some(pos) = cache.entries.iter().position(|e| e.key == key) {
        cache.total_bytes -= cache.entries[pos].size_bytes;
        cache.entries.remove(pos);
    }
    // Evict LRU if at capacity
    if cache.entries.len() >= cache.config.max_entries
        || cache.total_bytes + size > cache.config.max_bytes
    {
        ttl_cache_evict_lru(cache);
    }
    cache.entries.push(TtlCacheEntry {
        key: key.to_string(),
        size_bytes: size,
        last_access: now,
        access_count: 0,
    });
    cache.total_bytes += size;
}

#[allow(dead_code)]
pub fn ttl_cache_contains(cache: &TtlAssetCache, key: &str) -> bool {
    cache.entries.iter().any(|e| e.key == key)
}

#[allow(dead_code)]
pub fn ttl_cache_touch(cache: &mut TtlAssetCache, key: &str, now: u64) -> bool {
    if let Some(e) = cache.entries.iter_mut().find(|e| e.key == key) {
        e.last_access = now;
        e.access_count += 1;
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn ttl_cache_evict_lru(cache: &mut TtlAssetCache) -> Option<String> {
    if cache.entries.is_empty() {
        return None;
    }
    let lru_pos = cache
        .entries
        .iter()
        .enumerate()
        .min_by_key(|(_, e)| e.last_access)
        .map(|(i, _)| i);
    let Some(lru_pos) = lru_pos else { return None };
    let size = cache.entries[lru_pos].size_bytes;
    let key = cache.entries[lru_pos].key.clone();
    cache.entries.remove(lru_pos);
    cache.total_bytes -= size;
    Some(key)
}

#[allow(dead_code)]
pub fn ttl_cache_evict_expired(cache: &mut TtlAssetCache, now: u64) -> usize {
    let ttl = cache.config.ttl_seconds;
    let mut removed = 0;
    let mut i = 0;
    while i < cache.entries.len() {
        if now.saturating_sub(cache.entries[i].last_access) > ttl {
            cache.total_bytes -= cache.entries[i].size_bytes;
            cache.entries.remove(i);
            removed += 1;
        } else {
            i += 1;
        }
    }
    removed
}

#[allow(dead_code)]
pub fn ttl_cache_entry_count(cache: &TtlAssetCache) -> usize {
    cache.entries.len()
}

#[allow(dead_code)]
pub fn ttl_cache_total_bytes(cache: &TtlAssetCache) -> usize {
    cache.total_bytes
}

#[allow(dead_code)]
pub fn ttl_cache_remove(cache: &mut TtlAssetCache, key: &str) -> bool {
    if let Some(pos) = cache.entries.iter().position(|e| e.key == key) {
        cache.total_bytes -= cache.entries[pos].size_bytes;
        cache.entries.remove(pos);
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn ttl_cache_to_json(cache: &TtlAssetCache) -> String {
    let entries: Vec<String> = cache
        .entries
        .iter()
        .map(|e| {
            format!(
                r#"{{"key":"{}","size_bytes":{},"last_access":{},"access_count":{}}}"#,
                e.key, e.size_bytes, e.last_access, e.access_count
            )
        })
        .collect();
    format!(
        r#"{{"max_entries":{},"max_bytes":{},"total_bytes":{},"entry_count":{},"entries":[{}]}}"#,
        cache.config.max_entries,
        cache.config.max_bytes,
        cache.total_bytes,
        cache.entries.len(),
        entries.join(",")
    )
}

#[allow(dead_code)]
pub fn ttl_cache_is_full(cache: &TtlAssetCache) -> bool {
    cache.entries.len() >= cache.config.max_entries || cache.total_bytes >= cache.config.max_bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cache(max_entries: usize, max_bytes: usize) -> TtlAssetCache {
        new_asset_cache(CacheConfig {
            max_entries,
            max_bytes,
            ttl_seconds: 60,
        })
    }

    #[test]
    fn test_default_config() {
        let cfg = default_cache_config();
        assert_eq!(cfg.max_entries, 256);
        assert_eq!(cfg.ttl_seconds, 300);
    }

    #[test]
    fn test_insert_and_contains() {
        let mut c = make_cache(10, 1024);
        ttl_cache_insert(&mut c, "a", 100, 0);
        assert!(ttl_cache_contains(&c, "a"));
        assert!(!ttl_cache_contains(&c, "b"));
    }

    #[test]
    fn test_insert_replaces_same_key() {
        let mut c = make_cache(10, 1024);
        ttl_cache_insert(&mut c, "x", 50, 0);
        ttl_cache_insert(&mut c, "x", 80, 1);
        assert_eq!(ttl_cache_entry_count(&c), 1);
        assert_eq!(ttl_cache_total_bytes(&c), 80);
    }

    #[test]
    fn test_touch_updates_access() {
        let mut c = make_cache(10, 1024);
        ttl_cache_insert(&mut c, "k", 10, 100);
        let hit = ttl_cache_touch(&mut c, "k", 200);
        assert!(hit);
        let e = c.entries.iter().find(|e| e.key == "k").expect("should succeed");
        assert_eq!(e.last_access, 200);
        assert_eq!(e.access_count, 1);
    }

    #[test]
    fn test_touch_miss() {
        let mut c = make_cache(10, 1024);
        assert!(!ttl_cache_touch(&mut c, "missing", 0));
    }

    #[test]
    fn test_evict_lru() {
        let mut c = make_cache(10, 1024);
        ttl_cache_insert(&mut c, "old", 10, 1);
        ttl_cache_insert(&mut c, "new", 10, 100);
        let evicted = ttl_cache_evict_lru(&mut c);
        assert_eq!(evicted, Some("old".to_string()));
        assert!(!ttl_cache_contains(&c, "old"));
    }

    #[test]
    fn test_evict_lru_empty() {
        let mut c = make_cache(10, 1024);
        assert!(ttl_cache_evict_lru(&mut c).is_none());
    }

    #[test]
    fn test_evict_expired() {
        let mut c = make_cache(10, 1024);
        ttl_cache_insert(&mut c, "old", 10, 0);
        // fresh inserted at time 100, now=110, 110-100=10 < ttl=60 => not expired
        ttl_cache_insert(&mut c, "fresh", 10, 100);
        // now=110: old is at 0, 110-0=110 > 60 => expired; fresh 110-100=10 <= 60 => not
        let removed = ttl_cache_evict_expired(&mut c, 110);
        assert_eq!(removed, 1);
        assert!(!ttl_cache_contains(&c, "old"));
        assert!(ttl_cache_contains(&c, "fresh"));
    }

    #[test]
    fn test_remove() {
        let mut c = make_cache(10, 1024);
        ttl_cache_insert(&mut c, "del", 20, 0);
        assert!(ttl_cache_remove(&mut c, "del"));
        assert!(!ttl_cache_contains(&c, "del"));
        assert!(!ttl_cache_remove(&mut c, "del"));
    }

    #[test]
    fn test_total_bytes_tracking() {
        let mut c = make_cache(10, 1024);
        ttl_cache_insert(&mut c, "a", 30, 0);
        ttl_cache_insert(&mut c, "b", 50, 1);
        assert_eq!(ttl_cache_total_bytes(&c), 80);
        ttl_cache_remove(&mut c, "a");
        assert_eq!(ttl_cache_total_bytes(&c), 50);
    }

    #[test]
    fn test_to_json() {
        let mut c = make_cache(10, 1024);
        ttl_cache_insert(&mut c, "foo", 42, 0);
        let j = ttl_cache_to_json(&c);
        assert!(j.contains("\"foo\""));
        assert!(j.contains("\"size_bytes\":42"));
    }

    #[test]
    fn test_is_full_by_entry_count() {
        let mut c = make_cache(2, 1024);
        ttl_cache_insert(&mut c, "a", 1, 0);
        ttl_cache_insert(&mut c, "b", 1, 1);
        assert!(ttl_cache_is_full(&c));
    }
}
