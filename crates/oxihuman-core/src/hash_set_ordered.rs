#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

/// An ordered hash set that preserves insertion order.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OrderedHashSet {
    items: Vec<String>,
}

#[allow(dead_code)]
pub fn new_ordered_set() -> OrderedHashSet {
    OrderedHashSet { items: Vec::new() }
}

#[allow(dead_code)]
pub fn ordered_insert(set: &mut OrderedHashSet, item: &str) -> bool {
    if set.items.iter().any(|s| s == item) {
        return false;
    }
    set.items.push(item.to_string());
    true
}

#[allow(dead_code)]
pub fn ordered_contains(set: &OrderedHashSet, item: &str) -> bool {
    set.items.iter().any(|s| s == item)
}

#[allow(dead_code)]
pub fn ordered_remove(set: &mut OrderedHashSet, item: &str) -> bool {
    if let Some(pos) = set.items.iter().position(|s| s == item) {
        set.items.remove(pos);
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn ordered_set_len(set: &OrderedHashSet) -> usize {
    set.items.len()
}

#[allow(dead_code)]
pub fn ordered_to_vec(set: &OrderedHashSet) -> Vec<String> {
    set.items.clone()
}

#[allow(dead_code)]
pub fn ordered_set_clear(set: &mut OrderedHashSet) {
    set.items.clear();
}

#[allow(dead_code)]
pub fn ordered_union_set(a: &OrderedHashSet, b: &OrderedHashSet) -> OrderedHashSet {
    let mut result = a.clone();
    for item in &b.items {
        ordered_insert(&mut result, item);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_ordered_set() {
        let s = new_ordered_set();
        assert_eq!(ordered_set_len(&s), 0);
    }

    #[test]
    fn test_ordered_insert() {
        let mut s = new_ordered_set();
        assert!(ordered_insert(&mut s, "a"));
        assert!(!ordered_insert(&mut s, "a"));
    }

    #[test]
    fn test_ordered_contains() {
        let mut s = new_ordered_set();
        ordered_insert(&mut s, "x");
        assert!(ordered_contains(&s, "x"));
        assert!(!ordered_contains(&s, "y"));
    }

    #[test]
    fn test_ordered_remove() {
        let mut s = new_ordered_set();
        ordered_insert(&mut s, "x");
        assert!(ordered_remove(&mut s, "x"));
        assert!(!ordered_remove(&mut s, "x"));
    }

    #[test]
    fn test_ordered_set_len() {
        let mut s = new_ordered_set();
        ordered_insert(&mut s, "a");
        ordered_insert(&mut s, "b");
        assert_eq!(ordered_set_len(&s), 2);
    }

    #[test]
    fn test_ordered_to_vec() {
        let mut s = new_ordered_set();
        ordered_insert(&mut s, "a");
        ordered_insert(&mut s, "b");
        let v = ordered_to_vec(&s);
        assert_eq!(v, vec!["a", "b"]);
    }

    #[test]
    fn test_ordered_set_clear() {
        let mut s = new_ordered_set();
        ordered_insert(&mut s, "a");
        ordered_set_clear(&mut s);
        assert_eq!(ordered_set_len(&s), 0);
    }

    #[test]
    fn test_ordered_union_set() {
        let mut a = new_ordered_set();
        ordered_insert(&mut a, "x");
        let mut b = new_ordered_set();
        ordered_insert(&mut b, "y");
        ordered_insert(&mut b, "x");
        let u = ordered_union_set(&a, &b);
        assert_eq!(ordered_set_len(&u), 2);
    }

    #[test]
    fn test_preserves_order() {
        let mut s = new_ordered_set();
        ordered_insert(&mut s, "c");
        ordered_insert(&mut s, "a");
        ordered_insert(&mut s, "b");
        assert_eq!(ordered_to_vec(&s), vec!["c", "a", "b"]);
    }

    #[test]
    fn test_remove_middle() {
        let mut s = new_ordered_set();
        ordered_insert(&mut s, "a");
        ordered_insert(&mut s, "b");
        ordered_insert(&mut s, "c");
        ordered_remove(&mut s, "b");
        assert_eq!(ordered_to_vec(&s), vec!["a", "c"]);
    }
}
