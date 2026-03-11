// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A compact vector that stores small arrays inline before spilling to heap.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CompactVec<T: Clone + Default> {
    inline: [T; 4],
    inline_len: usize,
    overflow: Vec<T>,
}

#[allow(dead_code)]
impl<T: Clone + Default + PartialEq> CompactVec<T> {
    pub fn new() -> Self {
        Self {
            inline: Default::default(),
            inline_len: 0,
            overflow: Vec::new(),
        }
    }

    pub fn push(&mut self, val: T) {
        if self.inline_len < 4 {
            self.inline[self.inline_len] = val;
            self.inline_len += 1;
        } else {
            self.overflow.push(val);
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if let Some(v) = self.overflow.pop() {
            return Some(v);
        }
        if self.inline_len > 0 {
            self.inline_len -= 1;
            let val = self.inline[self.inline_len].clone();
            self.inline[self.inline_len] = T::default();
            Some(val)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.inline_len + self.overflow.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inline_len == 0 && self.overflow.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.inline_len {
            Some(&self.inline[index])
        } else {
            self.overflow.get(index - self.inline_len)
        }
    }

    pub fn is_inline(&self) -> bool {
        self.overflow.is_empty()
    }

    pub fn clear(&mut self) {
        for i in 0..self.inline_len {
            self.inline[i] = T::default();
        }
        self.inline_len = 0;
        self.overflow.clear();
    }

    pub fn contains(&self, val: &T) -> bool {
        for i in 0..self.inline_len {
            if &self.inline[i] == val {
                return true;
            }
        }
        self.overflow.contains(val)
    }

    pub fn to_vec(&self) -> Vec<T> {
        let mut v = Vec::with_capacity(self.len());
        for i in 0..self.inline_len {
            v.push(self.inline[i].clone());
        }
        v.extend_from_slice(&self.overflow);
        v
    }
}

impl<T: Clone + Default + PartialEq> Default for CompactVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let v: CompactVec<i32> = CompactVec::new();
        assert!(v.is_empty());
        assert!(v.is_inline());
    }

    #[test]
    fn test_push_inline() {
        let mut v = CompactVec::new();
        v.push(1);
        v.push(2);
        assert_eq!(v.len(), 2);
        assert!(v.is_inline());
    }

    #[test]
    fn test_push_overflow() {
        let mut v = CompactVec::new();
        for i in 0..6 {
            v.push(i);
        }
        assert_eq!(v.len(), 6);
        assert!(!v.is_inline());
    }

    #[test]
    fn test_get() {
        let mut v = CompactVec::new();
        v.push(10);
        v.push(20);
        v.push(30);
        v.push(40);
        v.push(50);
        assert_eq!(v.get(0), Some(&10));
        assert_eq!(v.get(4), Some(&50));
        assert_eq!(v.get(10), None);
    }

    #[test]
    fn test_pop() {
        let mut v = CompactVec::new();
        v.push(1);
        v.push(2);
        assert_eq!(v.pop(), Some(2));
        assert_eq!(v.pop(), Some(1));
        assert_eq!(v.pop(), None);
    }

    #[test]
    fn test_clear() {
        let mut v = CompactVec::new();
        v.push(1);
        v.push(2);
        v.push(3);
        v.push(4);
        v.push(5);
        v.clear();
        assert!(v.is_empty());
    }

    #[test]
    fn test_contains() {
        let mut v = CompactVec::new();
        v.push(10);
        v.push(20);
        assert!(v.contains(&10));
        assert!(!v.contains(&30));
    }

    #[test]
    fn test_to_vec() {
        let mut v = CompactVec::new();
        v.push(1);
        v.push(2);
        v.push(3);
        assert_eq!(v.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn test_pop_from_overflow() {
        let mut v = CompactVec::new();
        for i in 0..6 {
            v.push(i);
        }
        assert_eq!(v.pop(), Some(5));
        assert_eq!(v.len(), 5);
    }

    #[test]
    fn test_default() {
        let v: CompactVec<f32> = CompactVec::default();
        assert!(v.is_empty());
    }
}
