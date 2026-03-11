// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! String interning pool: deduplicates string allocations.

#![allow(dead_code)]

use std::collections::HashMap;

/// A pool that interns strings to avoid duplicate allocations.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StringPool {
    strings: Vec<String>,
    index: HashMap<String, usize>,
}

/// A handle to an interned string.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternedString(usize);

/// Create a new empty string pool.
#[allow(dead_code)]
pub fn new_string_pool() -> StringPool {
    StringPool {
        strings: Vec::new(),
        index: HashMap::new(),
    }
}

/// Intern a string, returning a handle. If the string is already interned, returns the existing handle.
#[allow(dead_code)]
pub fn intern(pool: &mut StringPool, s: &str) -> InternedString {
    if let Some(&id) = pool.index.get(s) {
        return InternedString(id);
    }
    let id = pool.strings.len();
    pool.strings.push(s.to_string());
    pool.index.insert(s.to_string(), id);
    InternedString(id)
}

/// Resolve an interned handle back to a string slice.
#[allow(dead_code)]
pub fn resolve(pool: &StringPool, handle: InternedString) -> Option<&str> {
    pool.strings.get(handle.0).map(|s| s.as_str())
}

/// Return the number of unique strings in the pool.
#[allow(dead_code)]
pub fn pool_size(pool: &StringPool) -> usize {
    pool.strings.len()
}

/// Check whether a string is already in the pool.
#[allow(dead_code)]
pub fn pool_contains(pool: &StringPool, s: &str) -> bool {
    pool.index.contains_key(s)
}

/// Clear all interned strings.
#[allow(dead_code)]
pub fn pool_clear(pool: &mut StringPool) {
    pool.strings.clear();
    pool.index.clear();
}

/// Return (count, total_bytes) statistics.
#[allow(dead_code)]
pub fn pool_stats(pool: &StringPool) -> (usize, usize) {
    let bytes: usize = pool.strings.iter().map(|s| s.len()).sum();
    (pool.strings.len(), bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pool_is_empty() {
        let pool = new_string_pool();
        assert_eq!(pool_size(&pool), 0);
    }

    #[test]
    fn test_intern_returns_handle() {
        let mut pool = new_string_pool();
        let h = intern(&mut pool, "hello");
        assert_eq!(pool_size(&pool), 1);
        assert_eq!(resolve(&pool, h), Some("hello"));
    }

    #[test]
    fn test_intern_deduplicates() {
        let mut pool = new_string_pool();
        let h1 = intern(&mut pool, "world");
        let h2 = intern(&mut pool, "world");
        assert_eq!(h1, h2);
        assert_eq!(pool_size(&pool), 1);
    }

    #[test]
    fn test_resolve_unknown_returns_none() {
        let pool = new_string_pool();
        assert!(resolve(&pool, InternedString(99)).is_none());
    }

    #[test]
    fn test_pool_contains() {
        let mut pool = new_string_pool();
        intern(&mut pool, "abc");
        assert!(pool_contains(&pool, "abc"));
        assert!(!pool_contains(&pool, "xyz"));
    }

    #[test]
    fn test_pool_clear() {
        let mut pool = new_string_pool();
        intern(&mut pool, "a");
        intern(&mut pool, "b");
        pool_clear(&mut pool);
        assert_eq!(pool_size(&pool), 0);
        assert!(!pool_contains(&pool, "a"));
    }

    #[test]
    fn test_pool_stats() {
        let mut pool = new_string_pool();
        intern(&mut pool, "hi");
        intern(&mut pool, "world");
        let (count, bytes) = pool_stats(&pool);
        assert_eq!(count, 2);
        assert_eq!(bytes, 7); // "hi"=2, "world"=5
    }

    #[test]
    fn test_multiple_strings() {
        let mut pool = new_string_pool();
        let h1 = intern(&mut pool, "alpha");
        let h2 = intern(&mut pool, "beta");
        let h3 = intern(&mut pool, "gamma");
        assert_ne!(h1, h2);
        assert_ne!(h2, h3);
        assert_eq!(resolve(&pool, h1), Some("alpha"));
        assert_eq!(resolve(&pool, h2), Some("beta"));
        assert_eq!(resolve(&pool, h3), Some("gamma"));
    }

    #[test]
    fn test_empty_string_internable() {
        let mut pool = new_string_pool();
        let h = intern(&mut pool, "");
        assert_eq!(resolve(&pool, h), Some(""));
        assert_eq!(pool_size(&pool), 1);
    }
}
