// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Welford's online algorithm for mean/variance/std.

/// Online running statistics (Welford's method).
pub struct RunningStatistics {
    count: u64,
    mean: f64,
    m2: f64,
    min: f64,
    max: f64,
}

/// Construct a new RunningStatistics.
pub fn new_running_stats() -> RunningStatistics {
    RunningStatistics {
        count: 0,
        mean: 0.0,
        m2: 0.0,
        min: f64::INFINITY,
        max: f64::NEG_INFINITY,
    }
}

impl RunningStatistics {
    /// Add a new value.
    pub fn add(&mut self, x: f64) {
        self.count += 1;
        let delta = x - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = x - self.mean;
        self.m2 += delta * delta2;
        if x < self.min {
            self.min = x;
        }
        if x > self.max {
            self.max = x;
        }
    }

    /// Add multiple values.
    pub fn add_slice(&mut self, xs: &[f64]) {
        for &x in xs {
            self.add(x);
        }
    }

    /// Number of values added.
    pub fn count(&self) -> u64 {
        self.count
    }

    /// Current mean.
    pub fn mean(&self) -> f64 {
        self.mean
    }

    /// Sample variance (n-1 denominator).
    pub fn variance(&self) -> f64 {
        if self.count < 2 {
            return 0.0;
        }
        self.m2 / (self.count - 1) as f64
    }

    /// Population variance (n denominator).
    pub fn pop_variance(&self) -> f64 {
        if self.count == 0 {
            return 0.0;
        }
        self.m2 / self.count as f64
    }

    /// Sample standard deviation.
    pub fn std_dev(&self) -> f64 {
        self.variance().sqrt()
    }

    /// Minimum value seen.
    pub fn min(&self) -> Option<f64> {
        if self.count == 0 {
            None
        } else {
            Some(self.min)
        }
    }

    /// Maximum value seen.
    pub fn max(&self) -> Option<f64> {
        if self.count == 0 {
            None
        } else {
            Some(self.max)
        }
    }

    /// Reset all statistics.
    pub fn reset(&mut self) {
        self.count = 0;
        self.mean = 0.0;
        self.m2 = 0.0;
        self.min = f64::INFINITY;
        self.max = f64::NEG_INFINITY;
    }
}

/// Compute mean of a slice (convenience).
pub fn slice_mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    xs.iter().sum::<f64>() / xs.len() as f64
}

/// Compute sample variance of a slice.
pub fn slice_variance(xs: &[f64]) -> f64 {
    let mut rs = new_running_stats();
    rs.add_slice(xs);
    rs.variance()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_count() {
        /* new stats has zero count */
        let rs = new_running_stats();
        assert_eq!(rs.count(), 0);
    }

    #[test]
    fn test_mean_single() {
        /* mean of single value is that value */
        let mut rs = new_running_stats();
        rs.add(7.0);
        assert!((rs.mean() - 7.0).abs() < 1e-12);
    }

    #[test]
    fn test_mean_multiple() {
        /* mean of [1,2,3,4,5] = 3 */
        let mut rs = new_running_stats();
        rs.add_slice(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!((rs.mean() - 3.0).abs() < 1e-12);
    }

    #[test]
    fn test_variance() {
        /* sample variance of [2, 4, 4, 4, 5, 5, 7, 9] = 4 */
        let mut rs = new_running_stats();
        rs.add_slice(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]);
        assert!(
            (rs.variance() - 4.571428).abs() < 0.01,
            "var={}",
            rs.variance()
        );
    }

    #[test]
    fn test_std_dev_constant() {
        /* std dev of constant data is 0 */
        let mut rs = new_running_stats();
        rs.add_slice(&[5.0, 5.0, 5.0, 5.0]);
        assert!(rs.std_dev() < 1e-12);
    }

    #[test]
    fn test_min_max() {
        /* min and max are tracked correctly */
        let mut rs = new_running_stats();
        rs.add_slice(&[3.0, 1.0, 4.0, 1.0, 5.0]);
        assert!((rs.min().expect("should succeed") - 1.0).abs() < 1e-12);
        assert!((rs.max().expect("should succeed") - 5.0).abs() < 1e-12);
    }

    #[test]
    fn test_reset() {
        /* reset clears all state */
        let mut rs = new_running_stats();
        rs.add_slice(&[1.0, 2.0, 3.0]);
        rs.reset();
        assert_eq!(rs.count(), 0);
        assert!(rs.min().is_none());
    }

    #[test]
    fn test_slice_mean() {
        /* slice_mean convenience function */
        assert!((slice_mean(&[10.0, 20.0, 30.0]) - 20.0).abs() < 1e-12);
    }

    #[test]
    fn test_slice_variance() {
        /* slice_variance convenience function */
        let v = slice_variance(&[1.0, 2.0, 3.0]);
        assert!((v - 1.0).abs() < 1e-9, "v={v}");
    }
}
