// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A pair of identically-typed buffers that can be swapped atomically.
/// Useful for producer/consumer patterns without locking.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SwapBuffer<T> {
    front: Vec<T>,
    back: Vec<T>,
    swap_count: usize,
}

#[allow(dead_code)]
impl<T> SwapBuffer<T> {
    pub fn new() -> Self {
        Self {
            front: Vec::new(),
            back: Vec::new(),
            swap_count: 0,
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            front: Vec::with_capacity(cap),
            back: Vec::with_capacity(cap),
            swap_count: 0,
        }
    }

    pub fn push_back(&mut self, item: T) {
        self.back.push(item);
    }

    pub fn swap(&mut self) {
        std::mem::swap(&mut self.front, &mut self.back);
        self.back.clear();
        self.swap_count += 1;
    }

    pub fn front(&self) -> &[T] {
        &self.front
    }

    pub fn back_len(&self) -> usize {
        self.back.len()
    }

    pub fn front_len(&self) -> usize {
        self.front.len()
    }

    pub fn is_front_empty(&self) -> bool {
        self.front.is_empty()
    }

    pub fn is_back_empty(&self) -> bool {
        self.back.is_empty()
    }

    pub fn swap_count(&self) -> usize {
        self.swap_count
    }

    pub fn clear_all(&mut self) {
        self.front.clear();
        self.back.clear();
    }

    pub fn drain_front(&mut self) -> Vec<T> {
        std::mem::take(&mut self.front)
    }

    pub fn total_items(&self) -> usize {
        self.front.len() + self.back.len()
    }
}

impl<T> Default for SwapBuffer<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let sb: SwapBuffer<i32> = SwapBuffer::new();
        assert!(sb.is_front_empty());
        assert!(sb.is_back_empty());
    }

    #[test]
    fn test_push_back() {
        let mut sb = SwapBuffer::new();
        sb.push_back(1);
        sb.push_back(2);
        assert_eq!(sb.back_len(), 2);
        assert!(sb.is_front_empty());
    }

    #[test]
    fn test_swap() {
        let mut sb = SwapBuffer::new();
        sb.push_back(10);
        sb.push_back(20);
        sb.swap();
        assert_eq!(sb.front(), &[10, 20]);
        assert!(sb.is_back_empty());
    }

    #[test]
    fn test_swap_count() {
        let mut sb: SwapBuffer<i32> = SwapBuffer::new();
        sb.swap();
        sb.swap();
        assert_eq!(sb.swap_count(), 2);
    }

    #[test]
    fn test_drain_front() {
        let mut sb = SwapBuffer::new();
        sb.push_back(5);
        sb.swap();
        let drained = sb.drain_front();
        assert_eq!(drained, vec![5]);
        assert!(sb.is_front_empty());
    }

    #[test]
    fn test_clear_all() {
        let mut sb = SwapBuffer::new();
        sb.push_back(1);
        sb.swap();
        sb.push_back(2);
        sb.clear_all();
        assert_eq!(sb.total_items(), 0);
    }

    #[test]
    fn test_total_items() {
        let mut sb = SwapBuffer::new();
        sb.push_back(1);
        sb.swap();
        sb.push_back(2);
        assert_eq!(sb.total_items(), 2);
    }

    #[test]
    fn test_double_swap() {
        let mut sb = SwapBuffer::new();
        sb.push_back(1);
        sb.swap();
        sb.push_back(2);
        sb.swap();
        assert_eq!(sb.front(), &[2]);
    }

    #[test]
    fn test_with_capacity() {
        let sb: SwapBuffer<i32> = SwapBuffer::with_capacity(16);
        assert!(sb.is_front_empty());
    }

    #[test]
    fn test_default() {
        let sb: SwapBuffer<f32> = SwapBuffer::default();
        assert!(sb.is_front_empty());
    }
}
