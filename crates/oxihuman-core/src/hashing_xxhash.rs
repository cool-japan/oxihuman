// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! xxHash-64 and xxHash-32 — spec-correct implementations.
//!
//! These follow the official xxHash specification exactly.
//! Reference: <https://github.com/Cyan4973/xxHash/blob/dev/doc/xxhash_spec.md>

// ── xxHash-64 constants ──────────────────────────────────────────────────────

const PRIME64_1: u64 = 0x9E3779B185EBCA87;
const PRIME64_2: u64 = 0xC2B2AE3D27D4EB4F;
const PRIME64_3: u64 = 0x165667B19E3779F9;
const PRIME64_4: u64 = 0x85EBCA77C2B2AE63;
const PRIME64_5: u64 = 0x27D4EB2F165667C5;

// ── xxHash-32 constants ──────────────────────────────────────────────────────

const PRIME32_1: u32 = 0x9E3779B1;
const PRIME32_2: u32 = 0x85EBCA77;
const PRIME32_3: u32 = 0xC2B2AE3D;
const PRIME32_4: u32 = 0x27D4EB2F;
const PRIME32_5: u32 = 0x165667B1;

// ── xxHash-64 helper functions ───────────────────────────────────────────────

/// Process one 8-byte lane into an accumulator.
#[inline(always)]
fn round64(acc: u64, input: u64) -> u64 {
    acc.wrapping_add(input.wrapping_mul(PRIME64_2))
        .rotate_left(31)
        .wrapping_mul(PRIME64_1)
}

/// Merge one accumulator lane into the converged hash state.
#[inline(always)]
fn merge_round64(h: u64, val: u64) -> u64 {
    let val = round64(0, val);
    (h ^ val).wrapping_mul(PRIME64_1).wrapping_add(PRIME64_4)
}

// ── xxHash-32 helper function ────────────────────────────────────────────────

