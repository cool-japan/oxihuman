// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cursor-based read buffer for sequential byte consumption.

/// A buffer with an internal read cursor.
#[derive(Debug, Clone)]
pub struct CursorBuffer {
    data: Vec<u8>,
    cursor: usize,
}

impl CursorBuffer {
    /// Create a new cursor buffer from existing bytes.
    pub fn new(data: Vec<u8>) -> Self {
        CursorBuffer { data, cursor: 0 }
    }

    /// Create an empty buffer.
    pub fn empty() -> Self {
        CursorBuffer::new(vec![])
    }

    /// Number of bytes remaining after the cursor.
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.cursor)
    }

    /// True if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Total buffer length.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// True if no bytes remain.
    pub fn is_done(&self) -> bool {
        self.cursor >= self.data.len()
    }

    /// Read exactly `n` bytes, advancing the cursor.
    pub fn read_bytes(&mut self, n: usize) -> Option<&[u8]> {
        if self.cursor + n > self.data.len() {
            return None;
        }
        let slice = &self.data[self.cursor..self.cursor + n];
        self.cursor += n;
        Some(slice)
    }

    /// Read a single byte.
    pub fn read_u8(&mut self) -> Option<u8> {
        self.read_bytes(1).map(|b| b[0])
    }

    /// Read a little-endian u16.
    pub fn read_u16_le(&mut self) -> Option<u16> {
        let b = self.read_bytes(2)?;
        Some(u16::from_le_bytes([b[0], b[1]]))
    }

    /// Read a little-endian u32.
    pub fn read_u32_le(&mut self) -> Option<u32> {
        let b = self.read_bytes(4)?;
        Some(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    /// Peek at the next byte without advancing.
    pub fn peek_u8(&self) -> Option<u8> {
        self.data.get(self.cursor).copied()
    }

    /// Rewind the cursor to the beginning.
    pub fn rewind(&mut self) {
        self.cursor = 0;
    }

    /// Current cursor position.
    pub fn position(&self) -> usize {
        self.cursor
    }

    /// Append bytes to the buffer.
    pub fn append(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }
}

/// Create a new cursor buffer.
pub fn new_cursor_buffer(data: Vec<u8>) -> CursorBuffer {
    CursorBuffer::new(data)
}

/// Read n bytes.
pub fn cb_read_bytes(buf: &mut CursorBuffer, n: usize) -> Option<Vec<u8>> {
    buf.read_bytes(n).map(|s| s.to_vec())
}

/// Read one byte.
pub fn cb_read_u8(buf: &mut CursorBuffer) -> Option<u8> {
    buf.read_u8()
}

/// Remaining bytes.
pub fn cb_remaining(buf: &CursorBuffer) -> usize {
    buf.remaining()
}

/// Rewind to start.
pub fn cb_rewind(buf: &mut CursorBuffer) {
    buf.rewind();
}

/// Current position.
pub fn cb_position(buf: &CursorBuffer) -> usize {
    buf.position()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_bytes() {
        let mut buf = new_cursor_buffer(vec![1, 2, 3, 4]);
        assert_eq!(cb_read_bytes(&mut buf, 2), Some(vec![1, 2]) /* first two */);
    }

    #[test]
    fn test_remaining() {
        let mut buf = new_cursor_buffer(vec![0; 10]);
        cb_read_bytes(&mut buf, 3);
        assert_eq!(cb_remaining(&buf), 7 /* 10 - 3 */);
    }

    #[test]
    fn test_read_u8() {
        let mut buf = new_cursor_buffer(vec![42, 0]);
        assert_eq!(cb_read_u8(&mut buf), Some(42u8) /* first byte */);
    }

    #[test]
    fn test_read_u16_le() {
        let mut buf = new_cursor_buffer(vec![0x01, 0x00]);
        assert_eq!(buf.read_u16_le(), Some(1u16) /* little endian 1 */);
    }

    #[test]
    fn test_read_u32_le() {
        let mut buf = new_cursor_buffer(vec![0x04, 0x00, 0x00, 0x00]);
        assert_eq!(buf.read_u32_le(), Some(4u32) /* little endian 4 */);
    }

    #[test]
    fn test_peek_u8() {
        let buf = new_cursor_buffer(vec![99, 0]);
        assert_eq!(buf.peek_u8(), Some(99u8) /* peek without advance */);
        assert_eq!(cb_position(&buf), 0 /* cursor unchanged */);
    }

    #[test]
    fn test_rewind() {
        let mut buf = new_cursor_buffer(vec![1, 2, 3]);
        cb_read_bytes(&mut buf, 3);
        cb_rewind(&mut buf);
        assert_eq!(cb_position(&buf), 0 /* back to start */);
    }

    #[test]
    fn test_read_past_end_returns_none() {
        let mut buf = new_cursor_buffer(vec![1, 2]);
        assert_eq!(cb_read_bytes(&mut buf, 5), None /* too many bytes */);
    }

    #[test]
    fn test_is_done() {
        let mut buf = new_cursor_buffer(vec![1]);
        buf.read_u8();
        assert!(buf.is_done() /* consumed all */);
    }

    #[test]
    fn test_append() {
        let mut buf = new_cursor_buffer(vec![1]);
        buf.append(&[2, 3]);
        assert_eq!(cb_remaining(&buf), 3 /* 3 bytes total */);
    }
}
