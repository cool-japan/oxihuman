// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fixed-capacity string (stack-allocated, max 64 bytes).

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub struct FixedString {
    pub buf: [u8; 64],
    pub len: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FixedStringConfig {
    pub max_len: usize,
}

#[allow(dead_code)]
pub fn default_fixed_string_config() -> FixedStringConfig {
    FixedStringConfig { max_len: 64 }
}

#[allow(dead_code)]
pub fn new_fixed_string() -> FixedString {
    FixedString { buf: [0u8; 64], len: 0 }
}

#[allow(dead_code)]
pub fn fs_push_str(fs: &mut FixedString, s: &str) -> bool {
    let bytes = s.as_bytes();
    if fs.len + bytes.len() > 64 {
        return false;
    }
    fs.buf[fs.len..fs.len + bytes.len()].copy_from_slice(bytes);
    fs.len += bytes.len();
    true
}

#[allow(dead_code)]
pub fn fs_as_str(fs: &FixedString) -> &str {
    core::str::from_utf8(&fs.buf[..fs.len]).unwrap_or("")
}

#[allow(dead_code)]
pub fn fs_len(fs: &FixedString) -> usize {
    fs.len
}

#[allow(dead_code)]
pub fn fs_is_empty(fs: &FixedString) -> bool {
    fs.len == 0
}

#[allow(dead_code)]
pub fn fs_clear(fs: &mut FixedString) {
    fs.len = 0;
}

#[allow(dead_code)]
pub fn fs_capacity() -> usize {
    64
}

#[allow(dead_code)]
pub fn fs_remaining(fs: &FixedString) -> usize {
    64 - fs.len
}

#[allow(dead_code)]
pub fn fs_starts_with(fs: &FixedString, prefix: &str) -> bool {
    fs_as_str(fs).starts_with(prefix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_is_empty() {
        let fs = new_fixed_string();
        assert!(fs_is_empty(&fs));
        assert_eq!(fs_len(&fs), 0);
    }

    #[test]
    fn test_push_str_basic() {
        let mut fs = new_fixed_string();
        assert!(fs_push_str(&mut fs, "hello"));
        assert_eq!(fs_as_str(&fs), "hello");
    }

    #[test]
    fn test_push_str_overflow() {
        let mut fs = new_fixed_string();
        // Push exactly 64 bytes first
        let long = "a".repeat(60);
        assert!(fs_push_str(&mut fs, &long));
        // Now push more than remaining
        assert!(!fs_push_str(&mut fs, "extra_longer_than_4"));
    }

    #[test]
    fn test_clear() {
        let mut fs = new_fixed_string();
        fs_push_str(&mut fs, "data");
        fs_clear(&mut fs);
        assert!(fs_is_empty(&fs));
        assert_eq!(fs_as_str(&fs), "");
    }

    #[test]
    fn test_capacity() {
        assert_eq!(fs_capacity(), 64);
    }

    #[test]
    fn test_remaining() {
        let mut fs = new_fixed_string();
        fs_push_str(&mut fs, "hi");
        assert_eq!(fs_remaining(&fs), 62);
    }

    #[test]
    fn test_starts_with() {
        let mut fs = new_fixed_string();
        fs_push_str(&mut fs, "hello world");
        assert!(fs_starts_with(&fs, "hello"));
        assert!(!fs_starts_with(&fs, "world"));
    }

    #[test]
    fn test_append_multiple() {
        let mut fs = new_fixed_string();
        fs_push_str(&mut fs, "foo");
        fs_push_str(&mut fs, "bar");
        assert_eq!(fs_as_str(&fs), "foobar");
    }

    #[test]
    fn test_len_after_push() {
        let mut fs = new_fixed_string();
        fs_push_str(&mut fs, "abcde");
        assert_eq!(fs_len(&fs), 5);
    }
}
