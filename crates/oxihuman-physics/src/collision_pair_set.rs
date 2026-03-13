// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! A set of collision pairs (unordered body-id pairs) for tracking contacts.

use std::collections::HashSet;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CollisionPairId {
    lo: u32,
    hi: u32,
}

#[allow(dead_code)]
impl CollisionPairId {
    pub fn new(a: u32, b: u32) -> Self {
        if a <= b { Self { lo: a, hi: b } } else { Self { lo: b, hi: a } }
    }

    pub fn lo(self) -> u32 { self.lo }
    pub fn hi(self) -> u32 { self.hi }

    pub fn contains(self, id: u32) -> bool {
        self.lo == id || self.hi == id
    }

    pub fn other(self, id: u32) -> Option<u32> {
        if id == self.lo { Some(self.hi) }
        else if id == self.hi { Some(self.lo) }
        else { None }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollisionPairSet {
    pairs: HashSet<CollisionPairId>,
}

#[allow(dead_code)]
impl CollisionPairSet {
    pub fn new() -> Self {
        Self { pairs: HashSet::new() }
    }

    pub fn insert(&mut self, a: u32, b: u32) -> bool {
        self.pairs.insert(CollisionPairId::new(a, b))
    }

    pub fn remove(&mut self, a: u32, b: u32) -> bool {
        self.pairs.remove(&CollisionPairId::new(a, b))
    }

    pub fn contains(&self, a: u32, b: u32) -> bool {
        self.pairs.contains(&CollisionPairId::new(a, b))
    }

    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    pub fn clear(&mut self) {
        self.pairs.clear();
    }

    pub fn pairs_for(&self, body_id: u32) -> Vec<CollisionPairId> {
        self.pairs.iter().filter(|p| p.contains(body_id)).copied().collect()
    }

    pub fn all_pairs(&self) -> Vec<CollisionPairId> {
        self.pairs.iter().copied().collect()
    }

    pub fn unique_bodies(&self) -> HashSet<u32> {
        let mut ids = HashSet::new();
        for p in &self.pairs {
            ids.insert(p.lo);
            ids.insert(p.hi);
        }
        ids
    }
}

impl Default for CollisionPairSet {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_contains() {
        let mut s = CollisionPairSet::new();
        s.insert(1, 2);
        assert!(s.contains(1, 2));
        assert!(s.contains(2, 1));
    }

    #[test]
    fn test_no_duplicates() {
        let mut s = CollisionPairSet::new();
        assert!(s.insert(1, 2));
        assert!(!s.insert(2, 1));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn test_remove() {
        let mut s = CollisionPairSet::new();
        s.insert(3, 4);
        assert!(s.remove(4, 3));
        assert!(!s.contains(3, 4));
    }

    #[test]
    fn test_clear() {
        let mut s = CollisionPairSet::new();
        s.insert(1, 2);
        s.clear();
        assert!(s.is_empty());
    }

    #[test]
    fn test_pairs_for() {
        let mut s = CollisionPairSet::new();
        s.insert(1, 2);
        s.insert(1, 3);
        s.insert(4, 5);
        let p = s.pairs_for(1);
        assert_eq!(p.len(), 2);
    }

    #[test]
    fn test_unique_bodies() {
        let mut s = CollisionPairSet::new();
        s.insert(1, 2);
        s.insert(2, 3);
        let bodies = s.unique_bodies();
        assert_eq!(bodies.len(), 3);
    }

    #[test]
    fn test_pair_id_other() {
        let p = CollisionPairId::new(5, 10);
        assert_eq!(p.other(5), Some(10));
        assert_eq!(p.other(10), Some(5));
        assert_eq!(p.other(99), None);
    }

    #[test]
    fn test_pair_id_contains() {
        let p = CollisionPairId::new(3, 7);
        assert!(p.contains(3));
        assert!(p.contains(7));
        assert!(!p.contains(5));
    }

    #[test]
    fn test_empty() {
        let s = CollisionPairSet::new();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn test_all_pairs() {
        let mut s = CollisionPairSet::new();
        s.insert(1, 2);
        s.insert(3, 4);
        assert_eq!(s.all_pairs().len(), 2);
    }
}
