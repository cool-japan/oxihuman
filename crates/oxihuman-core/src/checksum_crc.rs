// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! CRC-8 and CRC-16 checksum computation.

#![allow(dead_code)]

// CRC-8 with polynomial 0x07 (CRC-8/SMBUS).
const CRC8_POLY: u8 = 0x07;

#[allow(clippy::needless_range_loop)]
fn crc8_table() -> [u8; 256] {
    let mut table = [0u8; 256];
    for i in 0usize..256 {
        let mut crc = i as u8;
        for _ in 0..8 {
            if crc & 0x80 != 0 {
                crc = (crc << 1) ^ CRC8_POLY;
            } else {
                crc <<= 1;
            }
        }
        table[i] = crc;
    }
    table
}

/// Compute CRC-8 checksum of a byte slice.
#[allow(dead_code)]
pub fn crc8(data: &[u8]) -> u8 {
    let table = crc8_table();
    let mut crc = 0u8;
    for &byte in data {
        let idx = (crc ^ byte) as usize;
        crc = table[idx];
    }
    crc
}

// CRC-16/CCITT with polynomial 0x1021, init 0xFFFF.
const CRC16_POLY: u16 = 0x1021;
const CRC16_INIT: u16 = 0xFFFF;

#[allow(clippy::needless_range_loop)]
fn crc16_table() -> [u16; 256] {
    let mut table = [0u16; 256];
    for i in 0usize..256 {
        let mut crc = (i as u16) << 8;
        for _ in 0..8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ CRC16_POLY;
            } else {
                crc <<= 1;
            }
        }
        table[i] = crc;
    }
    table
}

/// Compute CRC-16/CCITT checksum of a byte slice.
#[allow(dead_code)]
pub fn crc16(data: &[u8]) -> u16 {
    let table = crc16_table();
    let mut crc = CRC16_INIT;
    for &byte in data {
        let idx = ((crc >> 8) ^ byte as u16) as usize;
        crc = (crc << 8) ^ table[idx];
    }
    crc
}

/// Verify CRC-8: recompute and compare.
#[allow(dead_code)]
pub fn crc8_verify(data: &[u8], expected: u8) -> bool {
    crc8(data) == expected
}

/// Verify CRC-16: recompute and compare.
#[allow(dead_code)]
pub fn crc16_verify(data: &[u8], expected: u16) -> bool {
    crc16(data) == expected
}

/// Simple additive checksum (byte sum mod 256).
#[allow(dead_code)]
pub fn additive_checksum(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b))
}

/// XOR checksum of all bytes.
#[allow(dead_code)]
pub fn xor_checksum(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, &b| acc ^ b)
}

/// Compute Fletcher-16 checksum.
#[allow(dead_code)]
pub fn fletcher16(data: &[u8]) -> u16 {
    let mut sum1 = 0u32;
    let mut sum2 = 0u32;
    for &b in data {
        sum1 = (sum1 + b as u32) % 255;
        sum2 = (sum2 + sum1) % 255;
    }
    ((sum2 << 8) | sum1) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc8_empty() {
        assert_eq!(crc8(&[]), 0);
    }

    #[test]
    fn crc8_deterministic() {
        let a = crc8(b"hello");
        let b = crc8(b"hello");
        assert_eq!(a, b);
    }

    #[test]
    fn crc8_differs_for_different_data() {
        let a = crc8(b"hello");
        let b = crc8(b"world");
        assert_ne!(a, b);
    }

    #[test]
    fn crc8_verify_correct() {
        let data = b"test data";
        let crc = crc8(data);
        assert!(crc8_verify(data, crc));
    }

    #[test]
    fn crc16_deterministic() {
        let a = crc16(b"hello world");
        let b = crc16(b"hello world");
        assert_eq!(a, b);
    }

    #[test]
    fn crc16_differs_for_different_data() {
        let a = crc16(b"abc");
        let b = crc16(b"xyz");
        assert_ne!(a, b);
    }

    #[test]
    fn crc16_verify_correct() {
        let data = b"verify me";
        let crc = crc16(data);
        assert!(crc16_verify(data, crc));
    }

    #[test]
    fn additive_checksum_basic() {
        let sum = additive_checksum(&[1, 2, 3, 4]);
        assert_eq!(sum, 10);
    }

    #[test]
    fn xor_checksum_basic() {
        let xor = xor_checksum(&[0xFF, 0xFF]);
        assert_eq!(xor, 0);
    }

    #[test]
    fn fletcher16_deterministic() {
        let a = fletcher16(b"abcde");
        let b = fletcher16(b"abcde");
        assert_eq!(a, b);
    }
}
