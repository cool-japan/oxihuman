// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Fixed-size sliding window over f32 samples with stats.

use std::collections::VecDeque;

/// A fixed-capacity sliding window of f32 samples.
#[allow(dead_code)]
pub struct SlidingWindow {
    capacity: usize,
    samples: VecDeque<f32>,
    sum: f32,
    min_val: f32,
    max_val: f32,
}

#[allow(dead_code)]
impl SlidingWindow {
    pub fn new(capacity: usize) -> Self {
        let cap = capacity.max(1);
        Self {
            capacity: cap,
            samples: VecDeque::with_capacity(cap),
            sum: 0.0,
            min_val: f32::INFINITY,
            max_val: f32::NEG_INFINITY,
        }
    }

    /// Push a sample, dropping the oldest if at capacity.
    pub fn push(&mut self, value: f32) {
        if self.samples.len() == self.capacity {
            self.samples.pop_front();
            // Recompute sum and extremes.
            self.sum = self.samples.iter().sum();
            self.min_val = self.samples.iter().cloned().fold(f32::INFINITY, f32::min);
            self.max_val = self
                .samples
                .iter()
                .cloned()
                .fold(f32::NEG_INFINITY, f32::max);
        }
        self.samples.push_back(value);
        self.sum += value;
        self.min_val = self.min_val.min(value);
        self.max_val = self.max_val.max(value);
    }

    pub fn mean(&self) -> f32 {
        if self.samples.is_empty() {
            0.0
        } else {
            self.sum / self.samples.len() as f32
        }
    }

    pub fn sum(&self) -> f32 {
        self.sum
    }

    pub fn min(&self) -> Option<f32> {
        if self.samples.is_empty() {
            None
        } else {
            Some(self.min_val)
        }
    }

    pub fn max(&self) -> Option<f32> {
        if self.samples.is_empty() {
            None
        } else {
            Some(self.max_val)
        }
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.samples.len() == self.capacity
    }

    pub fn as_slice(&self) -> Vec<f32> {
        self.samples.iter().cloned().collect()
    }

    pub fn clear(&mut self) {
        self.samples.clear();
        self.sum = 0.0;
        self.min_val = f32::INFINITY;
        self.max_val = f32::NEG_INFINITY;
    }

    /// Variance of the window.
    pub fn variance(&self) -> f32 {
        if self.samples.len() < 2 {
            return 0.0;
        }
        let m = self.mean();
        self.samples.iter().map(|&x| (x - m) * (x - m)).sum::<f32>() / self.samples.len() as f32
    }
}

impl Default for SlidingWindow {
    fn default() -> Self {
        Self::new(16)
    }
}

pub fn new_sliding_window(capacity: usize) -> SlidingWindow {
    SlidingWindow::new(capacity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_push_and_mean() {
        let mut w = new_sliding_window(4);
        w.push(1.0);
        w.push(2.0);
        w.push(3.0);
        w.push(4.0);
        assert!((w.mean() - 2.5).abs() < 1e-5);
    }

    #[test]
    fn drops_oldest_when_full() {
        let mut w = new_sliding_window(3);
        w.push(1.0);
        w.push(2.0);
        w.push(3.0);
        w.push(4.0); // drops 1.0
        assert_eq!(w.len(), 3);
        assert!((w.sum() - 9.0).abs() < 1e-5);
    }

    #[test]
    fn min_max() {
        let mut w = new_sliding_window(5);
        w.push(3.0);
        w.push(1.0);
        w.push(5.0);
        assert_eq!(w.min(), Some(1.0));
        assert_eq!(w.max(), Some(5.0));
    }

    #[test]
    fn empty_window() {
        let w = new_sliding_window(4);
        assert!(w.is_empty());
        assert_eq!(w.min(), None);
        assert_eq!(w.max(), None);
        assert!((w.mean()).abs() < 1e-6);
    }

    #[test]
    fn is_full() {
        let mut w = new_sliding_window(2);
        w.push(1.0);
        assert!(!w.is_full());
        w.push(2.0);
        assert!(w.is_full());
    }

    #[test]
    fn clear() {
        let mut w = new_sliding_window(4);
        w.push(5.0);
        w.clear();
        assert!(w.is_empty());
        assert!((w.sum()).abs() < 1e-6);
    }

    #[test]
    fn variance_uniform() {
        let mut w = new_sliding_window(4);
        for _ in 0..4 {
            w.push(2.0);
        }
        assert!(w.variance().abs() < 1e-5);
    }

    #[test]
    fn capacity_constant() {
        let w = new_sliding_window(8);
        assert_eq!(w.capacity(), 8);
    }

    #[test]
    fn as_slice_ordered() {
        let mut w = new_sliding_window(3);
        w.push(10.0);
        w.push(20.0);
        w.push(30.0);
        assert_eq!(w.as_slice(), vec![10.0, 20.0, 30.0]);
    }

    #[test]
    fn single_element_variance_zero() {
        let mut w = new_sliding_window(5);
        w.push(7.0);
        assert!(w.variance().abs() < 1e-6);
    }
}
