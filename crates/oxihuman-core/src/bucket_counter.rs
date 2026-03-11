// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Counts occurrences distributed into fixed-size buckets.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BucketCounter {
    buckets: Vec<u64>,
    bucket_size: f64,
    min_val: f64,
}

#[allow(dead_code)]
impl BucketCounter {
    pub fn new(min_val: f64, max_val: f64, num_buckets: usize) -> Self {
        assert!(num_buckets > 0);
        assert!(max_val > min_val);
        Self {
            buckets: vec![0; num_buckets],
            bucket_size: (max_val - min_val) / num_buckets as f64,
            min_val,
        }
    }

    pub fn record(&mut self, value: f64) {
        let idx = ((value - self.min_val) / self.bucket_size) as usize;
        let idx = idx.min(self.buckets.len() - 1);
        self.buckets[idx] += 1;
    }

    pub fn count(&self, bucket_idx: usize) -> u64 {
        self.buckets.get(bucket_idx).copied().unwrap_or(0)
    }

    pub fn total(&self) -> u64 {
        self.buckets.iter().sum()
    }

    pub fn num_buckets(&self) -> usize {
        self.buckets.len()
    }

    pub fn reset(&mut self) {
        self.buckets.iter_mut().for_each(|b| *b = 0);
    }

    pub fn max_bucket(&self) -> Option<usize> {
        self.buckets
            .iter()
            .enumerate()
            .max_by_key(|(_, &c)| c)
            .map(|(i, _)| i)
    }

    pub fn bucket_range(&self, idx: usize) -> (f64, f64) {
        let lo = self.min_val + idx as f64 * self.bucket_size;
        let hi = lo + self.bucket_size;
        (lo, hi)
    }

    pub fn percentile_bucket(&self, pct: f64) -> usize {
        let threshold = (self.total() as f64 * pct) as u64;
        let mut cumulative = 0u64;
        for (i, &c) in self.buckets.iter().enumerate() {
            cumulative += c;
            if cumulative >= threshold {
                return i;
            }
        }
        self.buckets.len() - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let bc = BucketCounter::new(0.0, 10.0, 5);
        assert_eq!(bc.num_buckets(), 5);
        assert_eq!(bc.total(), 0);
    }

    #[test]
    fn test_record() {
        let mut bc = BucketCounter::new(0.0, 10.0, 5);
        bc.record(1.0);
        bc.record(1.5);
        assert_eq!(bc.count(0), 2);
    }

    #[test]
    fn test_total() {
        let mut bc = BucketCounter::new(0.0, 10.0, 5);
        bc.record(1.0);
        bc.record(5.0);
        bc.record(9.0);
        assert_eq!(bc.total(), 3);
    }

    #[test]
    fn test_bucket_range() {
        let bc = BucketCounter::new(0.0, 10.0, 5);
        let (lo, hi) = bc.bucket_range(0);
        assert!((lo - 0.0).abs() < 1e-9);
        assert!((hi - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_max_bucket() {
        let mut bc = BucketCounter::new(0.0, 10.0, 5);
        bc.record(7.0);
        bc.record(7.5);
        bc.record(7.9);
        assert_eq!(bc.max_bucket(), Some(3));
    }

    #[test]
    fn test_reset() {
        let mut bc = BucketCounter::new(0.0, 10.0, 5);
        bc.record(1.0);
        bc.reset();
        assert_eq!(bc.total(), 0);
    }

    #[test]
    fn test_overflow_clamp() {
        let mut bc = BucketCounter::new(0.0, 10.0, 5);
        bc.record(100.0);
        assert_eq!(bc.count(4), 1);
    }

    #[test]
    fn test_percentile() {
        let mut bc = BucketCounter::new(0.0, 10.0, 5);
        for _ in 0..10 {
            bc.record(1.0);
        }
        for _ in 0..10 {
            bc.record(9.0);
        }
        let p50 = bc.percentile_bucket(0.5);
        assert_eq!(p50, 0);
    }

    #[test]
    fn test_single_bucket() {
        let mut bc = BucketCounter::new(0.0, 10.0, 1);
        bc.record(5.0);
        assert_eq!(bc.count(0), 1);
    }

    #[test]
    fn test_boundary_value() {
        let mut bc = BucketCounter::new(0.0, 10.0, 5);
        bc.record(2.0);
        assert_eq!(bc.count(1), 1);
    }
}
