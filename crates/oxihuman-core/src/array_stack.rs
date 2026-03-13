// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Fixed-capacity stack backed by a `Vec` with a maximum size.

/// A stack with a fixed maximum capacity.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArrayStack<T> {
    data: Vec<T>,
    capacity: usize,
}

#[allow(dead_code)]
impl<T: Clone> ArrayStack<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, val: T) -> bool {
        if self.data.len() >= self.capacity {
            return false;
        }
        self.data.push(val);
        true
    }

    pub fn pop(&mut self) -> Option<T> {
        self.data.pop()
    }

    pub fn peek(&self) -> Option<&T> {
        self.data.last()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.data.len() >= self.capacity
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn remaining(&self) -> usize {
        self.capacity - self.data.len()
    }

    pub fn as_slice(&self) -> &[T] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stack_is_empty() {
        let s = ArrayStack::<i32>::new(4);
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn push_and_pop() {
        let mut s = ArrayStack::new(4);
        assert!(s.push(10));
        assert!(s.push(20));
        assert_eq!(s.pop(), Some(20));
        assert_eq!(s.pop(), Some(10));
    }

    #[test]
    fn push_beyond_capacity_fails() {
        let mut s = ArrayStack::new(2);
        assert!(s.push(1));
        assert!(s.push(2));
        assert!(!s.push(3));
    }

    #[test]
    fn peek_returns_top() {
        let mut s = ArrayStack::new(4);
        s.push(5);
        s.push(10);
        assert_eq!(s.peek(), Some(&10));
    }

    #[test]
    fn is_full_at_capacity() {
        let mut s = ArrayStack::new(1);
        assert!(!s.is_full());
        s.push(42);
        assert!(s.is_full());
    }

    #[test]
    fn clear_empties_stack() {
        let mut s = ArrayStack::new(4);
        s.push(1);
        s.push(2);
        s.clear();
        assert!(s.is_empty());
    }

    #[test]
    fn remaining_tracks_space() {
        let mut s = ArrayStack::new(3);
        assert_eq!(s.remaining(), 3);
        s.push(1);
        assert_eq!(s.remaining(), 2);
    }

    #[test]
    fn as_slice_returns_data() {
        let mut s = ArrayStack::new(4);
        s.push(10);
        s.push(20);
        assert_eq!(s.as_slice(), &[10, 20]);
    }

    #[test]
    fn pop_empty_returns_none() {
        let mut s = ArrayStack::<i32>::new(4);
        assert_eq!(s.pop(), None);
    }

    #[test]
    fn capacity_returns_max() {
        let s = ArrayStack::<i32>::new(7);
        assert_eq!(s.capacity(), 7);
    }
}
