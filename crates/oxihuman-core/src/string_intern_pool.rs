#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! String interning pool for deduplication.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StringInternPool {
    strings: Vec<String>,
    index: HashMap<String, usize>,
}

#[allow(dead_code)]
pub fn new_string_intern_pool() -> StringInternPool {
    StringInternPool {
        strings: Vec::new(),
        index: HashMap::new(),
    }
}

#[allow(dead_code)]
pub fn sip_intern(pool: &mut StringInternPool, s: &str) -> usize {
    if let Some(&id) = pool.index.get(s) {
        return id;
    }
    let id = pool.strings.len();
    pool.strings.push(s.to_string());
    pool.index.insert(s.to_string(), id);
    id
}

#[allow(dead_code)]
pub fn sip_resolve(pool: &StringInternPool, id: usize) -> Option<&str> {
    pool.strings.get(id).map(|s| s.as_str())
}

#[allow(dead_code)]
pub fn sip_count(pool: &StringInternPool) -> usize {
    pool.strings.len()
}

#[allow(dead_code)]
pub fn sip_contains(pool: &StringInternPool, s: &str) -> bool {
    pool.index.contains_key(s)
}

#[allow(dead_code)]
pub fn sip_id_of(pool: &StringInternPool, s: &str) -> Option<usize> {
    pool.index.get(s).copied()
}

#[allow(dead_code)]
pub fn sip_to_vec(pool: &StringInternPool) -> Vec<String> {
    pool.strings.clone()
}

#[allow(dead_code)]
pub fn sip_clear(pool: &mut StringInternPool) {
    pool.strings.clear();
    pool.index.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pool() {
        let p = new_string_intern_pool();
        assert_eq!(sip_count(&p), 0);
    }

    #[test]
    fn test_intern() {
        let mut p = new_string_intern_pool();
        let id = sip_intern(&mut p, "hello");
        assert_eq!(id, 0);
    }

    #[test]
    fn test_intern_dedup() {
        let mut p = new_string_intern_pool();
        let id1 = sip_intern(&mut p, "hello");
        let id2 = sip_intern(&mut p, "hello");
        assert_eq!(id1, id2);
        assert_eq!(sip_count(&p), 1);
    }

    #[test]
    fn test_resolve() {
        let mut p = new_string_intern_pool();
        let id = sip_intern(&mut p, "world");
        assert_eq!(sip_resolve(&p, id), Some("world"));
    }

    #[test]
    fn test_resolve_invalid() {
        let p = new_string_intern_pool();
        assert_eq!(sip_resolve(&p, 99), None);
    }

    #[test]
    fn test_contains() {
        let mut p = new_string_intern_pool();
        sip_intern(&mut p, "yes");
        assert!(sip_contains(&p, "yes"));
        assert!(!sip_contains(&p, "no"));
    }

    #[test]
    fn test_id_of() {
        let mut p = new_string_intern_pool();
        sip_intern(&mut p, "test");
        assert_eq!(sip_id_of(&p, "test"), Some(0));
        assert_eq!(sip_id_of(&p, "nope"), None);
    }

    #[test]
    fn test_to_vec() {
        let mut p = new_string_intern_pool();
        sip_intern(&mut p, "a");
        sip_intern(&mut p, "b");
        assert_eq!(sip_to_vec(&p), vec!["a", "b"]);
    }

    #[test]
    fn test_clear() {
        let mut p = new_string_intern_pool();
        sip_intern(&mut p, "x");
        sip_clear(&mut p);
        assert_eq!(sip_count(&p), 0);
    }

    #[test]
    fn test_multiple_strings() {
        let mut p = new_string_intern_pool();
        let a = sip_intern(&mut p, "alpha");
        let b = sip_intern(&mut p, "beta");
        assert_ne!(a, b);
        assert_eq!(sip_count(&p), 2);
    }
}
