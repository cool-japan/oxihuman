// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Fixed-capacity circular buffer (ring buffer) for typed elements.

/// A circular buffer with a fixed capacity.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CircularBuffer<T> {
    data: Vec<Option<T>>,
    head: usize,
    len: usize,
    capacity: usize,
}

#[allow(dead_code)]
impl<T: Clone> CircularBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        let cap = capacity.max(1);
        Self {
            data: vec![None; cap],
            head: 0,
            len: 0,
            capacity: cap,
        }
    }

    pub fn push(&mut self, val: T) {
        let idx = (self.head + self.len) % self.capacity;
        self.data[idx] = Some(val);
        if self.len == self.capacity {
            self.head = (self.head + 1) % self.capacity;
        } else {
            self.len += 1;
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        let val = self.data[self.head].take();
        self.head = (self.head + 1) % self.capacity;
        self.len -= 1;
        val
    }

    pub fn peek_front(&self) -> Option<&T> {
        if self.len == 0 {
            return None;
        }
        self.data[self.head].as_ref()
    }

    pub fn peek_back(&self) -> Option<&T> {
        if self.len == 0 {
            return None;
        }
        let idx = (self.head + self.len - 1) % self.capacity;
        self.data[idx].as_ref()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }
        let idx = (self.head + index) % self.capacity;
        self.data[idx].as_ref()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_full(&self) -> bool {
        self.len == self.capacity
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn clear(&mut self) {
        for slot in &mut self.data {
            *slot = None;
        }
        self.head = 0;
        self.len = 0;
    }

    pub fn to_vec(&self) -> Vec<T> {
        let mut out = Vec::with_capacity(self.len);
        for i in 0..self.len {
            let idx = (self.head + i) % self.capacity;
            if let Some(v) = &self.data[idx] {
                out.push(v.clone());
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_buffer_is_empty() {
        let b = CircularBuffer::<i32>::new(4);
        assert!(b.is_empty());
        assert_eq!(b.len(), 0);
    }

    #[test]
    fn push_and_pop() {
        let mut b = CircularBuffer::new(4);
        b.push(1);
        b.push(2);
        assert_eq!(b.pop_front(), Some(1));
        assert_eq!(b.pop_front(), Some(2));
    }

    #[test]
    fn overwrites_when_full() {
        let mut b = CircularBuffer::new(2);
        b.push(1);
        b.push(2);
        b.push(3); // overwrites 1
        assert_eq!(b.pop_front(), Some(2));
        assert_eq!(b.pop_front(), Some(3));
    }

    #[test]
    fn is_full() {
        let mut b = CircularBuffer::new(2);
        b.push(10);
        b.push(20);
        assert!(b.is_full());
    }

    #[test]
    fn peek_front_and_back() {
        let mut b = CircularBuffer::new(4);
        b.push(1);
        b.push(2);
        b.push(3);
        assert_eq!(b.peek_front(), Some(&1));
        assert_eq!(b.peek_back(), Some(&3));
    }

    #[test]
    fn get_by_index() {
        let mut b = CircularBuffer::new(4);
        b.push(10);
        b.push(20);
        b.push(30);
        assert_eq!(b.get(1), Some(&20));
        assert_eq!(b.get(5), None);
    }

    #[test]
    fn clear_empties() {
        let mut b = CircularBuffer::new(4);
        b.push(1);
        b.clear();
        assert!(b.is_empty());
    }

    #[test]
    fn to_vec_returns_ordered() {
        let mut b = CircularBuffer::new(3);
        b.push(1);
        b.push(2);
        b.push(3);
        b.push(4); // overwrites 1
        assert_eq!(b.to_vec(), vec![2, 3, 4]);
    }

    #[test]
    fn capacity_returns_max() {
        let b = CircularBuffer::<i32>::new(8);
        assert_eq!(b.capacity(), 8);
    }

    #[test]
    fn pop_empty_returns_none() {
        let mut b = CircularBuffer::<i32>::new(4);
        assert_eq!(b.pop_front(), None);
    }
}
