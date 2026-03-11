#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Histogram / frequency count over f32 values.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HistogramSimple {
    pub bins: Vec<u64>,
    pub min: f32,
    pub max: f32,
    pub bin_count: usize,
}

#[allow(dead_code)]
pub fn new_histogram_simple(min: f32, max: f32, bins: usize) -> HistogramSimple {
    let bin_count = bins.max(1);
    HistogramSimple {
        bins: vec![0u64; bin_count],
        min,
        max,
        bin_count,
    }
}

#[allow(dead_code)]
pub fn histogram_simple_add(h: &mut HistogramSimple, val: f32) {
    let bin = histogram_simple_bin(h, val);
    if bin < h.bin_count {
        h.bins[bin] += 1;
    }
}

#[allow(dead_code)]
pub fn histogram_simple_bin(h: &HistogramSimple, val: f32) -> usize {
    if h.max <= h.min {
        return 0;
    }
    let ratio = (val - h.min) / (h.max - h.min);
    let idx = (ratio * h.bin_count as f32).floor() as isize;
    idx.clamp(0, (h.bin_count - 1) as isize) as usize
}

#[allow(dead_code)]
pub fn histogram_simple_count(h: &HistogramSimple) -> u64 {
    h.bins.iter().sum()
}

#[allow(dead_code)]
pub fn histogram_simple_max_bin(h: &HistogramSimple) -> usize {
    h.bins
        .iter()
        .enumerate()
        .max_by_key(|(_, &v)| v)
        .map(|(i, _)| i)
        .unwrap_or(0)
}

#[allow(dead_code)]
pub fn histogram_simple_to_probabilities(h: &HistogramSimple) -> Vec<f64> {
    let total = histogram_simple_count(h) as f64;
    if total == 0.0 {
        return vec![0.0; h.bin_count];
    }
    h.bins.iter().map(|&v| v as f64 / total).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_histogram_zero_counts() {
        let h = new_histogram_simple(0.0, 1.0, 10);
        assert_eq!(histogram_simple_count(&h), 0);
        assert_eq!(h.bin_count, 10);
    }

    #[test]
    fn add_value_increments_bin() {
        let mut h = new_histogram_simple(0.0, 1.0, 10);
        histogram_simple_add(&mut h, 0.5);
        assert_eq!(histogram_simple_count(&h), 1);
    }

    #[test]
    fn bin_index_min_value() {
        let h = new_histogram_simple(0.0, 10.0, 10);
        assert_eq!(histogram_simple_bin(&h, 0.0), 0);
    }

    #[test]
    fn bin_index_max_value() {
        let h = new_histogram_simple(0.0, 10.0, 10);
        assert_eq!(histogram_simple_bin(&h, 10.0), 9);
    }

    #[test]
    fn max_bin_returns_most_frequent() {
        let mut h = new_histogram_simple(0.0, 10.0, 10);
        for _ in 0..5 {
            histogram_simple_add(&mut h, 7.5);
        }
        histogram_simple_add(&mut h, 1.0);
        assert_eq!(histogram_simple_max_bin(&h), histogram_simple_bin(&h, 7.5));
    }

    #[test]
    fn probabilities_sum_to_one() {
        let mut h = new_histogram_simple(0.0, 1.0, 4);
        histogram_simple_add(&mut h, 0.1);
        histogram_simple_add(&mut h, 0.4);
        histogram_simple_add(&mut h, 0.7);
        histogram_simple_add(&mut h, 0.9);
        let probs = histogram_simple_to_probabilities(&h);
        let sum: f64 = probs.iter().sum();
        assert!((sum - 1.0).abs() < 1e-9);
    }

    #[test]
    fn probabilities_empty_returns_zeros() {
        let h = new_histogram_simple(0.0, 1.0, 3);
        let probs = histogram_simple_to_probabilities(&h);
        assert!(probs.iter().all(|&p| p == 0.0));
    }

    #[test]
    fn out_of_range_clamped() {
        let mut h = new_histogram_simple(0.0, 1.0, 5);
        histogram_simple_add(&mut h, -1.0);
        histogram_simple_add(&mut h, 2.0);
        // Both should land in edge bins, not panic
        assert_eq!(histogram_simple_count(&h), 2);
    }

    #[test]
    fn single_bin() {
        let mut h = new_histogram_simple(0.0, 1.0, 1);
        histogram_simple_add(&mut h, 0.3);
        histogram_simple_add(&mut h, 0.7);
        assert_eq!(histogram_simple_count(&h), 2);
        assert_eq!(histogram_simple_max_bin(&h), 0);
    }

    #[test]
    fn multiple_adds() {
        let mut h = new_histogram_simple(0.0, 100.0, 10);
        for i in 0..10usize {
            histogram_simple_add(&mut h, i as f32 * 10.0 + 5.0);
        }
        assert_eq!(histogram_simple_count(&h), 10);
    }

    #[test]
    fn bin_count_stored() {
        let h = new_histogram_simple(-1.0, 1.0, 20);
        assert_eq!(h.bin_count, 20);
        assert_eq!(h.bins.len(), 20);
    }
}
