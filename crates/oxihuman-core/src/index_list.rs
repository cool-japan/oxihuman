// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Index list: ordered list of integer indices with fast membership check.

use std::collections::HashSet;

/// Ordered index list with O(1) contains check.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct IndexList {
    order: Vec<usize>,
    set: HashSet<usize>,
}

/// Create a new IndexList.
#[allow(dead_code)]
pub fn new_index_list() -> IndexList {
    IndexList {
        order: Vec::new(),
        set: HashSet::new(),
    }
}

/// Append an index; returns false if already present.
#[allow(dead_code)]
pub fn il_push(list: &mut IndexList, idx: usize) -> bool {
    if list.set.contains(&idx) {
        return false;
    }
    list.order.push(idx);
    list.set.insert(idx);
    true
}

/// Remove an index; preserves order of remaining elements.
#[allow(dead_code)]
pub fn il_remove(list: &mut IndexList, idx: usize) -> bool {
    if !list.set.remove(&idx) {
        return false;
    }
    list.order.retain(|&x| x != idx);
    true
}

/// Whether index is in the list.
#[allow(dead_code)]
pub fn il_contains(list: &IndexList, idx: usize) -> bool {
    list.set.contains(&idx)
}

/// Number of indices.
#[allow(dead_code)]
pub fn il_len(list: &IndexList) -> usize {
    list.order.len()
}

/// Whether empty.
#[allow(dead_code)]
pub fn il_is_empty(list: &IndexList) -> bool {
    list.order.is_empty()
}

/// Get index at position.
#[allow(dead_code)]
pub fn il_get(list: &IndexList, pos: usize) -> Option<usize> {
    list.order.get(pos).copied()
}

/// Clear all indices.
#[allow(dead_code)]
pub fn il_clear(list: &mut IndexList) {
    list.order.clear();
    list.set.clear();
}

/// Ordered slice of all indices.
#[allow(dead_code)]
pub fn il_as_slice(list: &IndexList) -> &[usize] {
    &list.order
}

/// Merge another list into this one (skip duplicates).
#[allow(dead_code)]
pub fn il_merge(list: &mut IndexList, other: &IndexList) {
    for &idx in &other.order {
        il_push(list, idx);
    }
}

/// Retain only indices satisfying predicate.
#[allow(dead_code)]
pub fn il_retain<F: Fn(usize) -> bool>(list: &mut IndexList, f: F) {
    let old = std::mem::take(&mut list.order);
    list.set.clear();
    for idx in old {
        if f(idx) {
            list.order.push(idx);
            list.set.insert(idx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_contains() {
        let mut list = new_index_list();
        assert!(il_push(&mut list, 3));
        assert!(il_contains(&list, 3));
    }

    #[test]
    fn test_no_duplicates() {
        let mut list = new_index_list();
        il_push(&mut list, 5);
        assert!(!il_push(&mut list, 5));
        assert_eq!(il_len(&list), 1);
    }

    #[test]
    fn test_remove() {
        let mut list = new_index_list();
        il_push(&mut list, 7);
        assert!(il_remove(&mut list, 7));
        assert!(!il_contains(&list, 7));
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut list = new_index_list();
        assert!(!il_remove(&mut list, 99));
    }

    #[test]
    fn test_order_preserved() {
        let mut list = new_index_list();
        il_push(&mut list, 10);
        il_push(&mut list, 5);
        il_push(&mut list, 20);
        assert_eq!(il_as_slice(&list), &[10, 5, 20]);
    }

    #[test]
    fn test_clear() {
        let mut list = new_index_list();
        il_push(&mut list, 1);
        il_clear(&mut list);
        assert!(il_is_empty(&list));
    }

    #[test]
    fn test_merge() {
        let mut a = new_index_list();
        il_push(&mut a, 1);
        let mut b = new_index_list();
        il_push(&mut b, 2);
        il_push(&mut b, 1);
        il_merge(&mut a, &b);
        assert_eq!(il_len(&a), 2);
    }

    #[test]
    fn test_retain() {
        let mut list = new_index_list();
        for i in 0..6 {
            il_push(&mut list, i);
        }
        il_retain(&mut list, |x| x % 2 == 0);
        assert_eq!(il_as_slice(&list), &[0, 2, 4]);
    }

    #[test]
    fn test_get() {
        let mut list = new_index_list();
        il_push(&mut list, 42);
        assert_eq!(il_get(&list, 0), Some(42));
        assert_eq!(il_get(&list, 1), None);
    }
}
