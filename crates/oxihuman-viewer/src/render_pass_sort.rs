// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! RenderPassSort — sort keys and ordering for render passes.

#![allow(dead_code)]

/// A sort key encoding rendering order.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SortKey {
    /// Lower values are drawn first.
    pub key: u64,
}

/// Container for sorted render pass entries.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderPassSort {
    pub entries: Vec<SortEntry>,
}

/// An entry pairing a name with a sort key.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SortEntry {
    pub name: String,
    pub key: SortKey,
    pub depth: f32,
    pub material_id: u32,
    pub is_opaque: bool,
}

/// Create a new sort key from a raw u64.
#[allow(dead_code)]
pub fn new_sort_key(key: u64) -> SortKey {
    SortKey { key }
}

/// Compare two sort keys. Returns `Ordering`.
#[allow(dead_code)]
pub fn compare_sort_keys(a: &SortKey, b: &SortKey) -> std::cmp::Ordering {
    a.key.cmp(&b.key)
}

/// Sort all entries by their sort key (ascending).
#[allow(dead_code)]
pub fn sort_render_passes(rps: &mut RenderPassSort) {
    rps.entries.sort_by(|a, b| a.key.cmp(&b.key));
}

/// Sort entries by depth (front-to-back for opaque).
#[allow(dead_code)]
pub fn sort_by_depth(rps: &mut RenderPassSort) {
    rps.entries.sort_by(|a, b| a.depth.partial_cmp(&b.depth).unwrap_or(std::cmp::Ordering::Equal));
}

/// Sort entries by material id.
#[allow(dead_code)]
pub fn sort_by_material(rps: &mut RenderPassSort) {
    rps.entries.sort_by_key(|e| e.material_id);
}

/// Sort opaque entries first, then transparent.
#[allow(dead_code)]
pub fn sort_opaque_first(rps: &mut RenderPassSort) {
    rps.entries.sort_by_key(|e| if e.is_opaque { 0u8 } else { 1u8 });
}

/// Convert a sort key to its raw u64 representation.
#[allow(dead_code)]
pub fn sort_key_to_u64(key: &SortKey) -> u64 {
    key.key
}

/// Number of entries in the sort.
#[allow(dead_code)]
pub fn sort_pass_count(rps: &RenderPassSort) -> usize {
    rps.entries.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_rps() -> RenderPassSort {
        RenderPassSort {
            entries: vec![
                SortEntry { name: "b".into(), key: new_sort_key(20), depth: 5.0, material_id: 2, is_opaque: false },
                SortEntry { name: "a".into(), key: new_sort_key(10), depth: 1.0, material_id: 1, is_opaque: true },
                SortEntry { name: "c".into(), key: new_sort_key(15), depth: 3.0, material_id: 3, is_opaque: true },
            ],
        }
    }

    #[test]
    fn test_new_sort_key() {
        let k = new_sort_key(42);
        assert_eq!(k.key, 42);
    }

    #[test]
    fn test_compare_sort_keys() {
        let a = new_sort_key(1);
        let b = new_sort_key(2);
        assert_eq!(compare_sort_keys(&a, &b), std::cmp::Ordering::Less);
    }

    #[test]
    fn test_sort_render_passes() {
        let mut rps = sample_rps();
        sort_render_passes(&mut rps);
        assert_eq!(rps.entries[0].name, "a");
        assert_eq!(rps.entries[2].name, "b");
    }

    #[test]
    fn test_sort_by_depth() {
        let mut rps = sample_rps();
        sort_by_depth(&mut rps);
        assert!((rps.entries[0].depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_sort_by_material() {
        let mut rps = sample_rps();
        sort_by_material(&mut rps);
        assert_eq!(rps.entries[0].material_id, 1);
    }

    #[test]
    fn test_sort_opaque_first() {
        let mut rps = sample_rps();
        sort_opaque_first(&mut rps);
        assert!(rps.entries[0].is_opaque);
        assert!(!rps.entries.last().unwrap().is_opaque);
    }

    #[test]
    fn test_sort_key_to_u64() {
        let k = new_sort_key(99);
        assert_eq!(sort_key_to_u64(&k), 99);
    }

    #[test]
    fn test_sort_pass_count() {
        let rps = sample_rps();
        assert_eq!(sort_pass_count(&rps), 3);
    }

    #[test]
    fn test_sort_pass_count_empty() {
        let rps = RenderPassSort { entries: vec![] };
        assert_eq!(sort_pass_count(&rps), 0);
    }

    #[test]
    fn test_compare_equal_keys() {
        let a = new_sort_key(5);
        let b = new_sort_key(5);
        assert_eq!(compare_sort_keys(&a, &b), std::cmp::Ordering::Equal);
    }
}
