#![allow(dead_code)]
//! Cache for materials to avoid redundant creation.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct CachedMaterial {
    pub name: String,
    pub shader: String,
    pub hit_count: u32,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct MaterialCache {
    materials: HashMap<String, CachedMaterial>,
    capacity: usize,
    hits: u64,
    misses: u64,
}

#[allow(dead_code)]
pub fn new_material_cache(capacity: usize) -> MaterialCache {
    MaterialCache {
        materials: HashMap::new(),
        capacity,
        hits: 0,
        misses: 0,
    }
}

#[allow(dead_code)]
pub fn cache_material(c: &mut MaterialCache, name: &str, shader: &str) -> bool {
    if c.materials.len() >= c.capacity && !c.materials.contains_key(name) {
        return false;
    }
    c.materials.insert(
        name.to_string(),
        CachedMaterial {
            name: name.to_string(),
            shader: shader.to_string(),
            hit_count: 0,
        },
    );
    true
}

#[allow(dead_code)]
pub fn get_cached_material<'a>(c: &'a mut MaterialCache, name: &str) -> Option<&'a CachedMaterial> {
    if c.materials.contains_key(name) {
        c.hits += 1;
        if let Some(m) = c.materials.get_mut(name) {
            m.hit_count += 1;
        }
        c.materials.get(name)
    } else {
        c.misses += 1;
        None
    }
}

#[allow(dead_code)]
pub fn material_cache_count(c: &MaterialCache) -> usize {
    c.materials.len()
}

#[allow(dead_code)]
pub fn evict_material(c: &mut MaterialCache, name: &str) -> bool {
    c.materials.remove(name).is_some()
}

#[allow(dead_code)]
pub fn cache_is_full(c: &MaterialCache) -> bool {
    c.materials.len() >= c.capacity
}

#[allow(dead_code)]
pub fn cache_hit_rate(c: &MaterialCache) -> f32 {
    let total = c.hits + c.misses;
    if total == 0 {
        return 0.0;
    }
    c.hits as f32 / total as f32
}

#[allow(dead_code)]
pub fn clear_material_cache(c: &mut MaterialCache) {
    c.materials.clear();
    c.hits = 0;
    c.misses = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_material_cache() {
        let c = new_material_cache(10);
        assert_eq!(material_cache_count(&c), 0);
    }

    #[test]
    fn test_cache_material() {
        let mut c = new_material_cache(10);
        assert!(cache_material(&mut c, "pbr", "standard"));
        assert_eq!(material_cache_count(&c), 1);
    }

    #[test]
    fn test_cache_full() {
        let mut c = new_material_cache(1);
        cache_material(&mut c, "a", "s");
        assert!(!cache_material(&mut c, "b", "s"));
    }

    #[test]
    fn test_get_cached_material() {
        let mut c = new_material_cache(10);
        cache_material(&mut c, "mat", "shader");
        assert!(get_cached_material(&mut c, "mat").is_some());
    }

    #[test]
    fn test_get_cached_material_miss() {
        let mut c = new_material_cache(10);
        assert!(get_cached_material(&mut c, "nope").is_none());
    }

    #[test]
    fn test_evict_material() {
        let mut c = new_material_cache(10);
        cache_material(&mut c, "a", "s");
        assert!(evict_material(&mut c, "a"));
        assert_eq!(material_cache_count(&c), 0);
    }

    #[test]
    fn test_cache_is_full() {
        let mut c = new_material_cache(1);
        assert!(!cache_is_full(&c));
        cache_material(&mut c, "a", "s");
        assert!(cache_is_full(&c));
    }

    #[test]
    fn test_cache_hit_rate() {
        let mut c = new_material_cache(10);
        cache_material(&mut c, "a", "s");
        get_cached_material(&mut c, "a");
        get_cached_material(&mut c, "miss");
        assert!((cache_hit_rate(&c) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clear_material_cache() {
        let mut c = new_material_cache(10);
        cache_material(&mut c, "a", "s");
        clear_material_cache(&mut c);
        assert_eq!(material_cache_count(&c), 0);
    }

    #[test]
    fn test_cache_hit_rate_empty() {
        let c = new_material_cache(10);
        assert!(cache_hit_rate(&c).abs() < 1e-6);
    }
}
