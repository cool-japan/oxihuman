//! Cache management with LRU/LFU/FIFO eviction and size budgets.
//!
//! Note: `CacheEntry` and several `cache_*` functions exist in `asset_cache`.
//! This module uses distinct types: `MgrCacheEntry`, `CacheManager`, `CachePolicy`.

#[allow(dead_code)]
pub struct MgrCacheEntry {
    pub key: String,
    pub data: Vec<u8>,
    pub size_bytes: usize,
    pub access_count: u64,
    pub insert_order: u64,
    pub last_access_tick: u64,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CachePolicy {
    Lru,
    Lfu,
    Fifo,
}

#[allow(dead_code)]
pub struct CacheManager {
    pub entries: Vec<MgrCacheEntry>,
    pub capacity_bytes: usize,
    pub used_bytes: usize,
    pub policy: CachePolicy,
    pub tick: u64,
    pub insert_counter: u64,
    pub hits: u64,
    pub misses: u64,
}

#[allow(dead_code)]
pub fn new_cache_manager(capacity_bytes: usize, policy: CachePolicy) -> CacheManager {
    CacheManager {
        entries: Vec::new(),
        capacity_bytes,
        used_bytes: 0,
        policy,
        tick: 0,
        insert_counter: 0,
        hits: 0,
        misses: 0,
    }
}

#[allow(dead_code)]
pub fn cache_insert(mgr: &mut CacheManager, key: &str, data: Vec<u8>) {
    // Remove old entry with same key
    if let Some(pos) = mgr.entries.iter().position(|e| e.key == key) {
        let old_size = mgr.entries[pos].size_bytes;
        mgr.entries.remove(pos);
        mgr.used_bytes -= old_size;
    }
    mgr.tick += 1;
    mgr.insert_counter += 1;
    let size = data.len();
    let entry = MgrCacheEntry {
        key: key.to_string(),
        data,
        size_bytes: size,
        access_count: 0,
        insert_order: mgr.insert_counter,
        last_access_tick: mgr.tick,
    };
    mgr.entries.push(entry);
    mgr.used_bytes += size;
    cache_evict_to_budget(mgr);
}

#[allow(dead_code)]
pub fn cache_get<'a>(mgr: &'a mut CacheManager, key: &str) -> Option<&'a [u8]> {
    mgr.tick += 1;
    let tick = mgr.tick;
    if let Some(entry) = mgr.entries.iter_mut().find(|e| e.key == key) {
        entry.access_count += 1;
        entry.last_access_tick = tick;
        mgr.hits += 1;
        // Re-borrow immutably for return
        let Some(pos) = mgr.entries.iter().position(|e| e.key == key) else { return None; };
        return Some(&mgr.entries[pos].data);
    }
    mgr.misses += 1;
    None
}

