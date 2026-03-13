// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Simple B-tree stub backed by a sorted Vec of (key, value) pairs.

pub struct BTreeSimple {
    pub entries: Vec<(i64, i64)>,
}

pub fn new_btree_simple() -> BTreeSimple {
    BTreeSimple {
        entries: Vec::new(),
    }
}

pub fn btree_simple_insert(t: &mut BTreeSimple, key: i64, value: i64) {
    match t.entries.binary_search_by_key(&key, |&(k, _)| k) {
        Ok(pos) => t.entries[pos].1 = value, // update existing
        Err(pos) => t.entries.insert(pos, (key, value)),
    }
}

pub fn btree_simple_get(t: &BTreeSimple, key: i64) -> Option<i64> {
    t.entries
        .binary_search_by_key(&key, |&(k, _)| k)
        .ok()
        .map(|pos| t.entries[pos].1)
}

pub fn btree_simple_remove(t: &mut BTreeSimple, key: i64) -> bool {
    match t.entries.binary_search_by_key(&key, |&(k, _)| k) {
        Ok(pos) => {
            t.entries.remove(pos);
            true
        }
        Err(_) => false,
    }
}

pub fn btree_simple_len(t: &BTreeSimple) -> usize {
    t.entries.len()
}

pub fn btree_simple_range(t: &BTreeSimple, lo: i64, hi: i64) -> Vec<(i64, i64)> {
    t.entries
        .iter()
        .copied()
        .filter(|&(k, _)| k >= lo && k <= hi)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_btree_simple() {
        /* new tree is empty */
        let t = new_btree_simple();
        assert_eq!(btree_simple_len(&t), 0);
    }

    #[test]
    fn test_insert_and_get() {
        /* inserted key is retrievable */
        let mut t = new_btree_simple();
        btree_simple_insert(&mut t, 10, 100);
        assert_eq!(btree_simple_get(&t, 10), Some(100));
    }

    #[test]
    fn test_get_missing() {
        /* missing key returns None */
        let t = new_btree_simple();
        assert_eq!(btree_simple_get(&t, 99), None);
    }

    #[test]
    fn test_remove() {
        /* removing existing key returns true */
        let mut t = new_btree_simple();
        btree_simple_insert(&mut t, 5, 50);
        assert!(btree_simple_remove(&mut t, 5));
        assert_eq!(btree_simple_get(&t, 5), None);
    }

    #[test]
    fn test_remove_missing() {
        /* removing missing key returns false */
        let mut t = new_btree_simple();
        assert!(!btree_simple_remove(&mut t, 42));
    }

    #[test]
    fn test_range() {
        /* range query returns correct entries */
        let mut t = new_btree_simple();
        for k in [1i64, 3, 5, 7, 9] {
            btree_simple_insert(&mut t, k, k * 10);
        }
        let r = btree_simple_range(&t, 3, 7);
        assert_eq!(r.len(), 3);
        assert_eq!(r[0], (3, 30));
    }

    #[test]
    fn test_update_existing() {
        /* inserting same key updates the value */
        let mut t = new_btree_simple();
        btree_simple_insert(&mut t, 1, 10);
        btree_simple_insert(&mut t, 1, 99);
        assert_eq!(btree_simple_get(&t, 1), Some(99));
        assert_eq!(btree_simple_len(&t), 1);
    }
}
