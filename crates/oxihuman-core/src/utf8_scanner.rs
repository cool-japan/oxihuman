// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! UTF-8 byte scanner / validator.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ScannerConfig {
    pub max_len: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub char_count: usize,
    pub byte_count: usize,
    pub is_valid: bool,
    pub has_multibyte: bool,
}

#[allow(dead_code)]
pub fn default_scanner_config() -> ScannerConfig {
    ScannerConfig { max_len: 4096 }
}

#[allow(dead_code)]
pub fn scan_utf8(bytes: &[u8]) -> ScanResult {
    match std::str::from_utf8(bytes) {
        Ok(s) => {
            let char_count = s.chars().count();
            let has_multibyte = bytes.len() != char_count;
            ScanResult {
                char_count,
                byte_count: bytes.len(),
                is_valid: true,
                has_multibyte,
            }
        }
        Err(_) => ScanResult {
            char_count: 0,
            byte_count: bytes.len(),
            is_valid: false,
            has_multibyte: false,
        },
    }
}

#[allow(dead_code)]
pub fn scan_str(s: &str) -> ScanResult {
    scan_utf8(s.as_bytes())
}

#[allow(dead_code)]
pub fn count_utf8_chars(s: &str) -> usize {
    s.chars().count()
}

#[allow(dead_code)]
pub fn is_valid_utf8(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes).is_ok()
}

#[allow(dead_code)]
pub fn utf8_char_at(s: &str, char_index: usize) -> Option<char> {
    s.chars().nth(char_index)
}

#[allow(dead_code)]
pub fn utf8_truncate(s: &str, max_chars: usize) -> &str {
    for (char_count, (byte_idx, _)) in s.char_indices().enumerate() {
        if char_count == max_chars {
            return &s[..byte_idx];
        }
    }
    s
}

#[allow(dead_code)]
pub fn utf8_byte_len(s: &str) -> usize {
    s.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_ascii() {
        let res = scan_str("hello");
        assert!(res.is_valid);
        assert_eq!(res.char_count, 5);
        assert_eq!(res.byte_count, 5);
        assert!(!res.has_multibyte);
    }

    #[test]
    fn test_scan_multibyte() {
        let res = scan_str("こんにちは");
        assert!(res.is_valid);
        assert_eq!(res.char_count, 5);
        assert!(res.has_multibyte);
    }

    #[test]
    fn test_scan_invalid_bytes() {
        let bad = &[0xFF, 0xFE, 0xFD];
        let res = scan_utf8(bad);
        assert!(!res.is_valid);
    }

    #[test]
    fn test_count_utf8_chars() {
        assert_eq!(count_utf8_chars("abc"), 3);
        assert_eq!(count_utf8_chars(""), 0);
    }

    #[test]
    fn test_is_valid_utf8() {
        assert!(is_valid_utf8(b"valid"));
        assert!(!is_valid_utf8(&[0xFF]));
    }

    #[test]
    fn test_char_at() {
        assert_eq!(utf8_char_at("hello", 1), Some('e'));
        assert!(utf8_char_at("hello", 99).is_none());
    }

    #[test]
    fn test_truncate() {
        assert_eq!(utf8_truncate("hello world", 5), "hello");
        assert_eq!(utf8_truncate("hi", 10), "hi");
    }

    #[test]
    fn test_byte_len() {
        assert_eq!(utf8_byte_len("abc"), 3);
        let s = "こ"; // 3 bytes in UTF-8
        assert_eq!(utf8_byte_len(s), 3);
    }

    #[test]
    fn test_scan_empty() {
        let res = scan_str("");
        assert!(res.is_valid);
        assert_eq!(res.char_count, 0);
        assert_eq!(res.byte_count, 0);
    }
}
