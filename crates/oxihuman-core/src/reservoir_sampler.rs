// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Reservoir sampling (deterministic cycling via counter-based pseudo-random).

#![allow(dead_code)]

/// Reservoir sampler using Algorithm R with deterministic PRNG.
#[allow(dead_code)]
pub struct ReservoirSampler<T: Clone> {
    reservoir: Vec<T>,
    capacity: usize,
    count: usize,
    seed: u64,
}

/// Simple xorshift64 PRNG (deterministic, no rand dependency).
fn xorshift64(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

impl<T: Clone> ReservoirSampler<T> {
    #[allow(dead_code)]
    pub fn new(capacity: usize) -> Self {
        Self {
            reservoir: Vec::with_capacity(capacity),
            capacity: capacity.max(1),
            count: 0,
            seed: 0x517cc1b727220a95,
        }
    }

    #[allow(dead_code)]
    pub fn with_seed(capacity: usize, seed: u64) -> Self {
        Self {
            reservoir: Vec::with_capacity(capacity),
            capacity: capacity.max(1),
            count: 0,
            seed: seed | 1,
        }
    }

    #[allow(dead_code)]
    pub fn add(&mut self, item: T) {
        self.count += 1;
        if self.reservoir.len() < self.capacity {
            self.reservoir.push(item);
        } else {
            let j = (xorshift64(&mut self.seed) as usize) % self.count;
            if j < self.capacity {
                self.reservoir[j] = item;
            }
        }
    }

    #[allow(dead_code)]
    pub fn sample(&self) -> &[T] {
        &self.reservoir
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.reservoir.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.reservoir.is_empty()
    }

    #[allow(dead_code)]
    pub fn total_seen(&self) -> usize {
        self.count
    }

    #[allow(dead_code)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.reservoir.clear();
        self.count = 0;
    }

    #[allow(dead_code)]
    pub fn is_full(&self) -> bool {
        self.reservoir.len() >= self.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill_reservoir() {
        let mut rs = ReservoirSampler::new(5);
        for i in 0..5 {
            rs.add(i);
        }
        assert_eq!(rs.len(), 5);
        assert!(rs.is_full());
    }

    #[test]
    fn test_capacity_respected() {
        let mut rs = ReservoirSampler::new(3);
        for i in 0..100 {
            rs.add(i);
        }
        assert_eq!(rs.len(), 3);
    }

    #[test]
    fn test_total_seen() {
        let mut rs = ReservoirSampler::new(5);
        for i in 0..20 {
            rs.add(i);
        }
        assert_eq!(rs.total_seen(), 20);
    }

    #[test]
    fn test_empty() {
        let rs: ReservoirSampler<i32> = ReservoirSampler::new(5);
        assert!(rs.is_empty());
        assert_eq!(rs.total_seen(), 0);
    }

    #[test]
    fn test_reset() {
        let mut rs = ReservoirSampler::new(3);
        rs.add(1);
        rs.add(2);
        rs.reset();
        assert!(rs.is_empty());
        assert_eq!(rs.total_seen(), 0);
    }

    #[test]
    fn test_deterministic_with_seed() {
        let mut rs1 = ReservoirSampler::with_seed(5, 42);
        let mut rs2 = ReservoirSampler::with_seed(5, 42);
        for i in 0..100 {
            rs1.add(i);
            rs2.add(i);
        }
        assert_eq!(rs1.sample(), rs2.sample());
    }

    #[test]
    fn test_capacity_getter() {
        let rs: ReservoirSampler<i32> = ReservoirSampler::new(7);
        assert_eq!(rs.capacity(), 7);
    }

    #[test]
    fn test_min_capacity_one() {
        let mut rs = ReservoirSampler::new(0);
        rs.add(99);
        assert_eq!(rs.len(), 1);
    }

    #[test]
    fn test_sample_all_within_range() {
        let mut rs = ReservoirSampler::new(10);
        for i in 0..50i32 {
            rs.add(i);
        }
        for &v in rs.sample() {
            assert!((0..50).contains(&v));
        }
    }

    #[test]
    fn test_xorshift_not_zero() {
        let mut s = 0xdead_beef_cafe_u64;
        let v = xorshift64(&mut s);
        assert_ne!(v, 0);
    }
}
