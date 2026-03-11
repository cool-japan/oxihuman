// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Fixed-capacity bitset backed by a `Vec<u64>`.

/// A bitset with a fixed capacity set at construction time.
#[derive(Debug, Clone)]
pub struct FixedBitset {
    words: Vec<u64>,
    capacity: usize,
}

impl FixedBitset {
    /// Create a bitset that can hold `capacity` bits (all cleared).
    pub fn new(capacity: usize) -> Self {
        let words = capacity.div_ceil(64);
        FixedBitset { words: vec![0u64; words], capacity }
    }

    /// Set bit `i` to 1.
    pub fn set(&mut self, i: usize) {
        assert!(i < self.capacity);
        self.words[i / 64] |= 1u64 << (i % 64);
    }

    /// Clear bit `i`.
    pub fn clear(&mut self, i: usize) {
        assert!(i < self.capacity);
        self.words[i / 64] &= !(1u64 << (i % 64));
    }

    /// Test whether bit `i` is set.
    pub fn test(&self, i: usize) -> bool {
        assert!(i < self.capacity);
        (self.words[i / 64] >> (i % 64)) & 1 == 1
    }

    /// Toggle bit `i`.
    pub fn toggle(&mut self, i: usize) {
        assert!(i < self.capacity);
        self.words[i / 64] ^= 1u64 << (i % 64);
    }

    /// Count the number of set bits.
    pub fn count_ones(&self) -> u32 {
        self.words.iter().map(|w| w.count_ones()).sum()
    }

    /// Capacity (maximum number of bits).
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Reset all bits to 0.
    pub fn clear_all(&mut self) {
        for w in &mut self.words {
            *w = 0;
        }
    }

    /// Set all bits to 1 (within capacity).
    pub fn set_all(&mut self) {
        for w in &mut self.words {
            *w = u64::MAX;
        }
        /* mask the last word to avoid phantom bits */
        let rem = self.capacity % 64;
        if rem != 0 {
            if let Some(last) = self.words.last_mut() {
                *last = (1u64 << rem).saturating_sub(1);
            }
        }
    }
}

/// Create a new fixed bitset with `capacity` bits.
pub fn new_fixed_bitset(capacity: usize) -> FixedBitset {
    FixedBitset::new(capacity)
}

/// Set bit `i`.
pub fn fb_set(bs: &mut FixedBitset, i: usize) {
    bs.set(i);
}

/// Clear bit `i`.
pub fn fb_clear(bs: &mut FixedBitset, i: usize) {
    bs.clear(i);
}

/// Test bit `i`.
pub fn fb_test(bs: &FixedBitset, i: usize) -> bool {
    bs.test(i)
}

/// Count set bits.
pub fn fb_count_ones(bs: &FixedBitset) -> u32 {
    bs.count_ones()
}

/// Capacity of the bitset.
pub fn fb_capacity(bs: &FixedBitset) -> usize {
    bs.capacity()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_clear() {
        let bs = new_fixed_bitset(128);
        assert_eq!(fb_count_ones(&bs), 0 /* all zeros initially */);
    }

    #[test]
    fn test_set_and_test() {
        let mut bs = new_fixed_bitset(64);
        fb_set(&mut bs, 7);
        assert!(fb_test(&bs, 7) /* bit 7 set */);
        assert!(!fb_test(&bs, 0));
    }

    #[test]
    fn test_clear_bit() {
        let mut bs = new_fixed_bitset(64);
        fb_set(&mut bs, 3);
        fb_clear(&mut bs, 3);
        assert!(!fb_test(&bs, 3) /* cleared */);
    }

    #[test]
    fn test_count_ones() {
        let mut bs = new_fixed_bitset(64);
        fb_set(&mut bs, 0);
        fb_set(&mut bs, 1);
        fb_set(&mut bs, 63);
        assert_eq!(fb_count_ones(&bs), 3 /* three bits set */);
    }

    #[test]
    fn test_toggle() {
        let mut bs = new_fixed_bitset(64);
        bs.toggle(5);
        assert!(bs.test(5));
        bs.toggle(5);
        assert!(!bs.test(5) /* toggled back */);
    }

    #[test]
    fn test_set_all() {
        let mut bs = new_fixed_bitset(10);
        bs.set_all();
        assert_eq!(fb_count_ones(&bs), 10 /* all 10 bits set */);
    }

    #[test]
    fn test_clear_all() {
        let mut bs = new_fixed_bitset(32);
        bs.set_all();
        bs.clear_all();
        assert_eq!(fb_count_ones(&bs), 0 /* cleared */);
    }

    #[test]
    fn test_capacity() {
        let bs = new_fixed_bitset(200);
        assert_eq!(fb_capacity(&bs), 200 /* correct capacity */);
    }

    #[test]
    fn test_cross_word_boundary() {
        let mut bs = new_fixed_bitset(128);
        fb_set(&mut bs, 63);
        fb_set(&mut bs, 64);
        assert!(fb_test(&bs, 63) /* word boundary */);
        assert!(fb_test(&bs, 64));
    }

    #[test]
    fn test_large_index() {
        let mut bs = new_fixed_bitset(1000);
        fb_set(&mut bs, 999);
        assert!(fb_test(&bs, 999) /* last bit */);
    }
}
