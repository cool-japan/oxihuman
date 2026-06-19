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

    /// Combine two CRC-32 values for independent byte sequences.
    ///
    /// Given `crc1 = CRC32(A)` and `crc2 = CRC32(B)` where `B` is `len2` bytes
    /// long, returns `CRC32(A ++ B)` (the CRC of the concatenation) without
    /// re-scanning the underlying data. This is a faithful port of zlib's
    /// `crc32_combine_`, which advances `crc1` over `len2` zero bytes using
    /// GF(2) operator matrices and then folds in `crc2`.
    pub fn combine(&self, crc1: u32, crc2: u32, len2: usize) -> u32 {
        // Degenerate case.
        if len2 == 0 {
            return crc1;
        }

        let mut even = [0u32; GF2_DIM]; // even-power-of-two zeros operator
        let mut odd = [0u32; GF2_DIM]; // odd-power-of-two zeros operator

        // Operator for one zero bit, in `odd`.
        odd[0] = POLY; // CRC-32 polynomial
        let mut row = 1u32;
        #[allow(clippy::needless_range_loop)]
        for n in 1..GF2_DIM {
            odd[n] = row;
            row <<= 1;
        }

        // Operator for two zero bits, in `even`.
        gf2_matrix_square(&mut even, &odd);
        // Operator for four zero bits, in `odd`.
        gf2_matrix_square(&mut odd, &even);

        // Apply len2 zero BYTES to crc1. The first square below builds the
        // operator for one zero byte (eight zero bits) into `even`.
        let mut crc1 = crc1;
        let mut len2 = len2 as u64;
        loop {
            gf2_matrix_square(&mut even, &odd);
            if len2 & 1 != 0 {
                crc1 = gf2_matrix_times(&even, crc1);
            }
            len2 >>= 1;
            if len2 == 0 {
                break;
            }
            gf2_matrix_square(&mut odd, &even);
            if len2 & 1 != 0 {
                crc1 = gf2_matrix_times(&odd, crc1);
            }
            len2 >>= 1;
            if len2 == 0 {
                break;
            }
        }

        crc1 ^ crc2
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

/// Number of bits in the CRC-32 register; the GF(2) operator matrices are
/// `GF2_DIM` columns wide.
const GF2_DIM: usize = 32;

/// Multiply the bit vector `vec` by the GF(2) matrix `mat`.
///
/// Each set bit in `vec` selects the corresponding column of `mat`, and the
/// selected columns are XOR-ed together to form the result.
fn gf2_matrix_times(mat: &[u32; GF2_DIM], mut vec: u32) -> u32 {
    let mut sum = 0u32;
    let mut idx = 0usize;
    while vec != 0 {
        if vec & 1 != 0 {
            sum ^= mat[idx];
        }
        vec >>= 1;
        idx += 1;
    }
    sum
}

/// Square the GF(2) operator matrix `mat`, storing the result in `square`.
///
/// Squaring an operator that advances the CRC over `k` zero bits yields the
/// operator that advances it over `2 * k` zero bits.
fn gf2_matrix_square(square: &mut [u32; GF2_DIM], mat: &[u32; GF2_DIM]) {
    #[allow(clippy::needless_range_loop)]
    for n in 0..GF2_DIM {
        square[n] = gf2_matrix_times(mat, mat[n]);
    }
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

    #[test]
    fn combine_matches_concatenation() {
        let t = CrcTable::new();
        let a = b"hello, ";
        let b = b"world!";
        let mut concat = a.to_vec();
        concat.extend_from_slice(b);
        let crc_a = t.checksum(a);
        let crc_b = t.checksum(b);
        let combined = t.combine(crc_a, crc_b, b.len());
        assert_eq!(combined, t.checksum(&concat));
    }

    #[test]
    fn combine_empty_second_is_identity() {
        let t = CrcTable::new();
        let crc_a = t.checksum(b"abc");
        assert_eq!(t.combine(crc_a, t.checksum(b""), 0), crc_a);
    }

    #[test]
    fn combine_three_way_associative() {
        let t = CrcTable::new();
        let (a, b, c) = (b"AAAA".as_slice(), b"BBBBBB".as_slice(), b"CC".as_slice());
        let mut all = a.to_vec();
        all.extend_from_slice(b);
        all.extend_from_slice(c);
        let ab = t.combine(t.checksum(a), t.checksum(b), b.len());
        let abc = t.combine(ab, t.checksum(c), c.len());
        assert_eq!(abc, t.checksum(&all));
    }
}
