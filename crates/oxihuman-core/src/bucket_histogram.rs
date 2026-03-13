// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Fixed-bucket histogram.

#[derive(Debug, Clone)]
pub struct BucketHistogram {
    pub min: f64,
    pub max: f64,
    buckets: Vec<u64>,
    pub underflow: u64,
    pub overflow: u64,
    total: u64,
}

impl BucketHistogram {
    pub fn new(min: f64, max: f64, num_buckets: usize) -> Self {
        assert!(num_buckets > 0, "need at least one bucket");
        assert!(max > min, "max must be greater than min");
        BucketHistogram {
            min,
            max,
            buckets: vec![0u64; num_buckets],
            underflow: 0,
            overflow: 0,
            total: 0,
        }
    }

    pub fn num_buckets(&self) -> usize {
        self.buckets.len()
    }

    pub fn bucket_width(&self) -> f64 {
        (self.max - self.min) / self.buckets.len() as f64
    }

    pub fn add(&mut self, value: f64) {
        self.total += 1;
        if value < self.min {
            self.underflow += 1;
            return;
        }
        if value >= self.max {
            self.overflow += 1;
            return;
        }
        let idx = ((value - self.min) / self.bucket_width()) as usize;
        let idx = idx.min(self.buckets.len() - 1);
        self.buckets[idx] += 1;
    }

    pub fn count(&self, bucket: usize) -> u64 {
        self.buckets.get(bucket).copied().unwrap_or(0)
    }

    pub fn total(&self) -> u64 {
        self.total
    }

    pub fn mode_bucket(&self) -> usize {
        self.buckets
            .iter()
            .enumerate()
            .max_by_key(|(_, &c)| c)
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    pub fn bucket_lower(&self, idx: usize) -> f64 {
        self.min + idx as f64 * self.bucket_width()
    }

    pub fn bucket_upper(&self, idx: usize) -> f64 {
        self.bucket_lower(idx) + self.bucket_width()
    }

    pub fn clear(&mut self) {
        self.buckets.iter_mut().for_each(|c| *c = 0);
        self.underflow = 0;
        self.overflow = 0;
        self.total = 0;
    }
}

pub fn histogram_mean(hist: &BucketHistogram) -> Option<f64> {
    let in_range = hist.total.saturating_sub(hist.underflow + hist.overflow);
    if in_range == 0 {
        return None;
    }
    let sum: f64 = (0..hist.num_buckets())
        .map(|i| {
            let mid = (hist.bucket_lower(i) + hist.bucket_upper(i)) / 2.0;
            mid * hist.count(i) as f64
        })
        .sum();
    Some(sum / in_range as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_add() {
        let mut h = BucketHistogram::new(0.0, 10.0, 10);
        h.add(5.0);
        assert_eq!(h.total(), 1);
    }

    #[test]
    fn test_underflow() {
        let mut h = BucketHistogram::new(0.0, 10.0, 10);
        h.add(-1.0);
        assert_eq!(h.underflow, 1);
    }

    #[test]
    fn test_overflow() {
        let mut h = BucketHistogram::new(0.0, 10.0, 10);
        h.add(10.0);
        assert_eq!(h.overflow, 1);
    }

    #[test]
    fn test_correct_bucket() {
        let mut h = BucketHistogram::new(0.0, 10.0, 10);
        h.add(2.5);
        assert_eq!(h.count(2), 1 /* 2.5 falls in bucket 2 (2.0-3.0) */,);
    }

    #[test]
    fn test_mode_bucket() {
        let mut h = BucketHistogram::new(0.0, 10.0, 10);
        h.add(5.0);
        h.add(5.1);
        h.add(5.2);
        h.add(1.0);
        assert_eq!(h.mode_bucket(), 5 /* most values in bucket 5 */,);
    }

    #[test]
    fn test_clear() {
        let mut h = BucketHistogram::new(0.0, 10.0, 5);
        h.add(1.0);
        h.add(2.0);
        h.clear();
        assert_eq!(h.total(), 0);
        assert_eq!(h.count(0), 0);
    }

    #[test]
    fn test_bucket_width() {
        let h = BucketHistogram::new(0.0, 10.0, 5);
        assert!((h.bucket_width() - 2.0).abs() < 1e-10 /* 10/5 = 2 */,);
    }

    #[test]
    fn test_histogram_mean() {
        let mut h = BucketHistogram::new(0.0, 10.0, 10);
        for _ in 0..10 {
            h.add(5.0);
        }
        let m = histogram_mean(&h).expect("should succeed");
        assert!((m - 5.5).abs() < 1.0 /* midpoint of bucket 5 = 5.5 */,);
    }

    #[test]
    fn test_bucket_bounds() {
        let h = BucketHistogram::new(0.0, 10.0, 10);
        assert!((h.bucket_lower(0) - 0.0).abs() < 1e-10);
        assert!((h.bucket_upper(0) - 1.0).abs() < 1e-10);
    }
}
