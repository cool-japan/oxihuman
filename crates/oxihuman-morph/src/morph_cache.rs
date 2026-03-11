#![allow(dead_code)]

//! Morph target result cache with dirty tracking.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphCache {
    pub entries: HashMap<String, Vec<[f32; 3]>>,
    pub dirty_keys: Vec<String>,
    pub max_entries: usize,
}

#[allow(dead_code)]
pub fn new_morph_cache(max_entries: usize) -> MorphCache {
    MorphCache {
        entries: HashMap::new(),
        dirty_keys: Vec::new(),
        max_entries,
    }
}

#[allow(dead_code)]
pub fn mc_insert(cache: &mut MorphCache, key: &str, deltas: Vec<[f32; 3]>) {
    if cache.entries.len() >= cache.max_entries {
        if let Some(oldest) = cache.dirty_keys.first().cloned() {
            cache.entries.remove(&oldest);
            cache.dirty_keys.retain(|k| k != &oldest);
        }
    }
    cache.entries.insert(key.to_string(), deltas);
    cache.dirty_keys.retain(|k| k != key);
}

#[allow(dead_code)]
pub fn mc_get<'a>(cache: &'a MorphCache, key: &str) -> Option<&'a Vec<[f32; 3]>> {
    cache.entries.get(key)
}

#[allow(dead_code)]
pub fn mc_mark_dirty(cache: &mut MorphCache, key: &str) {
    if !cache.dirty_keys.contains(&key.to_string()) {
        cache.dirty_keys.push(key.to_string());
    }
}

#[allow(dead_code)]
pub fn mc_is_dirty(cache: &MorphCache, key: &str) -> bool {
    cache.dirty_keys.contains(&key.to_string())
}

#[allow(dead_code)]
pub fn mc_clear(cache: &mut MorphCache) {
    cache.entries.clear();
    cache.dirty_keys.clear();
}

#[allow(dead_code)]
pub fn mc_count(cache: &MorphCache) -> usize {
    cache.entries.len()
}

#[allow(dead_code)]
pub fn mc_dirty_count(cache: &MorphCache) -> usize {
    cache.dirty_keys.len()
}

#[allow(dead_code)]
pub fn mc_remove(cache: &mut MorphCache, key: &str) {
    cache.entries.remove(key);
    cache.dirty_keys.retain(|k| k != key);
}

#[allow(dead_code)]
pub fn mc_to_json(cache: &MorphCache) -> String {
    format!(
        "{{\"count\":{},\"dirty_count\":{},\"max_entries\":{}}}",
        cache.entries.len(),
        cache.dirty_keys.len(),
        cache.max_entries
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_morph_cache() {
        let cache = new_morph_cache(16);
        assert_eq!(mc_count(&cache), 0);
    }

    #[test]
    fn test_insert_and_get() {
        let mut cache = new_morph_cache(16);
        mc_insert(&mut cache, "blink", vec![[0.1, 0.0, 0.0]]);
        assert!(mc_get(&cache, "blink").is_some());
    }

    #[test]
    fn test_mark_dirty() {
        let mut cache = new_morph_cache(16);
        mc_mark_dirty(&mut cache, "jaw");
        assert!(mc_is_dirty(&cache, "jaw"));
    }

    #[test]
    fn test_not_dirty_after_insert() {
        let mut cache = new_morph_cache(16);
        mc_mark_dirty(&mut cache, "jaw");
        mc_insert(&mut cache, "jaw", vec![[0.0, 0.1, 0.0]]);
        assert!(!mc_is_dirty(&cache, "jaw"));
    }

    #[test]
    fn test_clear() {
        let mut cache = new_morph_cache(16);
        mc_insert(&mut cache, "a", vec![]);
        mc_clear(&mut cache);
        assert_eq!(mc_count(&cache), 0);
    }

    #[test]
    fn test_dirty_count() {
        let mut cache = new_morph_cache(16);
        mc_mark_dirty(&mut cache, "x");
        mc_mark_dirty(&mut cache, "y");
        assert_eq!(mc_dirty_count(&cache), 2);
    }

    #[test]
    fn test_remove() {
        let mut cache = new_morph_cache(16);
        mc_insert(&mut cache, "test", vec![]);
        mc_remove(&mut cache, "test");
        assert!(mc_get(&cache, "test").is_none());
    }

    #[test]
    fn test_to_json() {
        let cache = new_morph_cache(8);
        let json = mc_to_json(&cache);
        assert!(json.contains("max_entries"));
    }

    #[test]
    fn test_no_duplicate_dirty() {
        let mut cache = new_morph_cache(16);
        mc_mark_dirty(&mut cache, "dup");
        mc_mark_dirty(&mut cache, "dup");
        assert_eq!(mc_dirty_count(&cache), 1);
    }

    #[test]
    fn test_eviction_at_capacity() {
        let mut cache = new_morph_cache(2);
        mc_mark_dirty(&mut cache, "a");
        mc_insert(&mut cache, "a", vec![]);
        mc_insert(&mut cache, "b", vec![]);
        mc_mark_dirty(&mut cache, "a");
        mc_insert(&mut cache, "c", vec![]);
        assert!(mc_count(&cache) <= 2);
    }
}
