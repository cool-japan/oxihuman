// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A simple non-cryptographic hash digest for data fingerprinting.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DigestHash {
    value: u64,
}

#[allow(dead_code)]
impl DigestHash {
    pub const SEED: u64 = 0xcbf2_9ce4_8422_2325;

    pub fn new() -> Self {
        Self { value: Self::SEED }
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        let mut h = Self::new();
        h.update(data);
        h
    }

    pub fn from_text(s: &str) -> Self {
        Self::from_bytes(s.as_bytes())
    }

    pub fn update(&mut self, data: &[u8]) {
        for &b in data {
            self.value ^= b as u64;
            self.value = self.value.wrapping_mul(0x0100_0000_01b3);
        }
    }

    pub fn update_u32(&mut self, val: u32) {
        self.update(&val.to_le_bytes());
    }

    pub fn update_f32(&mut self, val: f32) {
        self.update(&val.to_le_bytes());
    }

    pub fn finish(&self) -> u64 {
        self.value
    }

    pub fn finish_u32(&self) -> u32 {
        (self.value ^ (self.value >> 32)) as u32
    }

    pub fn combine(&self, other: &DigestHash) -> DigestHash {
        DigestHash {
            value: self.value ^ other.value.wrapping_mul(0x9e37_79b9_7f4a_7c15),
        }
    }

    pub fn is_zero(&self) -> bool {
        self.value == 0
    }

    pub fn to_hex(&self) -> String {
        format!("{:016x}", self.value)
    }
}

impl Default for DigestHash {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let h = DigestHash::new();
        assert_eq!(h.finish(), DigestHash::SEED);
    }

    #[test]
    fn test_from_bytes() {
        let h = DigestHash::from_bytes(b"hello");
        assert_ne!(h.finish(), DigestHash::SEED);
    }

    #[test]
    fn test_deterministic() {
        let a = DigestHash::from_text("test");
        let b = DigestHash::from_text("test");
        assert_eq!(a, b);
    }

    #[test]
    fn test_different_inputs() {
        let a = DigestHash::from_text("alpha");
        let b = DigestHash::from_text("beta");
        assert_ne!(a, b);
    }

    #[test]
    fn test_update_u32() {
        let mut h = DigestHash::new();
        h.update_u32(42);
        assert_ne!(h.finish(), DigestHash::SEED);
    }

    #[test]
    fn test_update_f32() {
        let mut h = DigestHash::new();
        h.update_f32(1.5);
        assert_ne!(h.finish(), DigestHash::SEED);
    }

    #[test]
    fn test_finish_u32() {
        let h = DigestHash::from_text("data");
        let _v = h.finish_u32();
        // just ensure no panic
    }

    #[test]
    fn test_combine() {
        let a = DigestHash::from_text("a");
        let b = DigestHash::from_text("b");
        let c = a.combine(&b);
        assert_ne!(c, a);
        assert_ne!(c, b);
    }

    #[test]
    fn test_to_hex() {
        let h = DigestHash::new();
        let hex = h.to_hex();
        assert_eq!(hex.len(), 16);
    }

    #[test]
    fn test_default() {
        let h = DigestHash::default();
        assert_eq!(h.finish(), DigestHash::SEED);
    }
}
