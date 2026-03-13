// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Exponential histogram for sliding window quantiles.

#![allow(dead_code)]

/// A bucket in the exponential histogram.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpBucket {
    pub count: u64,
    pub timestamp: u64,
}

/// Exponential histogram for approximate quantile queries over sliding windows.
#[allow(dead_code)]
pub struct ExponentialHistogram {
    epsilon: f64,
    window: u64,
    buckets: Vec<ExpBucket>,
    total: u64,
    clock: u64,
}

impl ExponentialHistogram {
    #[allow(dead_code)]
    pub fn new(epsilon: f64, window: u64) -> Self {
        Self {
            epsilon: epsilon.max(1e-9),
            window,
            buckets: Vec::new(),
            total: 0,
            clock: 0,
        }
    }

    #[allow(dead_code)]
    pub fn tick(&mut self) {
        self.clock += 1;
        let cutoff = self.clock.saturating_sub(self.window);
        self.buckets.retain(|b| b.timestamp >= cutoff);
        self.total = self.buckets.iter().map(|b| b.count).sum();
    }

    #[allow(dead_code)]
    pub fn add(&mut self, count: u64) {
        self.buckets.push(ExpBucket { count, timestamp: self.clock });
        self.total = self.total.saturating_add(count);
        self.merge_buckets();
    }

    fn merge_buckets(&mut self) {
        let k = (1.0 / (2.0 * self.epsilon)).ceil() as usize;
        while self.buckets.len() > k.max(2) {
            let min_idx = self
                .buckets
                .windows(2)
                .enumerate()
                .min_by_key(|(_, w)| w[0].count + w[1].count)
                .map(|(i, _)| i)
                .unwrap_or(0);
            let next = self.buckets.remove(min_idx + 1);
            self.buckets[min_idx].count += next.count;
            self.buckets[min_idx].timestamp = self.buckets[min_idx].timestamp.max(next.timestamp);
        }
    }

    /// Estimate the count of elements in the window.
    #[allow(dead_code)]
    pub fn count(&self) -> u64 {
        self.total
    }

    /// Approximate quantile (rank in [0,1]).
    #[allow(dead_code)]
    pub fn quantile(&self, q: f64) -> u64 {
        if self.buckets.is_empty() {
            return 0;
        }
        let target = (q * self.total as f64) as u64;
        let mut cumulative = 0u64;
        for bucket in &self.buckets {
            cumulative += bucket.count;
            if cumulative >= target {
                return bucket.count;
            }
        }
        self.buckets.last().map_or(0, |b| b.count)
    }

    #[allow(dead_code)]
    pub fn bucket_count(&self) -> usize {
        self.buckets.len()
    }

    #[allow(dead_code)]
    pub fn epsilon(&self) -> f64 {
        self.epsilon
    }

    #[allow(dead_code)]
    pub fn window(&self) -> u64 {
        self.window
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.buckets.clear();
        self.total = 0;
        self.clock = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_count() {
        let eh = ExponentialHistogram::new(0.1, 100);
        assert_eq!(eh.count(), 0);
    }

    #[test]
    fn test_add_and_count() {
        let mut eh = ExponentialHistogram::new(0.5, 100);
        eh.add(5);
        eh.add(3);
        assert_eq!(eh.count(), 8);
    }

    #[test]
    fn test_tick_expires_old() {
        let mut eh = ExponentialHistogram::new(0.5, 2);
        eh.add(10);
        eh.tick();
        eh.tick();
        eh.tick();
        assert_eq!(eh.count(), 0);
    }

    #[test]
    fn test_bucket_merge_reduces_count() {
        let mut eh = ExponentialHistogram::new(0.5, 1000);
        for i in 1u64..=20 {
            eh.add(i);
        }
        assert!(eh.bucket_count() <= 20);
    }

    #[test]
    fn test_quantile_zero() {
        let mut eh = ExponentialHistogram::new(0.1, 100);
        eh.add(10);
        assert!(eh.quantile(0.0) <= 10);
    }

    #[test]
    fn test_quantile_one() {
        let mut eh = ExponentialHistogram::new(0.1, 100);
        eh.add(10);
        assert!(eh.quantile(1.0) > 0);
    }

    #[test]
    fn test_reset() {
        let mut eh = ExponentialHistogram::new(0.1, 100);
        eh.add(5);
        eh.reset();
        assert_eq!(eh.count(), 0);
        assert_eq!(eh.bucket_count(), 0);
    }

    #[test]
    fn test_epsilon_getter() {
        let eh = ExponentialHistogram::new(0.25, 50);
        assert!((eh.epsilon() - 0.25).abs() < 1e-9);
    }

    #[test]
    fn test_window_getter() {
        let eh = ExponentialHistogram::new(0.1, 77);
        assert_eq!(eh.window(), 77);
    }

    #[test]
    fn test_tick_multiple() {
        let mut eh = ExponentialHistogram::new(0.5, 5);
        eh.add(1);
        for _ in 0..6 {
            eh.tick();
        }
        assert_eq!(eh.count(), 0);
    }
}
