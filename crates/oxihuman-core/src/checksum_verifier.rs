// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! File checksum verifier (CRC32/SHA256 stub).

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

/// Simple CRC32 stub (polynomial 0xEDB88320).
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

/// Simple SHA-256 stub (FNV-based hash, not real SHA256).
pub fn sha256_stub(data: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    let bytes = h.to_le_bytes();
    for i in 0..32 {
        out[i] = bytes[i % 8].wrapping_add(i as u8);
    }
    out
}

/// Compute a checksum for bytes.
pub fn compute_checksum(algo: ChecksumAlgo, data: &[u8]) -> Checksum {
    let value = match &algo {
        ChecksumAlgo::Crc32 => crc32_bytes(data).to_le_bytes().to_vec(),
        ChecksumAlgo::Sha256 => sha256_stub(data).to_vec(),
        ChecksumAlgo::Xxhash64 => {
            let mut h: u64 = 0x27D4EB2F165667C5;
            for &b in data {
                h ^= b as u64;
                h = h.wrapping_mul(0xBF58476D1CE4E5B9);
            }
            h.to_le_bytes().to_vec()
        }
    };
    Checksum::new(algo, value)
}

/// Verify data against an expected checksum.
pub fn verify_checksum(data: &[u8], expected: &Checksum) -> bool {
    let actual = compute_checksum(expected.algo.clone(), data);
    actual.value == expected.value
}

/// Verify a checksum hex string.
pub fn verify_hex(data: &[u8], algo: ChecksumAlgo, hex: &str) -> bool {
    let computed = compute_checksum(algo, data);
    computed.hex() == hex
}

/// Build a checksum registry for multiple files (stub).
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
    }
}
