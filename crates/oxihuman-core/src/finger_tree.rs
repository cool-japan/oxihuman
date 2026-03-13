// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Finger tree sequence stub — simulates O(1) push/pop at both ends with
//! amortised O(log n) split/concat, represented here as a thin deque wrapper.

use std::collections::VecDeque;

/// A finger tree sequence.
pub struct FingerTree<T> {
    data: VecDeque<T>,
}

impl<T: Clone> FingerTree<T> {
    /// Create an empty finger tree.
    pub fn new() -> Self {
        Self {
            data: VecDeque::new(),
        }
    }

    /// Push an element onto the left (front).
    pub fn push_left(&mut self, value: T) {
        self.data.push_front(value);
    }

    /// Push an element onto the right (back).
    pub fn push_right(&mut self, value: T) {
        self.data.push_back(value);
    }

    /// Pop an element from the left (front).
    pub fn pop_left(&mut self) -> Option<T> {
        self.data.pop_front()
    }

    /// Pop an element from the right (back).
    pub fn pop_right(&mut self) -> Option<T> {
        self.data.pop_back()
    }

    /// Peek at the leftmost element.
    pub fn peek_left(&self) -> Option<&T> {
        self.data.front()
    }

    /// Peek at the rightmost element.
    pub fn peek_right(&self) -> Option<&T> {
        self.data.back()
    }

    /// Concatenate another finger tree onto the right of this one.
    pub fn concat(&mut self, other: FingerTree<T>) {
        for item in other.data {
            self.data.push_back(item);
        }
    }

    /// Split the tree at index `at`, returning the right portion.
    pub fn split_at(&mut self, at: usize) -> FingerTree<T> {
        let right: VecDeque<T> = self.data.split_off(at);
        FingerTree { data: right }
    }

    /// Number of elements in the tree.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// True if the tree is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Collect all elements into a Vec.
    pub fn to_vec(&self) -> Vec<T> {
        self.data.iter().cloned().collect()
    }
}

impl<T: Clone> Default for FingerTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new empty finger tree.
pub fn new_finger_tree<T: Clone>() -> FingerTree<T> {
    FingerTree::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_left() {
        let mut ft: FingerTree<i32> = FingerTree::new();
        ft.push_left(1);
        ft.push_left(2);
        assert_eq!(ft.peek_left(), Some(&2)); /* last pushed left is front */
    }

    #[test]
    fn test_push_right() {
        let mut ft: FingerTree<i32> = FingerTree::new();
        ft.push_right(1);
        ft.push_right(2);
        assert_eq!(ft.peek_right(), Some(&2)); /* last pushed right is back */
    }

    #[test]
    fn test_pop_left() {
        let mut ft: FingerTree<i32> = FingerTree::new();
        ft.push_right(10);
        assert_eq!(ft.pop_left(), Some(10)); /* pop from left */
    }

    #[test]
    fn test_pop_right() {
        let mut ft: FingerTree<i32> = FingerTree::new();
        ft.push_right(5);
        assert_eq!(ft.pop_right(), Some(5)); /* pop from right */
    }

    #[test]
    fn test_concat() {
        let mut left: FingerTree<i32> = FingerTree::new();
        let mut right: FingerTree<i32> = FingerTree::new();
        left.push_right(1);
        right.push_right(2);
        left.concat(right);
        assert_eq!(left.len(), 2); /* two elements after concat */
    }

    #[test]
    fn test_split_at() {
        let mut ft: FingerTree<i32> = FingerTree::new();
        for i in 0..6 {
            ft.push_right(i);
        }
        let right = ft.split_at(3);
        assert_eq!(ft.len(), 3); /* left half */
        assert_eq!(right.len(), 3); /* right half */
    }

    #[test]
    fn test_to_vec() {
        let mut ft: FingerTree<i32> = FingerTree::new();
        ft.push_right(1);
        ft.push_right(2);
        ft.push_right(3);
        assert_eq!(ft.to_vec(), vec![1, 2, 3]); /* ordered correctly */
    }

    #[test]
    fn test_len_and_is_empty() {
        let ft: FingerTree<i32> = FingerTree::new();
        assert!(ft.is_empty()); /* new tree is empty */
        let mut ft2 = ft;
        ft2.push_right(1);
        assert_eq!(ft2.len(), 1);
    }

    #[test]
    fn test_default() {
        let ft: FingerTree<i32> = FingerTree::default();
        assert!(ft.is_empty()); /* default is empty */
    }

    #[test]
    fn test_new_helper() {
        let ft = new_finger_tree::<u8>();
        assert!(ft.is_empty()); /* helper works */
    }
}
