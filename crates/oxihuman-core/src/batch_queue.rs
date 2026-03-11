// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Queue that drains items in fixed-size batches.

use std::collections::VecDeque;

/// A queue that yields items in batches of a configured size.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BatchQueue<T> {
    items: VecDeque<T>,
    batch_size: usize,
}

#[allow(dead_code)]
impl<T> BatchQueue<T> {
    pub fn new(batch_size: usize) -> Self {
        let bs = if batch_size == 0 { 1 } else { batch_size };
        Self {
            items: VecDeque::new(),
            batch_size: bs,
        }
    }

    pub fn enqueue(&mut self, item: T) {
        self.items.push_back(item);
    }

    pub fn enqueue_many(&mut self, iter: impl IntoIterator<Item = T>) {
        for item in iter {
            self.items.push_back(item);
        }
    }

    pub fn drain_batch(&mut self) -> Vec<T> {
        let n = self.batch_size.min(self.items.len());
        self.items.drain(..n).collect()
    }

    pub fn has_full_batch(&self) -> bool {
        self.items.len() >= self.batch_size
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn batch_size(&self) -> usize {
        self.batch_size
    }

    pub fn pending_batch_count(&self) -> usize {
        if self.items.is_empty() {
            0
        } else {
            self.items.len().div_ceil(self.batch_size)
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn peek_front(&self) -> Option<&T> {
        self.items.front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_queue_is_empty() {
        let q = BatchQueue::<i32>::new(4);
        assert!(q.is_empty());
    }

    #[test]
    fn enqueue_and_drain() {
        let mut q = BatchQueue::new(2);
        q.enqueue(1);
        q.enqueue(2);
        q.enqueue(3);
        let batch = q.drain_batch();
        assert_eq!(batch, vec![1, 2]);
        assert_eq!(q.len(), 1);
    }

    #[test]
    fn drain_partial_batch() {
        let mut q = BatchQueue::new(5);
        q.enqueue(10);
        let batch = q.drain_batch();
        assert_eq!(batch, vec![10]);
    }

    #[test]
    fn has_full_batch() {
        let mut q = BatchQueue::new(2);
        q.enqueue(1);
        assert!(!q.has_full_batch());
        q.enqueue(2);
        assert!(q.has_full_batch());
    }

    #[test]
    fn pending_batch_count() {
        let mut q = BatchQueue::new(3);
        q.enqueue_many(0..7);
        assert_eq!(q.pending_batch_count(), 3); // ceil(7/3)
    }

    #[test]
    fn clear_empties() {
        let mut q = BatchQueue::new(2);
        q.enqueue(1);
        q.clear();
        assert!(q.is_empty());
    }

    #[test]
    fn batch_size_zero_becomes_one() {
        let q = BatchQueue::<i32>::new(0);
        assert_eq!(q.batch_size(), 1);
    }

    #[test]
    fn peek_front() {
        let mut q = BatchQueue::new(4);
        assert!(q.peek_front().is_none());
        q.enqueue(42);
        assert_eq!(q.peek_front(), Some(&42));
    }

    #[test]
    fn enqueue_many_works() {
        let mut q = BatchQueue::new(3);
        q.enqueue_many(vec![1, 2, 3, 4]);
        assert_eq!(q.len(), 4);
    }

    #[test]
    fn drain_empty_returns_empty() {
        let mut q = BatchQueue::<i32>::new(4);
        assert!(q.drain_batch().is_empty());
    }
}
