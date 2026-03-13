// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Rolling hash (Rabin-Karp style) for substring search.

#![allow(dead_code)]

const DEFAULT_BASE: u64 = 257;
const DEFAULT_MODULUS: u64 = 1_000_000_007;

/// Rolling hash state for a sliding window over bytes.
#[allow(dead_code)]
pub struct RollingHash {
    pub window: Vec<u8>,
    pub hash: u64,
    pub base: u64,
    pub modulus: u64,
    pub window_size: usize,
}

/// Create a new rolling hash with the given window size, using default base and modulus.
#[allow(dead_code)]
pub fn new_rolling_hash(window_size: usize) -> RollingHash {
    RollingHash {
        window: Vec::with_capacity(window_size),
        hash: 0,
        base: DEFAULT_BASE,
        modulus: DEFAULT_MODULUS,
        window_size,
    }
}

/// Push a byte into the rolling hash window.
/// If the window is already full, the oldest byte is evicted.
#[allow(dead_code)]
pub fn rolling_hash_push(rh: &mut RollingHash, byte: u8) {
    let ws = rh.window_size;
    if ws == 0 {
        return;
    }
    if rh.window.len() == ws {
        // Recompute hash from scratch (simple correctness-first approach)
        rh.window.remove(0);
    }
    rh.window.push(byte);
    // Recompute hash over current window
    let mut h: u64 = 0;
    let base = rh.base;
    let modulus = rh.modulus;
    for &b in &rh.window {
        h = (h.wrapping_mul(base).wrapping_add(b as u64)) % modulus;
    }
    rh.hash = h;
}

/// Return the current hash value.
#[allow(dead_code)]
pub fn rolling_hash_value(rh: &RollingHash) -> u64 {
    rh.hash
}

/// Compute a simple polynomial hash of the given byte slice.
#[allow(dead_code)]
pub fn simple_hash(data: &[u8]) -> u64 {
    let mut h: u64 = 0;
    for &b in data {
        h = (h.wrapping_mul(DEFAULT_BASE).wrapping_add(b as u64)) % DEFAULT_MODULUS;
    }
    h
}

/// Find the first occurrence of `needle` in `haystack` using rolling hash.
/// Returns the starting index or None if not found.
#[allow(dead_code)]
pub fn find_pattern(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    if needle.len() > haystack.len() {
        return None;
    }
    let target_hash = simple_hash(needle);
    let mut rh = new_rolling_hash(needle.len());
    // Fill window
    for &b in &haystack[..needle.len()] {
        rolling_hash_push(&mut rh, b);
    }
    if rolling_hash_value(&rh) == target_hash && &haystack[..needle.len()] == needle {
        return Some(0);
    }
    for i in needle.len()..haystack.len() {
        rolling_hash_push(&mut rh, haystack[i]);
        let start = i + 1 - needle.len();
        if rolling_hash_value(&rh) == target_hash
            && &haystack[start..start + needle.len()] == needle
        {
            return Some(start);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rolling_hash_empty() {
        let rh = new_rolling_hash(4);
        assert_eq!(rolling_hash_value(&rh), 0);
        assert!(rh.window.is_empty());
    }

    #[test]
    fn simple_hash_same_input_same_output() {
        let h1 = simple_hash(b"hello");
        let h2 = simple_hash(b"hello");
        assert_eq!(h1, h2);
    }

    #[test]
    fn simple_hash_different_inputs() {
        let h1 = simple_hash(b"hello");
        let h2 = simple_hash(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn find_pattern_found_at_start() {
        assert_eq!(find_pattern(b"abcdef", b"abc"), Some(0));
    }

    #[test]
    fn find_pattern_found_at_end() {
        assert_eq!(find_pattern(b"abcdef", b"def"), Some(3));
    }

    #[test]
    fn find_pattern_found_in_middle() {
        assert_eq!(find_pattern(b"abcdef", b"bcd"), Some(1));
    }

    #[test]
    fn find_pattern_not_found() {
        assert!(find_pattern(b"abcdef", b"xyz").is_none());
    }

    #[test]
    fn find_pattern_empty_needle() {
        assert_eq!(find_pattern(b"abc", b""), Some(0));
    }

    #[test]
    fn find_pattern_needle_longer_than_haystack() {
        assert!(find_pattern(b"ab", b"abc").is_none());
    }

    #[test]
    fn rolling_hash_consistent_with_simple_hash() {
        let data = b"hello";
        let mut rh = new_rolling_hash(data.len());
        for &b in data.iter() {
            rolling_hash_push(&mut rh, b);
        }
        assert_eq!(rolling_hash_value(&rh), simple_hash(data));
    }
}
