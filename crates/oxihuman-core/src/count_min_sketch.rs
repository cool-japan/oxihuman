// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Count-Min Sketch for frequency estimation.

#![allow(dead_code)]

/// Count-Min Sketch using pairwise-independent hash families.
#[allow(dead_code)]
pub struct CountMinSketch {
    table: Vec<Vec<u64>>,
    depth: usize,
    width: usize,
    seeds: Vec<(u64, u64)>,
}

/// Polynomial hash with seed a, b: (a*x + b) mod prime.
fn poly_hash(data: &[u8], a: u64, b: u64, width: usize) -> usize {
    const PRIME: u64 = 0xFFFFFFFFFFFFFFC5;
    let mut x: u64 = 0;
    for &byte in data {
        x = x.wrapping_mul(31).wrapping_add(byte as u64);
    }
    (a.wrapping_mul(x).wrapping_add(b) % PRIME) as usize % width
}

impl CountMinSketch {
    #[allow(dead_code)]
    pub fn new(depth: usize, width: usize) -> Self {
        let seeds: Vec<(u64, u64)> = (0..depth)
            .map(|i| {
                let a = 0x9e3779b97f4a7c15u64.wrapping_mul(i as u64 + 1);
                let b = 0x6c62272e07bb0142u64.wrapping_mul(i as u64 + 3);
                (a | 1, b)
            })
            .collect();
        Self {
            table: vec![vec![0u64; width]; depth],
            depth,
            width,
            seeds,
        }
    }

    /// Create with error bounds epsilon and delta.
    #[allow(dead_code)]
    pub fn with_error(epsilon: f64, delta: f64) -> Self {
        let width = (2.0 / epsilon).ceil() as usize;
        let depth = (1.0 / delta).ln().ceil() as usize;
        Self::new(depth.max(1), width.max(4))
    }

    #[allow(dead_code)]
    pub fn update(&mut self, data: &[u8], count: u64) {
        for d in 0..self.depth {
            let (a, b) = self.seeds[d];
            let col = poly_hash(data, a, b, self.width);
            self.table[d][col] = self.table[d][col].saturating_add(count);
        }
    }

    #[allow(dead_code)]
    pub fn estimate(&self, data: &[u8]) -> u64 {
        (0..self.depth)
            .map(|d| {
                let (a, b) = self.seeds[d];
                let col = poly_hash(data, a, b, self.width);
                self.table[d][col]
            })
            .min()
            .unwrap_or(0)
    }

    #[allow(dead_code)]
    pub fn depth(&self) -> usize {
        self.depth
    }

    #[allow(dead_code)]
    pub fn width(&self) -> usize {
        self.width
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        for row in &mut self.table {
            for cell in row.iter_mut() {
                *cell = 0;
            }
        }
    }

    /// Total count across all updates (sum of row 0 / approx).
    #[allow(dead_code)]
    pub fn total(&self) -> u64 {
        self.table.first().map_or(0, |row| row.iter().sum())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_update_estimate() {
        let mut cms = CountMinSketch::new(4, 256);
        cms.update(b"hello", 3);
        assert!(cms.estimate(b"hello") >= 3);
    }

    #[test]
    fn test_zero_estimate_unseen() {
        let cms = CountMinSketch::new(4, 256);
        assert_eq!(cms.estimate(b"unseen"), 0);
    }

    #[test]
    fn test_multiple_items() {
        let mut cms = CountMinSketch::new(4, 512);
        cms.update(b"a", 10);
        cms.update(b"b", 5);
        assert!(cms.estimate(b"a") >= 10);
        assert!(cms.estimate(b"b") >= 5);
    }

    #[test]
    fn test_overcount_not_undercount() {
        let mut cms = CountMinSketch::new(4, 64);
        cms.update(b"item", 7);
        assert!(cms.estimate(b"item") >= 7);
    }

    #[test]
    fn test_reset() {
        let mut cms = CountMinSketch::new(3, 128);
        cms.update(b"x", 100);
        cms.reset();
        assert_eq!(cms.estimate(b"x"), 0);
    }

    #[test]
    fn test_with_error() {
        let cms = CountMinSketch::with_error(0.01, 0.01);
        assert!(cms.width() >= 4);
        assert!(cms.depth() >= 1);
    }

    #[test]
    fn test_depth_width() {
        let cms = CountMinSketch::new(5, 100);
        assert_eq!(cms.depth(), 5);
        assert_eq!(cms.width(), 100);
    }

    #[test]
    fn test_saturating_add() {
        let mut cms = CountMinSketch::new(2, 4);
        cms.update(b"k", u64::MAX);
        cms.update(b"k", 1);
        assert_eq!(cms.estimate(b"k"), u64::MAX);
    }

    #[test]
    fn test_update_multiple_times() {
        let mut cms = CountMinSketch::new(4, 256);
        for _ in 0..10 {
            cms.update(b"rep", 1);
        }
        assert!(cms.estimate(b"rep") >= 10);
    }

    #[test]
    fn test_empty_key() {
        let mut cms = CountMinSketch::new(3, 64);
        cms.update(b"", 2);
        assert!(cms.estimate(b"") >= 2);
    }
}
