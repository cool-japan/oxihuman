// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Rolling statistics: mean, variance, min, max over a sliding window.

#![allow(dead_code)]

use std::collections::VecDeque;

/// Rolling statistics accumulator.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RollingStats {
    pub window: usize,
    pub samples: VecDeque<f64>,
}

/// Create a new rolling stats with the given window size.
#[allow(dead_code)]
pub fn new_rolling_stats(window: usize) -> RollingStats {
    RollingStats {
        window: window.max(1),
        samples: VecDeque::with_capacity(window.max(1)),
    }
}

/// Push a new sample, dropping the oldest if the window is full.
#[allow(dead_code)]
pub fn rs_push(rs: &mut RollingStats, value: f64) {
    if rs.samples.len() == rs.window {
        rs.samples.pop_front();
    }
    rs.samples.push_back(value);
}

/// Compute the rolling mean. Returns 0.0 if empty.
#[allow(dead_code)]
pub fn rs_mean(rs: &RollingStats) -> f64 {
    if rs.samples.is_empty() {
        return 0.0;
    }
    rs.samples.iter().sum::<f64>() / rs.samples.len() as f64
}

/// Compute the rolling variance (population). Returns 0.0 if fewer than 2 samples.
#[allow(dead_code)]
pub fn rs_variance(rs: &RollingStats) -> f64 {
    if rs.samples.len() < 2 {
        return 0.0;
    }
    let mean = rs_mean(rs);
    let n = rs.samples.len() as f64;
    rs.samples
        .iter()
        .map(|&x| (x - mean) * (x - mean))
        .sum::<f64>()
        / n
}

/// Compute rolling standard deviation.
#[allow(dead_code)]
pub fn rs_std(rs: &RollingStats) -> f64 {
    rs_variance(rs).sqrt()
}

/// Rolling minimum. Returns f64::MAX if empty.
#[allow(dead_code)]
pub fn rs_min(rs: &RollingStats) -> f64 {
    rs.samples.iter().copied().fold(f64::MAX, f64::min)
}

/// Rolling maximum. Returns f64::MIN if empty.
#[allow(dead_code)]
pub fn rs_max(rs: &RollingStats) -> f64 {
    rs.samples.iter().copied().fold(f64::MIN, f64::max)
}

/// Number of samples currently stored.
#[allow(dead_code)]
pub fn rs_count(rs: &RollingStats) -> usize {
    rs.samples.len()
}

/// Check if the window is full.
#[allow(dead_code)]
pub fn rs_is_full(rs: &RollingStats) -> bool {
    rs.samples.len() == rs.window
}

/// Clear all samples.
#[allow(dead_code)]
pub fn rs_clear(rs: &mut RollingStats) {
    rs.samples.clear();
}

/// Rolling sum of all samples.
#[allow(dead_code)]
pub fn rs_sum(rs: &RollingStats) -> f64 {
    rs.samples.iter().sum()
}

/// Compute the median (approximate: uses sorted copy).
#[allow(dead_code)]
pub fn rs_median(rs: &RollingStats) -> f64 {
    if rs.samples.is_empty() {
        return 0.0;
    }
    let mut sorted: Vec<f64> = rs.samples.iter().copied().collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = sorted.len() / 2;
    if !sorted.len().is_multiple_of(2) {
        sorted[mid]
    } else {
        (sorted[mid - 1] + sorted[mid]) * 0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stats_empty() {
        let rs = new_rolling_stats(5);
        assert_eq!(rs_count(&rs), 0);
        assert!(!rs_is_full(&rs));
    }

    #[test]
    fn push_and_count() {
        let mut rs = new_rolling_stats(3);
        rs_push(&mut rs, 1.0);
        rs_push(&mut rs, 2.0);
        assert_eq!(rs_count(&rs), 2);
    }

    #[test]
    fn window_limit() {
        let mut rs = new_rolling_stats(3);
        for i in 0..5 {
            rs_push(&mut rs, i as f64);
        }
        assert_eq!(rs_count(&rs), 3);
        assert!(rs_is_full(&rs));
    }

    #[test]
    fn mean_correct() {
        let mut rs = new_rolling_stats(10);
        rs_push(&mut rs, 2.0);
        rs_push(&mut rs, 4.0);
        rs_push(&mut rs, 6.0);
        let m = rs_mean(&rs);
        assert!((m - 4.0).abs() < 1e-9);
    }

    #[test]
    fn variance_correct() {
        let mut rs = new_rolling_stats(10);
        rs_push(&mut rs, 2.0);
        rs_push(&mut rs, 4.0);
        rs_push(&mut rs, 6.0);
        let v = rs_variance(&rs);
        // mean=4, var = ((4+0+4)/3) = 8/3
        assert!((v - 8.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn min_max_correct() {
        let mut rs = new_rolling_stats(5);
        rs_push(&mut rs, 3.0);
        rs_push(&mut rs, 1.0);
        rs_push(&mut rs, 5.0);
        assert!((rs_min(&rs) - 1.0).abs() < 1e-9);
        assert!((rs_max(&rs) - 5.0).abs() < 1e-9);
    }

    #[test]
    fn sum_correct() {
        let mut rs = new_rolling_stats(5);
        rs_push(&mut rs, 10.0);
        rs_push(&mut rs, 20.0);
        rs_push(&mut rs, 30.0);
        assert!((rs_sum(&rs) - 60.0).abs() < 1e-9);
    }

    #[test]
    fn clear_empties() {
        let mut rs = new_rolling_stats(5);
        rs_push(&mut rs, 1.0);
        rs_clear(&mut rs);
        assert_eq!(rs_count(&rs), 0);
    }

    #[test]
    fn median_odd() {
        let mut rs = new_rolling_stats(10);
        rs_push(&mut rs, 5.0);
        rs_push(&mut rs, 1.0);
        rs_push(&mut rs, 3.0);
        assert!((rs_median(&rs) - 3.0).abs() < 1e-9);
    }

    #[test]
    fn mean_empty_zero() {
        let rs = new_rolling_stats(5);
        assert_eq!(rs_mean(&rs), 0.0);
    }

    #[test]
    fn std_nonnegative() {
        let mut rs = new_rolling_stats(5);
        rs_push(&mut rs, 1.0);
        rs_push(&mut rs, 2.0);
        rs_push(&mut rs, 3.0);
        assert!(rs_std(&rs) >= 0.0);
    }
}
