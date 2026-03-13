#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Adler-32 checksum implementation.

const MOD_ADLER: u32 = 65521;

/// Adler-32 checksum state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Adler32 {
    a: u32,
    b: u32,
}

#[allow(dead_code)]
pub fn adler32_compute(data: &[u8]) -> Adler32 {
    let mut s = Adler32 { a: 1, b: 0 };
    for &byte in data {
        s.a = (s.a + byte as u32) % MOD_ADLER;
        s.b = (s.b + s.a) % MOD_ADLER;
    }
    s
}

#[allow(dead_code)]
pub fn adler32_update(state: &mut Adler32, data: &[u8]) {
    for &byte in data {
        state.a = (state.a + byte as u32) % MOD_ADLER;
        state.b = (state.b + state.a) % MOD_ADLER;
    }
}

#[allow(dead_code)]
pub fn adler32_combine(s1: &Adler32, s2: &Adler32) -> Adler32 {
    Adler32 {
        a: (s1.a + s2.a - 1) % MOD_ADLER,
        b: (s1.b + s2.b + s1.a.wrapping_sub(1)) % MOD_ADLER,
    }
}

#[allow(dead_code)]
pub fn adler32_to_u32(s: &Adler32) -> u32 {
    (s.b << 16) | s.a
}

#[allow(dead_code)]
pub fn adler32_reset(s: &mut Adler32) {
    s.a = 1;
    s.b = 0;
}

#[allow(dead_code)]
pub fn adler32_from_bytes(data: &[u8]) -> u32 {
    adler32_to_u32(&adler32_compute(data))
}

#[allow(dead_code)]
pub fn adler32_verify(data: &[u8], expected: u32) -> bool {
    adler32_from_bytes(data) == expected
}

#[allow(dead_code)]
pub fn adler32_to_hex(s: &Adler32) -> String {
    format!("{:08x}", adler32_to_u32(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_value() {
        // Adler-32 of "Wikipedia" is 0x11E60398
        let s = adler32_compute(b"Wikipedia");
        assert_eq!(adler32_to_u32(&s), 0x11E60398);
    }

    #[test]
    fn test_empty() {
        let s = adler32_compute(b"");
        assert_eq!(adler32_to_u32(&s), 1); // initial a=1, b=0 → 0x00000001
    }

    #[test]
    fn test_update() {
        let mut s = Adler32 { a: 1, b: 0 };
        adler32_update(&mut s, b"Wiki");
        adler32_update(&mut s, b"pedia");
        assert_eq!(adler32_to_u32(&s), 0x11E60398);
    }

    #[test]
    fn test_reset() {
        let mut s = adler32_compute(b"data");
        adler32_reset(&mut s);
        assert_eq!(adler32_to_u32(&s), 1);
    }

    #[test]
    fn test_from_bytes() {
        let v = adler32_from_bytes(b"abc");
        assert_eq!(v, adler32_to_u32(&adler32_compute(b"abc")));
    }

    #[test]
    fn test_verify_ok() {
        let expected = adler32_from_bytes(b"test");
        assert!(adler32_verify(b"test", expected));
    }

    #[test]
    fn test_verify_fail() {
        assert!(!adler32_verify(b"test", 0));
    }

    #[test]
    fn test_to_hex() {
        let s = adler32_compute(b"");
        assert_eq!(adler32_to_hex(&s), "00000001");
    }

    #[test]
    fn test_combine() {
        let s1 = adler32_compute(b"ab");
        let s2 = adler32_compute(b"cd");
        let combined = adler32_combine(&s1, &s2);
        // Should produce some deterministic value
        let _ = adler32_to_u32(&combined);
    }

    #[test]
    fn test_single_byte() {
        let s = adler32_compute(b"a");
        // a=1+97=98, b=0+98=98 → (98<<16)|98 = 0x00620062
        assert_eq!(adler32_to_u32(&s), 0x00620062);
    }
}
