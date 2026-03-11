#![allow(dead_code)]

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SparseArray<T> {
    data: HashMap<usize, T>,
}

#[allow(dead_code)]
pub fn new_sparse_array<T>() -> SparseArray<T> {
    SparseArray {
        data: HashMap::new(),
    }
}

#[allow(dead_code)]
pub fn sparse_set_val<T>(arr: &mut SparseArray<T>, idx: usize, val: T) {
    arr.data.insert(idx, val);
}

#[allow(dead_code)]
pub fn sparse_get<T>(arr: &SparseArray<T>, idx: usize) -> Option<&T> {
    arr.data.get(&idx)
}

#[allow(dead_code)]
pub fn sparse_has<T>(arr: &SparseArray<T>, idx: usize) -> bool {
    arr.data.contains_key(&idx)
}

#[allow(dead_code)]
pub fn sparse_remove<T>(arr: &mut SparseArray<T>, idx: usize) -> bool {
    arr.data.remove(&idx).is_some()
}

#[allow(dead_code)]
pub fn sparse_count<T>(arr: &SparseArray<T>) -> usize {
    arr.data.len()
}

#[allow(dead_code)]
pub fn sparse_keys<T>(arr: &SparseArray<T>) -> Vec<usize> {
    let mut keys: Vec<usize> = arr.data.keys().copied().collect();
    keys.sort();
    keys
}

#[allow(dead_code)]
pub fn sparse_clear<T>(arr: &mut SparseArray<T>) {
    arr.data.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let arr: SparseArray<i32> = new_sparse_array();
        assert_eq!(sparse_count(&arr), 0);
    }

    #[test]
    fn test_set_and_get() {
        let mut arr = new_sparse_array();
        sparse_set_val(&mut arr, 10, "hello");
        assert_eq!(sparse_get(&arr, 10), Some(&"hello"));
    }

    #[test]
    fn test_has() {
        let mut arr = new_sparse_array();
        sparse_set_val(&mut arr, 5, 42);
        assert!(sparse_has(&arr, 5));
        assert!(!sparse_has(&arr, 6));
    }

    #[test]
    fn test_remove() {
        let mut arr = new_sparse_array();
        sparse_set_val(&mut arr, 0, 1);
        assert!(sparse_remove(&mut arr, 0));
        assert!(!sparse_remove(&mut arr, 0));
    }

    #[test]
    fn test_count() {
        let mut arr = new_sparse_array();
        sparse_set_val(&mut arr, 100, 'a');
        sparse_set_val(&mut arr, 200, 'b');
        assert_eq!(sparse_count(&arr), 2);
    }

    #[test]
    fn test_keys() {
        let mut arr = new_sparse_array();
        sparse_set_val(&mut arr, 30, 0);
        sparse_set_val(&mut arr, 10, 0);
        sparse_set_val(&mut arr, 20, 0);
        assert_eq!(sparse_keys(&arr), vec![10, 20, 30]);
    }

    #[test]
    fn test_clear() {
        let mut arr = new_sparse_array();
        sparse_set_val(&mut arr, 1, 1);
        sparse_clear(&mut arr);
        assert_eq!(sparse_count(&arr), 0);
    }

    #[test]
    fn test_overwrite() {
        let mut arr = new_sparse_array();
        sparse_set_val(&mut arr, 0, 1);
        sparse_set_val(&mut arr, 0, 2);
        assert_eq!(sparse_get(&arr, 0), Some(&2));
    }

    #[test]
    fn test_get_missing() {
        let arr: SparseArray<i32> = new_sparse_array();
        assert_eq!(sparse_get(&arr, 999), None);
    }

    #[test]
    fn test_empty_keys() {
        let arr: SparseArray<i32> = new_sparse_array();
        assert!(sparse_keys(&arr).is_empty());
    }
}
