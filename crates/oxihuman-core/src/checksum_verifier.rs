// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! File checksum verifier (CRC32 / SHA-256 / xxHash-64).

/// Checksum algorithm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChecksumAlgo {
    Crc32,
    Sha256,
    Xxhash64,
}

impl ChecksumAlgo {
    pub fn output_len(&self) -> usize {
        match self {
            ChecksumAlgo::Crc32 => 4,
            ChecksumAlgo::Sha256 => 32,
            ChecksumAlgo::Xxhash64 => 8,
        }
    }
}

/// A checksum value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checksum {
    pub algo: ChecksumAlgo,
    pub value: Vec<u8>,
}

impl Checksum {
    pub fn new(algo: ChecksumAlgo, value: Vec<u8>) -> Self {
        Checksum { algo, value }
    }

    pub fn hex(&self) -> String {
        self.value.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

/// CRC32 implementation using the standard IEEE 802.3 polynomial (0xEDB88320).
pub fn crc32_bytes(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

/// SHA-256 digest of `data`.
///
/// Delegates to the real SHA-256 implementation in `hashing_sha256`.
/// The name is kept as `sha256_stub` for backward compatibility with
/// existing callers and the public re-export in `_core_part3.rs`.
pub fn sha256_stub(data: &[u8]) -> [u8; 32] {
    crate::hashing_sha256::sha256_hash(data).0
}

/// Compute a checksum for `data` using the specified algorithm.
pub fn compute_checksum(algo: ChecksumAlgo, data: &[u8]) -> Checksum {
    let value = match &algo {
        ChecksumAlgo::Crc32 => crc32_bytes(data).to_le_bytes().to_vec(),
        ChecksumAlgo::Sha256 => sha256_stub(data).to_vec(),
        ChecksumAlgo::Xxhash64 => crate::hashing_xxhash::xxhash64(data, 0)
            .to_le_bytes()
            .to_vec(),
    };
    Checksum::new(algo, value)
}

/// Verify `data` against an expected checksum.
pub fn verify_checksum(data: &[u8], expected: &Checksum) -> bool {
    let actual = compute_checksum(expected.algo.clone(), data);
    actual.value == expected.value
}

/// Compute a checksum and compare it against a hex string.
pub fn verify_hex(data: &[u8], algo: ChecksumAlgo, hex: &str) -> bool {
    let computed = compute_checksum(algo, data);
    computed.hex() == hex
}

/// Build a checksum registry for a slice of named byte buffers.
pub fn checksum_map(items: &[(&str, &[u8])], algo: ChecksumAlgo) -> Vec<(String, Checksum)> {
    items
        .iter()
        .map(|(name, data)| (name.to_string(), compute_checksum(algo.clone(), data)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32_empty() {
        let c = crc32_bytes(&[]);
        assert_eq!(c, 0x0000_0000); /* standard CRC32 of empty = 0x00000000 */
    }

    #[test]
    fn test_crc32_deterministic() {
        let a = crc32_bytes(b"hello");
        let b = crc32_bytes(b"hello");
        assert_eq!(a, b);
    }

    #[test]
    fn test_crc32_differs_for_diff_data() {
        let a = crc32_bytes(b"hello");
        let b = crc32_bytes(b"world");
        assert_ne!(a, b);
    }

    #[test]
    fn test_compute_checksum_crc32() {
        let c = compute_checksum(ChecksumAlgo::Crc32, b"test");
        assert_eq!(c.value.len(), 4);
    }

    #[test]
    fn test_compute_checksum_sha256() {
        let c = compute_checksum(ChecksumAlgo::Sha256, b"test");
        assert_eq!(c.value.len(), 32);
    }

    #[test]
    fn test_compute_checksum_xxhash64() {
        let c = compute_checksum(ChecksumAlgo::Xxhash64, b"test");
        assert_eq!(c.value.len(), 8);
        // The value must match the real xxhash64 implementation.
        let expected = crate::hashing_xxhash::xxhash64(b"test", 0)
            .to_le_bytes()
            .to_vec();
        assert_eq!(c.value, expected);
    }

    #[test]
    fn test_xxhash64_empty_via_compute() {
        // Cross-check: compute_checksum for empty input must agree with
        // the KAT value from hashing_xxhash (0xef46db3751d8e999).
        let c = compute_checksum(ChecksumAlgo::Xxhash64, b"");
        let expected_hash: u64 = 0xef46db3751d8e999;
        assert_eq!(c.value, expected_hash.to_le_bytes().to_vec());
    }

    #[test]
    fn test_sha256_stub_uses_real_sha256() {
        // sha256_stub must return the same bytes as hashing_sha256::sha256_hash.
        let data = b"oxihuman";
        let via_stub = sha256_stub(data);
        let via_direct = crate::hashing_sha256::sha256_hash(data).0;
        assert_eq!(via_stub, via_direct);
    }

    #[test]
    fn test_verify_checksum_ok() {
        let data = b"oxihuman";
        let chk = compute_checksum(ChecksumAlgo::Crc32, data);
        assert!(verify_checksum(data, &chk));
    }

    #[test]
    fn test_verify_checksum_fail() {
        let chk = compute_checksum(ChecksumAlgo::Crc32, b"original");
        assert!(!verify_checksum(b"tampered", &chk));
    }

    #[test]
    fn test_hex_length_crc32() {
        let c = compute_checksum(ChecksumAlgo::Crc32, b"a");
        assert_eq!(c.hex().len(), 8); /* 4 bytes * 2 hex chars */
    }

    #[test]
    fn test_checksum_map() {
        let items: Vec<(&str, &[u8])> = vec![("a.txt", b"aaa"), ("b.txt", b"bbb")];
        let map = checksum_map(&items, ChecksumAlgo::Crc32);
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_algo_output_len() {
        assert_eq!(ChecksumAlgo::Sha256.output_len(), 32);
        assert_eq!(ChecksumAlgo::Crc32.output_len(), 4);
        assert_eq!(ChecksumAlgo::Xxhash64.output_len(), 8);
    }

    #[test]
    fn test_verify_hex_xxhash64() {
        let data = b"hello";
        let c = compute_checksum(ChecksumAlgo::Xxhash64, data);
        assert!(verify_hex(data, ChecksumAlgo::Xxhash64, &c.hex()));
    }
}
