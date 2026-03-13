// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A growable output buffer for accumulating byte data.
#[allow(dead_code)]
pub struct OutputBuffer {
    data: Vec<u8>,
    flush_count: u32,
}

#[allow(dead_code)]
impl OutputBuffer {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            flush_count: 0,
        }
    }
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            data: Vec::with_capacity(cap),
            flush_count: 0,
        }
    }
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }
    pub fn write_str(&mut self, s: &str) {
        self.data.extend_from_slice(s.as_bytes());
    }
    pub fn write_u8(&mut self, v: u8) {
        self.data.push(v);
    }
    pub fn write_u32_le(&mut self, v: u32) {
        self.data.extend_from_slice(&v.to_le_bytes());
    }
    pub fn flush(&mut self) -> Vec<u8> {
        self.flush_count += 1;
        std::mem::take(&mut self.data)
    }
    pub fn peek(&self) -> &[u8] {
        &self.data
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    pub fn flush_count(&self) -> u32 {
        self.flush_count
    }
    pub fn clear(&mut self) {
        self.data.clear();
    }
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }
    pub fn as_str_lossy(&self) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(&self.data)
    }
}

impl Default for OutputBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
pub fn new_output_buffer() -> OutputBuffer {
    OutputBuffer::new()
}
#[allow(dead_code)]
pub fn ob_write_bytes(b: &mut OutputBuffer, bytes: &[u8]) {
    b.write_bytes(bytes);
}
#[allow(dead_code)]
pub fn ob_write_str(b: &mut OutputBuffer, s: &str) {
    b.write_str(s);
}
#[allow(dead_code)]
pub fn ob_write_u8(b: &mut OutputBuffer, v: u8) {
    b.write_u8(v);
}
#[allow(dead_code)]
pub fn ob_flush(b: &mut OutputBuffer) -> Vec<u8> {
    b.flush()
}
#[allow(dead_code)]
pub fn ob_peek(b: &OutputBuffer) -> &[u8] {
    b.peek()
}
#[allow(dead_code)]
pub fn ob_len(b: &OutputBuffer) -> usize {
    b.len()
}
#[allow(dead_code)]
pub fn ob_is_empty(b: &OutputBuffer) -> bool {
    b.is_empty()
}
#[allow(dead_code)]
pub fn ob_clear(b: &mut OutputBuffer) {
    b.clear();
}
#[allow(dead_code)]
pub fn ob_flush_count(b: &OutputBuffer) -> u32 {
    b.flush_count()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_write_str_peek() {
        let mut b = new_output_buffer();
        ob_write_str(&mut b, "hello");
        assert_eq!(ob_peek(&b), b"hello");
    }
    #[test]
    fn test_write_bytes() {
        let mut b = new_output_buffer();
        ob_write_bytes(&mut b, &[1, 2, 3]);
        assert_eq!(ob_len(&b), 3);
    }
    #[test]
    fn test_flush_clears() {
        let mut b = new_output_buffer();
        ob_write_str(&mut b, "abc");
        let out = ob_flush(&mut b);
        assert_eq!(out, b"abc");
        assert!(ob_is_empty(&b));
    }
    #[test]
    fn test_flush_count() {
        let mut b = new_output_buffer();
        ob_flush(&mut b);
        ob_flush(&mut b);
        assert_eq!(ob_flush_count(&b), 2);
    }
    #[test]
    fn test_write_u8() {
        let mut b = new_output_buffer();
        ob_write_u8(&mut b, 42);
        assert_eq!(ob_peek(&b), &[42u8]);
    }
    #[test]
    fn test_write_u32_le() {
        let mut b = new_output_buffer();
        b.write_u32_le(0x01020304);
        assert_eq!(b.len(), 4);
        assert_eq!(b.peek()[0], 0x04);
    }
    #[test]
    fn test_clear() {
        let mut b = new_output_buffer();
        ob_write_str(&mut b, "data");
        ob_clear(&mut b);
        assert!(ob_is_empty(&b));
    }
    #[test]
    fn test_is_empty_initially() {
        let b = new_output_buffer();
        assert!(ob_is_empty(&b));
    }
    #[test]
    fn test_len_accumulates() {
        let mut b = new_output_buffer();
        ob_write_str(&mut b, "abc");
        ob_write_str(&mut b, "de");
        assert_eq!(ob_len(&b), 5);
    }
    #[test]
    fn test_as_str_lossy() {
        let mut b = new_output_buffer();
        ob_write_str(&mut b, "rust");
        let s = b.as_str_lossy();
        assert_eq!(s.as_ref(), "rust");
    }
}
