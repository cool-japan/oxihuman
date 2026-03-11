// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Count-Min Sketch v2 — frequency estimation in O(1) space per query.

const SEEDS: [u64; 8] = [
    0x517c_c1b7_2722_0a95,
    0x9e37_79b9_7f4a_7c15,
    0x6c62_272e_07bb_0142,
    0x0b5a_d4ec_ed1c_fd6c,
    0xbf58_476d_1ce4_e5b9,
    0x94d0_49bb_1331_11eb,
    0xd2a9_8b26_625e_ee7b,
    0xaaaa_5555_aaaa_5555,
];

/// Count-Min Sketch with configurable depth and width.
pub struct CountMinSketchV2 {
    table: Vec<Vec<u32>>,
    depth: usize,
    width: usize,
}

impl CountMinSketchV2 {
    /// Create a new sketch with `depth` rows and `width` counters per row.
    pub fn new(depth: usize, width: usize) -> Self {
        let d = depth.min(SEEDS.len()).max(1);
        let w = width.max(1);
        CountMinSketchV2 {
            table: vec![vec![0u32; w]; d],
            depth: d,
            width: w,
        }
    }

    fn row_index(&self, item: u64, row: usize) -> usize {
        let h = item
            .wrapping_mul(SEEDS[row])
            .rotate_left(33)
            .wrapping_add(SEEDS[(row + 1) % SEEDS.len()]);
        (h as usize) % self.width
    }

    /// Increment the frequency count for `item` by `delta`.
    pub fn add(&mut self, item: u64, delta: u32) {
        for row in 0..self.depth {
            let col = self.row_index(item, row);
            self.table[row][col] = self.table[row][col].saturating_add(delta);
        }
    }

    /// Estimate the frequency of `item` (upper bound).
    pub fn estimate(&self, item: u64) -> u32 {
        (0..self.depth)
            .map(|row| {
                let col = self.row_index(item, row);
                self.table[row][col]
            })
            .min()
            .unwrap_or(0)
    }

    /// Total depth (number of hash rows).
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Total width (counters per row).
    pub fn width(&self) -> usize {
        self.width
    }

    /// Reset all counters to zero.
    pub fn clear(&mut self) {
        for row in self.table.iter_mut() {
            for c in row.iter_mut() {
                *c = 0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_after_add() {
        let mut cms = CountMinSketchV2::new(4, 256);
        cms.add(7, 3);
        assert!(cms.estimate(7) >= 3 /* estimate must be at least true count */);
    }

    #[test]
    fn test_fresh_estimate_zero() {
        let cms = CountMinSketchV2::new(3, 128);
        assert_eq!(cms.estimate(42), 0 /* no inserts yet */);
    }

    #[test]
    fn test_add_multiple_items() {
        let mut cms = CountMinSketchV2::new(4, 512);
        cms.add(1, 5);
        cms.add(2, 10);
        assert!(cms.estimate(1) >= 5 /* item 1 must be at least 5 */);
        assert!(cms.estimate(2) >= 10 /* item 2 must be at least 10 */);
    }

    #[test]
    fn test_dimensions() {
        let cms = CountMinSketchV2::new(3, 64);
        assert_eq!(cms.depth(), 3 /* depth should be 3 */);
        assert_eq!(cms.width(), 64 /* width should be 64 */);
    }

    #[test]
    fn test_clear_resets() {
        let mut cms = CountMinSketchV2::new(2, 32);
        cms.add(10, 7);
        cms.clear();
        assert_eq!(cms.estimate(10), 0 /* after clear, estimate is 0 */);
    }

    #[test]
    fn test_saturating_add() {
        let mut cms = CountMinSketchV2::new(1, 4);
        cms.add(0, u32::MAX);
        cms.add(0, 1);
        assert_eq!(cms.estimate(0), u32::MAX /* saturates at max */);
    }

    #[test]
    fn test_depth_capped_at_seeds() {
        let cms = CountMinSketchV2::new(100, 16);
        assert!(cms.depth() <= SEEDS.len() /* depth capped at seed count */);
    }

    #[test]
    fn test_minimum_dimensions() {
        let cms = CountMinSketchV2::new(0, 0);
        assert!(cms.depth() >= 1 /* at least 1 row */);
        assert!(cms.width() >= 1 /* at least 1 column */);
    }

    #[test]
    fn test_heavy_hitter() {
        let mut cms = CountMinSketchV2::new(4, 256);
        for _ in 0..1000 {
            cms.add(99, 1);
        }
        assert!(cms.estimate(99) >= 1000 /* heavy hitter must be at least 1000 */);
    }
}
