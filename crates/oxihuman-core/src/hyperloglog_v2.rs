// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! HyperLogLog v2 — probabilistic cardinality estimation.

const HLL_SEED: u64 = 0x9e37_79b9_7f4a_7c15;
const ALPHA_16: f64 = 0.673;
const ALPHA_32: f64 = 0.697;
const ALPHA_64: f64 = 0.709;

/// HyperLogLog cardinality estimator with `2^b` registers.
pub struct HyperLogLogV2 {
    registers: Vec<u8>,
    b: u8,
    m: usize,
}

impl HyperLogLogV2 {
    /// Create a new HLL with `b` bits of precision (b in 4..=16).
    pub fn new(b: u8) -> Self {
        let b = b.clamp(4, 16);
        let m = 1usize << b;
        HyperLogLogV2 {
            registers: vec![0u8; m],
            b,
            m,
        }
    }

    fn hash_item(&self, item: u64) -> u64 {
        item.wrapping_mul(HLL_SEED)
            .rotate_left(27)
            .wrapping_add(HLL_SEED >> 17)
    }

    fn leading_zeros_plus_one(val: u64, bits: u8) -> u8 {
        let shifted = val << bits;
        shifted.leading_zeros() as u8 + 1
    }

    /// Add an element to the estimator.
    pub fn add(&mut self, item: u64) {
        let h = self.hash_item(item);
        let reg_idx = (h >> (64 - self.b)) as usize;
        let w = h << self.b;
        let rho = w.leading_zeros() as u8 + 1;
        if rho > self.registers[reg_idx] {
            self.registers[reg_idx] = rho;
        }
    }

    /// Estimate the cardinality.
    pub fn count(&self) -> u64 {
        let m = self.m as f64;
        let alpha = match self.m {
            16 => ALPHA_16,
            32 => ALPHA_32,
            64 => ALPHA_64,
            _ => 0.7213 / (1.0 + 1.079 / m),
        };
        let z: f64 = self
            .registers
            .iter()
            .map(|&r| 2.0f64.powi(-(r as i32)))
            .sum();
        let raw = alpha * m * m / z;
        /* small range correction: use linear counting when estimate is small */
        let zeros = self.registers.iter().filter(|&&r| r == 0).count();
        if raw < 2.5 * m && zeros > 0 {
            (m * (m / zeros as f64).ln()).round() as u64
        } else {
            raw.round() as u64
        }
    }

    /// Precision bits.
    pub fn precision(&self) -> u8 {
        self.b
    }

    /// Number of registers.
    pub fn num_registers(&self) -> usize {
        self.m
    }

    /// Merge another HLL into this one (must have same precision).
    pub fn merge(&mut self, other: &HyperLogLogV2) {
        if other.b == self.b {
            for (r, &o) in self.registers.iter_mut().zip(other.registers.iter()) {
                if o > *r {
                    *r = o;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_cardinality() {
        let hll = HyperLogLogV2::new(8);
        assert_eq!(hll.count(), 0 /* no elements added */);
    }

    #[test]
    fn test_single_element() {
        let mut hll = HyperLogLogV2::new(8);
        hll.add(1234);
        assert!(hll.count() >= 1 /* at least one element counted */);
    }

    #[test]
    fn test_many_elements_approximate() {
        let mut hll = HyperLogLogV2::new(12);
        for i in 0u64..1000 {
            hll.add(i);
        }
        let est = hll.count();
        /* estimate should be within 20% of 1000 */
        assert!((800..=1200).contains(&est));
    }

    #[test]
    fn test_precision_clamp() {
        let hll = HyperLogLogV2::new(2);
        assert_eq!(hll.precision(), 4 /* clamped to minimum 4 */);
    }

    #[test]
    fn test_num_registers() {
        let hll = HyperLogLogV2::new(8);
        assert_eq!(hll.num_registers(), 256 /* 2^8 registers */);
    }

    #[test]
    fn test_merge_same_precision() {
        let mut a = HyperLogLogV2::new(8);
        let mut b = HyperLogLogV2::new(8);
        for i in 0u64..500 {
            a.add(i);
        }
        for i in 500u64..1000 {
            b.add(i);
        }
        a.merge(&b);
        let est = a.count();
        assert!(est >= 700 /* merged estimate should approach 1000 */);
    }

    #[test]
    fn test_merge_different_precision_ignored() {
        let mut a = HyperLogLogV2::new(8);
        let b = HyperLogLogV2::new(10);
        /* merging different precision is a no-op */
        let before = a.count();
        a.merge(&b);
        assert_eq!(a.count(), before);
    }

    #[test]
    fn test_high_precision() {
        let hll = HyperLogLogV2::new(16);
        assert_eq!(hll.num_registers(), 65536 /* 2^16 registers */);
    }

    #[test]
    fn test_duplicate_inserts() {
        let mut hll = HyperLogLogV2::new(8);
        for _ in 0..100 {
            hll.add(42 /* same element 100 times */);
        }
        let est = hll.count();
        /* estimate should be close to 1 */
        assert!(est <= 10);
    }
}
