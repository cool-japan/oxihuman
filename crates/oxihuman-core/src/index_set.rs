#![allow(dead_code)]

use std::collections::BTreeSet;

/// An ordered set of unique indices.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IndexSet {
    set: BTreeSet<usize>,
}

/// Creates a new empty index set.
#[allow(dead_code)]
pub fn new_index_set() -> IndexSet {
    IndexSet {
        set: BTreeSet::new(),
    }
}

/// Inserts an index. Returns true if it was new.
#[allow(dead_code)]
pub fn index_set_insert(iset: &mut IndexSet, index: usize) -> bool {
    iset.set.insert(index)
}

/// Checks if the set contains an index.
#[allow(dead_code)]
pub fn index_set_contains(iset: &IndexSet, index: usize) -> bool {
    iset.set.contains(&index)
}

/// Removes an index. Returns true if it was present.
#[allow(dead_code)]
pub fn index_set_remove(iset: &mut IndexSet, index: usize) -> bool {
    iset.set.remove(&index)
}

/// Returns the number of indices.
#[allow(dead_code)]
pub fn index_set_len(iset: &IndexSet) -> usize {
    iset.set.len()
}

/// Returns all indices as a sorted Vec.
#[allow(dead_code)]
pub fn index_set_to_vec(iset: &IndexSet) -> Vec<usize> {
    iset.set.iter().copied().collect()
}

/// Clears the set.
#[allow(dead_code)]
pub fn index_set_clear(iset: &mut IndexSet) {
    iset.set.clear();
}

/// Returns the union of two index sets.
#[allow(dead_code)]
pub fn index_set_union(a: &IndexSet, b: &IndexSet) -> IndexSet {
    IndexSet {
        set: a.set.union(&b.set).copied().collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_index_set() {
        let iset = new_index_set();
        assert_eq!(index_set_len(&iset), 0);
    }

    #[test]
    fn test_insert() {
        let mut iset = new_index_set();
        assert!(index_set_insert(&mut iset, 5));
        assert!(!index_set_insert(&mut iset, 5));
    }

    #[test]
    fn test_contains() {
        let mut iset = new_index_set();
        index_set_insert(&mut iset, 3);
        assert!(index_set_contains(&iset, 3));
        assert!(!index_set_contains(&iset, 4));
    }

    #[test]
    fn test_remove() {
        let mut iset = new_index_set();
        index_set_insert(&mut iset, 10);
        assert!(index_set_remove(&mut iset, 10));
        assert!(!index_set_contains(&iset, 10));
    }

    #[test]
    fn test_len() {
        let mut iset = new_index_set();
        index_set_insert(&mut iset, 1);
        index_set_insert(&mut iset, 2);
        assert_eq!(index_set_len(&iset), 2);
    }

    #[test]
    fn test_to_vec() {
        let mut iset = new_index_set();
        index_set_insert(&mut iset, 3);
        index_set_insert(&mut iset, 1);
        index_set_insert(&mut iset, 2);
        assert_eq!(index_set_to_vec(&iset), vec![1, 2, 3]);
    }

    #[test]
    fn test_clear() {
        let mut iset = new_index_set();
        index_set_insert(&mut iset, 1);
        index_set_clear(&mut iset);
        assert_eq!(index_set_len(&iset), 0);
    }

    #[test]
    fn test_union() {
        let mut a = new_index_set();
        index_set_insert(&mut a, 1);
        index_set_insert(&mut a, 2);
        let mut b = new_index_set();
        index_set_insert(&mut b, 2);
        index_set_insert(&mut b, 3);
        let u = index_set_union(&a, &b);
        assert_eq!(index_set_to_vec(&u), vec![1, 2, 3]);
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut iset = new_index_set();
        assert!(!index_set_remove(&mut iset, 99));
    }

    #[test]
    fn test_duplicates() {
        let mut iset = new_index_set();
        index_set_insert(&mut iset, 5);
        index_set_insert(&mut iset, 5);
        index_set_insert(&mut iset, 5);
        assert_eq!(index_set_len(&iset), 1);
    }
}
