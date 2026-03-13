// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! xxHash fast hash stub.

const PRIME1: u64 = 0x9E3779B185EBCA87;
const PRIME2: u64 = 0xC2B2AE3D27D4EB4F;
const PRIME3: u64 = 0x165667B19E3779F9;

/// xxHash hasher stub.
#[derive(Debug, Clone, Default)]
pub struct XxHasher {
    buf: Vec<u8>,
    seed: u64,
}

impl XxHasher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_seed(seed: u64) -> Self {
        Self {
            buf: Vec::new(),
            seed,
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        self.buf.extend_from_slice(data);
    }

    pub fn finish(&self) -> u64 {
        xxhash64(&self.buf, self.seed)
    }

    pub fn reset(&mut self) {
        self.buf.clear();
    }
}

/// Compute an xxHash-64 stub digest.
pub fn xxhash64(data: &[u8], seed: u64) -> u64 {
    /* stub: mixing digest — not the real xxHash implementation */
    let mut h = seed.wrapping_add(PRIME3);
    for (i, &b) in data.iter().enumerate() {
        h = h
            .wrapping_add((b as u64).wrapping_mul(PRIME2))
            .wrapping_mul(PRIME1)
            .rotate_left((i % 31 + 1) as u32);
    }
    h ^= h >> 33;
    h = h.wrapping_mul(PRIME2);
    h ^= h >> 29;
    h.wrapping_mul(PRIME3)
}

/// Compute an xxHash-32 stub digest.
pub fn xxhash32(data: &[u8], seed: u32) -> u32 {
    /* stub: simplified 32-bit mixing */
    let mut h = seed.wrapping_add(0x9e3779b1);
    for &b in data {
        h = h
            .wrapping_add(b as u32)
            .wrapping_mul(0x165667b1)
            .rotate_left(13);
    }
    h ^= h >> 15;
    h = h.wrapping_mul(0x85ebca77);
    h ^= h >> 13;
    h
}

/// Return the hash as a hex string.
pub fn xxhash64_hex(data: &[u8], seed: u64) -> String {
    format!("{:016x}", xxhash64(data, seed))
}

/// Return whether two byte slices have the same hash.
pub fn xxhash64_eq(a: &[u8], b: &[u8]) -> bool {
    xxhash64(a, 0) == xxhash64(b, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_deterministic() {
        /* same input same output */
        assert_eq!(xxhash64(b"hello", 0), xxhash64(b"hello", 0));
    }

    #[test]
    fn test_hash_differs_on_input() {
        /* different input different output */
        assert_ne!(xxhash64(b"a", 0), xxhash64(b"b", 0));
    }

    #[test]
    fn test_seed_affects_output() {
        /* seed changes output */
        assert_ne!(xxhash64(b"data", 0), xxhash64(b"data", 1));
    }

    #[test]
    fn test_hash32_deterministic() {
        /* 32-bit variant is deterministic */
        assert_eq!(xxhash32(b"test", 0), xxhash32(b"test", 0));
    }

    #[test]
    fn test_hex_length() {
        /* hex string is 16 chars */
        assert_eq!(xxhash64_hex(b"hi", 0).len(), 16);
    }

    #[test]
    fn test_eq_same() {
        /* same data compares equal */
        assert!(xxhash64_eq(b"same", b"same"));
    }

    #[test]
    fn test_eq_different() {
        /* different data compares unequal */
        assert!(!xxhash64_eq(b"a", b"b"));
    }

    #[test]
    fn test_hasher_incremental() {
        /* incremental hasher matches direct call */
        let mut h = XxHasher::with_seed(42);
        h.update(b"hello");
        assert_eq!(h.finish(), xxhash64(b"hello", 42));
    }

    #[test]
    fn test_hasher_reset() {
        /* reset clears accumulated data */
        let mut h = XxHasher::new();
        h.update(b"something");
        h.reset();
        assert_eq!(h.finish(), xxhash64(&[], 0));
    }
}
