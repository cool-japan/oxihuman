// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A compact string that stores short strings inline (up to 23 bytes on 64-bit).
/// Falls back to heap allocation for longer strings.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompactString {
    inner: CompactInner,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum CompactInner {
    Inline { buf: [u8; 23], len: u8 },
    Heap(String),
}

const INLINE_CAP: usize = 23;

#[allow(dead_code)]
impl CompactString {
    pub fn new(s: &str) -> Self {
        if s.len() <= INLINE_CAP {
            let mut buf = [0u8; INLINE_CAP];
            buf[..s.len()].copy_from_slice(s.as_bytes());
            Self {
                inner: CompactInner::Inline {
                    buf,
                    len: s.len() as u8,
                },
            }
        } else {
            Self {
                inner: CompactInner::Heap(s.to_string()),
            }
        }
    }

    pub fn as_str(&self) -> &str {
        match &self.inner {
            CompactInner::Inline { buf, len } => {
                std::str::from_utf8(&buf[..*len as usize]).unwrap_or_default()
            }
            CompactInner::Heap(s) => s.as_str(),
        }
    }

    pub fn len(&self) -> usize {
        match &self.inner {
            CompactInner::Inline { len, .. } => *len as usize,
            CompactInner::Heap(s) => s.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_inline(&self) -> bool {
        matches!(self.inner, CompactInner::Inline { .. })
    }

    pub fn is_heap(&self) -> bool {
        matches!(self.inner, CompactInner::Heap(_))
    }

    pub fn to_string_owned(&self) -> String {
        self.as_str().to_string()
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
}

impl std::fmt::Display for CompactString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_inline() {
        let s = CompactString::new("hello");
        assert!(s.is_inline());
        assert_eq!(s.as_str(), "hello");
    }

    #[test]
    fn test_long_heap() {
        let long = "a]".repeat(20);
        let s = CompactString::new(&long);
        assert!(s.is_heap());
        assert_eq!(s.as_str(), long);
    }

    #[test]
    fn test_empty() {
        let s = CompactString::new("");
        assert!(s.is_empty());
        assert!(s.is_inline());
    }

    #[test]
    fn test_len() {
        let s = CompactString::new("abc");
        assert_eq!(s.len(), 3);
    }

    #[test]
    fn test_boundary_23() {
        let exactly = "a".repeat(23);
        let s = CompactString::new(&exactly);
        assert!(s.is_inline());
        assert_eq!(s.len(), 23);
    }

    #[test]
    fn test_boundary_24() {
        let over = "a".repeat(24);
        let s = CompactString::new(&over);
        assert!(s.is_heap());
    }

    #[test]
    fn test_starts_with() {
        let s = CompactString::new("hello world");
        assert!(s.starts_with("hello"));
        assert!(!s.starts_with("world"));
    }

    #[test]
    fn test_ends_with() {
        let s = CompactString::new("test.rs");
        assert!(s.ends_with(".rs"));
    }

    #[test]
    fn test_display() {
        let s = CompactString::new("display");
        assert_eq!(format!("{s}"), "display");
    }

    #[test]
    fn test_eq() {
        let a = CompactString::new("same");
        let b = CompactString::new("same");
        assert_eq!(a, b);
    }
}
