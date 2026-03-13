// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Export a cache manifest describing asset dependencies and versions.

/// Cache manifest entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub key: String,
    pub version: u32,
    pub size_bytes: usize,
    pub checksum: u64,
}

/// Cache manifest.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CacheManifest {
    pub entries: Vec<CacheEntry>,
}

#[allow(dead_code)]
pub fn new_cache_manifest() -> CacheManifest {
    CacheManifest { entries: Vec::new() }
}

#[allow(dead_code)]
pub fn cache_add_entry(manifest: &mut CacheManifest, key: &str, version: u32, size: usize) {
    let checksum = simple_hash(key.as_bytes());
    manifest.entries.push(CacheEntry {
        key: key.to_string(), version, size_bytes: size, checksum,
    });
}

#[allow(dead_code)]
pub fn cache_entry_count(manifest: &CacheManifest) -> usize {
    manifest.entries.len()
}

#[allow(dead_code)]
pub fn cache_total_size(manifest: &CacheManifest) -> usize {
    manifest.entries.iter().map(|e| e.size_bytes).sum()
}

#[allow(dead_code)]
pub fn cache_find_entry(manifest: &CacheManifest, key: &str) -> Option<&CacheEntry> {
    manifest.entries.iter().find(|e| e.key == key)
}

#[allow(dead_code)]
pub fn cache_manifest_to_json(manifest: &CacheManifest) -> String {
    let entries: Vec<String> = manifest.entries.iter().map(|e| {
        format!(r#"{{"key":"{}","version":{},"size":{},"checksum":{}}}"#,
            e.key, e.version, e.size_bytes, e.checksum)
    }).collect();
    format!(r#"{{"entries":[{}]}}"#, entries.join(","))
}

#[allow(dead_code)]
pub fn cache_remove_entry(manifest: &mut CacheManifest, key: &str) -> bool {
    let before = manifest.entries.len();
    manifest.entries.retain(|e| e.key != key);
    manifest.entries.len() < before
}

#[allow(dead_code)]
pub fn cache_clear(manifest: &mut CacheManifest) {
    manifest.entries.clear();
}

fn simple_hash(data: &[u8]) -> u64 {
    let mut h: u64 = 5381;
    for &b in data {
        h = h.wrapping_mul(33).wrapping_add(b as u64);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_manifest() {
        let m = new_cache_manifest();
        assert_eq!(cache_entry_count(&m), 0);
    }

    #[test]
    fn test_add_entry() {
        let mut m = new_cache_manifest();
        cache_add_entry(&mut m, "mesh.glb", 1, 1024);
        assert_eq!(cache_entry_count(&m), 1);
    }

    #[test]
    fn test_total_size() {
        let mut m = new_cache_manifest();
        cache_add_entry(&mut m, "a", 1, 100);
        cache_add_entry(&mut m, "b", 1, 200);
        assert_eq!(cache_total_size(&m), 300);
    }

    #[test]
    fn test_find_entry() {
        let mut m = new_cache_manifest();
        cache_add_entry(&mut m, "tex.png", 2, 500);
        let e = cache_find_entry(&m, "tex.png");
        assert!(e.is_some());
        assert_eq!(e.expect("should succeed").version, 2);
    }

    #[test]
    fn test_find_missing() {
        let m = new_cache_manifest();
        assert!(cache_find_entry(&m, "nope").is_none());
    }

    #[test]
    fn test_to_json() {
        let mut m = new_cache_manifest();
        cache_add_entry(&mut m, "data", 1, 64);
        let json = cache_manifest_to_json(&m);
        assert!(json.contains("data"));
    }

    #[test]
    fn test_remove_entry() {
        let mut m = new_cache_manifest();
        cache_add_entry(&mut m, "x", 1, 10);
        assert!(cache_remove_entry(&mut m, "x"));
        assert_eq!(cache_entry_count(&m), 0);
    }

    #[test]
    fn test_clear() {
        let mut m = new_cache_manifest();
        cache_add_entry(&mut m, "a", 1, 10);
        cache_add_entry(&mut m, "b", 1, 20);
        cache_clear(&mut m);
        assert_eq!(cache_entry_count(&m), 0);
    }

    #[test]
    fn test_checksum_deterministic() {
        let mut m1 = new_cache_manifest();
        let mut m2 = new_cache_manifest();
        cache_add_entry(&mut m1, "same", 1, 10);
        cache_add_entry(&mut m2, "same", 1, 10);
        assert_eq!(m1.entries[0].checksum, m2.entries[0].checksum);
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut m = new_cache_manifest();
        assert!(!cache_remove_entry(&mut m, "nope"));
    }

}
