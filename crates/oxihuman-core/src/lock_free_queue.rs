#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Simple lock-free-style queue (single-threaded stub).

use std::collections::VecDeque;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LockFreeQueue<T> {
    data: VecDeque<T>,
}

#[allow(dead_code)]
pub fn new_lock_free_queue<T>() -> LockFreeQueue<T> {
    LockFreeQueue {
        data: VecDeque::new(),
    }
}

#[allow(dead_code)]
pub fn lfq_push<T>(q: &mut LockFreeQueue<T>, val: T) {
    q.data.push_back(val);
}

#[allow(dead_code)]
pub fn lfq_pop<T>(q: &mut LockFreeQueue<T>) -> Option<T> {
    q.data.pop_front()
}

#[allow(dead_code)]
pub fn lfq_peek<T>(q: &LockFreeQueue<T>) -> Option<&T> {
    q.data.front()
}

#[allow(dead_code)]
pub fn lfq_len<T>(q: &LockFreeQueue<T>) -> usize {
    q.data.len()
}

#[allow(dead_code)]
pub fn lfq_is_empty<T>(q: &LockFreeQueue<T>) -> bool {
    q.data.is_empty()
}

#[allow(dead_code)]
pub fn lfq_clear<T>(q: &mut LockFreeQueue<T>) {
    q.data.clear();
}

#[allow(dead_code)]
pub fn lfq_to_vec<T: Clone>(q: &LockFreeQueue<T>) -> Vec<T> {
    q.data.iter().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_queue() {
        let q = new_lock_free_queue::<i32>();
        assert!(lfq_is_empty(&q));
    }

    #[test]
    fn test_push_pop() {
        let mut q = new_lock_free_queue::<i32>();
        lfq_push(&mut q, 42);
        assert_eq!(lfq_pop(&mut q), Some(42));
    }

    #[test]
    fn test_fifo() {
        let mut q = new_lock_free_queue::<i32>();
        lfq_push(&mut q, 1);
        lfq_push(&mut q, 2);
        assert_eq!(lfq_pop(&mut q), Some(1));
        assert_eq!(lfq_pop(&mut q), Some(2));
    }

    #[test]
    fn test_peek() {
        let mut q = new_lock_free_queue::<i32>();
        lfq_push(&mut q, 10);
        assert_eq!(lfq_peek(&q), Some(&10));
        assert_eq!(lfq_len(&q), 1);
    }

    #[test]
    fn test_pop_empty() {
        let mut q = new_lock_free_queue::<i32>();
        assert_eq!(lfq_pop(&mut q), None);
    }

    #[test]
    fn test_len() {
        let mut q = new_lock_free_queue::<i32>();
        lfq_push(&mut q, 1);
        lfq_push(&mut q, 2);
        assert_eq!(lfq_len(&q), 2);
    }

    #[test]
    fn test_clear() {
        let mut q = new_lock_free_queue::<i32>();
        lfq_push(&mut q, 1);
        lfq_clear(&mut q);
        assert!(lfq_is_empty(&q));
    }

    #[test]
    fn test_to_vec() {
        let mut q = new_lock_free_queue::<i32>();
        lfq_push(&mut q, 1);
        lfq_push(&mut q, 2);
        assert_eq!(lfq_to_vec(&q), vec![1, 2]);
    }

    #[test]
    fn test_is_empty() {
        let q = new_lock_free_queue::<i32>();
        assert!(lfq_is_empty(&q));
    }

    #[test]
    fn test_peek_empty() {
        let q = new_lock_free_queue::<i32>();
        assert_eq!(lfq_peek(&q), None);
    }
}
