// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! SHA-256 hash stub.

/// 32-byte SHA-256 digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sha256Digest(pub [u8; 32]);

impl Sha256Digest {
    /// Return the digest as a lowercase hex string.
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Return the raw bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// SHA-256 hasher stub (accumulates input, finalizes on demand).
#[derive(Debug, Clone, Default)]
pub struct Sha256Hasher {
    accumulated: Vec<u8>,
}

impl Sha256Hasher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, data: &[u8]) {
        /* stub: append data */
        self.accumulated.extend_from_slice(data);
    }

    pub fn finalize(&self) -> Sha256Digest {
        /* stub: derive 32-byte digest from accumulated bytes */
        sha256_hash(&self.accumulated)
    }

    pub fn reset(&mut self) {
        self.accumulated.clear();
    }
}

/// Compute a SHA-256 stub digest of the given bytes.
pub fn sha256_hash(data: &[u8]) -> Sha256Digest {
    /* stub: 32-byte digest via mixing — not cryptographic */
    let mut state = [
        0x6a09e667u32,
        0xbb67ae85,
        0x3c6ef372,
        0xa54ff53a,
        0x510e527f,
        0x9b05688c,
        0x1f83d9ab,
        0x5be0cd19,
    ];
    for (i, &b) in data.iter().enumerate() {
        let idx = i % 8;
        state[idx] = state[idx]
            .wrapping_add(b as u32)
            .wrapping_add(state[(idx + 1) % 8])
            .rotate_left(3);
    }
    let mut digest = [0u8; 32];
    for (i, &s) in state.iter().enumerate() {
        let bytes = s.to_be_bytes();
        digest[i * 4..(i + 1) * 4].copy_from_slice(&bytes);
    }
    Sha256Digest(digest)
}

/// Compute HMAC-SHA256 stub.
pub fn hmac_sha256_stub(key: &[u8], data: &[u8]) -> Sha256Digest {
    /* stub: combine key and data then hash */
    let mut combined = Vec::with_capacity(key.len() + data.len());
    combined.extend_from_slice(key);
    combined.extend_from_slice(data);
    sha256_hash(&combined)
}

/// Compare two digests in constant-time (stub).
pub fn sha256_eq(a: &Sha256Digest, b: &Sha256Digest) -> bool {
    a.0.iter()
        .zip(b.0.iter())
        .fold(0u8, |acc, (&x, &y)| acc | (x ^ y))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_returns_32_bytes() {
        /* digest is always 32 bytes */
        let d = sha256_hash(b"hello");
        assert_eq!(d.0.len(), 32);
    }

    #[test]
    fn test_hash_empty() {
        /* empty input returns 32-byte digest */
        let d = sha256_hash(&[]);
        assert_eq!(d.0.len(), 32);
    }

    #[test]
    fn test_hash_deterministic() {
        /* same input gives same output */
        let d1 = sha256_hash(b"test");
        let d2 = sha256_hash(b"test");
        assert_eq!(d1, d2);
    }

    #[test]
    fn test_hash_different_inputs() {
        /* different input gives different output */
        let d1 = sha256_hash(b"aaa");
        let d2 = sha256_hash(b"bbb");
        assert_ne!(d1, d2);
    }

    #[test]
    fn test_to_hex_length() {
        /* hex string is 64 chars */
        let d = sha256_hash(b"hello");
        assert_eq!(d.to_hex().len(), 64);
    }

    #[test]
    fn test_hasher_roundtrip() {
        /* incremental hasher matches direct hash */
        let mut h = Sha256Hasher::new();
        h.update(b"hello");
        let d1 = h.finalize();
        let d2 = sha256_hash(b"hello");
        assert_eq!(d1, d2);
    }

    #[test]
    fn test_hasher_reset() {
        /* reset clears state */
        let mut h = Sha256Hasher::new();
        h.update(b"abc");
        h.reset();
        let d = h.finalize();
        assert_eq!(d, sha256_hash(&[]));
    }

    #[test]
    fn test_sha256_eq() {
        /* eq works for same and different */
        let d1 = sha256_hash(b"same");
        let d2 = sha256_hash(b"same");
        assert!(sha256_eq(&d1, &d2));
        let d3 = sha256_hash(b"other");
        assert!(!sha256_eq(&d1, &d3));
    }

    #[test]
    fn test_hmac_stub() {
        /* hmac returns 32-byte digest */
        let d = hmac_sha256_stub(b"key", b"message");
        assert_eq!(d.0.len(), 32);
    }
}
