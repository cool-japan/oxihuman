// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Zipper list — a list with a movable focus/cursor, representing the
//! list as a pair of reversed prefix and suffix.

/// A zipper list with a focus on the current element.
pub struct ZipperList<T> {
    left: Vec<T>, /* reversed prefix — top is element to the left of focus */
    focus: Option<T>,
    right: Vec<T>, /* suffix — first element is the one to the right of focus */
}

impl<T: Clone> ZipperList<T> {
    /// Create a zipper from a slice, focusing on the first element.
    pub fn from_slice(items: &[T]) -> Self {
        if items.is_empty() {
            return Self {
                left: Vec::new(),
                focus: None,
                right: Vec::new(),
            };
        }
        Self {
            left: Vec::new(),
            focus: Some(items[0].clone()),
            right: items[1..].to_vec(),
        }
    }

    /// True if there is a focused element.
    pub fn has_focus(&self) -> bool {
        self.focus.is_some()
    }

    /// Borrow the focused element.
    pub fn peek(&self) -> Option<&T> {
        self.focus.as_ref()
    }

    /// Move focus one step to the right.
    pub fn move_right(&mut self) -> bool {
        if self.right.is_empty() {
            return false;
        }
        if let Some(f) = self.focus.take() {
            self.left.push(f);
        }
        self.focus = Some(self.right.remove(0));
        true
    }

    /// Move focus one step to the left.
    pub fn move_left(&mut self) -> bool {
        if self.left.is_empty() {
            return false;
        }
        if let Some(f) = self.focus.take() {
            self.right.insert(0, f);
        }
        self.focus = self.left.pop();
        true
    }

    /// Insert `value` before the current focus.
    pub fn insert_before(&mut self, value: T) {
        self.left.insert(0, value);
    }

    /// Insert `value` after the current focus.
    pub fn insert_after(&mut self, value: T) {
        self.right.insert(0, value);
    }

    /// Remove and return the focused element, focusing on the next right.
    pub fn remove_focus(&mut self) -> Option<T> {
        let removed = self.focus.take()?;
        if !self.right.is_empty() {
            self.focus = Some(self.right.remove(0));
        } else if !self.left.is_empty() {
            self.focus = self.left.pop();
        }
        Some(removed)
    }

    /// Collect all elements in order into a Vec.
    pub fn to_vec(&self) -> Vec<T> {
        let mut v: Vec<T> = self.left.iter().rev().cloned().collect();
        if let Some(f) = &self.focus {
            v.push(f.clone());
        }
        v.extend(self.right.iter().cloned());
        v
    }

    /// Total number of elements (left + focus + right).
    pub fn len(&self) -> usize {
        self.left.len() + self.focus.as_ref().map(|_| 1).unwrap_or(0) + self.right.len()
    }

    /// True if the zipper contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Create a zipper from a slice.
pub fn new_zipper_list<T: Clone>(items: &[T]) -> ZipperList<T> {
    ZipperList::from_slice(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_first_element() {
        let z = ZipperList::from_slice(&[1, 2, 3]);
        assert_eq!(z.peek(), Some(&1)); /* focus on first */
    }

    #[test]
    fn test_move_right() {
        let mut z = ZipperList::from_slice(&[1, 2, 3]);
        z.move_right();
        assert_eq!(z.peek(), Some(&2)); /* moved to second */
    }

    #[test]
    fn test_move_left() {
        let mut z = ZipperList::from_slice(&[1, 2, 3]);
        z.move_right();
        z.move_left();
        assert_eq!(z.peek(), Some(&1)); /* back to first */
    }

    #[test]
    fn test_to_vec_preserves_order() {
        let z = ZipperList::from_slice(&[1, 2, 3]);
        assert_eq!(z.to_vec(), vec![1, 2, 3]); /* order preserved */
    }

    #[test]
    fn test_insert_before() {
        let mut z = ZipperList::from_slice(&[1, 3]);
        z.move_right();
        z.insert_before(2);
        assert_eq!(z.to_vec(), vec![1, 2, 3]); /* inserted before focus */
    }

    #[test]
    fn test_insert_after() {
        let mut z = ZipperList::from_slice(&[1, 3]);
        z.insert_after(2);
        assert_eq!(z.to_vec(), vec![1, 2, 3]); /* inserted after focus */
    }

    #[test]
    fn test_remove_focus() {
        let mut z = ZipperList::from_slice(&[1, 2, 3]);
        let removed = z.remove_focus();
        assert_eq!(removed, Some(1)); /* removed first */
        assert_eq!(z.peek(), Some(&2)); /* focus moves to next */
    }

    #[test]
    fn test_len() {
        let z = ZipperList::from_slice(&[1, 2, 3]);
        assert_eq!(z.len(), 3); /* total length */
    }

    #[test]
    fn test_is_empty() {
        let z: ZipperList<i32> = ZipperList::from_slice(&[]);
        assert!(z.is_empty()); /* empty slice gives empty zipper */
    }

    #[test]
    fn test_new_helper() {
        let z = new_zipper_list(&[10, 20]);
        assert_eq!(z.peek(), Some(&10)); /* helper works */
    }
}
