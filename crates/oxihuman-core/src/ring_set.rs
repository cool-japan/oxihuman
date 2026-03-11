// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A fixed-capacity ring-buffer set that tracks the most recent N unique items.

use std::collections::HashSet;
use std::hash::Hash;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RingSet<T: Eq + Hash + Clone> {
    buf: Vec<Option<T>>,
    head: usize,
    len: usize,
    set: HashSet<T>,
}

#[allow(dead_code)]
impl<T: Eq + Hash + Clone> RingSet<T> {
    pub fn new(capacity: usize) -> Self {
        let cap = capacity.max(1);
        let mut buf = Vec::with_capacity(cap);
        buf.resize_with(cap, || None);
        Self { buf, head: 0, len: 0, set: HashSet::new() }
    }

    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn contains(&self, item: &T) -> bool {
        self.set.contains(item)
    }

    pub fn insert(&mut self, item: T) -> bool {
        if self.set.contains(&item) {
            return false;
        }
        let cap = self.buf.len();
        let idx = (self.head + self.len) % cap;

        if self.len == cap {
            // Evict oldest
            if let Some(old) = self.buf[self.head].take() {
                self.set.remove(&old);
            }
            self.head = (self.head + 1) % cap;
        } else {
            self.len += 1;
        }

        self.set.insert(item.clone());
        self.buf[idx] = Some(item);
        true
    }

    pub fn oldest(&self) -> Option<&T> {
        if self.len == 0 { None } else { self.buf[self.head].as_ref() }
    }

    pub fn newest(&self) -> Option<&T> {
        if self.len == 0 {
            None
        } else {
            let idx = (self.head + self.len - 1) % self.buf.len();
            self.buf[idx].as_ref()
        }
    }

    pub fn clear(&mut self) {
        for slot in &mut self.buf {
            *slot = None;
        }
        self.set.clear();
        self.head = 0;
        self.len = 0;
    }

    pub fn to_vec(&self) -> Vec<T> {
        (0..self.len)
            .filter_map(|i| {
                let idx = (self.head + i) % self.buf.len();
                self.buf[idx].clone()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_contains() {
        let mut s = RingSet::new(4);
        assert!(s.insert(1));
        assert!(s.contains(&1));
    }

    #[test]
    fn test_no_duplicates() {
        let mut s = RingSet::new(4);
        assert!(s.insert(1));
        assert!(!s.insert(1));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn test_eviction() {
        let mut s = RingSet::new(2);
        s.insert(1);
        s.insert(2);
        s.insert(3); // evicts 1
        assert!(!s.contains(&1));
        assert!(s.contains(&2));
        assert!(s.contains(&3));
    }

    #[test]
    fn test_oldest_newest() {
        let mut s = RingSet::new(3);
        s.insert(10);
        s.insert(20);
        assert_eq!(s.oldest(), Some(&10));
        assert_eq!(s.newest(), Some(&20));
    }

    #[test]
    fn test_clear() {
        let mut s = RingSet::new(4);
        s.insert(1);
        s.clear();
        assert!(s.is_empty());
        assert!(!s.contains(&1));
    }

    #[test]
    fn test_to_vec() {
        let mut s = RingSet::new(3);
        s.insert(1);
        s.insert(2);
        assert_eq!(s.to_vec(), vec![1, 2]);
    }

    #[test]
    fn test_capacity() {
        let s: RingSet<i32> = RingSet::new(10);
        assert_eq!(s.capacity(), 10);
    }

    #[test]
    fn test_empty_oldest_newest() {
        let s: RingSet<i32> = RingSet::new(4);
        assert!(s.oldest().is_none());
        assert!(s.newest().is_none());
    }

    #[test]
    fn test_full_wrap() {
        let mut s = RingSet::new(3);
        for i in 0..6 {
            s.insert(i);
        }
        assert_eq!(s.len(), 3);
        assert!(s.contains(&3));
        assert!(s.contains(&4));
        assert!(s.contains(&5));
    }

    #[test]
    fn test_string_items() {
        let mut s = RingSet::new(2);
        s.insert("hello".to_string());
        s.insert("world".to_string());
        assert!(s.contains(&"hello".to_string()));
    }
}
