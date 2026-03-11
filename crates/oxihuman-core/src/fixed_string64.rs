// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fixed-capacity string (stack allocated up to 64 bytes).

#![allow(dead_code)]

/// A stack-allocated string with capacity of 64 bytes.
#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub struct FixedString64 {
    pub data: [u8; 64],
    pub len: usize,
}

/// Create a new empty FixedString64.
#[allow(dead_code)]
pub fn new_fixed_string64() -> FixedString64 {
    FixedString64 {
        data: [0u8; 64],
        len: 0,
    }
}

/// Create a FixedString64 from a str slice, returning None if too long.
#[allow(dead_code)]
pub fn fixed_string64_from_str(s: &str) -> Option<FixedString64> {
    let bytes = s.as_bytes();
    if bytes.len() > 64 {
        return None;
    }
    let mut fs = new_fixed_string64();
    fs.data[..bytes.len()].copy_from_slice(bytes);
    fs.len = bytes.len();
    Some(fs)
}

/// Return the string contents as a str.
#[allow(dead_code)]
pub fn fixed_string64_as_str(fs: &FixedString64) -> &str {
    core::str::from_utf8(&fs.data[..fs.len]).unwrap_or("")
}

/// Push a char onto the string. Returns false if there is no space.
#[allow(dead_code)]
pub fn fixed_string64_push(fs: &mut FixedString64, c: char) -> bool {
    let mut buf = [0u8; 4];
    let encoded = c.encode_utf8(&mut buf);
    let bytes = encoded.as_bytes();
    if fs.len + bytes.len() > 64 {
        return false;
    }
    fs.data[fs.len..fs.len + bytes.len()].copy_from_slice(bytes);
    fs.len += bytes.len();
    true
}

/// Return the current byte length.
#[allow(dead_code)]
pub fn fixed_string64_len(fs: &FixedString64) -> usize {
    fs.len
}

/// Clear the string (reset to empty).
#[allow(dead_code)]
pub fn fixed_string64_clear(fs: &mut FixedString64) {
    fs.len = 0;
}

/// Return the maximum capacity in bytes.
#[allow(dead_code)]
pub fn fixed_string64_capacity() -> usize {
    64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_is_empty() {
        let fs = new_fixed_string64();
        assert_eq!(fixed_string64_len(&fs), 0);
        assert_eq!(fixed_string64_as_str(&fs), "");
    }

    #[test]
    fn test_from_str_ok() {
        let fs = fixed_string64_from_str("hello").unwrap();
        assert_eq!(fixed_string64_as_str(&fs), "hello");
    }

    #[test]
    fn test_from_str_too_long() {
        let long = "x".repeat(65);
        assert!(fixed_string64_from_str(&long).is_none());
    }

    #[test]
    fn test_push_char() {
        let mut fs = new_fixed_string64();
        assert!(fixed_string64_push(&mut fs, 'A'));
        assert_eq!(fixed_string64_as_str(&fs), "A");
    }

    #[test]
    fn test_push_overflow() {
        let mut fs = fixed_string64_from_str(&"a".repeat(64)).unwrap();
        assert!(!fixed_string64_push(&mut fs, 'B'));
    }

    #[test]
    fn test_len() {
        let fs = fixed_string64_from_str("abc").unwrap();
        assert_eq!(fixed_string64_len(&fs), 3);
    }

    #[test]
    fn test_clear() {
        let mut fs = fixed_string64_from_str("data").unwrap();
        fixed_string64_clear(&mut fs);
        assert_eq!(fixed_string64_len(&fs), 0);
        assert_eq!(fixed_string64_as_str(&fs), "");
    }

    #[test]
    fn test_capacity() {
        assert_eq!(fixed_string64_capacity(), 64);
    }

    #[test]
    fn test_exact_capacity_fits() {
        let s = "a".repeat(64);
        let fs = fixed_string64_from_str(&s);
        assert!(fs.is_some());
        assert_eq!(fixed_string64_len(&fs.unwrap()), 64);
    }
}
