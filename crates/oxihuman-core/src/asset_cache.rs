// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! LRU asset cache with size limits and eviction.

#[allow(dead_code)]
pub struct CacheEntry {
    pub key: String,
    pub data: Vec<u8>,
    pub size_bytes: usize,
    pub access_count: u64,
    pub last_access: u64,
}

#[allow(dead_code)]
pub struct AssetCache {
    pub entries: Vec<CacheEntry>,
    pub max_bytes: usize,
    pub total_bytes: usize,
    pub tick: u64,
    pub hits: u64,
    pub misses: u64,
}

#[allow(dead_code)]
pub fn new_cache(max_bytes: usize) -> AssetCache {
    AssetCache {
        entries: Vec::new(),
        max_bytes,
        total_bytes: 0,
        tick: 0,
        hits: 0,
        misses: 0,
    }
}

#[allow(dead_code)]
pub fn cache_insert(cache: &mut AssetCache, key: &str, data: Vec<u8>) {
    // Remove existing entry with same key
    if let Some(pos) = cache.entries.iter().position(|e| e.key == key) {
        let old_size = cache.entries[pos].size_bytes;
        cache.entries.remove(pos);
        cache.total_bytes -= old_size;
    }
    let size = data.len();
    cache.tick += 1;
    let entry = CacheEntry {
        key: key.to_string(),
        data,
        size_bytes: size,
        access_count: 0,
        last_access: cache.tick,
    };
    cache.entries.push(entry);
    cache.total_bytes += size;
    evict_until_fits(cache);
}

#[allow(dead_code)]
pub fn cache_get<'a>(cache: &'a mut AssetCache, key: &str) -> Option<&'a [u8]> {
    cache.tick += 1;
    let tick = cache.tick;
    if let Some(pos) = cache.entries.iter().position(|e| e.key == key) {
        cache.entries[pos].access_count += 1;
        cache.entries[pos].last_access = tick;
        cache.hits += 1;
        Some(&cache.entries[pos].data)
    } else {
        cache.misses += 1;
        None
    }
}

