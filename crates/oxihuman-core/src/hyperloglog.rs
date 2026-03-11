// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! HyperLogLog cardinality estimator.

#![allow(dead_code)]

/// HyperLogLog with configurable precision (b bits for register count).
#[allow(dead_code)]
pub struct HyperLogLog {
    registers: Vec<u8>,
    b: usize,
    m: usize,
}

fn fnv1a(data: &[u8]) -> u64 {
    const FNV_PRIME: u64 = 0x00000100_000001B3;
    const FNV_OFFSET: u64 = 0xcbf29ce4_84222325;
    let mut h = FNV_OFFSET;
    for &byte in data {
        h ^= byte as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

fn leading_zeros_plus_one(x: u64, bits: usize) -> u8 {
    let shifted = x << bits;
    if shifted == 0 {
        (64 - bits + 1) as u8
    } else {
        shifted.leading_zeros() as u8 + 1
    }
}

/// Alpha correction factor for HyperLogLog.
fn alpha(m: usize) -> f64 {
    match m {
        16 => 0.673,
        32 => 0.697,
        64 => 0.709,
        _ => 0.7213 / (1.0 + 1.079 / m as f64),
    }
}

impl HyperLogLog {
    /// Create with precision b (4..=18). m = 2^b registers.
    #[allow(dead_code)]
    pub fn new(b: usize) -> Self {
        let b = b.clamp(4, 18);
        let m = 1usize << b;
        Self {
            registers: vec![0u8; m],
            b,
            m,
        }
    }

    #[allow(dead_code)]
    pub fn add(&mut self, data: &[u8]) {
        let hash = fnv1a(data);
        let index = (hash >> (64 - self.b)) as usize;
        let w = hash << self.b | ((1u64 << self.b) - 1);
        let rho = leading_zeros_plus_one(w, self.b);
        if rho > self.registers[index] {
            self.registers[index] = rho;
        }
    }

    /// Estimate the number of distinct elements.
    #[allow(dead_code)]
    pub fn count(&self) -> u64 {
        let m = self.m as f64;
        let alpha = alpha(self.m);
        let sum: f64 = self
            .registers
            .iter()
            .map(|&r| 2.0_f64.powi(-(r as i32)))
            .sum();
        let estimate = alpha * m * m / sum;

        let zero_count = self.registers.iter().filter(|&&r| r == 0).count();
        if estimate < 2.5 * m && zero_count > 0 {
            (m * (m / zero_count as f64).ln()) as u64
        } else {
            estimate as u64
        }
    }

    #[allow(dead_code)]
    pub fn merge(&mut self, other: &HyperLogLog) {
        if self.m != other.m {
            return;
        }
        for (a, &b) in self.registers.iter_mut().zip(other.registers.iter()) {
            if b > *a {
                *a = b;
            }
        }
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        for r in &mut self.registers {
            *r = 0;
        }
    }

    #[allow(dead_code)]
    pub fn precision(&self) -> usize {
        self.b
    }

    #[allow(dead_code)]
    pub fn num_registers(&self) -> usize {
        self.m
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_count_zero() {
        let hll = HyperLogLog::new(10);
        assert_eq!(hll.count(), 0);
    }

    #[test]
    fn test_single_element() {
        let mut hll = HyperLogLog::new(10);
        hll.add(b"hello");
        assert!(hll.count() >= 1);
    }

    #[test]
    fn test_distinct_estimate_approximate() {
        let mut hll = HyperLogLog::new(12);
        for i in 0u32..1000 {
            hll.add(&i.to_le_bytes());
        }
        let est = hll.count();
        assert!(est > 800 && est < 1200, "estimate={est}");
    }

    #[test]
    fn test_duplicates_not_overcounted() {
        let mut hll = HyperLogLog::new(12);
        for _ in 0..100 {
            hll.add(b"same");
        }
        let est = hll.count();
        assert!(est < 10, "estimate={est}");
    }

    #[test]
    fn test_merge() {
        let mut hll1 = HyperLogLog::new(10);
        let mut hll2 = HyperLogLog::new(10);
        for i in 0u32..100 {
            hll1.add(&i.to_le_bytes());
        }
        for i in 100u32..200 {
            hll2.add(&i.to_le_bytes());
        }
        hll1.merge(&hll2);
        let est = hll1.count();
        /* HyperLogLog is probabilistic; allow ~30% error around 200 */
        assert!((100..=300).contains(&est), "estimate={est}");
    }

    #[test]
    fn test_reset() {
        let mut hll = HyperLogLog::new(10);
        hll.add(b"x");
        hll.reset();
        assert_eq!(hll.count(), 0);
    }

    #[test]
    fn test_precision_clamp() {
        let hll = HyperLogLog::new(2);
        assert_eq!(hll.precision(), 4);
        let hll2 = HyperLogLog::new(20);
        assert_eq!(hll2.precision(), 18);
    }

    #[test]
    fn test_num_registers() {
        let hll = HyperLogLog::new(8);
        assert_eq!(hll.num_registers(), 256);
    }

    #[test]
    fn test_merge_same_data() {
        let mut hll1 = HyperLogLog::new(10);
        let mut hll2 = HyperLogLog::new(10);
        hll1.add(b"a");
        hll2.add(b"a");
        hll1.merge(&hll2);
        let est = hll1.count();
        assert!(est < 5, "estimate={est}");
    }

    #[test]
    fn test_add_empty() {
        let mut hll = HyperLogLog::new(10);
        hll.add(b"");
        assert!(hll.count() >= 1);
    }
}
