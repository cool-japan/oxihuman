// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A simple per-logical-context object pool (not truly thread-local, but models the API).

use std::collections::VecDeque;

/// A reusable-object pool for a single logical context.
#[allow(dead_code)]
pub struct ThreadLocalPool<T> {
    free: VecDeque<T>,
    capacity: usize,
    alloc_count: u64,
    recycle_count: u64,
    overflow_count: u64,
}

#[allow(dead_code)]
impl<T> ThreadLocalPool<T> {
    /// Create a pool with maximum `capacity` free objects.
    pub fn new(capacity: usize) -> Self {
        Self {
            free: VecDeque::with_capacity(capacity),
            capacity,
            alloc_count: 0,
            recycle_count: 0,
            overflow_count: 0,
        }
    }

    /// Pre-populate the pool with objects produced by `factory`.
    pub fn warm_up(&mut self, count: usize, factory: impl Fn() -> T) {
        for _ in 0..count.min(self.capacity) {
            self.free.push_back(factory());
        }
    }

    /// Obtain an object from the pool, or allocate a new one via `factory`.
    pub fn acquire(&mut self, factory: impl FnOnce() -> T) -> T {
        self.alloc_count += 1;
        self.free.pop_front().unwrap_or_else(factory)
    }

    /// Return an object to the pool. If the pool is at capacity, the object is dropped.
    pub fn release(&mut self, obj: T) {
        if self.free.len() < self.capacity {
            self.free.push_back(obj);
            self.recycle_count += 1;
        } else {
            self.overflow_count += 1;
        }
    }

    /// Number of objects currently in the free pool.
    pub fn free_count(&self) -> usize {
        self.free.len()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn alloc_count(&self) -> u64 {
        self.alloc_count
    }

    pub fn recycle_count(&self) -> u64 {
        self.recycle_count
    }

    pub fn overflow_count(&self) -> u64 {
        self.overflow_count
    }

    pub fn is_empty(&self) -> bool {
        self.free.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.free.len() >= self.capacity
    }

    /// Drain all free objects (e.g. at shutdown).
    pub fn drain(&mut self) -> Vec<T> {
        self.free.drain(..).collect()
    }
}

pub fn new_thread_local_pool<T>(capacity: usize) -> ThreadLocalPool<T> {
    ThreadLocalPool::new(capacity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pool_empty() {
        let p: ThreadLocalPool<i32> = new_thread_local_pool(4);
        assert!(p.is_empty());
        assert_eq!(p.free_count(), 0);
    }

    #[test]
    fn acquire_creates_when_empty() {
        let mut p: ThreadLocalPool<i32> = new_thread_local_pool(4);
        let v = p.acquire(|| 42);
        assert_eq!(v, 42);
        assert_eq!(p.alloc_count(), 1);
    }

    #[test]
    fn release_and_reuse() {
        let mut p: ThreadLocalPool<i32> = new_thread_local_pool(4);
        p.release(99);
        let v = p.acquire(|| 0);
        assert_eq!(v, 99);
        assert_eq!(p.recycle_count(), 1);
    }

    #[test]
    fn overflow_when_full() {
        let mut p: ThreadLocalPool<i32> = new_thread_local_pool(1);
        p.release(1);
        p.release(2); // overflow
        assert_eq!(p.overflow_count(), 1);
        assert_eq!(p.free_count(), 1);
    }

    #[test]
    fn warm_up_fills_pool() {
        let mut p: ThreadLocalPool<Vec<u8>> = new_thread_local_pool(4);
        p.warm_up(3, Vec::new);
        assert_eq!(p.free_count(), 3);
    }

    #[test]
    fn capacity_respected() {
        let p: ThreadLocalPool<i32> = new_thread_local_pool(8);
        assert_eq!(p.capacity(), 8);
    }

    #[test]
    fn is_full_detection() {
        let mut p: ThreadLocalPool<i32> = new_thread_local_pool(2);
        p.release(1);
        p.release(2);
        assert!(p.is_full());
    }

    #[test]
    fn drain_empties_pool() {
        let mut p: ThreadLocalPool<i32> = new_thread_local_pool(4);
        p.release(1);
        p.release(2);
        let drained = p.drain();
        assert_eq!(drained.len(), 2);
        assert!(p.is_empty());
    }

    #[test]
    fn warm_up_capped_at_capacity() {
        let mut p: ThreadLocalPool<i32> = new_thread_local_pool(2);
        p.warm_up(10, || 0);
        assert_eq!(p.free_count(), 2);
    }

    #[test]
    fn alloc_count_increments() {
        let mut p: ThreadLocalPool<i32> = new_thread_local_pool(4);
        p.acquire(|| 1);
        p.acquire(|| 2);
        assert_eq!(p.alloc_count(), 2);
    }
}
