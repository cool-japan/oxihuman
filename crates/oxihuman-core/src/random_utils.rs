// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Pseudo-random number utilities based on a linear congruential generator (LCG).

/// Linear Congruential Generator state.
pub struct Lcg {
    state: u64,
}

impl Lcg {
    /// Create a new LCG with the given seed.
    pub fn new(seed: u64) -> Self {
        Lcg { state: seed }
    }

    /// Advance the state and return the next raw u64.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.state
    }

    /// Return a float in [0.0, 1.0).
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 11) as f32 / (1u64 << 53) as f32
    }

    /// Return a float in [lo, hi).
    pub fn next_range_f32(&mut self, lo: f32, hi: f32) -> f32 {
        lo + self.next_f32() * (hi - lo)
    }

    /// Return a u64 in [0, n).
    pub fn next_range_u64(&mut self, n: u64) -> u64 {
        if n == 0 {
            return 0;
        }
        self.next_u64() % n
    }
}

/// Shuffle a slice in-place using an Lcg.
pub fn lcg_shuffle<T>(rng: &mut Lcg, slice: &mut [T]) {
    let n = slice.len();
    for i in (1..n).rev() {
        let j = rng.next_range_u64((i + 1) as u64) as usize;
        slice.swap(i, j);
    }
}

/// Generate `n` f32 values in [0, 1) into a Vec.
pub fn lcg_sample_uniform(rng: &mut Lcg, n: usize) -> Vec<f32> {
    (0..n).map(|_| rng.next_f32()).collect()
}

/// Pick a random element from a non-empty slice. Returns None if empty.
pub fn lcg_choose<'a, T>(rng: &mut Lcg, slice: &'a [T]) -> Option<&'a T> {
    if slice.is_empty() {
        return None;
    }
    let idx = rng.next_range_u64(slice.len() as u64) as usize;
    Some(&slice[idx])
}

/// Box-Muller transform: returns a normally distributed f32 (mean=0, std=1).
pub fn lcg_normal(rng: &mut Lcg) -> f32 {
    use std::f32::consts::TAU;
    let u1 = (rng.next_f32() + f32::EPSILON).min(1.0 - f32::EPSILON);
    let u2 = rng.next_f32();
    (-2.0 * u1.ln()).sqrt() * (TAU * u2).cos()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lcg_range() {
        /* output in [0, 1) */
        let mut rng = Lcg::new(42);
        for _ in 0..100 {
            let v = rng.next_f32();
            assert!((0.0..1.0).contains(&v), "out of range: {v}");
        }
    }

    #[test]
    fn test_lcg_deterministic() {
        /* same seed -> same sequence */
        let mut r1 = Lcg::new(1234);
        let mut r2 = Lcg::new(1234);
        for _ in 0..10 {
            assert_eq!(r1.next_u64(), r2.next_u64());
        }
    }

    #[test]
    fn test_range_f32() {
        /* range outputs within [lo, hi) */
        let mut rng = Lcg::new(7);
        for _ in 0..100 {
            let v = rng.next_range_f32(5.0, 10.0);
            assert!((5.0..10.0).contains(&v), "out of range: {v}");
        }
    }

    #[test]
    fn test_range_u64() {
        /* modular range */
        let mut rng = Lcg::new(3);
        for _ in 0..50 {
            let v = rng.next_range_u64(10);
            assert!(v < 10, "out of range: {v}");
        }
    }

    #[test]
    fn test_shuffle_length_preserved() {
        /* shuffle keeps all elements */
        let mut rng = Lcg::new(99);
        let mut v: Vec<u32> = (0..10).collect();
        lcg_shuffle(&mut rng, &mut v);
        assert_eq!(v.len(), 10);
        let mut sorted = v.clone();
        sorted.sort();
        assert_eq!(sorted, (0u32..10).collect::<Vec<_>>());
    }

    #[test]
    fn test_sample_uniform_count() {
        /* returns correct number of samples */
        let mut rng = Lcg::new(0);
        let s = lcg_sample_uniform(&mut rng, 20);
        assert_eq!(s.len(), 20);
    }

    #[test]
    fn test_choose_empty() {
        /* empty slice returns None */
        let mut rng = Lcg::new(1);
        let empty: &[i32] = &[];
        assert!(lcg_choose(&mut rng, empty).is_none());
    }

    #[test]
    fn test_normal_roughly_unit_std() {
        /* normal samples have mean near 0 */
        let mut rng = Lcg::new(123456);
        let samples: Vec<f32> = (0..1000).map(|_| lcg_normal(&mut rng)).collect();
        let mean: f32 = samples.iter().sum::<f32>() / samples.len() as f32;
        assert!(mean.abs() < 0.2, "mean too large: {mean}");
    }
}