#[allow(dead_code)]
pub fn cache_remove(cache: &mut AssetCache, key: &str) -> bool {
    if let Some(pos) = cache.entries.iter().position(|e| e.key == key) {
        let size = cache.entries[pos].size_bytes;
        cache.entries.remove(pos);
        cache.total_bytes -= size;
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn cache_contains(cache: &AssetCache, key: &str) -> bool {
    cache.entries.iter().any(|e| e.key == key)
}

#[allow(dead_code)]
pub fn evict_lru(cache: &mut AssetCache) {
    if cache.entries.is_empty() {
        return;
    }
    let lru_pos = cache
        .entries
        .iter()
        .enumerate()
        .min_by_key(|(_, e)| e.last_access)
        .map(|(i, _)| i);
    let Some(lru_pos) = lru_pos else { return };
    let size = cache.entries[lru_pos].size_bytes;
    cache.entries.remove(lru_pos);
    cache.total_bytes -= size;
}

#[allow(dead_code)]
pub fn evict_until_fits(cache: &mut AssetCache) {
    while cache.total_bytes > cache.max_bytes && !cache.entries.is_empty() {
        evict_lru(cache);
    }
}

#[allow(dead_code)]
pub fn cache_size(cache: &AssetCache) -> usize {
    cache.total_bytes
}

#[allow(dead_code)]
pub fn cache_count(cache: &AssetCache) -> usize {
    cache.entries.len()
}

#[allow(dead_code)]
pub fn cache_hit_rate(cache: &AssetCache) -> f32 {
    let total = cache.hits + cache.misses;
    if total == 0 {
        return 0.0;
    }
    cache.hits as f32 / total as f32
}

#[allow(dead_code)]
pub fn cache_clear(cache: &mut AssetCache) {
    cache.entries.clear();
    cache.total_bytes = 0;
}

#[allow(dead_code)]
pub fn most_accessed(cache: &AssetCache) -> Option<&CacheEntry> {
    cache.entries.iter().max_by_key(|e| e.access_count)
}

#[allow(dead_code)]
pub fn cache_stats(cache: &AssetCache) -> (usize, usize, f32) {
    (cache_count(cache), cache_size(cache), cache_hit_rate(cache))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut cache = new_cache(1024);
        cache_insert(&mut cache, "foo", vec![1, 2, 3]);
        let data = cache_get(&mut cache, "foo");
        assert_eq!(data, Some([1u8, 2, 3].as_slice()));
    }

    #[test]
    fn test_get_miss_returns_none() {
        let mut cache = new_cache(1024);
        let data = cache_get(&mut cache, "missing");
        assert!(data.is_none());
    }

    #[test]
    fn test_eviction_when_over_max() {
        let mut cache = new_cache(10);
        cache_insert(&mut cache, "a", vec![0u8; 6]);
        cache_insert(&mut cache, "b", vec![0u8; 6]);
        // "a" should be evicted as LRU
        assert!(cache.total_bytes <= 10);
    }

    #[test]
    fn test_hit_rate_calculation() {
        let mut cache = new_cache(1024);
        cache_insert(&mut cache, "x", vec![1]);
        cache_get(&mut cache, "x");
        cache_get(&mut cache, "x");
        cache_get(&mut cache, "missing");
        let rate = cache_hit_rate(&cache);
        assert!((rate - 2.0 / 3.0).abs() < 1e-4);
    }

    #[test]
    fn test_hit_rate_no_access() {
        let cache = new_cache(1024);
        assert_eq!(cache_hit_rate(&cache), 0.0);
    }

    #[test]
    fn test_contains() {
        let mut cache = new_cache(1024);
        cache_insert(&mut cache, "k", vec![9]);
        assert!(cache_contains(&cache, "k"));
        assert!(!cache_contains(&cache, "nope"));
    }

    #[test]
    fn test_remove() {
        let mut cache = new_cache(1024);
        cache_insert(&mut cache, "del", vec![1, 2]);
        assert!(cache_remove(&mut cache, "del"));
        assert!(!cache_contains(&cache, "del"));
        assert!(!cache_remove(&mut cache, "del"));
    }

    #[test]
    fn test_clear() {
        let mut cache = new_cache(1024);
        cache_insert(&mut cache, "a", vec![1]);
        cache_insert(&mut cache, "b", vec![2]);
        cache_clear(&mut cache);
        assert_eq!(cache_count(&cache), 0);
        assert_eq!(cache_size(&cache), 0);
    }

    #[test]
    fn test_most_accessed() {
        let mut cache = new_cache(1024);
        cache_insert(&mut cache, "a", vec![1]);
        cache_insert(&mut cache, "b", vec![2]);
        cache_get(&mut cache, "b");
        cache_get(&mut cache, "b");
        cache_get(&mut cache, "a");
        let top = most_accessed(&cache);
        assert!(top.is_some());
        assert_eq!(top.expect("should succeed").key, "b");
    }

    #[test]
    fn test_most_accessed_empty() {
        let cache = new_cache(1024);
        assert!(most_accessed(&cache).is_none());
    }

    #[test]
    fn test_lru_eviction_order() {
        let mut cache = new_cache(20);
        cache_insert(&mut cache, "first", vec![0u8; 8]);
        cache_insert(&mut cache, "second", vec![0u8; 8]);
        // Access "first" to make it more recent
        cache_get(&mut cache, "first");
        // Insert large entry that forces eviction
        cache_insert(&mut cache, "third", vec![0u8; 8]);
        // "second" should be evicted as LRU (last_access is oldest)
        assert!(!cache_contains(&cache, "second"));
        assert!(cache_contains(&cache, "first"));
    }

    #[test]
    fn test_cache_size_tracking() {
        let mut cache = new_cache(1024);
        cache_insert(&mut cache, "a", vec![1, 2, 3]);
        cache_insert(&mut cache, "b", vec![4, 5]);
        assert_eq!(cache_size(&cache), 5);
    }

    #[test]
    fn test_cache_count() {
        let mut cache = new_cache(1024);
        assert_eq!(cache_count(&cache), 0);
        cache_insert(&mut cache, "a", vec![1]);
        assert_eq!(cache_count(&cache), 1);
        cache_insert(&mut cache, "b", vec![2]);
        assert_eq!(cache_count(&cache), 2);
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = new_cache(1024);
        cache_insert(&mut cache, "x", vec![9]);
        cache_get(&mut cache, "x");
        let (count, size, rate) = cache_stats(&cache);
        assert_eq!(count, 1);
        assert_eq!(size, 1);
        assert!((rate - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_insert_overwrite_same_key() {
        let mut cache = new_cache(1024);
        cache_insert(&mut cache, "k", vec![1, 2, 3]);
        cache_insert(&mut cache, "k", vec![9]);
        assert_eq!(cache_count(&cache), 1);
        assert_eq!(cache_size(&cache), 1);
    }

    #[test]
    fn test_evict_lru_removes_oldest() {
        let mut cache = new_cache(1024);
        cache_insert(&mut cache, "old", vec![1]);
        cache_insert(&mut cache, "new", vec![2]);
        evict_lru(&mut cache);
        assert!(!cache_contains(&cache, "old"));
        assert!(cache_contains(&cache, "new"));
    }
}
