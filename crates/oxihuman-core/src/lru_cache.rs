//! Fixed-capacity LRU (Least-Recently-Used) cache mapping string keys to f32 values.
//!
//! Implemented as an ordered list of entries. The most-recently-used entry is moved to
//! the back of the list on every access. When the cache is full and a new entry is
//! inserted, the least-recently-used (front) entry is evicted.

/// Configuration for the LRU cache.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LruCacheConfig {
    /// Maximum number of entries the cache can hold.
    pub capacity: usize,
}

#[allow(dead_code)]
impl LruCacheConfig {
    fn new() -> Self {
        Self { capacity: 16 }
    }
}

/// Returns the default LRU cache configuration.
#[allow(dead_code)]
pub fn default_lru_config() -> LruCacheConfig {
    LruCacheConfig::new()
}

/// A single LRU cache entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LruEntry {
    pub key: String,
    pub value: f32,
}

/// Fixed-capacity LRU cache.
///
/// Internal representation: `entries[0]` is the LRU item; `entries[last]` is the MRU item.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LruCache {
    config: LruCacheConfig,
    entries: Vec<LruEntry>,
}

/// Creates a new `LruCache` with the given configuration.
#[allow(dead_code)]
pub fn new_lru_cache(config: LruCacheConfig) -> LruCache {
    LruCache {
        config,
        entries: Vec::new(),
    }
}

/// Retrieves the value for `key`, updating its recency. Returns `None` if not found.
#[allow(dead_code)]
pub fn lru_get(cache: &mut LruCache, key: &str) -> Option<f32> {
    if let Some(pos) = cache.entries.iter().position(|e| e.key == key) {
        let entry = cache.entries.remove(pos);
        let value = entry.value;
        cache.entries.push(entry);
        Some(value)
    } else {
        None
    }
}

/// Inserts or updates `key` with `value`.
/// If the cache is full and `key` is new, the LRU entry is evicted.
#[allow(dead_code)]
pub fn lru_insert(cache: &mut LruCache, key: &str, value: f32) {
    // If key already present, update in place and move to MRU position.
    if let Some(pos) = cache.entries.iter().position(|e| e.key == key) {
        let mut entry = cache.entries.remove(pos);
        entry.value = value;
        cache.entries.push(entry);
        return;
    }
    // Evict LRU if at capacity.
    if cache.entries.len() >= cache.config.capacity {
        cache.entries.remove(0);
    }
    cache.entries.push(LruEntry {
        key: key.to_string(),
        value,
    });
}

/// Removes an entry by key. Returns `true` if it was found and removed.
#[allow(dead_code)]
pub fn lru_remove(cache: &mut LruCache, key: &str) -> bool {
    if let Some(pos) = cache.entries.iter().position(|e| e.key == key) {
        cache.entries.remove(pos);
        true
    } else {
        false
    }
}

/// Returns `true` if the cache contains `key`.
#[allow(dead_code)]
pub fn lru_contains(cache: &LruCache, key: &str) -> bool {
    cache.entries.iter().any(|e| e.key == key)
}

/// Returns the number of entries currently in the cache.
#[allow(dead_code)]
pub fn lru_len(cache: &LruCache) -> usize {
    cache.entries.len()
}

/// Returns the cache capacity.
#[allow(dead_code)]
pub fn lru_capacity(cache: &LruCache) -> usize {
    cache.config.capacity
}

/// Evicts the oldest (LRU) entry. Returns the evicted entry, or `None` if empty.
#[allow(dead_code)]
pub fn lru_evict_oldest(cache: &mut LruCache) -> Option<LruEntry> {
    if cache.entries.is_empty() {
        return None;
    }
    Some(cache.entries.remove(0))
}

/// Serialises the cache to a simple JSON string.
#[allow(dead_code)]
pub fn lru_to_json(cache: &LruCache) -> String {
    let items: Vec<String> = cache
        .entries
        .iter()
        .map(|e| format!("{{\"key\":\"{}\",\"value\":{:.4}}}", e.key, e.value))
        .collect();
    format!(
        "{{\"capacity\":{},\"len\":{},\"entries\":[{}]}}",
        cache.config.capacity,
        cache.entries.len(),
        items.join(",")
    )
}

// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make_cache() -> LruCache {
        let mut c = new_lru_cache(LruCacheConfig { capacity: 3 });
        lru_insert(&mut c, "a", 1.0);
        lru_insert(&mut c, "b", 2.0);
        lru_insert(&mut c, "c", 3.0);
        c
    }

    #[test]
    fn test_basic_insert_get() {
        let mut c = make_cache();
        assert!((lru_get(&mut c, "a").unwrap() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_len() {
        let c = make_cache();
        assert_eq!(lru_len(&c), 3);
    }

    #[test]
    fn test_capacity() {
        let c = make_cache();
        assert_eq!(lru_capacity(&c), 3);
    }

    #[test]
    fn test_eviction_on_overflow() {
        let mut c = make_cache();
        // "a" was accessed (moved to MRU) in test above but in this test it wasn't.
        // Order: a(LRU), b, c(MRU). Inserting "d" should evict "a".
        lru_insert(&mut c, "d", 4.0);
        assert!(!lru_contains(&c, "a"));
        assert!(lru_contains(&c, "d"));
    }

    #[test]
    fn test_get_updates_recency() {
        let mut c = make_cache();
        // Access "a" → it becomes MRU. Order becomes: b, c, a.
        lru_get(&mut c, "a");
        // Insert "d" → evict "b" (now LRU).
        lru_insert(&mut c, "d", 4.0);
        assert!(!lru_contains(&c, "b"));
        assert!(lru_contains(&c, "a"));
    }

    #[test]
    fn test_remove() {
        let mut c = make_cache();
        assert!(lru_remove(&mut c, "b"));
        assert!(!lru_contains(&c, "b"));
        assert_eq!(lru_len(&c), 2);
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut c = make_cache();
        assert!(!lru_remove(&mut c, "z"));
    }

    #[test]
    fn test_evict_oldest() {
        let mut c = make_cache();
        let evicted = lru_evict_oldest(&mut c).unwrap();
        assert_eq!(evicted.key, "a");
        assert_eq!(lru_len(&c), 2);
    }

    #[test]
    fn test_update_existing_key() {
        let mut c = make_cache();
        lru_insert(&mut c, "a", 99.0);
        assert_eq!(lru_len(&c), 3); // no eviction
        assert!((lru_get(&mut c, "a").unwrap() - 99.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_to_json() {
        let c = make_cache();
        let json = lru_to_json(&c);
        assert!(json.contains("capacity"));
        assert!(json.contains("entries"));
    }
}