#[allow(dead_code)]
pub fn cache_evict_one(mgr: &mut CacheManager) -> bool {
    if mgr.entries.is_empty() {
        return false;
    }
    let victim = match mgr.policy {
        CachePolicy::Lru => {
            // evict least recently used
            mgr.entries
                .iter()
                .enumerate()
                .min_by_key(|(_, e)| e.last_access_tick)
                .map(|(i, _)| i)
        }
        CachePolicy::Lfu => {
            // evict least frequently used; break ties by LRU
            mgr.entries
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    a.access_count
                        .cmp(&b.access_count)
                        .then(a.last_access_tick.cmp(&b.last_access_tick))
                })
                .map(|(i, _)| i)
        }
        CachePolicy::Fifo => {
            // evict oldest insert
            mgr.entries
                .iter()
                .enumerate()
                .min_by_key(|(_, e)| e.insert_order)
                .map(|(i, _)| i)
        }
    };
    if let Some(idx) = victim {
        let size = mgr.entries[idx].size_bytes;
        mgr.entries.remove(idx);
        mgr.used_bytes -= size;
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn cache_evict_to_budget(mgr: &mut CacheManager) {
    while mgr.used_bytes > mgr.capacity_bytes && !mgr.entries.is_empty() {
        cache_evict_one(mgr);
    }
}

#[allow(dead_code)]
pub fn cache_size(mgr: &CacheManager) -> usize {
    mgr.used_bytes
}

#[allow(dead_code)]
pub fn cache_capacity(mgr: &CacheManager) -> usize {
    mgr.capacity_bytes
}

#[allow(dead_code)]
pub fn cache_hit_rate(mgr: &CacheManager) -> f32 {
    let total = mgr.hits + mgr.misses;
    if total == 0 {
        0.0
    } else {
        mgr.hits as f32 / total as f32
    }
}

#[allow(dead_code)]
pub fn cache_miss_count(mgr: &CacheManager) -> u64 {
    mgr.misses
}

#[allow(dead_code)]
pub fn cache_hit_count(mgr: &CacheManager) -> u64 {
    mgr.hits
}

#[allow(dead_code)]
pub fn clear_cache(mgr: &mut CacheManager) {
    mgr.entries.clear();
    mgr.used_bytes = 0;
}

#[allow(dead_code)]
pub fn cache_to_json(mgr: &CacheManager) -> String {
    let mut parts = Vec::new();
    parts.push(format!("\"entry_count\":{}", mgr.entries.len()));
    parts.push(format!("\"used_bytes\":{}", mgr.used_bytes));
    parts.push(format!("\"capacity_bytes\":{}", mgr.capacity_bytes));
    parts.push(format!("\"hits\":{}", mgr.hits));
    parts.push(format!("\"misses\":{}", mgr.misses));
    let hit_rate = cache_hit_rate(mgr);
    parts.push(format!("\"hit_rate\":{hit_rate:.4}"));
    let policy_str = match mgr.policy {
        CachePolicy::Lru => "lru",
        CachePolicy::Lfu => "lfu",
        CachePolicy::Fifo => "fifo",
    };
    parts.push(format!("\"policy\":\"{policy_str}\""));
    format!("{{{}}}", parts.join(","))
}

#[allow(dead_code)]
pub fn set_cache_policy(mgr: &mut CacheManager, policy: CachePolicy) {
    mgr.policy = policy;
}

#[allow(dead_code)]
pub fn cache_contains(mgr: &CacheManager, key: &str) -> bool {
    mgr.entries.iter().any(|e| e.key == key)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mgr() -> CacheManager {
        new_cache_manager(1024, CachePolicy::Lru)
    }

    #[test]
    fn test_new_cache_manager() {
        let mgr = make_mgr();
        assert_eq!(cache_capacity(&mgr), 1024);
        assert_eq!(cache_size(&mgr), 0);
    }

    #[test]
    fn test_cache_insert_and_contains() {
        let mut mgr = make_mgr();
        cache_insert(&mut mgr, "key1", vec![0u8; 100]);
        assert!(cache_contains(&mgr, "key1"));
    }

    #[test]
    fn test_cache_get_hit() {
        let mut mgr = make_mgr();
        cache_insert(&mut mgr, "a", vec![1, 2, 3]);
        let data = cache_get(&mut mgr, "a");
        assert!(data.is_some());
        assert_eq!(data.unwrap(), &[1, 2, 3]);
    }

    #[test]
    fn test_cache_get_miss() {
        let mut mgr = make_mgr();
        let data = cache_get(&mut mgr, "missing");
        assert!(data.is_none());
    }

    #[test]
    fn test_cache_hit_rate_all_hits() {
        let mut mgr = make_mgr();
        cache_insert(&mut mgr, "x", vec![0u8; 10]);
        cache_get(&mut mgr, "x");
        cache_get(&mut mgr, "x");
        assert!((cache_hit_rate(&mgr) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_cache_hit_rate_mixed() {
        let mut mgr = make_mgr();
        cache_insert(&mut mgr, "x", vec![0u8; 10]);
        cache_get(&mut mgr, "x"); // hit
        cache_get(&mut mgr, "none"); // miss
        let rate = cache_hit_rate(&mgr);
        assert!((rate - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_cache_miss_count() {
        let mut mgr = make_mgr();
        cache_get(&mut mgr, "nope");
        cache_get(&mut mgr, "nope2");
        assert_eq!(cache_miss_count(&mgr), 2);
    }

    #[test]
    fn test_cache_hit_count() {
        let mut mgr = make_mgr();
        cache_insert(&mut mgr, "y", vec![0u8; 5]);
        cache_get(&mut mgr, "y");
        assert_eq!(cache_hit_count(&mgr), 1);
    }

    #[test]
    fn test_clear_cache() {
        let mut mgr = make_mgr();
        cache_insert(&mut mgr, "a", vec![0u8; 50]);
        cache_insert(&mut mgr, "b", vec![0u8; 50]);
        clear_cache(&mut mgr);
        assert_eq!(cache_size(&mgr), 0);
        assert!(!cache_contains(&mgr, "a"));
    }

    #[test]
    fn test_lru_eviction() {
        let mut mgr = new_cache_manager(200, CachePolicy::Lru);
        cache_insert(&mut mgr, "a", vec![0u8; 100]);
        cache_insert(&mut mgr, "b", vec![0u8; 100]);
        // Access "a" to make "b" LRU
        cache_get(&mut mgr, "a");
        // Insert "c" which forces eviction
        cache_insert(&mut mgr, "c", vec![0u8; 100]);
        // "b" should be evicted (LRU)
        assert!(!cache_contains(&mgr, "b"));
        assert!(cache_contains(&mgr, "a"));
        assert!(cache_contains(&mgr, "c"));
    }

    #[test]
    fn test_fifo_eviction() {
        let mut mgr = new_cache_manager(200, CachePolicy::Fifo);
        cache_insert(&mut mgr, "first", vec![0u8; 100]);
        cache_insert(&mut mgr, "second", vec![0u8; 100]);
        cache_insert(&mut mgr, "third", vec![0u8; 100]);
        // "first" should be evicted
        assert!(!cache_contains(&mgr, "first"));
        assert!(cache_contains(&mgr, "second"));
    }

    #[test]
    fn test_lfu_eviction() {
        let mut mgr = new_cache_manager(200, CachePolicy::Lfu);
        cache_insert(&mut mgr, "rare", vec![0u8; 100]);
        cache_insert(&mut mgr, "freq", vec![0u8; 100]);
        // Access "freq" more times
        cache_get(&mut mgr, "freq");
        cache_get(&mut mgr, "freq");
        // Inserting another entry triggers eviction - "rare" has fewer accesses
        cache_insert(&mut mgr, "new", vec![0u8; 100]);
        assert!(!cache_contains(&mgr, "rare"));
        assert!(cache_contains(&mgr, "freq"));
    }

    #[test]
    fn test_set_cache_policy() {
        let mut mgr = make_mgr();
        set_cache_policy(&mut mgr, CachePolicy::Fifo);
        assert_eq!(mgr.policy, CachePolicy::Fifo);
    }

    #[test]
    fn test_cache_to_json() {
        let mgr = make_mgr();
        let json = cache_to_json(&mgr);
        assert!(json.contains("capacity_bytes"));
        assert!(json.contains("hit_rate"));
        assert!(json.contains("policy"));
    }

    #[test]
    fn test_cache_evict_one_empty() {
        let mut mgr = make_mgr();
        let evicted = cache_evict_one(&mut mgr);
        assert!(!evicted);
    }

    #[test]
    fn test_insert_overwrites_same_key() {
        let mut mgr = make_mgr();
        cache_insert(&mut mgr, "dup", vec![1u8; 10]);
        cache_insert(&mut mgr, "dup", vec![2u8; 20]);
        // Should not double-count
        assert!(cache_size(&mgr) <= 20 + 10);
        let count = mgr.entries.iter().filter(|e| e.key == "dup").count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_cache_hit_rate_no_accesses() {
        let mgr = make_mgr();
        assert!((cache_hit_rate(&mgr)).abs() < 1e-6);
    }
}
