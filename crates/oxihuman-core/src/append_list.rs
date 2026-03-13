// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// An append-only list that never invalidates indices.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AppendList<T> {
    items: Vec<T>,
}

#[allow(dead_code)]
impl<T> AppendList<T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            items: Vec::with_capacity(cap),
        }
    }

    pub fn push(&mut self, item: T) -> usize {
        let idx = self.items.len();
        self.items.push(item);
        idx
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.items.iter()
    }

    pub fn last(&self) -> Option<&T> {
        self.items.last()
    }

    pub fn as_slice(&self) -> &[T] {
        &self.items
    }

    pub fn contains_index(&self, index: usize) -> bool {
        index < self.items.len()
    }

    pub fn capacity(&self) -> usize {
        self.items.capacity()
    }
}

impl<T> Default for AppendList<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_is_empty() {
        let list: AppendList<i32> = AppendList::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_push_returns_index() {
        let mut list = AppendList::new();
        assert_eq!(list.push(10), 0);
        assert_eq!(list.push(20), 1);
        assert_eq!(list.push(30), 2);
    }

    #[test]
    fn test_get_valid() {
        let mut list = AppendList::new();
        list.push(42);
        assert_eq!(list.get(0), Some(&42));
    }

    #[test]
    fn test_get_out_of_bounds() {
        let list: AppendList<i32> = AppendList::new();
        assert!(list.get(0).is_none());
    }

    #[test]
    fn test_last() {
        let mut list = AppendList::new();
        list.push(1);
        list.push(2);
        assert_eq!(list.last(), Some(&2));
    }

    #[test]
    fn test_iter() {
        let mut list = AppendList::new();
        list.push(1);
        list.push(2);
        list.push(3);
        let v: Vec<_> = list.iter().copied().collect();
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn test_with_capacity() {
        let list: AppendList<i32> = AppendList::with_capacity(16);
        assert!(list.capacity() >= 16);
        assert!(list.is_empty());
    }

    #[test]
    fn test_contains_index() {
        let mut list = AppendList::new();
        list.push(100);
        assert!(list.contains_index(0));
        assert!(!list.contains_index(1));
    }

    #[test]
    fn test_as_slice() {
        let mut list = AppendList::new();
        list.push(5);
        list.push(6);
        assert_eq!(list.as_slice(), &[5, 6]);
    }

    #[test]
    fn test_default() {
        let list: AppendList<f32> = AppendList::default();
        assert!(list.is_empty());
    }
}
