#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Short, compact identifier type.

/// A short identifier backed by a `u64`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub struct ShortId(u64);

/// Create a new `ShortId` from a counter value.
#[allow(dead_code)]
pub fn new_short_id(counter: u64) -> ShortId {
    ShortId(counter)
}

/// Create a `ShortId` from a raw `u64` value.
#[allow(dead_code)]
pub fn short_id_from_u64(v: u64) -> ShortId {
    ShortId(v)
}

/// Convert a `ShortId` to its base-36 string representation.
#[allow(dead_code)]
pub fn short_id_to_string(id: ShortId) -> String {
    const CHARS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    if id.0 == 0 {
        return "0".to_string();
    }
    let mut n = id.0;
    let mut buf = Vec::new();
    while n > 0 {
        buf.push(CHARS[(n % 36) as usize] as char);
        n /= 36;
    }
    buf.into_iter().rev().collect()
}

/// Parse a base-36 string into a `ShortId`.
#[allow(dead_code)]
pub fn short_id_from_str(s: &str) -> Option<ShortId> {
    if s.is_empty() {
        return None;
    }
    let mut v: u64 = 0;
    for ch in s.chars() {
        let digit = match ch {
            '0'..='9' => ch as u64 - '0' as u64,
            'a'..='z' => ch as u64 - 'a' as u64 + 10,
            _ => return None,
        };
        v = v.checked_mul(36)?.checked_add(digit)?;
    }
    Some(ShortId(v))
}

/// Check equality of two `ShortId`s.
#[allow(dead_code)]
pub fn short_ids_equal(a: ShortId, b: ShortId) -> bool {
    a == b
}

/// Hash a `ShortId` to a `u64` (the inner value XOR-shifted).
#[allow(dead_code)]
pub fn short_id_hash(id: ShortId) -> u64 {
    let mut h = id.0;
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51afd7ed558ccd);
    h ^= h >> 33;
    h
}

/// Return true if a `ShortId` string is valid (non-empty base-36).
#[allow(dead_code)]
pub fn short_id_is_valid(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c: char| c.is_ascii_digit() || c.is_ascii_lowercase())
}

/// Return the length of the string representation.
#[allow(dead_code)]
pub fn short_id_len(id: ShortId) -> usize {
    short_id_to_string(id).len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_short_id() {
        let id = new_short_id(42);
        assert_eq!(id.0, 42);
    }

    #[test]
    fn test_from_u64() {
        let id = short_id_from_u64(100);
        assert_eq!(id.0, 100);
    }

    #[test]
    fn test_to_string_zero() {
        assert_eq!(short_id_to_string(ShortId(0)), "0");
    }

    #[test]
    fn test_to_string_roundtrip() {
        let id = ShortId(12345);
        let s = short_id_to_string(id);
        let parsed = short_id_from_str(&s).unwrap();
        assert_eq!(parsed, id);
    }

    #[test]
    fn test_from_str_invalid() {
        assert!(short_id_from_str("UPPER").is_none());
        assert!(short_id_from_str("").is_none());
    }

    #[test]
    fn test_short_ids_equal() {
        let a = ShortId(7);
        let b = ShortId(7);
        assert!(short_ids_equal(a, b));
        assert!(!short_ids_equal(a, ShortId(8)));
    }

    #[test]
    fn test_short_id_hash() {
        let id = ShortId(1);
        let h = short_id_hash(id);
        assert_ne!(h, 1); // should be transformed
    }

    #[test]
    fn test_short_id_is_valid() {
        assert!(short_id_is_valid("abc123"));
        assert!(!short_id_is_valid(""));
        assert!(!short_id_is_valid("ABC"));
    }

    #[test]
    fn test_short_id_len() {
        let id = ShortId(0);
        assert_eq!(short_id_len(id), 1);
        let id2 = ShortId(36);
        assert_eq!(short_id_len(id2), 2);
    }
}
