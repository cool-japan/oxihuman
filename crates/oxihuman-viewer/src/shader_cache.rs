// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A cached shader program entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShaderCacheEntry {
    pub name: String,
    pub vert_hash: u64,
    pub frag_hash: u64,
    pub program_id: u32,
}

/// Shader program cache.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ShaderCache {
    pub entries: Vec<ShaderCacheEntry>,
}

/// Create a new empty shader cache.
#[allow(dead_code)]
pub fn new_shader_cache() -> ShaderCache {
    ShaderCache { entries: Vec::new() }
}

/// Insert a shader into the cache.
#[allow(dead_code)]
pub fn cache_shader(cache: &mut ShaderCache, name: &str, vh: u64, fh: u64, prog: u32) {
    cache.entries.push(ShaderCacheEntry {
        name: name.to_string(),
        vert_hash: vh,
        frag_hash: fh,
        program_id: prog,
    });
}

/// Look up a shader by (vert_hash, frag_hash), returning the program ID.
#[allow(dead_code)]
pub fn find_shader(cache: &ShaderCache, vh: u64, fh: u64) -> Option<u32> {
    cache.entries.iter().find(|e| e.vert_hash == vh && e.frag_hash == fh).map(|e| e.program_id)
}

/// Remove a shader entry by name.
#[allow(dead_code)]
pub fn evict_shader(cache: &mut ShaderCache, name: &str) {
    cache.entries.retain(|e| e.name != name);
}

/// Return the number of cached shaders.
#[allow(dead_code)]
pub fn shader_count(cache: &ShaderCache) -> usize {
    cache.entries.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_cache_empty() {
        let c = new_shader_cache();
        assert_eq!(shader_count(&c), 0);
    }

    #[test]
    fn cache_shader_increments_count() {
        let mut c = new_shader_cache();
        cache_shader(&mut c, "pbr", 1, 2, 10);
        assert_eq!(shader_count(&c), 1);
    }

    #[test]
    fn find_shader_existing() {
        let mut c = new_shader_cache();
        cache_shader(&mut c, "pbr", 111, 222, 42);
        assert_eq!(find_shader(&c, 111, 222), Some(42));
    }

    #[test]
    fn find_shader_missing() {
        let c = new_shader_cache();
        assert!(find_shader(&c, 0, 0).is_none());
    }

    #[test]
    fn evict_removes_entry() {
        let mut c = new_shader_cache();
        cache_shader(&mut c, "test", 1, 2, 1);
        evict_shader(&mut c, "test");
        assert_eq!(shader_count(&c), 0);
    }

    #[test]
    fn evict_nonexistent_is_ok() {
        let mut c = new_shader_cache();
        evict_shader(&mut c, "ghost"); // should not panic
        assert_eq!(shader_count(&c), 0);
    }

    #[test]
    fn find_after_evict_returns_none() {
        let mut c = new_shader_cache();
        cache_shader(&mut c, "s", 5, 6, 99);
        evict_shader(&mut c, "s");
        assert!(find_shader(&c, 5, 6).is_none());
    }

    #[test]
    fn name_stored_correctly() {
        let mut c = new_shader_cache();
        cache_shader(&mut c, "unlit", 7, 8, 1);
        assert_eq!(c.entries[0].name, "unlit");
    }

    #[test]
    fn multiple_entries_distinct() {
        let mut c = new_shader_cache();
        cache_shader(&mut c, "a", 1, 2, 10);
        cache_shader(&mut c, "b", 3, 4, 20);
        assert_eq!(find_shader(&c, 1, 2), Some(10));
        assert_eq!(find_shader(&c, 3, 4), Some(20));
    }

    #[test]
    fn evict_only_named_entry() {
        let mut c = new_shader_cache();
        cache_shader(&mut c, "keep", 1, 2, 1);
        cache_shader(&mut c, "remove", 3, 4, 2);
        evict_shader(&mut c, "remove");
        assert_eq!(shader_count(&c), 1);
        assert_eq!(c.entries[0].name, "keep");
    }
}
