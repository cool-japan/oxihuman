// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::collections::VecDeque;

/// A fixed-capacity FIFO queue that drops the oldest element on overflow.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoundedQueue<T> {
    buf: VecDeque<T>,
    capacity: usize,
}

#[allow(dead_code)]
impl<T> BoundedQueue<T> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "capacity must be > 0");
        Self {
            buf: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, item: T) -> Option<T> {
        let evicted = if self.buf.len() >= self.capacity {
            self.buf.pop_front()
        } else {
            None
        };
        self.buf.push_back(item);
        evicted
    }

    pub fn pop(&mut self) -> Option<T> {
        self.buf.pop_front()
    }

    pub fn peek(&self) -> Option<&T> {
        self.buf.front()
    }

    pub fn peek_back(&self) -> Option<&T> {
        self.buf.back()
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.buf.len() >= self.capacity
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn clear(&mut self) {
        self.buf.clear();
    }

    pub fn iter(&self) -> std::collections::vec_deque::Iter<'_, T> {
        self.buf.iter()
    }

    pub fn remaining(&self) -> usize {
        self.capacity - self.buf.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let q: BoundedQueue<i32> = BoundedQueue::new(4);
        assert!(q.is_empty());
        assert_eq!(q.capacity(), 4);
    }

    #[test]
    fn test_push_pop() {
        let mut q = BoundedQueue::new(4);
        q.push(1);
        q.push(2);
        assert_eq!(q.pop(), Some(1));
        assert_eq!(q.pop(), Some(2));
    }

    #[test]
    fn test_overflow_evicts_oldest() {
        let mut q = BoundedQueue::new(2);
        assert!(q.push(1).is_none());
        assert!(q.push(2).is_none());
        let evicted = q.push(3);
        assert_eq!(evicted, Some(1));
        assert_eq!(q.peek(), Some(&2));
    }

    #[test]
    fn test_is_full() {
        let mut q = BoundedQueue::new(2);
        assert!(!q.is_full());
        q.push(1);
        q.push(2);
        assert!(q.is_full());
    }

    #[test]
    fn test_clear() {
        let mut q = BoundedQueue::new(4);
        q.push(1);
        q.push(2);
        q.clear();
        assert!(q.is_empty());
    }

    #[test]
    fn test_peek_back() {
        let mut q = BoundedQueue::new(4);
        q.push(10);
        q.push(20);
        assert_eq!(q.peek_back(), Some(&20));
    }

    #[test]
    fn test_iter() {
        let mut q = BoundedQueue::new(4);
        q.push(1);
        q.push(2);
        q.push(3);
        let v: Vec<_> = q.iter().copied().collect();
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn test_remaining() {
        let mut q = BoundedQueue::new(3);
        assert_eq!(q.remaining(), 3);
        q.push(1);
        assert_eq!(q.remaining(), 2);
    }

    #[test]
    fn test_pop_empty() {
        let mut q: BoundedQueue<i32> = BoundedQueue::new(2);
        assert!(q.pop().is_none());
    }

    #[test]
    fn test_multiple_overflows() {
        let mut q = BoundedQueue::new(2);
        q.push(1);
        q.push(2);
        q.push(3);
        q.push(4);
        assert_eq!(q.len(), 2);
        assert_eq!(q.pop(), Some(3));
        assert_eq!(q.pop(), Some(4));
    }
}
