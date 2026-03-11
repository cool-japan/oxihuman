// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Histogram with configurable bins, count, and normalize.

/// A single histogram bin.
#[derive(Debug, Clone, PartialEq)]
pub struct HistBin {
    pub low: f32,
    pub high: f32,
    pub count: usize,
}

/// Histogram builder.
pub struct HistogramBuilder {
    min: f32,
    max: f32,
    bins: Vec<HistBin>,
}

/// Construct a new HistogramBuilder.
pub fn new_histogram(min: f32, max: f32, num_bins: usize) -> HistogramBuilder {
    let num_bins = num_bins.max(1);
    let width = (max - min) / num_bins as f32;
    let bins = (0..num_bins)
        .map(|i| HistBin {
            low: min + i as f32 * width,
            high: min + (i + 1) as f32 * width,
            count: 0,
        })
        .collect();
    HistogramBuilder { min, max, bins }
}

impl HistogramBuilder {
    /// Add a value to the histogram.
    pub fn add(&mut self, value: f32) {
        if self.bins.is_empty() {
            return;
        }
        if !(self.min..=self.max).contains(&value) {
            return;
        }
        let n = self.bins.len();
        let idx = if (value - self.max).abs() < 1e-9 {
            n - 1
        } else {
            let width = (self.max - self.min) / n as f32;
            ((value - self.min) / width) as usize
        };
        if idx < n {
            self.bins[idx].count += 1;
        }
    }

    /// Add many values.
    pub fn add_many(&mut self, values: &[f32]) {
        for &v in values {
            self.add(v);
        }
    }

    /// Return raw counts.
    pub fn counts(&self) -> Vec<usize> {
        self.bins.iter().map(|b| b.count).collect()
    }

    /// Return normalized frequencies (sum to 1.0).
    pub fn normalized(&self) -> Vec<f32> {
        let total: usize = self.bins.iter().map(|b| b.count).sum();
        if total == 0 {
            return vec![0.0; self.bins.len()];
        }
        self.bins
            .iter()
            .map(|b| b.count as f32 / total as f32)
            .collect()
    }

    /// Total count.
    pub fn total(&self) -> usize {
        self.bins.iter().map(|b| b.count).sum()
    }

    /// Number of bins.
    pub fn bin_count(&self) -> usize {
        self.bins.len()
    }

    /// Return the bin containing `value`.
    pub fn bin_of(&self, value: f32) -> Option<&HistBin> {
        self.bins.iter().find(|b| (b.low..b.high).contains(&value))
    }

    /// Reset all counts.
    pub fn reset(&mut self) {
        for b in &mut self.bins {
            b.count = 0;
        }
    }

    /// Return the mode bin (highest count).
    pub fn mode_bin(&self) -> Option<&HistBin> {
        self.bins.iter().max_by_key(|b| b.count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_histogram() {
        /* histogram with no values has zero total */
        let h = new_histogram(0.0, 10.0, 5);
        assert_eq!(h.total(), 0);
    }

    #[test]
    fn test_add_single() {
        /* adding one value increments one bin */
        let mut h = new_histogram(0.0, 10.0, 10);
        h.add(5.0);
        assert_eq!(h.total(), 1);
    }

    #[test]
    fn test_add_many_counts() {
        /* add_many correctly accumulates counts */
        let mut h = new_histogram(0.0, 4.0, 4);
        h.add_many(&[0.5, 1.5, 2.5, 3.5]);
        assert_eq!(h.total(), 4);
    }

    #[test]
    fn test_normalized_sums_to_one() {
        /* normalized frequencies sum to ~1.0 */
        let mut h = new_histogram(0.0, 10.0, 5);
        h.add_many(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);
        let sum: f32 = h.normalized().iter().sum();
        assert!((sum - 1.0).abs() < 1e-5, "sum={sum}");
    }

    #[test]
    fn test_out_of_range_ignored() {
        /* values outside [min, max] are ignored */
        let mut h = new_histogram(0.0, 10.0, 5);
        h.add(-1.0);
        h.add(11.0);
        assert_eq!(h.total(), 0);
    }

    #[test]
    fn test_bin_count() {
        /* bin_count returns number of bins */
        let h = new_histogram(0.0, 100.0, 20);
        assert_eq!(h.bin_count(), 20);
    }

    #[test]
    fn test_reset() {
        /* reset clears all counts */
        let mut h = new_histogram(0.0, 10.0, 5);
        h.add_many(&[1.0, 2.0, 3.0]);
        h.reset();
        assert_eq!(h.total(), 0);
    }

    #[test]
    fn test_mode_bin() {
        /* mode_bin returns bin with highest count */
        let mut h = new_histogram(0.0, 4.0, 4);
        h.add_many(&[0.5, 0.5, 0.5, 1.5, 2.5]);
        let mode = h.mode_bin().unwrap();
        assert_eq!(mode.count, 3);
    }

    #[test]
    fn test_counts_length() {
        /* counts() length equals bin_count */
        let h = new_histogram(0.0, 10.0, 7);
        assert_eq!(h.counts().len(), 7);
    }
}
