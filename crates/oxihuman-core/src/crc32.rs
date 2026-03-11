// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! CRC32 checksum (table-driven implementation).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Crc32Config {
    pub polynomial: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Crc32State {
    pub value: u32,
}

// Standard CRC32 polynomial (IEEE 802.3)
const CRC32_POLY: u32 = 0xEDB88320;

fn make_table(poly: u32) -> [u32; 256] {
    let mut table = [0u32; 256];
    for (i, slot) in table.iter_mut().enumerate() {
        let mut crc = i as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ poly;
            } else {
                crc >>= 1;
            }
        }
        *slot = crc;
    }
    table
}

#[allow(dead_code)]
pub fn default_crc32_config() -> Crc32Config {
    Crc32Config { polynomial: CRC32_POLY }
}

#[allow(dead_code)]
pub fn new_crc32_state() -> Crc32State {
    Crc32State { value: 0xFFFFFFFF }
}

#[allow(dead_code)]
pub fn crc32_update(state: &mut Crc32State, data: &[u8]) {
    let table = make_table(CRC32_POLY);
    for &b in data {
        let idx = ((state.value ^ b as u32) & 0xff) as usize;
        state.value = (state.value >> 8) ^ table[idx];
    }
}

#[allow(dead_code)]
pub fn crc32_finalize(state: &Crc32State) -> u32 {
    state.value ^ 0xFFFFFFFF
}

#[allow(dead_code)]
pub fn crc32_of(data: &[u8]) -> u32 {
    let mut s = new_crc32_state();
    crc32_update(&mut s, data);
    crc32_finalize(&s)
}

#[allow(dead_code)]
pub fn crc32_of_str(s: &str) -> u32 {
    crc32_of(s.as_bytes())
}

#[allow(dead_code)]
pub fn crc32_combine(a: u32, _b: u32) -> u32 {
    // Simple XOR combination (not the true CRC combine; serves as a stub)
    a ^ _b
}

#[allow(dead_code)]
pub fn crc32_reset(state: &mut Crc32State) {
    state.value = 0xFFFFFFFF;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_crc32_config();
        assert_eq!(cfg.polynomial, CRC32_POLY);
    }

    #[test]
    fn test_crc32_empty() {
        assert_eq!(crc32_of(b""), 0x00000000);
    }

    #[test]
    fn test_crc32_hello() {
        // Known CRC32 of "hello" = 0x3610A686
        let result = crc32_of(b"hello");
        assert_eq!(result, 0x3610A686);
    }

    #[test]
    fn test_crc32_consistency() {
        let r1 = crc32_of(b"test data");
        let r2 = crc32_of(b"test data");
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_crc32_different_inputs() {
        let r1 = crc32_of(b"foo");
        let r2 = crc32_of(b"bar");
        assert_ne!(r1, r2);
    }

    #[test]
    fn test_crc32_incremental() {
        let mut s = new_crc32_state();
        crc32_update(&mut s, b"hel");
        crc32_update(&mut s, b"lo");
        let incremental = crc32_finalize(&s);
        let full = crc32_of(b"hello");
        assert_eq!(incremental, full);
    }

    #[test]
    fn test_crc32_of_str() {
        let r1 = crc32_of_str("hello");
        let r2 = crc32_of(b"hello");
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_crc32_reset() {
        let mut s = new_crc32_state();
        crc32_update(&mut s, b"something");
        crc32_reset(&mut s);
        assert_eq!(s.value, 0xFFFFFFFF);
    }

    #[test]
    fn test_crc32_combine_self_inverse() {
        // XOR combine: a ^ a = 0
        let r = crc32_combine(0xDEADBEEF, 0xDEADBEEF);
        assert_eq!(r, 0);
    }
}
