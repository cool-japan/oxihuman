// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Bounded histogram for metrics SDK.

/// A fixed-bucket histogram for recording distributions.
#[derive(Debug, Clone)]
pub struct MetricsHistogramSdk {
    boundaries: Vec<f64>,
    buckets: Vec<u64>,
    count: u64,
    sum: f64,
}

impl MetricsHistogramSdk {
    /// Create histogram with explicit boundary edges.
    pub fn new(boundaries: Vec<f64>) -> Self {
        let n = boundaries.len() + 1;
        Self {
            boundaries,
            buckets: vec![0; n],
            count: 0,
            sum: 0.0,
        }
    }

    pub fn record(&mut self, value: f64) {
        self.count += 1;
        self.sum += value;
        let idx = self
            .boundaries
            .iter()
            .position(|&b| value < b)
            .unwrap_or(self.boundaries.len());
        self.buckets[idx] += 1;
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn sum(&self) -> f64 {
        self.sum
    }

    pub fn mean(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum / self.count as f64
        }
    }

    pub fn bucket_count(&self, index: usize) -> u64 {
        self.buckets.get(index).copied().unwrap_or(0)
    }

    pub fn num_buckets(&self) -> usize {
        self.buckets.len()
    }

    pub fn reset(&mut self) {
        for b in &mut self.buckets {
            *b = 0;
        }
        self.count = 0;
        self.sum = 0.0;
    }
}

pub fn new_histogram_sdk(boundaries: Vec<f64>) -> MetricsHistogramSdk {
    MetricsHistogramSdk::new(boundaries)
}

pub fn hist_record(h: &mut MetricsHistogramSdk, value: f64) {
    h.record(value);
}

pub fn hist_count(h: &MetricsHistogramSdk) -> u64 {
    h.count()
}

pub fn hist_sum(h: &MetricsHistogramSdk) -> f64 {
    h.sum()
}

pub fn hist_mean(h: &MetricsHistogramSdk) -> f64 {
    h.mean()
}

pub fn hist_bucket(h: &MetricsHistogramSdk, idx: usize) -> u64 {
    h.bucket_count(idx)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_hist() -> MetricsHistogramSdk {
        /* boundaries: 10, 50, 100 → 4 buckets */
        new_histogram_sdk(vec![10.0, 50.0, 100.0])
    }

    #[test]
    fn test_record_and_count() {
        let mut h = make_hist();
        hist_record(&mut h, 5.0);
        hist_record(&mut h, 5.0);
        assert_eq!(hist_count(&h), 2);
    }

    #[test]
    fn test_sum() {
        let mut h = make_hist();
        hist_record(&mut h, 10.0);
        hist_record(&mut h, 20.0);
        assert!((hist_sum(&h) - 30.0).abs() < 1e-10);
    }

    #[test]
    fn test_mean() {
        let mut h = make_hist();
        hist_record(&mut h, 10.0);
        hist_record(&mut h, 30.0);
        assert!((hist_mean(&h) - 20.0).abs() < 1e-10);
    }

    #[test]
    fn test_mean_empty() {
        let h = make_hist();
        assert_eq!(hist_mean(&h), 0.0);
    }

    #[test]
    fn test_bucket_routing() {
        /* value 5 < 10 → bucket 0 */
        let mut h = make_hist();
        hist_record(&mut h, 5.0);
        assert_eq!(hist_bucket(&h, 0), 1);
    }

    #[test]
    fn test_overflow_bucket() {
        /* value 200 >= 100 → last bucket */
        let mut h = make_hist();
        hist_record(&mut h, 200.0);
        assert_eq!(hist_bucket(&h, 3), 1);
    }

    #[test]
    fn test_reset() {
        let mut h = make_hist();
        hist_record(&mut h, 5.0);
        h.reset();
        assert_eq!(hist_count(&h), 0);
        assert_eq!(hist_sum(&h), 0.0);
    }

    #[test]
    fn test_num_buckets() {
        let h = make_hist();
        assert_eq!(h.num_buckets(), 4);
    }

    #[test]
    fn test_boundary_value_goes_to_next_bucket() {
        /* value 10.0 is NOT < 10 → goes to bucket 1 */
        let mut h = make_hist();
        hist_record(&mut h, 10.0);
        assert_eq!(hist_bucket(&h, 1), 1);
    }
}
