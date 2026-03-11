// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Reservoir sampling using Vitter's Algorithm R.

/// A reservoir sampler that maintains a uniform random sample of size `k`.
pub struct ReservoirSampler<T> {
    reservoir: Vec<T>,
    capacity: usize,
    count: usize,
    rng_state: u64,
}

impl<T: Clone> ReservoirSampler<T> {
    /// Create a new reservoir sampler with capacity `k`.
    pub fn new(k: usize) -> Self {
        ReservoirSampler {
            reservoir: Vec::with_capacity(k),
            capacity: k.max(1),
            count: 0,
            rng_state: 6364136223846793005,
        }
    }

    fn next_rand(&mut self) -> u64 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 7;
        self.rng_state ^= self.rng_state << 17;
        self.rng_state
    }

    /// Feed a new item into the sampler (Vitter's Algorithm R).
    pub fn feed(&mut self, item: T) {
        self.count += 1;
        if self.reservoir.len() < self.capacity {
            self.reservoir.push(item);
        } else {
            let j = (self.next_rand() % self.count as u64) as usize;
            if j < self.capacity {
                self.reservoir[j] = item;
            }
        }
    }

    /// Return the current sample.
    pub fn sample(&self) -> &[T] {
        &self.reservoir
    }

    /// Number of items seen so far.
    pub fn items_seen(&self) -> usize {
        self.count
    }

    /// Sample size (may be less than capacity if fewer items fed).
    pub fn sample_size(&self) -> usize {
        self.reservoir.len()
    }

    /// Maximum sample capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Return true if the reservoir is full.
    pub fn is_full(&self) -> bool {
        self.reservoir.len() >= self.capacity
    }

    /// Clear the sampler.
    pub fn clear(&mut self) {
        self.reservoir.clear();
        self.count = 0;
    }

    /// Drain and return the sample.
    pub fn drain(self) -> Vec<T> {
        self.reservoir
    }
}

/// Sample `k` items from a slice using Algorithm R.
pub fn sample_slice<T: Clone>(data: &[T], k: usize) -> Vec<T> {
    let mut sampler = ReservoirSampler::new(k);
    for item in data {
        sampler.feed(item.clone());
    }
    sampler.reservoir
}

/// Sample `k` indices from `0..n` without replacement.
pub fn sample_indices(n: usize, k: usize) -> Vec<usize> {
    sample_slice(&(0..n).collect::<Vec<_>>(), k)
}

/// Create a new reservoir sampler.
pub fn new_reservoir_sampler<T: Clone>(k: usize) -> ReservoirSampler<T> {
    ReservoirSampler::new(k)
}

/// Weighted reservoir: feed item with integer weight (feed it weight times).
pub fn feed_weighted<T: Clone>(sampler: &mut ReservoirSampler<T>, item: T, weight: usize) {
    for _ in 0..weight {
        sampler.feed(item.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fills_up_to_capacity() {
        let mut s: ReservoirSampler<i32> = ReservoirSampler::new(5);
        for i in 0..5 {
            s.feed(i);
        }
        assert_eq!(s.sample_size(), 5);
        assert!(s.is_full());
    }

    #[test]
    fn test_does_not_exceed_capacity() {
        let mut s: ReservoirSampler<i32> = ReservoirSampler::new(3);
        for i in 0..100 {
            s.feed(i);
        }
        assert_eq!(s.sample_size(), 3);
        assert_eq!(s.items_seen(), 100);
    }

    #[test]
    fn test_all_items_from_small_set() {
        let data: Vec<i32> = (0..4).collect();
        let result = sample_slice(&data, 10);
        /* k > n: result contains all items */
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_sample_indices() {
        let idx = sample_indices(20, 5);
        assert_eq!(idx.len(), 5);
        /* All indices in range */
        for &i in &idx {
            assert!(i < 20);
        }
    }

    #[test]
    fn test_clear() {
        let mut s: ReservoirSampler<i32> = ReservoirSampler::new(5);
        s.feed(1);
        s.feed(2);
        s.clear();
        assert_eq!(s.items_seen(), 0);
        assert_eq!(s.sample_size(), 0);
    }

    #[test]
    fn test_capacity() {
        let s: ReservoirSampler<i32> = new_reservoir_sampler(7);
        assert_eq!(s.capacity(), 7);
    }

    #[test]
    fn test_drain() {
        let mut s: ReservoirSampler<i32> = ReservoirSampler::new(3);
        s.feed(10);
        s.feed(20);
        s.feed(30);
        let v = s.drain();
        assert_eq!(v.len(), 3);
    }

    #[test]
    fn test_sample_slice_correct_size() {
        let data: Vec<i32> = (0..50).collect();
        let result = sample_slice(&data, 10);
        assert_eq!(result.len(), 10);
    }
}
