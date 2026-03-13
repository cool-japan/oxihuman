// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! BLAKE3 hash stub.

/// 32-byte BLAKE3 digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blake3Digest(pub [u8; 32]);

impl Blake3Digest {
    /// Return the digest as a lowercase hex string.
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{:02x}", b)).collect()
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// BLAKE3 hasher stub.
#[derive(Debug, Clone, Default)]
pub struct Blake3Hasher {
    buf: Vec<u8>,
    key: Option<[u8; 32]>,
}

impl Blake3Hasher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_keyed(key: [u8; 32]) -> Self {
        Self {
            buf: Vec::new(),
            key: Some(key),
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        self.buf.extend_from_slice(data);
    }

    pub fn finalize(&self) -> Blake3Digest {
        if let Some(k) = &self.key {
            blake3_keyed_hash(k, &self.buf)
        } else {
            blake3_hash(&self.buf)
        }
    }

    pub fn reset(&mut self) {
        self.buf.clear();
    }
}

/// Compute a BLAKE3 stub digest.
pub fn blake3_hash(data: &[u8]) -> Blake3Digest {
    /* stub: mixing-based digest — not cryptographic */
    let mut state = [
        0x6b08e647u32,
        0xbb67ae05,
        0x3c6ef3a2,
        0xa5cf053a,
        0x514e527f,
        0x9b0568fc,
        0x1f09d9ab,
        0x5be0cd19,
    ];
    for (i, &b) in data.iter().enumerate() {
        let idx = i % 8;
        state[idx] = state[idx]
            .wrapping_mul(0x27220a95u32)
            .wrapping_add(b as u32)
            .rotate_left(5);
    }
    let mut digest = [0u8; 32];
    for (i, &s) in state.iter().enumerate() {
        digest[i * 4..(i + 1) * 4].copy_from_slice(&s.to_le_bytes());
    }
    Blake3Digest(digest)
}

/// Compute a keyed BLAKE3 stub digest.
pub fn blake3_keyed_hash(key: &[u8; 32], data: &[u8]) -> Blake3Digest {
    /* stub: mix key into initial state */
    let mut combined = Vec::with_capacity(32 + data.len());
    combined.extend_from_slice(key);
    combined.extend_from_slice(data);
    blake3_hash(&combined)
}

/// Return output length of BLAKE3 in bytes (always 32 for default output).
pub fn blake3_output_len() -> usize {
    32
}

/// Verify round-trip: hash twice and check equality.
pub fn blake3_stable(data: &[u8]) -> bool {
    blake3_hash(data) == blake3_hash(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_len() {
        /* digest is 32 bytes */
        let d = blake3_hash(b"test");
        assert_eq!(d.0.len(), 32);
    }

    #[test]
    fn test_hash_empty() {
        /* empty input returns 32-byte digest */
        let d = blake3_hash(&[]);
        assert_eq!(d.0.len(), 32);
    }

    #[test]
    fn test_hash_deterministic() {
        /* same input gives same digest */
        assert_eq!(blake3_hash(b"abc"), blake3_hash(b"abc"));
    }

    #[test]
    fn test_hash_distinct_inputs() {
        /* different input gives different digest */
        assert_ne!(blake3_hash(b"x"), blake3_hash(b"y"));
    }

    #[test]
    fn test_hex_length() {
        /* hex string is 64 chars */
        assert_eq!(blake3_hash(b"hi").to_hex().len(), 64);
    }

    #[test]
    fn test_keyed_differs_from_plain() {
        /* keyed hash differs from plain hash */
        let key = [0xFFu8; 32];
        assert_ne!(blake3_hash(b"msg"), blake3_keyed_hash(&key, b"msg"));
    }

    #[test]
    fn test_hasher_incremental() {
        /* incremental hasher matches direct hash */
        let mut h = Blake3Hasher::new();
        h.update(b"hello");
        assert_eq!(h.finalize(), blake3_hash(b"hello"));
    }

    #[test]
    fn test_hasher_reset() {
        /* reset clears state */
        let mut h = Blake3Hasher::new();
        h.update(b"something");
        h.reset();
        assert_eq!(h.finalize(), blake3_hash(&[]));
    }

    #[test]
    fn test_output_len() {
        /* output len constant */
        assert_eq!(blake3_output_len(), 32);
    }

    #[test]
    fn test_stable() {
        /* hash is deterministic */
        assert!(blake3_stable(b"stability check"));
    }
}
