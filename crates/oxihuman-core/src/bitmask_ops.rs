// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Bit manipulation utilities: count, toggle, ranges.

#![allow(dead_code)]

/// Count the number of set bits (popcount) in a u64.
#[allow(dead_code)]
pub fn popcount(x: u64) -> u32 {
    x.count_ones()
}

/// Count trailing zeros (index of lowest set bit). Returns 64 if x is 0.
#[allow(dead_code)]
pub fn count_trailing_zeros(x: u64) -> u32 {
    if x == 0 {
        64
    } else {
        x.trailing_zeros()
    }
}

/// Count leading zeros.
#[allow(dead_code)]
pub fn count_leading_zeros(x: u64) -> u32 {
    x.leading_zeros()
}

/// Toggle a specific bit in a u64 mask.
#[allow(dead_code)]
pub fn toggle_bit(mask: u64, bit: u32) -> u64 {
    mask ^ (1u64 << (bit & 63))
}

/// Set a specific bit.
#[allow(dead_code)]
pub fn set_bit(mask: u64, bit: u32) -> u64 {
    mask | (1u64 << (bit & 63))
}

/// Clear a specific bit.
#[allow(dead_code)]
pub fn clear_bit(mask: u64, bit: u32) -> u64 {
    mask & !(1u64 << (bit & 63))
}

/// Check if a specific bit is set.
#[allow(dead_code)]
pub fn is_bit_set(mask: u64, bit: u32) -> bool {
    mask & (1u64 << (bit & 63)) != 0
}

/// Create a bitmask with bits set in the range `[lo, hi)`.
#[allow(dead_code)]
pub fn range_mask(lo: u32, hi: u32) -> u64 {
    if lo >= 64 || hi == 0 || lo >= hi {
        return 0;
    }
    let hi = hi.min(64);
    let lo = lo.min(63);
    if hi >= 64 {
        !0u64 << lo
    } else {
        ((1u64 << hi) - 1) & (!0u64 << lo)
    }
}

/// Extract bits in the range `[lo, hi)` (zero-indexed from lo).
#[allow(dead_code)]
pub fn extract_range(mask: u64, lo: u32, hi: u32) -> u64 {
    (mask & range_mask(lo, hi)) >> lo.min(63)
}

/// Rotate bits left by `n` within a 64-bit word.
#[allow(dead_code)]
pub fn rotate_left(mask: u64, n: u32) -> u64 {
    mask.rotate_left(n)
}

/// Rotate bits right by `n` within a 64-bit word.
#[allow(dead_code)]
pub fn rotate_right(mask: u64, n: u32) -> u64 {
    mask.rotate_right(n)
}

/// Return the index of the highest set bit, or None if mask is 0.
#[allow(dead_code)]
pub fn highest_bit(mask: u64) -> Option<u32> {
    if mask == 0 {
        None
    } else {
        Some(63 - mask.leading_zeros())
    }
}

/// Return the index of the lowest set bit, or None if mask is 0.
#[allow(dead_code)]
pub fn lowest_bit(mask: u64) -> Option<u32> {
    if mask == 0 {
        None
    } else {
        Some(mask.trailing_zeros())
    }
}

/// Collect all set bit indices into a Vec.
#[allow(dead_code)]
pub fn set_bit_indices(mut mask: u64) -> Vec<u32> {
    let mut out = Vec::new();
    while mask != 0 {
        let bit = mask.trailing_zeros();
        out.push(bit);
        mask &= mask - 1;
    }
    out
}

/// Parity: true if the number of set bits is odd.
#[allow(dead_code)]
pub fn parity(x: u64) -> bool {
    !x.count_ones().is_multiple_of(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popcount_basic() {
        assert_eq!(popcount(0b1011), 3);
        assert_eq!(popcount(0), 0);
        assert_eq!(popcount(u64::MAX), 64);
    }

    #[test]
    fn toggle_bit_roundtrip() {
        let m = 0u64;
        let m2 = toggle_bit(m, 5);
        assert!(is_bit_set(m2, 5));
        let m3 = toggle_bit(m2, 5);
        assert!(!is_bit_set(m3, 5));
    }

    #[test]
    fn set_and_clear_bit() {
        let m = set_bit(0, 10);
        assert!(is_bit_set(m, 10));
        let m2 = clear_bit(m, 10);
        assert!(!is_bit_set(m2, 10));
    }

    #[test]
    fn range_mask_basic() {
        let m = range_mask(0, 4);
        assert_eq!(m, 0b1111);
    }

    #[test]
    fn range_mask_empty() {
        assert_eq!(range_mask(5, 5), 0);
        assert_eq!(range_mask(10, 5), 0);
    }

    #[test]
    fn highest_lowest_bit() {
        let m = 0b10100u64;
        assert_eq!(highest_bit(m), Some(4));
        assert_eq!(lowest_bit(m), Some(2));
    }

    #[test]
    fn set_bit_indices_correct() {
        let m = 0b1011u64;
        let indices = set_bit_indices(m);
        assert_eq!(indices, vec![0, 1, 3]);
    }

    #[test]
    fn rotate_left_basic() {
        let m = 1u64;
        assert_eq!(rotate_left(m, 4), 16);
    }

    #[test]
    fn parity_even_odd() {
        assert!(!parity(0b1100));
        assert!(parity(0b1110));
    }

    #[test]
    fn count_trailing_zeros_zero() {
        assert_eq!(count_trailing_zeros(0), 64);
    }
}
