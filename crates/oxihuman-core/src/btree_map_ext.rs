// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[allow(dead_code)]
pub struct BTreeMapExt {
    pub map: std::collections::BTreeMap<String, f64>,
}

#[allow(dead_code)]
pub fn new_btree_map_ext() -> BTreeMapExt {
    BTreeMapExt { map: std::collections::BTreeMap::new() }
}

#[allow(dead_code)]
pub fn bte_insert(m: &mut BTreeMapExt, key: &str, val: f64) {
    m.map.insert(key.to_string(), val);
}

#[allow(dead_code)]
pub fn bte_get(m: &BTreeMapExt, key: &str) -> Option<f64> {
    m.map.get(key).copied()
}

#[allow(dead_code)]
pub fn bte_sum(m: &BTreeMapExt) -> f64 {
    m.map.values().sum()
}

#[allow(dead_code)]
pub fn bte_min_val(m: &BTreeMapExt) -> Option<f64> {
    m.map.values().copied().reduce(f64::min)
}

#[allow(dead_code)]
pub fn bte_max_val(m: &BTreeMapExt) -> Option<f64> {
    m.map.values().copied().reduce(f64::max)
}

#[allow(dead_code)]
pub fn bte_count(m: &BTreeMapExt) -> usize {
    m.map.len()
}

#[allow(dead_code)]
pub fn bte_remove(m: &mut BTreeMapExt, key: &str) -> bool {
    m.map.remove(key).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_get() {
        let mut m = new_btree_map_ext();
        bte_insert(&mut m, "a", 1.0);
        assert_eq!(bte_get(&m, "a"), Some(1.0));
    }

    #[test]
    fn test_get_missing() {
        let m = new_btree_map_ext();
        assert_eq!(bte_get(&m, "missing"), None);
    }

    #[test]
    fn test_sum() {
        let mut m = new_btree_map_ext();
        bte_insert(&mut m, "a", 1.0);
        bte_insert(&mut m, "b", 2.0);
        bte_insert(&mut m, "c", 3.0);
        assert!((bte_sum(&m) - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_min_val() {
        let mut m = new_btree_map_ext();
        bte_insert(&mut m, "a", 3.0);
        bte_insert(&mut m, "b", 1.0);
        bte_insert(&mut m, "c", 2.0);
        assert_eq!(bte_min_val(&m), Some(1.0));
    }

    #[test]
    fn test_max_val() {
        let mut m = new_btree_map_ext();
        bte_insert(&mut m, "a", 3.0);
        bte_insert(&mut m, "b", 1.0);
        bte_insert(&mut m, "c", 2.0);
        assert_eq!(bte_max_val(&m), Some(3.0));
    }

    #[test]
    fn test_count() {
        let mut m = new_btree_map_ext();
        assert_eq!(bte_count(&m), 0);
        bte_insert(&mut m, "x", 5.0);
        assert_eq!(bte_count(&m), 1);
    }

    #[test]
    fn test_remove() {
        let mut m = new_btree_map_ext();
        bte_insert(&mut m, "k", 9.0);
        assert!(bte_remove(&mut m, "k"));
        assert_eq!(bte_get(&m, "k"), None);
    }

    #[test]
    fn test_remove_missing() {
        let mut m = new_btree_map_ext();
        assert!(!bte_remove(&mut m, "nope"));
    }
}
