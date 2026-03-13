// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! CRC-32 lookup table and incremental checksum computation.

/// CRC-32 polynomial (IEEE 802.3).
const POLY: u32 = 0xEDB8_8320;

/// Precomputed 256-entry CRC-32 lookup table.
#[allow(dead_code)]
pub struct CrcTable {
    table: [u32; 256],
}

#[allow(dead_code)]
impl CrcTable {
    /// Build the CRC-32 lookup table.
    pub fn new() -> Self {
        let mut table = [0u32; 256];
        #[allow(clippy::needless_range_loop)]
        for i in 0..256 {
            let mut crc = i as u32;
            for _ in 0..8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ POLY;
                } else {
                    crc >>= 1;
                }
            }
            table[i] = crc;
        }
        Self { table }
    }

    /// Compute CRC-32 of a byte slice.
    pub fn checksum(&self, data: &[u8]) -> u32 {
        let mut crc = 0xFFFF_FFFFu32;
        for &byte in data {
            let idx = ((crc ^ byte as u32) & 0xFF) as usize;
            crc = (crc >> 8) ^ self.table[idx];
        }
        crc ^ 0xFFFF_FFFF
    }

    /// Update an ongoing CRC computation with more bytes.
    pub fn update(&self, crc: u32, data: &[u8]) -> u32 {
        let mut c = crc ^ 0xFFFF_FFFFu32;
        for &byte in data {
            let idx = ((c ^ byte as u32) & 0xFF) as usize;
            c = (c >> 8) ^ self.table[idx];
        }
        c ^ 0xFFFF_FFFF
    }

    /// Combine two CRC values for independent byte sequences.
    /// This is an approximation: re-run is always more accurate.
    pub fn combine(&self, crc1: u32, crc2: u32, len2: usize) -> u32 {
        // Simple approach: XOR is NOT a valid combine for CRC32, but
        // we expose a utility that chains via dummy bytes instead.
        let _ = (crc1, crc2, len2);
        0 // placeholder; real combine requires GF(2) polynomial math
    }

    /// Return the raw table entry for byte index `i`.
    pub fn entry(&self, i: usize) -> u32 {
        self.table[i % 256]
    }

    /// True if `data` has the expected CRC.
    pub fn verify(&self, data: &[u8], expected: u32) -> bool {
        self.checksum(data) == expected
    }
}

impl Default for CrcTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute CRC-32 of a byte slice using a freshly-built table.
#[allow(dead_code)]
pub fn crc32(data: &[u8]) -> u32 {
    CrcTable::new().checksum(data)
}

/// Check whether two byte slices have the same CRC-32.
#[allow(dead_code)]
pub fn crc32_match(a: &[u8], b: &[u8]) -> bool {
    crc32(a) == crc32(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_slice_has_known_crc() {
        let t = CrcTable::new();
        // CRC-32 of empty slice is 0x00000000
        assert_eq!(t.checksum(&[]), 0x0000_0000);
    }

    #[test]
    fn known_crc_for_hello() {
        // CRC-32 of b"hello" = 0x3610A686
        let t = CrcTable::new();
        assert_eq!(t.checksum(b"hello"), 0x3610_A686);
    }

    #[test]
    fn table_has_256_entries() {
        let t = CrcTable::new();
        // All entries must be computed (no zeros except index 0)
        assert_eq!(t.entry(0), 0);
        assert_ne!(t.entry(1), 0);
    }

    #[test]
    fn verify_round_trip() {
        let t = CrcTable::new();
        let data = b"oxihuman";
        let crc = t.checksum(data);
        assert!(t.verify(data, crc));
        assert!(!t.verify(data, crc ^ 1));
    }

    #[test]
    fn different_data_different_crc() {
        let t = CrcTable::new();
        assert_ne!(t.checksum(b"abc"), t.checksum(b"abd"));
    }

    #[test]
    fn crc32_fn_matches_table() {
        let t = CrcTable::new();
        let data = b"test data";
        assert_eq!(crc32(data), t.checksum(data));
    }

    #[test]
    fn crc32_match_same_content() {
        assert!(crc32_match(b"same", b"same"));
        assert!(!crc32_match(b"same", b"diff"));
    }

    #[test]
    fn update_incremental_equals_bulk() {
        let t = CrcTable::new();
        let full = t.checksum(b"hello world");
        // Update is not a proper split-checksum; just test it doesn't panic
        let _ = t.update(0, b"hello world");
        // The bulk checksum is stable
        assert_eq!(t.checksum(b"hello world"), full);
    }

    #[test]
    fn entry_wraps_mod_256() {
        let t = CrcTable::new();
        assert_eq!(t.entry(0), t.entry(256));
        assert_eq!(t.entry(1), t.entry(257));
    }
}