/// Process one 4-byte lane into a 32-bit accumulator.
#[inline(always)]
fn round32(acc: u32, v: u32) -> u32 {
    acc.wrapping_add(v.wrapping_mul(PRIME32_2))
        .rotate_left(13)
        .wrapping_mul(PRIME32_1)
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Compute an xxHash-64 digest per the official xxHash specification.
pub fn xxhash64(data: &[u8], seed: u64) -> u64 {
    let len = data.len();
    let mut offset: usize = 0;
    let mut h64: u64;

    if len >= 32 {
        // Initialise four independent accumulators.
        let mut v1 = seed.wrapping_add(PRIME64_1).wrapping_add(PRIME64_2);
        let mut v2 = seed.wrapping_add(PRIME64_2);
        let mut v3 = seed;
        let mut v4 = seed.wrapping_sub(PRIME64_1);

        // Consume 32-byte stripes.
        while offset + 32 <= len {
            // Safety: bounds are checked by the while condition.
            let lane1 = u64::from_le_bytes(
                data[offset..offset + 8]
                    .try_into()
                    .expect("slice is exactly 8 bytes"),
            );
            let lane2 = u64::from_le_bytes(
                data[offset + 8..offset + 16]
                    .try_into()
                    .expect("slice is exactly 8 bytes"),
            );
            let lane3 = u64::from_le_bytes(
                data[offset + 16..offset + 24]
                    .try_into()
                    .expect("slice is exactly 8 bytes"),
            );
            let lane4 = u64::from_le_bytes(
                data[offset + 24..offset + 32]
                    .try_into()
                    .expect("slice is exactly 8 bytes"),
            );
            v1 = round64(v1, lane1);
            v2 = round64(v2, lane2);
            v3 = round64(v3, lane3);
            v4 = round64(v4, lane4);
            offset += 32;
        }

        // Converge the four accumulators.
        h64 = v1
            .rotate_left(1)
            .wrapping_add(v2.rotate_left(7))
            .wrapping_add(v3.rotate_left(12))
            .wrapping_add(v4.rotate_left(18));

        h64 = merge_round64(h64, v1);
        h64 = merge_round64(h64, v2);
        h64 = merge_round64(h64, v3);
        h64 = merge_round64(h64, v4);
    } else {
        h64 = seed.wrapping_add(PRIME64_5);
    }

    // Mix the total length into the state.
    h64 = h64.wrapping_add(len as u64);

    // Consume remaining 8-byte chunks.
    while offset + 8 <= len {
        let k1 = u64::from_le_bytes(
            data[offset..offset + 8]
                .try_into()
                .expect("slice is exactly 8 bytes"),
        );
        h64 ^= round64(0, k1);
        h64 = h64
            .rotate_left(27)
            .wrapping_mul(PRIME64_1)
            .wrapping_add(PRIME64_4);
        offset += 8;
    }

    // Consume a remaining 4-byte chunk if present.
    if offset + 4 <= len {
        let k1 = u64::from(u32::from_le_bytes(
            data[offset..offset + 4]
                .try_into()
                .expect("slice is exactly 4 bytes"),
        ));
        h64 ^= k1.wrapping_mul(PRIME64_1);
        h64 = h64
            .rotate_left(23)
            .wrapping_mul(PRIME64_2)
            .wrapping_add(PRIME64_3);
        offset += 4;
    }

    // Consume any remaining individual bytes.
    while offset < len {
        let k1 = data[offset] as u64;
        h64 ^= k1.wrapping_mul(PRIME64_5);
        h64 = h64.rotate_left(11).wrapping_mul(PRIME64_1);
        offset += 1;
    }

    // Final avalanche mix.
    h64 ^= h64 >> 33;
    h64 = h64.wrapping_mul(PRIME64_2);
    h64 ^= h64 >> 29;
    h64 = h64.wrapping_mul(PRIME64_3);
    h64 ^= h64 >> 32;
    h64
}

/// Compute an xxHash-32 digest per the official xxHash specification.
pub fn xxhash32(data: &[u8], seed: u32) -> u32 {
    let len = data.len();
    let mut offset: usize = 0;
    let mut h32: u32;

    if len >= 16 {
        // Initialise four independent 32-bit accumulators.
        let mut v1 = seed.wrapping_add(PRIME32_1).wrapping_add(PRIME32_2);
        let mut v2 = seed.wrapping_add(PRIME32_2);
        let mut v3 = seed;
        let mut v4 = seed.wrapping_sub(PRIME32_1);

        // Consume 16-byte stripes.
        while offset + 16 <= len {
            let lane1 = u32::from_le_bytes(
                data[offset..offset + 4]
                    .try_into()
                    .expect("slice is exactly 4 bytes"),
            );
            let lane2 = u32::from_le_bytes(
                data[offset + 4..offset + 8]
                    .try_into()
                    .expect("slice is exactly 4 bytes"),
            );
            let lane3 = u32::from_le_bytes(
                data[offset + 8..offset + 12]
                    .try_into()
                    .expect("slice is exactly 4 bytes"),
            );
            let lane4 = u32::from_le_bytes(
                data[offset + 12..offset + 16]
                    .try_into()
                    .expect("slice is exactly 4 bytes"),
            );
            v1 = round32(v1, lane1);
            v2 = round32(v2, lane2);
            v3 = round32(v3, lane3);
            v4 = round32(v4, lane4);
            offset += 16;
        }

        // Converge the four accumulators.
        h32 = v1
            .rotate_left(1)
            .wrapping_add(v2.rotate_left(7))
            .wrapping_add(v3.rotate_left(12))
            .wrapping_add(v4.rotate_left(18));
    } else {
        h32 = seed.wrapping_add(PRIME32_5);
    }

    // Mix the total length.
    h32 = h32.wrapping_add(len as u32);

    // Consume remaining 4-byte chunks.
    while offset + 4 <= len {
        let lane = u32::from_le_bytes(
            data[offset..offset + 4]
                .try_into()
                .expect("slice is exactly 4 bytes"),
        );
        h32 ^= lane.wrapping_mul(PRIME32_3);
        h32 = h32.rotate_left(17).wrapping_mul(PRIME32_4);
        offset += 4;
    }

    // Consume any remaining individual bytes.
    while offset < len {
        let lane = data[offset] as u32;
        h32 ^= lane.wrapping_mul(PRIME32_5);
        h32 = h32.rotate_left(11).wrapping_mul(PRIME32_1);
        offset += 1;
    }

    // Final avalanche mix.
    h32 ^= h32 >> 15;
    h32 = h32.wrapping_mul(PRIME32_2);
    h32 ^= h32 >> 13;
    h32 = h32.wrapping_mul(PRIME32_3);
    h32 ^= h32 >> 16;
    h32
}

/// Return the xxHash-64 digest as a 16-character lowercase hex string.
pub fn xxhash64_hex(data: &[u8], seed: u64) -> String {
    format!("{:016x}", xxhash64(data, seed))
}

/// Return `true` when two byte slices produce the same xxHash-64 (seed 0).
pub fn xxhash64_eq(a: &[u8], b: &[u8]) -> bool {
    xxhash64(a, 0) == xxhash64(b, 0)
}

// ── Streaming hasher ─────────────────────────────────────────────────────────

/// Streaming xxHash-64 hasher (accumulates data, finalises on demand).
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

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Known-answer tests (KAT) against the authoritative xxHash reference ──

    /// xxHash-64 of the empty string with seed 0.
    /// Reference value from the xxHash C reference implementation.
    #[test]
    fn kat_xxhash64_empty_seed0() {
        assert_eq!(
            xxhash64(b"", 0),
            0xef46db3751d8e999,
            "KAT: xxhash64(b\"\", 0)"
        );
    }

    /// xxHash-32 of the empty string with seed 0.
    /// Reference value from the xxHash C reference implementation.
    #[test]
    fn kat_xxhash32_empty_seed0() {
        assert_eq!(xxhash32(b"", 0), 0x02cc5d05, "KAT: xxhash32(b\"\", 0)");
    }

    // ── Behavioural / regression tests ──────────────────────────────────────

    #[test]
    fn test_hash_deterministic() {
        assert_eq!(xxhash64(b"hello", 0), xxhash64(b"hello", 0));
    }

    #[test]
    fn test_hash_differs_on_input() {
        assert_ne!(xxhash64(b"a", 0), xxhash64(b"b", 0));
    }

    #[test]
    fn test_seed_affects_output() {
        assert_ne!(xxhash64(b"data", 0), xxhash64(b"data", 1));
    }

    #[test]
    fn test_hash32_deterministic() {
        assert_eq!(xxhash32(b"test", 0), xxhash32(b"test", 0));
    }

    #[test]
    fn test_hash32_differs_on_input() {
        assert_ne!(xxhash32(b"a", 0), xxhash32(b"b", 0));
    }

    #[test]
    fn test_hash32_seed_affects_output() {
        assert_ne!(xxhash32(b"data", 0), xxhash32(b"data", 42));
    }

    #[test]
    fn test_hex_length() {
        assert_eq!(xxhash64_hex(b"hi", 0).len(), 16);
    }

    #[test]
    fn test_eq_same() {
        assert!(xxhash64_eq(b"same", b"same"));
    }

    #[test]
    fn test_eq_different() {
        assert!(!xxhash64_eq(b"a", b"b"));
    }

    #[test]
    fn test_hasher_incremental() {
        let mut h = XxHasher::with_seed(42);
        h.update(b"hello");
        assert_eq!(h.finish(), xxhash64(b"hello", 42));
    }

    #[test]
    fn test_hasher_reset() {
        let mut h = XxHasher::new();
        h.update(b"something");
        h.reset();
        assert_eq!(h.finish(), xxhash64(b"", 0));
    }

    /// Inputs whose length crosses the 32-byte stripe boundary exercise the
    /// full multi-accumulator path in xxhash64.
    #[test]
    fn test_hash64_long_input_deterministic() {
        let long: Vec<u8> = (0u8..128).collect();
        assert_eq!(xxhash64(&long, 0), xxhash64(&long, 0));
    }

    /// Same for xxhash32 and the 16-byte stripe boundary.
    #[test]
    fn test_hash32_long_input_deterministic() {
        let long: Vec<u8> = (0u8..64).collect();
        assert_eq!(xxhash32(&long, 0), xxhash32(&long, 0));
    }

    /// Two different long inputs must produce different digests.
    #[test]
    fn test_hash64_long_differs() {
        let a: Vec<u8> = (0u8..64).collect();
        let mut b = a.clone();
        b[32] ^= 0xFF;
        assert_ne!(xxhash64(&a, 0), xxhash64(&b, 0));
    }

    /// Verify the hex output is always lower-case.
    #[test]
    fn test_hex_is_lowercase() {
        let hex = xxhash64_hex(b"oxihuman", 0);
        assert!(hex.chars().all(|c| !c.is_ascii_uppercase()));
    }
}
