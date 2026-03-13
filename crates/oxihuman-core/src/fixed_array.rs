// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Fixed-capacity stack-allocated array with runtime length tracking.

/// Fixed-capacity array with runtime length up to CAP.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FixedArray<T: Copy + Default, const CAP: usize> {
    data: [T; CAP],
    len: usize,
}

impl<T: Copy + Default, const CAP: usize> FixedArray<T, CAP> {
    /// Create an empty FixedArray.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            data: [T::default(); CAP],
            len: 0,
        }
    }

    /// Push a value; returns false if full.
    #[allow(dead_code)]
    pub fn push(&mut self, val: T) -> bool {
        if self.len >= CAP {
            return false;
        }
        self.data[self.len] = val;
        self.len += 1;
        true
    }

    /// Pop the last value.
    #[allow(dead_code)]
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        Some(self.data[self.len])
    }

    /// Current length.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Whether at capacity.
    #[allow(dead_code)]
    pub fn is_full(&self) -> bool {
        self.len >= CAP
    }

    /// Capacity.
    #[allow(dead_code)]
    pub fn capacity(&self) -> usize {
        CAP
    }

    /// Get element at index.
    #[allow(dead_code)]
    pub fn get(&self, idx: usize) -> Option<T> {
        if idx < self.len {
            Some(self.data[idx])
        } else {
            None
        }
    }

    /// Set element at index; returns false if out of bounds.
    #[allow(dead_code)]
    pub fn set(&mut self, idx: usize, val: T) -> bool {
        if idx < self.len {
            self.data[idx] = val;
            true
        } else {
            false
        }
    }

    /// Clear all elements.
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Slice of active elements.
    #[allow(dead_code)]
    pub fn as_slice(&self) -> &[T] {
        &self.data[..self.len]
    }

    /// Swap two elements by index.
    #[allow(dead_code)]
    pub fn swap(&mut self, a: usize, b: usize) -> bool {
        if a < self.len && b < self.len {
            self.data.swap(a, b);
            true
        } else {
            false
        }
    }
}

impl<T: Copy + Default, const CAP: usize> Default for FixedArray<T, CAP> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop() {
        let mut fa: FixedArray<i32, 4> = FixedArray::new();
        assert!(fa.push(10));
        assert!(fa.push(20));
        assert_eq!(fa.pop(), Some(20));
        assert_eq!(fa.len(), 1);
    }

    #[test]
    fn test_full() {
        let mut fa: FixedArray<u8, 2> = FixedArray::new();
        assert!(fa.push(1));
        assert!(fa.push(2));
        assert!(!fa.push(3));
        assert!(fa.is_full());
    }

    #[test]
    fn test_empty_pop() {
        let mut fa: FixedArray<u8, 4> = FixedArray::new();
        assert_eq!(fa.pop(), None);
    }

    #[test]
    fn test_get_set() {
        let mut fa: FixedArray<u32, 4> = FixedArray::new();
        fa.push(5);
        assert_eq!(fa.get(0), Some(5));
        fa.set(0, 99);
        assert_eq!(fa.get(0), Some(99));
    }

    #[test]
    fn test_out_of_bounds() {
        let fa: FixedArray<u32, 4> = FixedArray::new();
        assert_eq!(fa.get(0), None);
    }

    #[test]
    fn test_clear() {
        let mut fa: FixedArray<i32, 4> = FixedArray::new();
        fa.push(1);
        fa.push(2);
        fa.clear();
        assert!(fa.is_empty());
    }

    #[test]
    fn test_as_slice() {
        let mut fa: FixedArray<i32, 8> = FixedArray::new();
        fa.push(3);
        fa.push(7);
        assert_eq!(fa.as_slice(), &[3, 7]);
    }

    #[test]
    fn test_swap() {
        let mut fa: FixedArray<i32, 4> = FixedArray::new();
        fa.push(1);
        fa.push(2);
        fa.swap(0, 1);
        assert_eq!(fa.get(0), Some(2));
        assert_eq!(fa.get(1), Some(1));
    }

    #[test]
    fn test_capacity() {
        let fa: FixedArray<f32, 16> = FixedArray::new();
        assert_eq!(fa.capacity(), 16);
    }
}
