// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A reusable string formatting buffer to reduce allocations.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FormatBuffer {
    buf: String,
    max_size: usize,
}

#[allow(dead_code)]
impl FormatBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            buf: String::with_capacity(max_size.min(4096)),
            max_size,
        }
    }

    pub fn write_str(&mut self, s: &str) {
        let remaining = self.max_size.saturating_sub(self.buf.len());
        let take = s.len().min(remaining);
        self.buf.push_str(&s[..take]);
    }

    pub fn write_u32(&mut self, val: u32) {
        let s = val.to_string();
        self.write_str(&s);
    }

    pub fn write_f32(&mut self, val: f32, decimals: usize) {
        let s = format!("{val:.decimals$}");
        self.write_str(&s);
    }

    pub fn write_char(&mut self, c: char) {
        if self.buf.len() < self.max_size {
            self.buf.push(c);
        }
    }

    pub fn write_newline(&mut self) {
        self.write_char('\n');
    }

    pub fn as_str(&self) -> &str {
        &self.buf
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub fn remaining(&self) -> usize {
        self.max_size.saturating_sub(self.buf.len())
    }

    pub fn is_full(&self) -> bool {
        self.buf.len() >= self.max_size
    }

    pub fn clear(&mut self) {
        self.buf.clear();
    }

    pub fn into_string(self) -> String {
        self.buf
    }

    pub fn write_padded(&mut self, s: &str, width: usize) {
        self.write_str(s);
        for _ in s.len()..width {
            self.write_char(' ');
        }
    }
}

impl Default for FormatBuffer {
    fn default() -> Self {
        Self::new(8192)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let fb = FormatBuffer::new(100);
        assert!(fb.is_empty());
        assert_eq!(fb.remaining(), 100);
    }

    #[test]
    fn test_write_str() {
        let mut fb = FormatBuffer::new(100);
        fb.write_str("hello");
        assert_eq!(fb.as_str(), "hello");
        assert_eq!(fb.len(), 5);
    }

    #[test]
    fn test_write_u32() {
        let mut fb = FormatBuffer::new(100);
        fb.write_u32(42);
        assert_eq!(fb.as_str(), "42");
    }

    #[test]
    fn test_write_f32() {
        let mut fb = FormatBuffer::new(100);
        fb.write_f32(2.75, 2);
        assert_eq!(fb.as_str(), "2.75");
    }

    #[test]
    fn test_overflow_truncation() {
        let mut fb = FormatBuffer::new(5);
        fb.write_str("abcdefgh");
        assert_eq!(fb.len(), 5);
        assert!(fb.is_full());
    }

    #[test]
    fn test_clear() {
        let mut fb = FormatBuffer::new(100);
        fb.write_str("data");
        fb.clear();
        assert!(fb.is_empty());
    }

    #[test]
    fn test_into_string() {
        let mut fb = FormatBuffer::new(100);
        fb.write_str("result");
        let s = fb.into_string();
        assert_eq!(s, "result");
    }

    #[test]
    fn test_write_padded() {
        let mut fb = FormatBuffer::new(100);
        fb.write_padded("hi", 5);
        assert_eq!(fb.as_str(), "hi   ");
    }

    #[test]
    fn test_newline() {
        let mut fb = FormatBuffer::new(100);
        fb.write_str("line1");
        fb.write_newline();
        fb.write_str("line2");
        assert_eq!(fb.as_str(), "line1\nline2");
    }

    #[test]
    fn test_default() {
        let fb = FormatBuffer::default();
        assert_eq!(fb.remaining(), 8192);
    }
}
