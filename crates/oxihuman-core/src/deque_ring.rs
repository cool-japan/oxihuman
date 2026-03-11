// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A fixed-capacity ring-buffer-backed deque. Once full, new pushes
//! overwrite the oldest element.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DequeRing<T> {
    buf: Vec<Option<T>>,
    head: usize,
    len: usize,
}

#[allow(dead_code)]
impl<T> DequeRing<T> {
    pub fn new(capacity: usize) -> Self {
        let cap = capacity.max(1);
        let mut buf = Vec::with_capacity(cap);
        buf.resize_with(cap, || None);
        Self { buf, head: 0, len: 0 }
    }

    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_full(&self) -> bool {
        self.len == self.buf.len()
    }

    pub fn push_back(&mut self, value: T) {
        let cap = self.buf.len();
        let idx = (self.head + self.len) % cap;
        self.buf[idx] = Some(value);
        if self.len == cap {
            self.head = (self.head + 1) % cap;
        } else {
            self.len += 1;
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        let val = self.buf[self.head].take();
        self.head = (self.head + 1) % self.buf.len();
        self.len -= 1;
        val
    }

    pub fn front(&self) -> Option<&T> {
        if self.len == 0 { None } else { self.buf[self.head].as_ref() }
    }

    pub fn back(&self) -> Option<&T> {
        if self.len == 0 {
            None
        } else {
            let idx = (self.head + self.len - 1) % self.buf.len();
            self.buf[idx].as_ref()
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }
        let idx = (self.head + index) % self.buf.len();
        self.buf[idx].as_ref()
    }

    pub fn clear(&mut self) {
        for slot in &mut self.buf {
            *slot = None;
        }
        self.head = 0;
        self.len = 0;
    }

    pub fn to_vec(&self) -> Vec<&T> {
        (0..self.len).filter_map(|i| self.get(i)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_pop() {
        let mut d = DequeRing::new(4);
        d.push_back(1);
        d.push_back(2);
        assert_eq!(d.pop_front(), Some(1));
        assert_eq!(d.pop_front(), Some(2));
    }

    #[test]
    fn test_wrap_around() {
        let mut d = DequeRing::new(3);
        d.push_back(1);
        d.push_back(2);
        d.push_back(3);
        d.push_back(4); // overwrites 1
        assert_eq!(d.front(), Some(&2));
        assert_eq!(d.back(), Some(&4));
    }

    #[test]
    fn test_len_and_capacity() {
        let mut d = DequeRing::new(5);
        assert!(d.is_empty());
        d.push_back(10);
        assert_eq!(d.len(), 1);
        assert_eq!(d.capacity(), 5);
    }

    #[test]
    fn test_is_full() {
        let mut d = DequeRing::new(2);
        d.push_back(1);
        d.push_back(2);
        assert!(d.is_full());
    }

    #[test]
    fn test_get() {
        let mut d = DequeRing::new(4);
        d.push_back(10);
        d.push_back(20);
        d.push_back(30);
        assert_eq!(d.get(0), Some(&10));
        assert_eq!(d.get(2), Some(&30));
        assert_eq!(d.get(3), None);
    }

    #[test]
    fn test_clear() {
        let mut d = DequeRing::new(4);
        d.push_back(1);
        d.push_back(2);
        d.clear();
        assert!(d.is_empty());
    }

    #[test]
    fn test_pop_empty() {
        let mut d: DequeRing<i32> = DequeRing::new(4);
        assert_eq!(d.pop_front(), None);
    }

    #[test]
    fn test_front_back() {
        let mut d = DequeRing::new(4);
        d.push_back(5);
        d.push_back(10);
        assert_eq!(d.front(), Some(&5));
        assert_eq!(d.back(), Some(&10));
    }

    #[test]
    fn test_to_vec() {
        let mut d = DequeRing::new(3);
        d.push_back(1);
        d.push_back(2);
        let v = d.to_vec();
        assert_eq!(v, vec![&1, &2]);
    }

    #[test]
    fn test_overwrite_multiple() {
        let mut d = DequeRing::new(2);
        d.push_back(1);
        d.push_back(2);
        d.push_back(3);
        d.push_back(4);
        assert_eq!(d.front(), Some(&3));
        assert_eq!(d.back(), Some(&4));
    }
}
