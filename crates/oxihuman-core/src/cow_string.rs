// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Copy-on-write string with inline small-string optimization up to 23 bytes.

/// A copy-on-write string that avoids allocation for short strings.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum CowString {
    Inline { buf: [u8; 23], len: u8 },
    Heap(String),
}

#[allow(dead_code)]
impl CowString {
    pub const INLINE_CAP: usize = 23;

    pub fn new(s: &str) -> Self {
        if s.len() <= Self::INLINE_CAP {
            let mut buf = [0u8; 23];
            buf[..s.len()].copy_from_slice(s.as_bytes());
            CowString::Inline { buf, len: s.len() as u8 }
        } else {
            CowString::Heap(s.to_string())
        }
    }

    pub fn empty() -> Self {
        CowString::Inline { buf: [0u8; 23], len: 0 }
    }

    pub fn as_str(&self) -> &str {
        match self {
            CowString::Inline { buf, len } => {
                std::str::from_utf8(&buf[..*len as usize]).unwrap_or("")
            }
            CowString::Heap(s) => s.as_str(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            CowString::Inline { len, .. } => *len as usize,
            CowString::Heap(s) => s.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_inline(&self) -> bool {
        matches!(self, CowString::Inline { .. })
    }

    pub fn is_heap(&self) -> bool {
        matches!(self, CowString::Heap(_))
    }

    pub fn to_owned_string(&self) -> String {
        self.as_str().to_string()
    }

    pub fn push_str(&mut self, s: &str) {
        let current = self.to_owned_string();
        let new_str = current + s;
        *self = CowString::new(&new_str);
    }

    pub fn starts_with(&self, prefix: &str) -> bool {
        self.as_str().starts_with(prefix)
    }

    pub fn ends_with(&self, suffix: &str) -> bool {
        self.as_str().ends_with(suffix)
    }

    pub fn contains_str(&self, needle: &str) -> bool {
        self.as_str().contains(needle)
    }

    pub fn to_uppercase(&self) -> CowString {
        CowString::new(&self.as_str().to_uppercase())
    }

    pub fn to_lowercase(&self) -> CowString {
        CowString::new(&self.as_str().to_lowercase())
    }
}

impl PartialEq for CowString {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for CowString {}

impl std::fmt::Display for CowString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_inline() {
        let s = CowString::new("hello");
        assert!(s.is_inline());
        assert_eq!(s.as_str(), "hello");
    }

    #[test]
    fn test_long_heap() {
        let long = "a]".repeat(20);
        let s = CowString::new(&long);
        assert!(s.is_heap());
        assert_eq!(s.as_str(), long);
    }

    #[test]
    fn test_empty() {
        let s = CowString::empty();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn test_equality() {
        let a = CowString::new("test");
        let b = CowString::new("test");
        assert_eq!(a, b);
    }

    #[test]
    fn test_push_str() {
        let mut s = CowString::new("hello");
        s.push_str(" world");
        assert_eq!(s.as_str(), "hello world");
    }

    #[test]
    fn test_starts_ends_with() {
        let s = CowString::new("foobar");
        assert!(s.starts_with("foo"));
        assert!(s.ends_with("bar"));
    }

    #[test]
    fn test_contains() {
        let s = CowString::new("abcdef");
        assert!(s.contains_str("cde"));
        assert!(!s.contains_str("xyz"));
    }

    #[test]
    fn test_case_conversion() {
        let s = CowString::new("Hello");
        assert_eq!(s.to_uppercase().as_str(), "HELLO");
        assert_eq!(s.to_lowercase().as_str(), "hello");
    }

    #[test]
    fn test_display() {
        let s = CowString::new("display");
        assert_eq!(format!("{}", s), "display");
    }

    #[test]
    fn test_exact_boundary() {
        let s23 = "a".repeat(23);
        let cow = CowString::new(&s23);
        assert!(cow.is_inline());
        let s24 = "a".repeat(24);
        let cow2 = CowString::new(&s24);
        assert!(cow2.is_heap());
    }
}
