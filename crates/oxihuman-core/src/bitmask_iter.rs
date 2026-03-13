// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Utilities for iterating over set bits in a bitmask.

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bitmask(u64);

#[allow(dead_code)]
impl Bitmask {
    pub const EMPTY: Self = Self(0);
    pub const ALL: Self = Self(u64::MAX);

    pub fn new(bits: u64) -> Self {
        Self(bits)
    }

    pub fn from_bit(bit: u32) -> Self {
        if bit < 64 { Self(1u64 << bit) } else { Self::EMPTY }
    }

    pub fn bits(self) -> u64 {
        self.0
    }

    pub fn set(&mut self, bit: u32) {
        if bit < 64 {
            self.0 |= 1u64 << bit;
        }
    }

    pub fn clear_bit(&mut self, bit: u32) {
        if bit < 64 {
            self.0 &= !(1u64 << bit);
        }
    }

    pub fn is_set(self, bit: u32) -> bool {
        bit < 64 && (self.0 & (1u64 << bit)) != 0
    }

    pub fn count_ones(self) -> u32 {
        self.0.count_ones()
    }

    pub fn count_zeros(self) -> u32 {
        self.0.count_zeros()
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    pub fn difference(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    pub fn complement(self) -> Self {
        Self(!self.0)
    }

    pub fn iter(self) -> BitmaskIter {
        BitmaskIter { remaining: self.0 }
    }

    pub fn lowest_set_bit(self) -> Option<u32> {
        if self.0 == 0 { None } else { Some(self.0.trailing_zeros()) }
    }

    pub fn highest_set_bit(self) -> Option<u32> {
        if self.0 == 0 { None } else { Some(63 - self.0.leading_zeros()) }
    }
}

#[allow(dead_code)]
pub struct BitmaskIter {
    remaining: u64,
}

impl Iterator for BitmaskIter {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.remaining == 0 {
            return None;
        }
        let bit = self.remaining.trailing_zeros();
        self.remaining &= self.remaining - 1;
        Some(bit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_is_set() {
        let mut m = Bitmask::EMPTY;
        m.set(5);
        assert!(m.is_set(5));
        assert!(!m.is_set(4));
    }

    #[test]
    fn test_clear_bit() {
        let mut m = Bitmask::new(0xFF);
        m.clear_bit(0);
        assert!(!m.is_set(0));
        assert!(m.is_set(1));
    }

    #[test]
    fn test_count_ones() {
        let m = Bitmask::new(0b10110);
        assert_eq!(m.count_ones(), 3);
    }

    #[test]
    fn test_iter() {
        let m = Bitmask::new(0b1010);
        let bits: Vec<u32> = m.iter().collect();
        assert_eq!(bits, vec![1, 3]);
    }

    #[test]
    fn test_union() {
        let a = Bitmask::new(0b1100);
        let b = Bitmask::new(0b0011);
        assert_eq!(a.union(b).bits(), 0b1111);
    }

    #[test]
    fn test_intersection() {
        let a = Bitmask::new(0b1100);
        let b = Bitmask::new(0b1010);
        assert_eq!(a.intersection(b).bits(), 0b1000);
    }

    #[test]
    fn test_difference() {
        let a = Bitmask::new(0b1111);
        let b = Bitmask::new(0b0011);
        assert_eq!(a.difference(b).bits(), 0b1100);
    }

    #[test]
    fn test_from_bit() {
        let m = Bitmask::from_bit(3);
        assert!(m.is_set(3));
        assert_eq!(m.count_ones(), 1);
    }

    #[test]
    fn test_lowest_highest() {
        let m = Bitmask::new(0b101000);
        assert_eq!(m.lowest_set_bit(), Some(3));
        assert_eq!(m.highest_set_bit(), Some(5));
    }

    #[test]
    fn test_empty() {
        assert!(Bitmask::EMPTY.is_empty());
        assert_eq!(Bitmask::EMPTY.iter().count(), 0);
    }
}
