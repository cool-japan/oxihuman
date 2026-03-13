// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! A fixed-capacity stack-allocated vector backed by a plain array.

/// A vector-like container with a fixed compile-time capacity `N`.
#[allow(dead_code)]
pub struct StaticVec<T, const N: usize> {
    data: [Option<T>; N],
    len: usize,
}

impl<T: Default + Clone, const N: usize> StaticVec<T, N> {
    /// Creates an empty [`StaticVec`].
    pub fn new() -> Self {
        Self {
            data: std::array::from_fn(|_| None),
            len: 0,
        }
    }

    /// Pushes a value, returning `false` if capacity is exhausted.
    pub fn push(&mut self, val: T) -> bool {
        if self.len >= N {
            return false;
        }
        self.data[self.len] = Some(val);
        self.len += 1;
        true
    }

    /// Pops the last value.
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        self.data[self.len].take()
    }

    /// Returns a reference to the element at `idx`.
    pub fn get(&self, idx: usize) -> Option<&T> {
        if idx < self.len {
            self.data[idx].as_ref()
        } else {
            None
        }
    }

    /// Returns the number of elements.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the compile-time capacity.
    pub fn capacity(&self) -> usize {
        N
    }

    /// Returns `true` if full.
    pub fn is_full(&self) -> bool {
        self.len >= N
    }

    /// Clears all elements.
    pub fn clear(&mut self) {
        for i in 0..self.len {
            self.data[i] = None;
        }
        self.len = 0;
    }

    /// Returns the last element without removing it.
    pub fn last(&self) -> Option<&T> {
        if self.len == 0 {
            None
        } else {
            self.data[self.len - 1].as_ref()
        }
    }

    /// Iterates over references to all stored elements.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data[..self.len].iter().filter_map(|x| x.as_ref())
    }
}

impl<T: Default + Clone, const N: usize> Default for StaticVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_static_vec<T: Default + Clone, const N: usize>() -> StaticVec<T, N> {
    StaticVec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty() {
        let v: StaticVec<i32, 4> = new_static_vec();
        assert!(v.is_empty());
        assert_eq!(v.len(), 0);
    }

    #[test]
    fn push_and_pop() {
        let mut v: StaticVec<i32, 4> = new_static_vec();
        assert!(v.push(10));
        assert!(v.push(20));
        assert_eq!(v.pop(), Some(20));
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn capacity_limit() {
        let mut v: StaticVec<i32, 2> = new_static_vec();
        assert!(v.push(1));
        assert!(v.push(2));
        assert!(!v.push(3)); // full
        assert!(v.is_full());
    }

    #[test]
    fn get_in_bounds() {
        let mut v: StaticVec<i32, 4> = new_static_vec();
        v.push(42);
        assert_eq!(v.get(0), Some(&42));
        assert_eq!(v.get(1), None);
    }

    #[test]
    fn clear_resets() {
        let mut v: StaticVec<i32, 4> = new_static_vec();
        v.push(1);
        v.push(2);
        v.clear();
        assert!(v.is_empty());
    }

    #[test]
    fn last_element() {
        let mut v: StaticVec<i32, 4> = new_static_vec();
        v.push(5);
        v.push(7);
        assert_eq!(v.last(), Some(&7));
    }

    #[test]
    fn iter_all() {
        let mut v: StaticVec<i32, 4> = new_static_vec();
        v.push(1);
        v.push(2);
        v.push(3);
        let collected: Vec<i32> = v.iter().copied().collect();
        assert_eq!(collected, vec![1, 2, 3]);
    }

    #[test]
    fn pop_empty_returns_none() {
        let mut v: StaticVec<i32, 4> = new_static_vec();
        assert_eq!(v.pop(), None);
    }

    #[test]
    fn capacity_is_n() {
        let v: StaticVec<i32, 8> = new_static_vec();
        assert_eq!(v.capacity(), 8);
    }

    #[test]
    fn refill_after_clear() {
        let mut v: StaticVec<i32, 2> = new_static_vec();
        v.push(1);
        v.push(2);
        v.clear();
        assert!(v.push(3));
        assert_eq!(v.len(), 1);
    }
}
