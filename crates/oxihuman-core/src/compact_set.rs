// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Compact ordered set of u32 values backed by a sorted Vec.

/// A compact, sorted set of `u32` values.
#[allow(dead_code)]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CompactSet {
    values: Vec<u32>,
}

#[allow(dead_code)]
impl CompactSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, value: u32) -> bool {
        match self.values.binary_search(&value) {
            Ok(_) => false,
            Err(pos) => {
                self.values.insert(pos, value);
                true
            }
        }
    }

    pub fn remove(&mut self, value: u32) -> bool {
        match self.values.binary_search(&value) {
            Ok(pos) => {
                self.values.remove(pos);
                true
            }
            Err(_) => false,
        }
    }

    pub fn contains(&self, value: u32) -> bool {
        self.values.binary_search(&value).is_ok()
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, u32> {
        self.values.iter()
    }

    pub fn min(&self) -> Option<u32> {
        self.values.first().copied()
    }

    pub fn max(&self) -> Option<u32> {
        self.values.last().copied()
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }

    pub fn union(&self, other: &CompactSet) -> CompactSet {
        let mut result = self.clone();
        for &v in &other.values {
            result.insert(v);
        }
        result
    }

    pub fn intersection(&self, other: &CompactSet) -> CompactSet {
        let mut result = CompactSet::new();
        for &v in &self.values {
            if other.contains(v) {
                result.insert(v);
            }
        }
        result
    }

    pub fn difference(&self, other: &CompactSet) -> CompactSet {
        let mut result = CompactSet::new();
        for &v in &self.values {
            if !other.contains(v) {
                result.insert(v);
            }
        }
        result
    }

    pub fn to_vec(&self) -> Vec<u32> {
        self.values.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        assert!(CompactSet::new().is_empty());
    }

    #[test]
    fn insert_and_contains() {
        let mut s = CompactSet::new();
        assert!(s.insert(5));
        assert!(s.contains(5));
        assert!(!s.contains(6));
    }

    #[test]
    fn duplicate_insert_returns_false() {
        let mut s = CompactSet::new();
        assert!(s.insert(3));
        assert!(!s.insert(3));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn remove_works() {
        let mut s = CompactSet::new();
        s.insert(7);
        assert!(s.remove(7));
        assert!(s.is_empty());
        assert!(!s.remove(7));
    }

    #[test]
    fn sorted_ordering() {
        let mut s = CompactSet::new();
        s.insert(10);
        s.insert(2);
        s.insert(6);
        let v = s.to_vec();
        assert_eq!(v, vec![2, 6, 10]);
    }

    #[test]
    fn min_max() {
        let mut s = CompactSet::new();
        s.insert(4);
        s.insert(1);
        s.insert(9);
        assert_eq!(s.min(), Some(1));
        assert_eq!(s.max(), Some(9));
    }

    #[test]
    fn union_op() {
        let mut a = CompactSet::new();
        a.insert(1);
        a.insert(2);
        let mut b = CompactSet::new();
        b.insert(2);
        b.insert(3);
        let u = a.union(&b);
        assert_eq!(u.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn intersection_op() {
        let mut a = CompactSet::new();
        a.insert(1);
        a.insert(2);
        a.insert(3);
        let mut b = CompactSet::new();
        b.insert(2);
        b.insert(4);
        let i = a.intersection(&b);
        assert_eq!(i.to_vec(), vec![2]);
    }

    #[test]
    fn difference_op() {
        let mut a = CompactSet::new();
        a.insert(1);
        a.insert(2);
        a.insert(3);
        let mut b = CompactSet::new();
        b.insert(2);
        let d = a.difference(&b);
        assert_eq!(d.to_vec(), vec![1, 3]);
    }

    #[test]
    fn clear_empties() {
        let mut s = CompactSet::new();
        s.insert(1);
        s.insert(2);
        s.clear();
        assert!(s.is_empty());
    }
}
