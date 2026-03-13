// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub fn to_gray(n: u32) -> u32 {
    n ^ (n >> 1)
}

pub fn from_gray(mut g: u32) -> u32 {
    let mut mask = g >> 1;
    while mask != 0 {
        g ^= mask;
        mask >>= 1;
    }
    g
}

pub fn gray_distance(a: u32, b: u32) -> u32 {
    (a ^ b).count_ones()
}

pub fn gray_next(g: u32) -> u32 {
    to_gray(from_gray(g).wrapping_add(1))
}

pub fn gray_prev(g: u32) -> u32 {
    to_gray(from_gray(g).wrapping_sub(1))
}

pub fn gray_bits(g: u32) -> u32 {
    g.count_ones()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gray_roundtrip() {
        /* encoding then decoding returns original value */
        for n in 0u32..=15 {
            assert_eq!(from_gray(to_gray(n)), n);
        }
    }

    #[test]
    fn consecutive_gray_differ_by_one_bit() {
        /* adjacent Gray codes differ by exactly 1 bit */
        for n in 0u32..=14 {
            let diff = to_gray(n) ^ to_gray(n + 1);
            assert_eq!(diff.count_ones(), 1);
        }
    }

    #[test]
    fn gray_distance_self() {
        /* distance from a code to itself is zero */
        assert_eq!(gray_distance(0b101, 0b101), 0);
    }

    #[test]
    fn gray_next_advances() {
        /* gray_next increments the binary value by 1 */
        let g = to_gray(5);
        assert_eq!(gray_next(g), to_gray(6));
    }

    #[test]
    fn gray_prev_decrements() {
        /* gray_prev decrements the binary value by 1 */
        let g = to_gray(5);
        assert_eq!(gray_prev(g), to_gray(4));
    }

    #[test]
    fn gray_bits_count() {
        /* gray_bits counts set bits */
        assert_eq!(gray_bits(0b1010), 2);
    }

    #[test]
    fn to_gray_zero() {
        /* zero maps to zero */
        assert_eq!(to_gray(0), 0);
    }
}
