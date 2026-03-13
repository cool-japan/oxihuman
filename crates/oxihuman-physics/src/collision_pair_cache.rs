#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Cache for collision pair tracking.

use std::collections::HashSet;

/// A unique key for a collision pair.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CollisionPairKey {
    pub a: u32,
    pub b: u32,
}

/// Cache of active collision pairs.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PairCache {
    pairs: HashSet<CollisionPairKey>,
}

#[allow(dead_code)]
pub fn new_pair_cache() -> PairCache {
    PairCache { pairs: HashSet::new() }
}

#[allow(dead_code)]
pub fn add_pair(cache: &mut PairCache, a: u32, b: u32) -> bool {
    let key = pair_key(a, b);
    cache.pairs.insert(key)
}

#[allow(dead_code)]
pub fn remove_pair(cache: &mut PairCache, a: u32, b: u32) -> bool {
    let key = pair_key(a, b);
    cache.pairs.remove(&key)
}

#[allow(dead_code)]
pub fn has_pair(cache: &PairCache, a: u32, b: u32) -> bool {
    let key = pair_key(a, b);
    cache.pairs.contains(&key)
}

#[allow(dead_code)]
pub fn pair_count(cache: &PairCache) -> usize {
    cache.pairs.len()
}

#[allow(dead_code)]
pub fn clear_pairs(cache: &mut PairCache) {
    cache.pairs.clear();
}

#[allow(dead_code)]
pub fn all_pairs(cache: &PairCache) -> Vec<CollisionPairKey> {
    cache.pairs.iter().copied().collect()
}

#[allow(dead_code)]
pub fn pair_key(a: u32, b: u32) -> CollisionPairKey {
    // Canonical ordering so (a,b) == (b,a).
    if a <= b {
        CollisionPairKey { a, b }
    } else {
        CollisionPairKey { a: b, b: a }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let c = new_pair_cache();
        assert_eq!(pair_count(&c), 0);
    }

    #[test]
    fn test_add() {
        let mut c = new_pair_cache();
        assert!(add_pair(&mut c, 1, 2));
        assert_eq!(pair_count(&c), 1);
    }

    #[test]
    fn test_duplicate() {
        let mut c = new_pair_cache();
        add_pair(&mut c, 1, 2);
        assert!(!add_pair(&mut c, 1, 2));
    }

    #[test]
    fn test_symmetric() {
        let mut c = new_pair_cache();
        add_pair(&mut c, 1, 2);
        assert!(has_pair(&c, 2, 1));
    }

    #[test]
    fn test_remove() {
        let mut c = new_pair_cache();
        add_pair(&mut c, 3, 4);
        assert!(remove_pair(&mut c, 3, 4));
        assert!(!has_pair(&c, 3, 4));
    }

    #[test]
    fn test_remove_missing() {
        let mut c = new_pair_cache();
        assert!(!remove_pair(&mut c, 1, 2));
    }

    #[test]
    fn test_clear() {
        let mut c = new_pair_cache();
        add_pair(&mut c, 1, 2);
        add_pair(&mut c, 3, 4);
        clear_pairs(&mut c);
        assert_eq!(pair_count(&c), 0);
    }

    #[test]
    fn test_all_pairs() {
        let mut c = new_pair_cache();
        add_pair(&mut c, 1, 2);
        let all = all_pairs(&c);
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn test_pair_key_canonical() {
        let k1 = pair_key(5, 3);
        let k2 = pair_key(3, 5);
        assert_eq!(k1, k2);
    }

    #[test]
    fn test_has_pair_missing() {
        let c = new_pair_cache();
        assert!(!has_pair(&c, 0, 1));
    }
}
