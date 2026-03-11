#![allow(dead_code)]

//! Deformer result caching with invalidation.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DeformerCacheEntry {
    pub deformer_name: String,
    pub result: Vec<[f32; 3]>,
    pub version: u64,
    pub valid: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DeformerCache {
    pub entries: HashMap<String, DeformerCacheEntry>,
    pub global_version: u64,
}

#[allow(dead_code)]
pub fn new_deformer_cache() -> DeformerCache {
    DeformerCache {
        entries: HashMap::new(),
        global_version: 0,
    }
}

#[allow(dead_code)]
pub fn dc_store(cache: &mut DeformerCache, deformer: &str, result: Vec<[f32; 3]>) {
    cache.global_version += 1;
    cache.entries.insert(
        deformer.to_string(),
        DeformerCacheEntry {
            deformer_name: deformer.to_string(),
            result,
            version: cache.global_version,
            valid: true,
        },
    );
}

#[allow(dead_code)]
pub fn dc_get<'a>(cache: &'a DeformerCache, deformer: &str) -> Option<&'a Vec<[f32; 3]>> {
    cache
        .entries
        .get(deformer)
        .filter(|e| e.valid)
        .map(|e| &e.result)
}

#[allow(dead_code)]
pub fn dc_invalidate(cache: &mut DeformerCache, deformer: &str) {
    if let Some(e) = cache.entries.get_mut(deformer) {
        e.valid = false;
    }
}

#[allow(dead_code)]
pub fn dc_invalidate_all(cache: &mut DeformerCache) {
    for e in cache.entries.values_mut() {
        e.valid = false;
    }
}

#[allow(dead_code)]
pub fn dc_is_valid(cache: &DeformerCache, deformer: &str) -> bool {
    cache
        .entries
        .get(deformer)
        .is_some_and(|e| e.valid)
}

#[allow(dead_code)]
pub fn dc_count(cache: &DeformerCache) -> usize {
    cache.entries.len()
}

#[allow(dead_code)]
pub fn dc_valid_count(cache: &DeformerCache) -> usize {
    cache.entries.values().filter(|e| e.valid).count()
}

#[allow(dead_code)]
pub fn dc_remove(cache: &mut DeformerCache, deformer: &str) {
    cache.entries.remove(deformer);
}

#[allow(dead_code)]
pub fn dc_clear(cache: &mut DeformerCache) {
    cache.entries.clear();
}

#[allow(dead_code)]
pub fn dc_to_json(cache: &DeformerCache) -> String {
    format!(
        "{{\"count\":{},\"valid_count\":{},\"global_version\":{}}}",
        cache.entries.len(),
        dc_valid_count(cache),
        cache.global_version
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cache() {
        let c = new_deformer_cache();
        assert_eq!(dc_count(&c), 0);
    }

    #[test]
    fn test_store_and_get() {
        let mut c = new_deformer_cache();
        dc_store(&mut c, "smooth", vec![[1.0, 0.0, 0.0]]);
        assert!(dc_get(&c, "smooth").is_some());
    }

    #[test]
    fn test_get_nonexistent() {
        let c = new_deformer_cache();
        assert!(dc_get(&c, "missing").is_none());
    }

    #[test]
    fn test_invalidate() {
        let mut c = new_deformer_cache();
        dc_store(&mut c, "twist", vec![]);
        dc_invalidate(&mut c, "twist");
        assert!(!dc_is_valid(&c, "twist"));
    }

    #[test]
    fn test_get_invalid_returns_none() {
        let mut c = new_deformer_cache();
        dc_store(&mut c, "twist", vec![]);
        dc_invalidate(&mut c, "twist");
        assert!(dc_get(&c, "twist").is_none());
    }

    #[test]
    fn test_invalidate_all() {
        let mut c = new_deformer_cache();
        dc_store(&mut c, "a", vec![]);
        dc_store(&mut c, "b", vec![]);
        dc_invalidate_all(&mut c);
        assert_eq!(dc_valid_count(&c), 0);
    }

    #[test]
    fn test_remove() {
        let mut c = new_deformer_cache();
        dc_store(&mut c, "d", vec![]);
        dc_remove(&mut c, "d");
        assert_eq!(dc_count(&c), 0);
    }

    #[test]
    fn test_clear() {
        let mut c = new_deformer_cache();
        dc_store(&mut c, "a", vec![]);
        dc_store(&mut c, "b", vec![]);
        dc_clear(&mut c);
        assert_eq!(dc_count(&c), 0);
    }

    #[test]
    fn test_global_version_increments() {
        let mut c = new_deformer_cache();
        dc_store(&mut c, "a", vec![]);
        dc_store(&mut c, "b", vec![]);
        assert!(c.global_version >= 2);
    }

    #[test]
    fn test_to_json() {
        let c = new_deformer_cache();
        let json = dc_to_json(&c);
        assert!(json.contains("global_version"));
    }
}
