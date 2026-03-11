// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// An ordered pair of body indices for collision/constraint lookups.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BodyPair {
    pub a: u32,
    pub b: u32,
}

#[allow(dead_code)]
impl BodyPair {
    /// Creates a canonical (ordered) body pair where a <= b.
    pub fn new(a: u32, b: u32) -> Self {
        if a <= b {
            Self { a, b }
        } else {
            Self { a: b, b: a }
        }
    }

    pub fn contains(&self, id: u32) -> bool {
        self.a == id || self.b == id
    }

    pub fn other(&self, id: u32) -> Option<u32> {
        if self.a == id {
            Some(self.b)
        } else if self.b == id {
            Some(self.a)
        } else {
            None
        }
    }

    pub fn is_self_pair(&self) -> bool {
        self.a == self.b
    }

    pub fn to_key(&self) -> u64 {
        ((self.a as u64) << 32) | (self.b as u64)
    }

    pub fn from_key(key: u64) -> Self {
        let a = (key >> 32) as u32;
        let b = (key & 0xFFFF_FFFF) as u32;
        Self { a, b }
    }
}

/// A set of unique body pairs.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct BodyPairSet {
    pairs: Vec<BodyPair>,
}

#[allow(dead_code)]
impl BodyPairSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, pair: BodyPair) -> bool {
        if self.pairs.contains(&pair) {
            false
        } else {
            self.pairs.push(pair);
            true
        }
    }

    pub fn contains(&self, pair: &BodyPair) -> bool {
        self.pairs.contains(pair)
    }

    pub fn remove(&mut self, pair: &BodyPair) -> bool {
        let before = self.pairs.len();
        self.pairs.retain(|p| p != pair);
        self.pairs.len() < before
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

    pub fn pairs(&self) -> &[BodyPair] {
        &self.pairs
    }

    pub fn pairs_involving(&self, id: u32) -> Vec<BodyPair> {
        self.pairs.iter().filter(|p| p.contains(id)).copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_order() {
        let p = BodyPair::new(5, 3);
        assert_eq!(p.a, 3);
        assert_eq!(p.b, 5);
    }

    #[test]
    fn test_contains() {
        let p = BodyPair::new(1, 2);
        assert!(p.contains(1));
        assert!(p.contains(2));
        assert!(!p.contains(3));
    }

    #[test]
    fn test_other() {
        let p = BodyPair::new(1, 2);
        assert_eq!(p.other(1), Some(2));
        assert_eq!(p.other(2), Some(1));
        assert!(p.other(3).is_none());
    }

    #[test]
    fn test_is_self_pair() {
        assert!(BodyPair::new(5, 5).is_self_pair());
        assert!(!BodyPair::new(1, 2).is_self_pair());
    }

    #[test]
    fn test_key_roundtrip() {
        let p = BodyPair::new(100, 200);
        let key = p.to_key();
        let p2 = BodyPair::from_key(key);
        assert_eq!(p, p2);
    }

    #[test]
    fn test_set_insert() {
        let mut set = BodyPairSet::new();
        assert!(set.insert(BodyPair::new(1, 2)));
        assert!(!set.insert(BodyPair::new(2, 1)));
    }

    #[test]
    fn test_set_remove() {
        let mut set = BodyPairSet::new();
        set.insert(BodyPair::new(1, 2));
        assert!(set.remove(&BodyPair::new(1, 2)));
        assert!(set.is_empty());
    }

    #[test]
    fn test_pairs_involving() {
        let mut set = BodyPairSet::new();
        set.insert(BodyPair::new(1, 2));
        set.insert(BodyPair::new(1, 3));
        set.insert(BodyPair::new(4, 5));
        let involving = set.pairs_involving(1);
        assert_eq!(involving.len(), 2);
    }

    #[test]
    fn test_set_contains() {
        let mut set = BodyPairSet::new();
        set.insert(BodyPair::new(10, 20));
        assert!(set.contains(&BodyPair::new(10, 20)));
    }

    #[test]
    fn test_set_clear() {
        let mut set = BodyPairSet::new();
        set.insert(BodyPair::new(1, 2));
        set.clear();
        assert!(set.is_empty());
    }
}
