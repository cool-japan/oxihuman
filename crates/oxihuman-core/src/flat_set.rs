// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A set backed by a sorted Vec for cache-friendly iteration and binary search.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlatSet<T: Ord + Clone> {
    data: Vec<T>,
}

#[allow(dead_code)]
impl<T: Ord + Clone> FlatSet<T> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            data: Vec::with_capacity(cap),
        }
    }

    pub fn insert(&mut self, value: T) -> bool {
        match self.data.binary_search(&value) {
            Ok(_) => false,
            Err(pos) => {
                self.data.insert(pos, value);
                true
            }
        }
    }

    pub fn remove(&mut self, value: &T) -> bool {
        match self.data.binary_search(value) {
            Ok(pos) => {
                self.data.remove(pos);
                true
            }
            Err(_) => false,
        }
    }

    pub fn contains(&self, value: &T) -> bool {
        self.data.binary_search(value).is_ok()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.data.iter()
    }

    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    pub fn first(&self) -> Option<&T> {
        self.data.first()
    }

    pub fn last(&self) -> Option<&T> {
        self.data.last()
    }

    pub fn union(&self, other: &FlatSet<T>) -> FlatSet<T> {
        let mut result = self.clone();
        for item in &other.data {
            result.insert(item.clone());
        }
        result
    }

    pub fn intersection(&self, other: &FlatSet<T>) -> FlatSet<T> {
        let mut result = FlatSet::new();
        for item in &self.data {
            if other.contains(item) {
                result.data.push(item.clone());
            }
        }
        result
    }
}

impl<T: Ord + Clone> Default for FlatSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_contains() {
        let mut s = FlatSet::new();
        assert!(s.insert(5));
        assert!(s.contains(&5));
    }

    #[test]
    fn test_insert_duplicate() {
        let mut s = FlatSet::new();
        s.insert(5);
        assert!(!s.insert(5));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn test_remove() {
        let mut s = FlatSet::new();
        s.insert(3);
        assert!(s.remove(&3));
        assert!(!s.contains(&3));
    }

    #[test]
    fn test_sorted_order() {
        let mut s = FlatSet::new();
        s.insert(3);
        s.insert(1);
        s.insert(2);
        assert_eq!(s.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn test_clear() {
        let mut s = FlatSet::new();
        s.insert(1);
        s.clear();
        assert!(s.is_empty());
    }

    #[test]
    fn test_first_last() {
        let mut s = FlatSet::new();
        s.insert(10);
        s.insert(1);
        s.insert(5);
        assert_eq!(s.first(), Some(&1));
        assert_eq!(s.last(), Some(&10));
    }

    #[test]
    fn test_union() {
        let mut a = FlatSet::new();
        a.insert(1);
        a.insert(2);
        let mut b = FlatSet::new();
        b.insert(2);
        b.insert(3);
        let u = a.union(&b);
        assert_eq!(u.len(), 3);
    }

    #[test]
    fn test_intersection() {
        let mut a = FlatSet::new();
        a.insert(1);
        a.insert(2);
        let mut b = FlatSet::new();
        b.insert(2);
        b.insert(3);
        let i = a.intersection(&b);
        assert_eq!(i.len(), 1);
        assert!(i.contains(&2));
    }

    #[test]
    fn test_iter() {
        let mut s = FlatSet::new();
        s.insert(2);
        s.insert(1);
        let v: Vec<_> = s.iter().copied().collect();
        assert_eq!(v, vec![1, 2]);
    }

    #[test]
    fn test_empty() {
        let s: FlatSet<i32> = FlatSet::new();
        assert!(s.is_empty());
        assert!(s.first().is_none());
    }
}
