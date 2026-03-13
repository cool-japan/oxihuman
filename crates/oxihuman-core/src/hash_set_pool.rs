// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::collections::HashSet;

/// A pool of reusable HashSets to reduce allocation overhead.
#[allow(dead_code)]
#[derive(Debug)]
pub struct HashSetPool {
    pool: Vec<HashSet<u64>>,
    max_pool_size: usize,
}

#[allow(dead_code)]
impl HashSetPool {
    pub fn new(max_pool_size: usize) -> Self {
        Self {
            pool: Vec::new(),
            max_pool_size,
        }
    }

    pub fn acquire(&mut self) -> HashSet<u64> {
        self.pool.pop().unwrap_or_default()
    }

    pub fn release(&mut self, mut set: HashSet<u64>) {
        if self.pool.len() < self.max_pool_size {
            set.clear();
            self.pool.push(set);
        }
    }

    pub fn available(&self) -> usize {
        self.pool.len()
    }

    pub fn max_size(&self) -> usize {
        self.max_pool_size
    }

    pub fn is_empty(&self) -> bool {
        self.pool.is_empty()
    }

    pub fn clear(&mut self) {
        self.pool.clear();
    }

    pub fn prewarm(&mut self, count: usize) {
        let to_add = count.min(self.max_pool_size - self.pool.len().min(self.max_pool_size));
        for _ in 0..to_add {
            self.pool.push(HashSet::new());
        }
    }

    pub fn shrink_to(&mut self, target: usize) {
        while self.pool.len() > target {
            self.pool.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let pool = HashSetPool::new(10);
        assert!(pool.is_empty());
        assert_eq!(pool.max_size(), 10);
    }

    #[test]
    fn test_acquire_empty() {
        let mut pool = HashSetPool::new(5);
        let set = pool.acquire();
        assert!(set.is_empty());
    }

    #[test]
    fn test_release_acquire() {
        let mut pool = HashSetPool::new(5);
        let mut set = pool.acquire();
        set.insert(42);
        pool.release(set);
        assert_eq!(pool.available(), 1);
        let reused = pool.acquire();
        assert!(reused.is_empty()); // was cleared
    }

    #[test]
    fn test_pool_limit() {
        let mut pool = HashSetPool::new(2);
        pool.release(HashSet::new());
        pool.release(HashSet::new());
        pool.release(HashSet::new()); // should be dropped
        assert_eq!(pool.available(), 2);
    }

    #[test]
    fn test_prewarm() {
        let mut pool = HashSetPool::new(10);
        pool.prewarm(5);
        assert_eq!(pool.available(), 5);
    }

    #[test]
    fn test_clear() {
        let mut pool = HashSetPool::new(10);
        pool.prewarm(3);
        pool.clear();
        assert!(pool.is_empty());
    }

    #[test]
    fn test_shrink_to() {
        let mut pool = HashSetPool::new(10);
        pool.prewarm(5);
        pool.shrink_to(2);
        assert_eq!(pool.available(), 2);
    }

    #[test]
    fn test_prewarm_respects_max() {
        let mut pool = HashSetPool::new(3);
        pool.prewarm(10);
        assert_eq!(pool.available(), 3);
    }

    #[test]
    fn test_release_clears_set() {
        let mut pool = HashSetPool::new(5);
        let mut set = HashSet::new();
        set.insert(1);
        set.insert(2);
        pool.release(set);
        let acquired = pool.acquire();
        assert!(acquired.is_empty());
    }

    #[test]
    fn test_multiple_acquire_release() {
        let mut pool = HashSetPool::new(5);
        let s1 = pool.acquire();
        let s2 = pool.acquire();
        pool.release(s1);
        pool.release(s2);
        assert_eq!(pool.available(), 2);
    }
}
