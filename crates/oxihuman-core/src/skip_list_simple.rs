// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Skip list stub backed by a sorted Vec of i64 keys.

pub struct SkipListSimple {
    pub entries: Vec<i64>,
    pub max_level: usize,
}

pub fn new_skip_list_simple(max_level: usize) -> SkipListSimple {
    SkipListSimple {
        entries: Vec::new(),
        max_level,
    }
}

pub fn skip_simple_insert(s: &mut SkipListSimple, key: i64) {
    match s.entries.binary_search(&key) {
        Ok(_) => {} // already exists
        Err(pos) => s.entries.insert(pos, key),
    }
}

pub fn skip_simple_contains(s: &SkipListSimple, key: i64) -> bool {
    s.entries.binary_search(&key).is_ok()
}

pub fn skip_simple_remove(s: &mut SkipListSimple, key: i64) -> bool {
    match s.entries.binary_search(&key) {
        Ok(pos) => {
            s.entries.remove(pos);
            true
        }
        Err(_) => false,
    }
}

pub fn skip_simple_len(s: &SkipListSimple) -> usize {
    s.entries.len()
}

pub fn skip_simple_range(s: &SkipListSimple, lo: i64, hi: i64) -> Vec<i64> {
    s.entries
        .iter()
        .copied()
        .filter(|&k| k >= lo && k <= hi)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_skip_list_simple() {
        /* new list is empty */
        let s = new_skip_list_simple(4);
        assert_eq!(skip_simple_len(&s), 0);
    }

    #[test]
    fn test_insert_and_contains() {
        /* inserted key is contained */
        let mut s = new_skip_list_simple(4);
        skip_simple_insert(&mut s, 10);
        assert!(skip_simple_contains(&s, 10));
    }

    #[test]
    fn test_not_contains() {
        /* not inserted key not found */
        let mut s = new_skip_list_simple(4);
        skip_simple_insert(&mut s, 5);
        assert!(!skip_simple_contains(&s, 99));
    }

    #[test]
    fn test_remove() {
        /* removing existing key returns true and removes it */
        let mut s = new_skip_list_simple(4);
        skip_simple_insert(&mut s, 7);
        assert!(skip_simple_remove(&mut s, 7));
        assert!(!skip_simple_contains(&s, 7));
    }

    #[test]
    fn test_remove_nonexistent() {
        /* removing missing key returns false */
        let mut s = new_skip_list_simple(4);
        assert!(!skip_simple_remove(&mut s, 42));
    }

    #[test]
    fn test_range_query() {
        /* range query returns correct subset */
        let mut s = new_skip_list_simple(4);
        for k in [1i64, 3, 5, 7, 9] {
            skip_simple_insert(&mut s, k);
        }
        let r = skip_simple_range(&s, 3, 7);
        assert_eq!(r, vec![3, 5, 7]);
    }

    #[test]
    fn test_sorted_order() {
        /* entries remain sorted */
        let mut s = new_skip_list_simple(4);
        for k in [5i64, 1, 3, 2, 4] {
            skip_simple_insert(&mut s, k);
        }
        assert_eq!(s.entries, vec![1, 2, 3, 4, 5]);
    }
}
