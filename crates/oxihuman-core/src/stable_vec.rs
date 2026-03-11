// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A stable vector where indices remain valid after removal (slots become holes).

/// A vector with stable indices - removed elements leave holes that are reused.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StableVec<T> {
    slots: Vec<Option<T>>,
    free_list: Vec<usize>,
    count: usize,
}

#[allow(dead_code)]
impl<T> StableVec<T> {
    pub fn new() -> Self {
        Self { slots: Vec::new(), free_list: Vec::new(), count: 0 }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self { slots: Vec::with_capacity(cap), free_list: Vec::new(), count: 0 }
    }

    pub fn insert(&mut self, value: T) -> usize {
        self.count += 1;
        if let Some(idx) = self.free_list.pop() {
            self.slots[idx] = Some(value);
            idx
        } else {
            let idx = self.slots.len();
            self.slots.push(Some(value));
            idx
        }
    }

    pub fn remove(&mut self, idx: usize) -> Option<T> {
        if idx < self.slots.len() {
            if let Some(val) = self.slots[idx].take() {
                self.free_list.push(idx);
                self.count -= 1;
                return Some(val);
            }
        }
        None
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        self.slots.get(idx).and_then(|s| s.as_ref())
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        self.slots.get_mut(idx).and_then(|s| s.as_mut())
    }

    pub fn contains(&self, idx: usize) -> bool {
        self.slots.get(idx).map(|s| s.is_some()).unwrap_or(false)
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn capacity(&self) -> usize {
        self.slots.len()
    }

    pub fn hole_count(&self) -> usize {
        self.free_list.len()
    }

    /// Iterate over (index, &value) for occupied slots.
    pub fn iter(&self) -> impl Iterator<Item = (usize, &T)> {
        self.slots.iter().enumerate().filter_map(|(i, s)| s.as_ref().map(|v| (i, v)))
    }

    pub fn indices(&self) -> Vec<usize> {
        self.slots.iter().enumerate()
            .filter_map(|(i, s)| if s.is_some() { Some(i) } else { None })
            .collect()
    }

    pub fn clear(&mut self) {
        self.slots.clear();
        self.free_list.clear();
        self.count = 0;
    }
}

impl<T> Default for StableVec<T> {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_get() {
        let mut sv = StableVec::new();
        let i = sv.insert(42);
        assert_eq!(sv.get(i), Some(&42));
    }

    #[test]
    fn test_remove() {
        let mut sv = StableVec::new();
        let i = sv.insert(10);
        assert_eq!(sv.remove(i), Some(10));
        assert_eq!(sv.get(i), None);
    }

    #[test]
    fn test_stable_indices() {
        let mut sv = StableVec::new();
        let a = sv.insert("a");
        let b = sv.insert("b");
        let c = sv.insert("c");
        sv.remove(b);
        assert_eq!(sv.get(a), Some(&"a"));
        assert_eq!(sv.get(c), Some(&"c"));
        assert_eq!(sv.get(b), None);
    }

    #[test]
    fn test_reuse() {
        let mut sv = StableVec::new();
        let a = sv.insert(1);
        sv.remove(a);
        let b = sv.insert(2);
        assert_eq!(a, b); // reused slot
        assert_eq!(sv.get(b), Some(&2));
    }

    #[test]
    fn test_len() {
        let mut sv = StableVec::new();
        sv.insert(1);
        sv.insert(2);
        sv.insert(3);
        assert_eq!(sv.len(), 3);
        sv.remove(1);
        assert_eq!(sv.len(), 2);
    }

    #[test]
    fn test_hole_count() {
        let mut sv = StableVec::new();
        sv.insert(1);
        sv.insert(2);
        sv.remove(0);
        assert_eq!(sv.hole_count(), 1);
    }

    #[test]
    fn test_iter() {
        let mut sv = StableVec::new();
        sv.insert(10);
        sv.insert(20);
        sv.insert(30);
        sv.remove(1);
        let vals: Vec<_> = sv.iter().map(|(_, v)| *v).collect();
        assert_eq!(vals, vec![10, 30]);
    }

    #[test]
    fn test_clear() {
        let mut sv = StableVec::new();
        sv.insert(1);
        sv.clear();
        assert!(sv.is_empty());
    }

    #[test]
    fn test_get_mut() {
        let mut sv = StableVec::new();
        let i = sv.insert(5);
        *sv.get_mut(i).unwrap() = 10;
        assert_eq!(sv.get(i), Some(&10));
    }

    #[test]
    fn test_indices() {
        let mut sv = StableVec::new();
        sv.insert('a');
        sv.insert('b');
        sv.insert('c');
        sv.remove(1);
        assert_eq!(sv.indices(), vec![0, 2]);
    }
}
