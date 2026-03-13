#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

/// Computes a deterministic hash for a string.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StringHash(pub u64);

#[allow(dead_code)]
pub fn compute_string_hash(s: &str) -> StringHash {
    StringHash(string_hash_u64(s))
}

#[allow(dead_code)]
pub fn string_hash_u32(s: &str) -> u32 {
    let mut h: u32 = 5381;
    for b in s.bytes() {
        h = h.wrapping_mul(33).wrapping_add(u32::from(b));
    }
    h
}

#[allow(dead_code)]
pub fn string_hash_u64(s: &str) -> u64 {
    let mut h: u64 = 14_695_981_039_346_656_037;
    for b in s.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(1_099_511_628_211);
    }
    h
}

#[allow(dead_code)]
pub fn string_hashes_equal(a: &str, b: &str) -> bool {
    string_hash_u64(a) == string_hash_u64(b)
}

#[allow(dead_code)]
pub fn hash_combine_strings(a: &str, b: &str) -> u64 {
    let ha = string_hash_u64(a);
    let hb = string_hash_u64(b);
    ha ^ (hb
        .wrapping_add(0x9e37_79b9_7f4a_7c15)
        .wrapping_add(ha << 6)
        .wrapping_add(ha >> 2))
}

#[allow(dead_code)]
pub fn hash_to_hex_sh(hash: u64) -> String {
    format!("{:016x}", hash)
}

#[allow(dead_code)]
pub fn string_hash_seed(s: &str, seed: u64) -> u64 {
    let mut h = seed;
    for b in s.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(1_099_511_628_211);
    }
    h
}

#[allow(dead_code)]
pub fn hash_empty_string() -> u64 {
    string_hash_u64("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_string_hash() {
        let h = compute_string_hash("hello");
        assert_ne!(h.0, 0);
    }

    #[test]
    fn test_string_hash_u32() {
        let h = string_hash_u32("test");
        assert_ne!(h, 0);
    }

    #[test]
    fn test_string_hash_u64() {
        let h = string_hash_u64("test");
        assert_ne!(h, 0);
    }

    #[test]
    fn test_string_hashes_equal() {
        assert!(string_hashes_equal("abc", "abc"));
        assert!(!string_hashes_equal("abc", "def"));
    }

    #[test]
    fn test_hash_combine_strings() {
        let h = hash_combine_strings("hello", "world");
        assert_ne!(h, string_hash_u64("hello"));
    }

    #[test]
    fn test_hash_to_hex_sh() {
        let hex = hash_to_hex_sh(255);
        assert_eq!(hex, "00000000000000ff");
    }

    #[test]
    fn test_string_hash_seed() {
        let h1 = string_hash_seed("test", 42);
        let h2 = string_hash_seed("test", 99);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hash_empty_string() {
        let h = hash_empty_string();
        assert_eq!(h, string_hash_u64(""));
    }

    #[test]
    fn test_deterministic() {
        assert_eq!(string_hash_u64("abc"), string_hash_u64("abc"));
    }

    #[test]
    fn test_different_strings() {
        assert_ne!(string_hash_u64("abc"), string_hash_u64("xyz"));
    }
}
