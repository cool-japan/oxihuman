// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A growable array with explicit growth factor control and statistics.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GrowableArray<T> {
    data: Vec<T>,
    growth_factor: f32,
    grow_count: usize,
}

#[allow(dead_code)]
impl<T> GrowableArray<T> {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            growth_factor: 2.0,
            grow_count: 0,
        }
    }

    pub fn with_growth_factor(factor: f32) -> Self {
        assert!(factor > 1.0, "growth factor must be > 1.0");
        Self {
            data: Vec::new(),
            growth_factor: factor,
            grow_count: 0,
        }
    }

    pub fn push(&mut self, value: T) {
        if self.data.len() == self.data.capacity() && !self.data.is_empty() {
            let new_cap = ((self.data.capacity() as f32 * self.growth_factor) as usize).max(self.data.capacity() + 1);
            self.data.reserve(new_cap - self.data.capacity());
            self.grow_count += 1;
        }
        self.data.push(value);
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    pub fn grow_count(&self) -> usize {
        self.grow_count
    }

    pub fn growth_factor(&self) -> f32 {
        self.growth_factor
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn pop(&mut self) -> Option<T> {
        self.data.pop()
    }

    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    pub fn last(&self) -> Option<&T> {
        self.data.last()
    }

    pub fn truncate(&mut self, len: usize) {
        self.data.truncate(len);
    }
}

impl<T> Default for GrowableArray<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let a: GrowableArray<i32> = GrowableArray::new();
        assert!(a.is_empty());
    }

    #[test]
    fn test_push_and_get() {
        let mut a = GrowableArray::new();
        a.push(42);
        assert_eq!(a.get(0), Some(&42));
    }

    #[test]
    fn test_len() {
        let mut a = GrowableArray::new();
        a.push(1);
        a.push(2);
        assert_eq!(a.len(), 2);
    }

    #[test]
    fn test_pop() {
        let mut a = GrowableArray::new();
        a.push(10);
        assert_eq!(a.pop(), Some(10));
        assert!(a.is_empty());
    }

    #[test]
    fn test_clear() {
        let mut a = GrowableArray::new();
        a.push(1);
        a.push(2);
        a.clear();
        assert!(a.is_empty());
    }

    #[test]
    fn test_growth_factor() {
        let a = GrowableArray::<i32>::with_growth_factor(1.5);
        assert!((a.growth_factor() - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_as_slice() {
        let mut a = GrowableArray::new();
        a.push(1);
        a.push(2);
        assert_eq!(a.as_slice(), &[1, 2]);
    }

    #[test]
    fn test_last() {
        let mut a = GrowableArray::new();
        a.push(5);
        a.push(10);
        assert_eq!(a.last(), Some(&10));
    }

    #[test]
    fn test_truncate() {
        let mut a = GrowableArray::new();
        a.push(1);
        a.push(2);
        a.push(3);
        a.truncate(1);
        assert_eq!(a.len(), 1);
    }

    #[test]
    fn test_get_mut() {
        let mut a = GrowableArray::new();
        a.push(1);
        *a.get_mut(0).expect("should succeed") = 99;
        assert_eq!(a.get(0), Some(&99));
    }
}
