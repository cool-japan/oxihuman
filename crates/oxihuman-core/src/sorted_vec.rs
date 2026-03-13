#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

/// A vector that maintains sorted order.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SortedVec {
    data: Vec<i64>,
}

#[allow(dead_code)]
pub fn new_sorted_vec() -> SortedVec {
    SortedVec { data: Vec::new() }
}

#[allow(dead_code)]
pub fn sorted_insert(sv: &mut SortedVec, value: i64) {
    let pos = sv.data.partition_point(|&x| x < value);
    sv.data.insert(pos, value);
}

#[allow(dead_code)]
pub fn sorted_contains(sv: &SortedVec, value: i64) -> bool {
    sv.data.binary_search(&value).is_ok()
}

#[allow(dead_code)]
pub fn sorted_remove(sv: &mut SortedVec, value: i64) -> bool {
    if let Ok(pos) = sv.data.binary_search(&value) {
        sv.data.remove(pos);
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn sorted_at(sv: &SortedVec, index: usize) -> Option<i64> {
    sv.data.get(index).copied()
}

#[allow(dead_code)]
pub fn sorted_len(sv: &SortedVec) -> usize {
    sv.data.len()
}

#[allow(dead_code)]
pub fn sorted_first(sv: &SortedVec) -> Option<i64> {
    sv.data.first().copied()
}

#[allow(dead_code)]
pub fn sorted_last(sv: &SortedVec) -> Option<i64> {
    sv.data.last().copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_sorted_vec() {
        let sv = new_sorted_vec();
        assert_eq!(sorted_len(&sv), 0);
    }

    #[test]
    fn test_sorted_insert() {
        let mut sv = new_sorted_vec();
        sorted_insert(&mut sv, 30);
        sorted_insert(&mut sv, 10);
        sorted_insert(&mut sv, 20);
        assert_eq!(sv.data, vec![10, 20, 30]);
    }

    #[test]
    fn test_sorted_contains() {
        let mut sv = new_sorted_vec();
        sorted_insert(&mut sv, 5);
        assert!(sorted_contains(&sv, 5));
        assert!(!sorted_contains(&sv, 6));
    }

    #[test]
    fn test_sorted_remove() {
        let mut sv = new_sorted_vec();
        sorted_insert(&mut sv, 5);
        assert!(sorted_remove(&mut sv, 5));
        assert!(!sorted_remove(&mut sv, 5));
    }

    #[test]
    fn test_sorted_at() {
        let mut sv = new_sorted_vec();
        sorted_insert(&mut sv, 10);
        sorted_insert(&mut sv, 20);
        assert_eq!(sorted_at(&sv, 0), Some(10));
        assert_eq!(sorted_at(&sv, 1), Some(20));
    }

    #[test]
    fn test_sorted_at_out_of_bounds() {
        let sv = new_sorted_vec();
        assert!(sorted_at(&sv, 0).is_none());
    }

    #[test]
    fn test_sorted_len() {
        let mut sv = new_sorted_vec();
        sorted_insert(&mut sv, 1);
        sorted_insert(&mut sv, 2);
        assert_eq!(sorted_len(&sv), 2);
    }

    #[test]
    fn test_sorted_first() {
        let mut sv = new_sorted_vec();
        sorted_insert(&mut sv, 30);
        sorted_insert(&mut sv, 10);
        assert_eq!(sorted_first(&sv), Some(10));
    }

    #[test]
    fn test_sorted_last() {
        let mut sv = new_sorted_vec();
        sorted_insert(&mut sv, 10);
        sorted_insert(&mut sv, 30);
        assert_eq!(sorted_last(&sv), Some(30));
    }

    #[test]
    fn test_empty_first_last() {
        let sv = new_sorted_vec();
        assert!(sorted_first(&sv).is_none());
        assert!(sorted_last(&sv).is_none());
    }
}
