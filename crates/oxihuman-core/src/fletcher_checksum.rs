// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub fn fletcher16(data: &[u8]) -> u16 {
    let mut sum1 = 0u32;
    let mut sum2 = 0u32;
    for &b in data {
        sum1 = (sum1 + b as u32) % 255;
        sum2 = (sum2 + sum1) % 255;
    }
    ((sum2 << 8) | sum1) as u16
}

pub fn fletcher32(data: &[u16]) -> u32 {
    let mut sum1 = 0u32;
    let mut sum2 = 0u32;
    for &w in data {
        sum1 = (sum1 + w as u32) % 65535;
        sum2 = (sum2 + sum1) % 65535;
    }
    (sum2 << 16) | sum1
}

pub fn fletcher16_check(data: &[u8], expected: u16) -> bool {
    fletcher16(data) == expected
}

pub fn fletcher32_check(data: &[u16], expected: u32) -> bool {
    fletcher32(data) == expected
}

pub fn fletcher16_combine(sum1: u8, sum2: u8) -> u16 {
    ((sum2 as u16) << 8) | (sum1 as u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fletcher16_empty() {
        /* empty input gives zero */
        assert_eq!(fletcher16(b""), 0);
    }

    #[test]
    fn fletcher16_known() {
        /* "abcd" produces a non-zero checksum */
        assert_ne!(fletcher16(b"abcd"), 0);
    }

    #[test]
    fn fletcher16_check_pass() {
        /* check passes with correct checksum */
        let data = b"hello";
        assert!(fletcher16_check(data, fletcher16(data)));
    }

    #[test]
    fn fletcher32_empty() {
        /* empty input gives zero */
        assert_eq!(fletcher32(&[]), 0);
    }

    #[test]
    fn fletcher32_check_pass() {
        /* check passes with correct checksum */
        let data: &[u16] = &[1, 2, 3];
        assert!(fletcher32_check(data, fletcher32(data)));
    }

    #[test]
    fn combine_matches_formula() {
        /* combine function matches manual formula */
        assert_eq!(fletcher16_combine(0x12, 0x34), 0x3412);
    }
}
