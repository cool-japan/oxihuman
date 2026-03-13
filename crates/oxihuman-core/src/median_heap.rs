// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Dynamic median with dual-heap (max-heap lower half + min-heap upper half).

#![allow(dead_code)]

use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// Ordered float wrapper for BinaryHeap (total order, NaN-safe).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
struct OrdF64(f64);

impl Eq for OrdF64 {}

impl PartialOrd for OrdF64 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrdF64 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap_or(std::cmp::Ordering::Equal)
    }
}

/// Dynamic median tracker using two heaps.
#[allow(dead_code)]
pub struct MedianHeap {
    lower: BinaryHeap<OrdF64>,
    upper: BinaryHeap<Reverse<OrdF64>>,
}

impl MedianHeap {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            lower: BinaryHeap::new(),
            upper: BinaryHeap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn push(&mut self, value: f64) {
        if self.lower.is_empty() || value <= self.lower.peek().map_or(f64::INFINITY, |v| v.0) {
            self.lower.push(OrdF64(value));
        } else {
            self.upper.push(Reverse(OrdF64(value)));
        }
        self.rebalance();
    }

    fn rebalance(&mut self) {
        while self.lower.len() > self.upper.len() + 1 {
            let Some(top) = self.lower.pop() else { return; };
            self.upper.push(Reverse(top));
        }
        while self.upper.len() > self.lower.len() {
            let Some(Reverse(top)) = self.upper.pop() else { return; };
            self.lower.push(top);
        }
    }

    /// Returns median, or None if empty.
    #[allow(dead_code)]
    pub fn median(&self) -> Option<f64> {
        if self.lower.is_empty() {
            return None;
        }
        if self.lower.len() == self.upper.len() {
            let lo = self.lower.peek()?.0;
            let hi = self.upper.peek().map_or(lo, |Reverse(v)| v.0);
            Some((lo + hi) / 2.0)
        } else {
            self.lower.peek().map(|v| v.0)
        }
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.lower.len() + self.upper.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.lower.is_empty() && self.upper.is_empty()
    }

    #[allow(dead_code)]
    pub fn lower_max(&self) -> Option<f64> {
        self.lower.peek().map(|v| v.0)
    }

    #[allow(dead_code)]
    pub fn upper_min(&self) -> Option<f64> {
        self.upper.peek().map(|Reverse(v)| v.0)
    }
}

impl Default for MedianHeap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_median() {
        let mh = MedianHeap::new();
        assert!(mh.median().is_none());
    }

    #[test]
    fn test_single_element() {
        let mut mh = MedianHeap::new();
        mh.push(42.0);
        assert_eq!(mh.median(), Some(42.0));
    }

    #[test]
    fn test_two_elements() {
        let mut mh = MedianHeap::new();
        mh.push(10.0);
        mh.push(20.0);
        assert_eq!(mh.median(), Some(15.0));
    }

    #[test]
    fn test_odd_count() {
        let mut mh = MedianHeap::new();
        for v in [1.0, 2.0, 3.0, 4.0, 5.0] {
            mh.push(v);
        }
        assert_eq!(mh.median(), Some(3.0));
    }

    #[test]
    fn test_even_count() {
        let mut mh = MedianHeap::new();
        for v in [1.0, 2.0, 3.0, 4.0] {
            mh.push(v);
        }
        assert_eq!(mh.median(), Some(2.5));
    }

    #[test]
    fn test_len() {
        let mut mh = MedianHeap::new();
        assert_eq!(mh.len(), 0);
        mh.push(1.0);
        mh.push(2.0);
        assert_eq!(mh.len(), 2);
    }

    #[test]
    fn test_is_empty() {
        let mh = MedianHeap::new();
        assert!(mh.is_empty());
    }

    #[test]
    fn test_lower_upper_bounds() {
        let mut mh = MedianHeap::new();
        for v in [5.0, 1.0, 3.0, 7.0] {
            mh.push(v);
        }
        assert!(mh.lower_max().expect("should succeed") <= mh.upper_min().expect("should succeed"));
    }

    #[test]
    fn test_reverse_order_insert() {
        let mut mh = MedianHeap::new();
        for v in [10.0, 8.0, 6.0, 4.0, 2.0] {
            mh.push(v);
        }
        assert_eq!(mh.median(), Some(6.0));
    }

    #[test]
    fn test_duplicate_values() {
        let mut mh = MedianHeap::new();
        for _ in 0..5 {
            mh.push(3.0);
        }
        assert_eq!(mh.median(), Some(3.0));
    }
}
