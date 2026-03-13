// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Lightweight CRC-32 (ISO 3309 / ITU-T V.42) hasher without external deps.

#[allow(dead_code)]
const CRC32_TABLE: [u32; 256] = {
    let mut table = [0u32; 256];
    let mut i = 0u32;
    while i < 256 {
        let mut crc = i;
        let mut j = 0;
        while j < 8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
            j += 1;
        }
        table[i as usize] = crc;
        i += 1;
    }
    table
};

#[allow(dead_code)]
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        let idx = ((crc ^ byte as u32) & 0xFF) as usize;
        crc = (crc >> 8) ^ CRC32_TABLE[idx];
    }
    !crc
}

#[allow(dead_code)]
pub fn crc32_update(crc: u32, data: &[u8]) -> u32 {
    let mut c = !crc;
    for &byte in data {
        let idx = ((c ^ byte as u32) & 0xFF) as usize;
        c = (c >> 8) ^ CRC32_TABLE[idx];
    }
    !c
}

#[allow(dead_code)]
pub fn crc32_combine(crc_a: u32, crc_b: u32) -> u32 {
    crc_a ^ crc_b
}

#[allow(dead_code)]
pub fn crc32_hex(crc: u32) -> String {
    format!("{crc:08x}")
}

#[allow(dead_code)]
pub fn crc32_str(s: &str) -> u32 {
    crc32(s.as_bytes())
}

#[allow(dead_code)]
pub fn crc32_verify(data: &[u8], expected: u32) -> bool {
    crc32(data) == expected
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Crc32Hasher {
    state: u32,
}

#[allow(dead_code)]
impl Crc32Hasher {
    pub fn new() -> Self {
        Self { state: 0xFFFF_FFFF }
    }

    pub fn update(&mut self, data: &[u8]) {
        for &byte in data {
            let idx = ((self.state ^ byte as u32) & 0xFF) as usize;
            self.state = (self.state >> 8) ^ CRC32_TABLE[idx];
        }
    }

    pub fn finalize(&self) -> u32 {
        !self.state
    }

    pub fn reset(&mut self) {
        self.state = 0xFFFF_FFFF;
    }
}

impl Default for Crc32Hasher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        assert_eq!(crc32(&[]), 0x0000_0000);
    }

    #[test]
    fn test_known_value() {
        // CRC-32 of "123456789" is 0xCBF43926
        let data = b"123456789";
        assert_eq!(crc32(data), 0xCBF4_3926);
    }

    #[test]
    fn test_crc32_str() {
        assert_eq!(crc32_str("123456789"), 0xCBF4_3926);
    }

    #[test]
    fn test_hex() {
        assert_eq!(crc32_hex(0xCBF4_3926), "cbf43926");
    }

    #[test]
    fn test_verify() {
        let data = b"hello";
        let c = crc32(data);
        assert!(crc32_verify(data, c));
        assert!(!crc32_verify(data, c.wrapping_add(1)));
    }

    #[test]
    fn test_streaming_hasher() {
        let mut h = Crc32Hasher::new();
        h.update(b"1234");
        h.update(b"56789");
        assert_eq!(h.finalize(), 0xCBF4_3926);
    }

    #[test]
    fn test_hasher_reset() {
        let mut h = Crc32Hasher::new();
        h.update(b"abc");
        h.reset();
        h.update(b"123456789");
        assert_eq!(h.finalize(), 0xCBF4_3926);
    }

    #[test]
    fn test_single_byte() {
        let c = crc32(&[0x00]);
        assert_ne!(c, 0);
    }

    #[test]
    fn test_deterministic() {
        let a = crc32(b"test data");
        let b = crc32(b"test data");
        assert_eq!(a, b);
    }

    #[test]
    fn test_different_data_different_hash() {
        assert_ne!(crc32(b"aaa"), crc32(b"bbb"));
    }
}
