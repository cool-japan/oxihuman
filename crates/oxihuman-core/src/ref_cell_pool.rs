#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Object pool using interior mutability.

/// A simple pool of reusable objects.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RefCellPool<T: Clone> {
    available: Vec<T>,
    total: usize,
    max_size: usize,
}

#[allow(dead_code)]
pub fn new_ref_cell_pool<T: Clone>(max_size: usize) -> RefCellPool<T> {
    RefCellPool {
        available: Vec::with_capacity(max_size),
        total: 0,
        max_size,
    }
}

#[allow(dead_code)]
pub fn pool_borrow<T: Clone>(pool: &mut RefCellPool<T>) -> Option<T> {
    pool.available.pop()
}

#[allow(dead_code)]
pub fn pool_return<T: Clone>(pool: &mut RefCellPool<T>, item: T) -> bool {
    if pool.available.len() < pool.max_size {
        pool.available.push(item);
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn pool_size<T: Clone>(pool: &RefCellPool<T>) -> usize {
    pool.available.len()
}

#[allow(dead_code)]
pub fn pool_available<T: Clone>(pool: &RefCellPool<T>) -> usize {
    pool.available.len()
}

#[allow(dead_code)]
pub fn pool_total<T: Clone>(pool: &RefCellPool<T>) -> usize {
    pool.total
}

#[allow(dead_code)]
pub fn pool_clear<T: Clone>(pool: &mut RefCellPool<T>) {
    pool.available.clear();
    pool.total = 0;
}

#[allow(dead_code)]
pub fn pool_is_full<T: Clone>(pool: &RefCellPool<T>) -> bool {
    pool.available.len() >= pool.max_size
}

impl<T: Clone> RefCellPool<T> {
    /// Seed the pool with initial objects.
    #[allow(dead_code)]
    pub fn seed(&mut self, items: Vec<T>) {
        for item in items {
            if self.available.len() < self.max_size {
                self.total += 1;
                self.available.push(item);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pool() {
        let pool: RefCellPool<i32> = new_ref_cell_pool(5);
        assert_eq!(pool_size(&pool), 0);
    }

    #[test]
    fn test_return_and_borrow() {
        let mut pool = new_ref_cell_pool(5);
        pool_return(&mut pool, 42);
        assert_eq!(pool_borrow(&mut pool), Some(42));
    }

    #[test]
    fn test_borrow_empty() {
        let mut pool: RefCellPool<i32> = new_ref_cell_pool(5);
        assert_eq!(pool_borrow(&mut pool), None);
    }

    #[test]
    fn test_pool_full() {
        let mut pool = new_ref_cell_pool(1);
        assert!(pool_return(&mut pool, 1));
        assert!(!pool_return(&mut pool, 2));
    }

    #[test]
    fn test_is_full() {
        let mut pool = new_ref_cell_pool(1);
        assert!(!pool_is_full(&pool));
        pool_return(&mut pool, 1);
        assert!(pool_is_full(&pool));
    }

    #[test]
    fn test_available() {
        let mut pool = new_ref_cell_pool(10);
        pool_return(&mut pool, 1);
        pool_return(&mut pool, 2);
        assert_eq!(pool_available(&pool), 2);
    }

    #[test]
    fn test_clear() {
        let mut pool = new_ref_cell_pool(10);
        pool_return(&mut pool, 1);
        pool_clear(&mut pool);
        assert_eq!(pool_size(&pool), 0);
    }

    #[test]
    fn test_seed() {
        let mut pool: RefCellPool<i32> = new_ref_cell_pool(10);
        pool.seed(vec![1, 2, 3]);
        assert_eq!(pool_size(&pool), 3);
        assert_eq!(pool_total(&pool), 3);
    }

    #[test]
    fn test_lifo_order() {
        let mut pool = new_ref_cell_pool(10);
        pool_return(&mut pool, 1);
        pool_return(&mut pool, 2);
        assert_eq!(pool_borrow(&mut pool), Some(2)); // LIFO
    }

    #[test]
    fn test_pool_total_after_clear() {
        let mut pool: RefCellPool<i32> = new_ref_cell_pool(10);
        pool.seed(vec![1, 2]);
        pool_clear(&mut pool);
        assert_eq!(pool_total(&pool), 0);
    }
}
