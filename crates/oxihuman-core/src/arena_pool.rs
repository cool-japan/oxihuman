// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! A simple arena-based pool allocator that hands out indices into a
//! contiguous backing store. Freed slots are recycled via a free-list.

#[allow(dead_code)]
pub struct ArenaPool<T> {
    storage: Vec<Option<T>>,
    free_list: Vec<usize>,
}

#[allow(dead_code)]
impl<T> ArenaPool<T> {
    pub fn new() -> Self {
        Self {
            storage: Vec::new(),
            free_list: Vec::new(),
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            storage: Vec::with_capacity(cap),
            free_list: Vec::new(),
        }
    }

    pub fn alloc(&mut self, value: T) -> usize {
        if let Some(idx) = self.free_list.pop() {
            self.storage[idx] = Some(value);
            idx
        } else {
            let idx = self.storage.len();
            self.storage.push(Some(value));
            idx
        }
    }

    pub fn free(&mut self, idx: usize) -> Option<T> {
        if idx < self.storage.len() {
            let val = self.storage[idx].take();
            if val.is_some() {
                self.free_list.push(idx);
            }
            val
        } else {
            None
        }
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        self.storage.get(idx).and_then(|s| s.as_ref())
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        self.storage.get_mut(idx).and_then(|s| s.as_mut())
    }

    pub fn active_count(&self) -> usize {
        self.storage.iter().filter(|s| s.is_some()).count()
    }

    pub fn capacity(&self) -> usize {
        self.storage.len()
    }

    pub fn free_count(&self) -> usize {
        self.free_list.len()
    }

    pub fn is_active(&self, idx: usize) -> bool {
        self.storage.get(idx).is_some_and(|s| s.is_some())
    }

    pub fn clear(&mut self) {
        self.storage.clear();
        self.free_list.clear();
    }
}

impl<T> Default for ArenaPool<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_and_get() {
        let mut pool = ArenaPool::new();
        let idx = pool.alloc(42);
        assert_eq!(pool.get(idx), Some(&42));
    }

    #[test]
    fn test_free_and_reuse() {
        let mut pool = ArenaPool::new();
        let idx0 = pool.alloc(10);
        pool.free(idx0);
        let idx1 = pool.alloc(20);
        assert_eq!(idx0, idx1);
        assert_eq!(pool.get(idx1), Some(&20));
    }

    #[test]
    fn test_active_count() {
        let mut pool = ArenaPool::new();
        pool.alloc(1);
        pool.alloc(2);
        assert_eq!(pool.active_count(), 2);
        pool.free(0);
        assert_eq!(pool.active_count(), 1);
    }

    #[test]
    fn test_get_freed_returns_none() {
        let mut pool = ArenaPool::new();
        let idx = pool.alloc(99);
        pool.free(idx);
        assert_eq!(pool.get(idx), None);
    }

    #[test]
    fn test_free_returns_value() {
        let mut pool = ArenaPool::new();
        let idx = pool.alloc("hello".to_string());
        let val = pool.free(idx);
        assert_eq!(val, Some("hello".to_string()));
    }

    #[test]
    fn test_is_active() {
        let mut pool = ArenaPool::new();
        let idx = pool.alloc(5);
        assert!(pool.is_active(idx));
        pool.free(idx);
        assert!(!pool.is_active(idx));
    }

    #[test]
    fn test_clear() {
        let mut pool = ArenaPool::new();
        pool.alloc(1);
        pool.alloc(2);
        pool.clear();
        assert_eq!(pool.active_count(), 0);
        assert_eq!(pool.capacity(), 0);
    }

    #[test]
    fn test_with_capacity() {
        let pool: ArenaPool<i32> = ArenaPool::with_capacity(16);
        assert_eq!(pool.active_count(), 0);
    }

    #[test]
    fn test_get_out_of_bounds() {
        let pool: ArenaPool<i32> = ArenaPool::new();
        assert_eq!(pool.get(100), None);
    }

    #[test]
    fn test_get_mut() {
        let mut pool = ArenaPool::new();
        let idx = pool.alloc(10);
        if let Some(v) = pool.get_mut(idx) {
            *v = 20;
        }
        assert_eq!(pool.get(idx), Some(&20));
    }
}
