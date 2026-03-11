//! Sparse set data structure for fast iteration and O(1) membership test (u32 keys).
//!
//! Provides O(1) insert, remove, and contains operations backed by two arrays:
//! a dense array for iteration and a sparse array for key→index lookup.

#![allow(dead_code)]

/// Configuration for a `SparseSet`.
#[derive(Debug, Clone)]
pub struct SparseSetConfig {
    /// Maximum key value that can be stored (exclusive upper bound).
    pub universe_size: usize,
    /// Maximum number of elements stored simultaneously.
    pub max_elements: usize,
}

/// A sparse set mapping u32 keys to their positions in the dense array.
#[derive(Debug, Clone)]
pub struct SparseSet {
    config: SparseSetConfig,
    /// Sparse array: sparse[key] = index in dense (or usize::MAX if absent).
    sparse: Vec<usize>,
    /// Dense array of stored keys.
    dense: Vec<u32>,
}

const ABSENT: usize = usize::MAX;

/// Build a default `SparseSetConfig`.
#[allow(dead_code)]
pub fn default_sparse_set_config() -> SparseSetConfig {
    SparseSetConfig {
        universe_size: 4096,
        max_elements: 1024,
    }
}

/// Create a new `SparseSet`.
#[allow(dead_code)]
pub fn new_sparse_set(config: SparseSetConfig) -> SparseSet {
    let sparse = vec![ABSENT; config.universe_size];
    SparseSet {
        config,
        sparse,
        dense: Vec::new(),
    }
}

/// Insert a key. Returns `true` if inserted, `false` if already present or out of range.
#[allow(dead_code)]
pub fn sparse_insert(set: &mut SparseSet, key: u32) -> bool {
    let k = key as usize;
    if k >= set.config.universe_size {
        return false;
    }
    if set.sparse[k] != ABSENT {
        return false; // already present
    }
    if set.dense.len() >= set.config.max_elements {
        return false;
    }
    let idx = set.dense.len();
    set.sparse[k] = idx;
    set.dense.push(key);
    true
}

/// Remove a key. Returns `true` if removed, `false` if not found.
#[allow(dead_code)]
pub fn sparse_remove(set: &mut SparseSet, key: u32) -> bool {
    let k = key as usize;
    if k >= set.config.universe_size || set.sparse[k] == ABSENT {
        return false;
    }
    let idx = set.sparse[k];
    let Some(&last) = set.dense.last() else { return false; };
    // Swap with last
    set.dense[idx] = last;
    set.sparse[last as usize] = idx;
    set.dense.pop();
    set.sparse[k] = ABSENT;
    true
}

/// Return `true` if the key is present.
#[allow(dead_code)]
pub fn sparse_contains(set: &SparseSet, key: u32) -> bool {
    let k = key as usize;
    k < set.config.universe_size && set.sparse[k] != ABSENT
}

/// Return the number of elements stored.
#[allow(dead_code)]
pub fn sparse_len(set: &SparseSet) -> usize {
    set.dense.len()
}

/// Return `true` if no elements are stored.
#[allow(dead_code)]
pub fn sparse_is_empty(set: &SparseSet) -> bool {
    set.dense.is_empty()
}

/// Return a slice over all stored keys (dense array).
#[allow(dead_code)]
pub fn sparse_iter_all(set: &SparseSet) -> &[u32] {
    &set.dense
}

/// Serialize the set state to a JSON string.
#[allow(dead_code)]
pub fn sparse_to_json(set: &SparseSet) -> String {
    let keys: Vec<String> = set.dense.iter().map(|k| k.to_string()).collect();
    format!(
        "{{\"len\":{},\"universe_size\":{},\"keys\":[{}]}}",
        set.dense.len(),
        set.config.universe_size,
        keys.join(",")
    )
}

/// Remove all elements.
#[allow(dead_code)]
pub fn sparse_clear(set: &mut SparseSet) {
    for &k in &set.dense {
        set.sparse[k as usize] = ABSENT;
    }
    set.dense.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_set() -> SparseSet {
        new_sparse_set(default_sparse_set_config())
    }

    #[test]
    fn test_empty_initially() {
        let s = make_set();
        assert!(sparse_is_empty(&s));
        assert_eq!(sparse_len(&s), 0);
    }

    #[test]
    fn test_insert_and_contains() {
        let mut s = make_set();
        assert!(sparse_insert(&mut s, 42));
        assert!(sparse_contains(&s, 42));
        assert!(!sparse_contains(&s, 43));
    }

    #[test]
    fn test_duplicate_insert_fails() {
        let mut s = make_set();
        assert!(sparse_insert(&mut s, 7));
        assert!(!sparse_insert(&mut s, 7));
        assert_eq!(sparse_len(&s), 1);
    }

    #[test]
    fn test_remove() {
        let mut s = make_set();
        sparse_insert(&mut s, 10);
        assert!(sparse_remove(&mut s, 10));
        assert!(!sparse_contains(&s, 10));
        assert_eq!(sparse_len(&s), 0);
    }

    #[test]
    fn test_remove_absent_fails() {
        let mut s = make_set();
        assert!(!sparse_remove(&mut s, 99));
    }

    #[test]
    fn test_iter_all() {
        let mut s = make_set();
        sparse_insert(&mut s, 1);
        sparse_insert(&mut s, 2);
        sparse_insert(&mut s, 3);
        let keys = sparse_iter_all(&s);
        assert_eq!(keys.len(), 3);
    }

    #[test]
    fn test_clear() {
        let mut s = make_set();
        sparse_insert(&mut s, 5);
        sparse_insert(&mut s, 10);
        sparse_clear(&mut s);
        assert!(sparse_is_empty(&s));
        assert!(!sparse_contains(&s, 5));
    }

    #[test]
    fn test_to_json_contains_len() {
        let mut s = make_set();
        sparse_insert(&mut s, 3);
        let json = sparse_to_json(&s);
        assert!(json.contains("\"len\":1"));
    }

    #[test]
    fn test_out_of_range_key_ignored() {
        let cfg = SparseSetConfig {
            universe_size: 10,
            max_elements: 10,
        };
        let mut s = new_sparse_set(cfg);
        assert!(!sparse_insert(&mut s, 10)); // exactly at limit, excluded
        assert!(!sparse_contains(&s, 10));
    }

    #[test]
    fn test_remove_swap_maintains_integrity() {
        let mut s = make_set();
        sparse_insert(&mut s, 0);
        sparse_insert(&mut s, 1);
        sparse_insert(&mut s, 2);
        sparse_remove(&mut s, 0); // swap 0 with last (2)
        assert!(!sparse_contains(&s, 0));
        assert!(sparse_contains(&s, 1));
        assert!(sparse_contains(&s, 2));
    }
}
