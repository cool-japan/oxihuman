// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub fn crc8_update(crc: u8, byte: u8) -> u8 {
    let mut c = crc ^ byte;
    for _ in 0..8 {
        if c & 0x80 != 0 {
            c = (c << 1) ^ 0x07;
        } else {
            c <<= 1;
        }
    }
    c
}

pub fn crc8(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, &b| crc8_update(acc, b))
}

pub fn crc8_check(data: &[u8], expected: u8) -> bool {
    crc8(data) == expected
}

pub fn crc16_update(crc: u16, byte: u8) -> u16 {
    let mut c = crc ^ ((byte as u16) << 8);
    for _ in 0..8 {
        if c & 0x8000 != 0 {
            c = (c << 1) ^ 0x8005;
        } else {
            c <<= 1;
        }
    }
    c
}

pub fn crc16(data: &[u8]) -> u16 {
    data.iter().fold(0u16, |acc, &b| crc16_update(acc, b))
}

pub fn crc16_check(data: &[u8], expected: u16) -> bool {
    crc16(data) == expected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc8_empty_is_zero() {
        /* CRC-8 of empty data is zero */
        assert_eq!(crc8(b""), 0);
    }

    #[test]
    fn crc8_known() {
        /* CRC-8 of 0xFF is deterministic */
        let v = crc8(&[0xFF]);
        assert_ne!(v, 0); /* not zero for non-zero input */
    }

    #[test]
    fn crc8_check_pass() {
        /* check passes when expected matches */
        let data = b"hello";
        assert!(crc8_check(data, crc8(data)));
    }

    #[test]
    fn crc8_check_fail() {
        /* check fails with wrong expected */
        assert!(!crc8_check(b"hello", 0x00));
    }

    #[test]
    fn crc16_empty_is_zero() {
        /* CRC-16 of empty data is zero */
        assert_eq!(crc16(b""), 0);
    }

    #[test]
    fn crc16_check_pass() {
        /* check passes when expected matches */
        let data = b"test";
        assert!(crc16_check(data, crc16(data)));
    }

    #[test]
    fn crc16_update_consistency() {
        /* byte-by-byte update matches bulk */
        let data = b"abcd";
        let bulk = crc16(data);
        let incr = data.iter().fold(0u16, |acc, &b| crc16_update(acc, b));
        assert_eq!(bulk, incr);
    }
}
