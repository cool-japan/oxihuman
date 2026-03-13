// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! A work-stealing deque for task-parallel scheduling.
//! Each worker has a local deque; idle workers steal from others.

use std::collections::VecDeque;

/// A single worker's deque that supports push/pop from one end and steal from the other.
#[allow(dead_code)]
#[derive(Debug)]
pub struct WorkDeque<T> {
    items: VecDeque<T>,
    steal_count: u64,
    push_count: u64,
}

#[allow(dead_code)]
impl<T> WorkDeque<T> {
    pub fn new() -> Self {
        Self {
            items: VecDeque::new(),
            steal_count: 0,
            push_count: 0,
        }
    }

    pub fn push(&mut self, item: T) {
        self.items.push_back(item);
        self.push_count += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        self.items.pop_back()
    }

    pub fn steal(&mut self) -> Option<T> {
        let item = self.items.pop_front();
        if item.is_some() {
            self.steal_count += 1;
        }
        item
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn steal_count(&self) -> u64 {
        self.steal_count
    }

    pub fn push_count(&self) -> u64 {
        self.push_count
    }
}

impl<T> Default for WorkDeque<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// A pool of work-stealing deques.
#[allow(dead_code)]
#[derive(Debug)]
pub struct WorkStealingPool<T> {
    deques: Vec<WorkDeque<T>>,
}

#[allow(dead_code)]
impl<T> WorkStealingPool<T> {
    pub fn new(worker_count: usize) -> Self {
        let mut deques = Vec::with_capacity(worker_count);
        for _ in 0..worker_count {
            deques.push(WorkDeque::new());
        }
        Self { deques }
    }

    pub fn worker_count(&self) -> usize {
        self.deques.len()
    }

    pub fn push(&mut self, worker_id: usize, item: T) {
        if worker_id < self.deques.len() {
            self.deques[worker_id].push(item);
        }
    }

    pub fn pop(&mut self, worker_id: usize) -> Option<T> {
        if worker_id < self.deques.len() {
            self.deques[worker_id].pop()
        } else {
            None
        }
    }

    /// Steal from the fullest other worker.
    pub fn steal_for(&mut self, worker_id: usize) -> Option<T> {
        let victim = self.find_victim(worker_id)?;
        self.deques[victim].steal()
    }

    fn find_victim(&self, exclude: usize) -> Option<usize> {
        let mut best = None;
        let mut best_len = 0;
        for (i, d) in self.deques.iter().enumerate() {
            if i != exclude && d.len() > best_len {
                best = Some(i);
                best_len = d.len();
            }
        }
        best
    }

    pub fn total_items(&self) -> usize {
        self.deques.iter().map(|d| d.len()).sum()
    }

    pub fn total_steals(&self) -> u64 {
        self.deques.iter().map(|d| d.steal_count()).sum()
    }

    pub fn worker_load(&self, worker_id: usize) -> usize {
        self.deques.get(worker_id).map(|d| d.len()).unwrap_or(0)
    }

    /// Balance: move half of the fullest deque to the emptiest.
    pub fn rebalance(&mut self) {
        if self.deques.len() < 2 {
            return;
        }
        let (fullest, emptiest) = {
            let mut fi = 0;
            let mut ei = 0;
            for i in 1..self.deques.len() {
                if self.deques[i].len() > self.deques[fi].len() {
                    fi = i;
                }
                if self.deques[i].len() < self.deques[ei].len() {
                    ei = i;
                }
            }
            (fi, ei)
        };
        if fullest == emptiest {
            return;
        }
        let move_count = self.deques[fullest].len() / 2;
        for _ in 0..move_count {
            if let Some(item) = self.deques[fullest].steal() {
                self.deques[emptiest].push(item);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deque_push_pop() {
        let mut d = WorkDeque::new();
        d.push(1);
        d.push(2);
        assert_eq!(d.pop(), Some(2));
        assert_eq!(d.pop(), Some(1));
    }

    #[test]
    fn test_deque_steal() {
        let mut d = WorkDeque::new();
        d.push(10);
        d.push(20);
        assert_eq!(d.steal(), Some(10)); // steals from front
        assert_eq!(d.steal_count(), 1);
    }

    #[test]
    fn test_pool_basic() {
        let mut pool = WorkStealingPool::new(3);
        pool.push(0, 1);
        pool.push(0, 2);
        pool.push(1, 3);
        assert_eq!(pool.total_items(), 3);
    }

    #[test]
    fn test_pool_steal() {
        let mut pool = WorkStealingPool::new(2);
        pool.push(0, 10);
        pool.push(0, 20);
        let stolen = pool.steal_for(1);
        assert_eq!(stolen, Some(10));
    }

    #[test]
    fn test_pool_steal_empty() {
        let mut pool: WorkStealingPool<i32> = WorkStealingPool::new(2);
        assert_eq!(pool.steal_for(0), None);
    }

    #[test]
    fn test_rebalance() {
        let mut pool = WorkStealingPool::new(2);
        for i in 0..10 {
            pool.push(0, i);
        }
        assert_eq!(pool.worker_load(0), 10);
        assert_eq!(pool.worker_load(1), 0);
        pool.rebalance();
        assert!(pool.worker_load(0) <= 6);
        assert!(pool.worker_load(1) >= 4);
    }

    #[test]
    fn test_worker_count() {
        let pool: WorkStealingPool<i32> = WorkStealingPool::new(4);
        assert_eq!(pool.worker_count(), 4);
    }

    #[test]
    fn test_total_steals() {
        let mut pool = WorkStealingPool::new(2);
        pool.push(0, 1);
        pool.push(0, 2);
        pool.steal_for(1);
        pool.steal_for(1);
        assert_eq!(pool.total_steals(), 2);
    }

    #[test]
    fn test_pop_empty() {
        let mut pool: WorkStealingPool<i32> = WorkStealingPool::new(1);
        assert_eq!(pool.pop(0), None);
    }

    #[test]
    fn test_push_invalid_worker() {
        let mut pool: WorkStealingPool<i32> = WorkStealingPool::new(1);
        pool.push(99, 42); // should not crash
        assert_eq!(pool.total_items(), 0);
    }
}
