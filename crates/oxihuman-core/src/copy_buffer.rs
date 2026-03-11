// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Fixed-capacity byte buffer with copy-in / copy-out semantics.

/// A fixed-capacity byte buffer supporting copy-in and copy-out operations.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CopyBuffer {
    data: Vec<u8>,
    capacity: usize,
}

#[allow(dead_code)]
impl CopyBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn remaining(&self) -> usize {
        self.capacity.saturating_sub(self.data.len())
    }

    /// Copy bytes in. Returns number of bytes actually written.
    pub fn copy_in(&mut self, src: &[u8]) -> usize {
        let avail = self.remaining();
        let n = src.len().min(avail);
        self.data.extend_from_slice(&src[..n]);
        n
    }

    /// Copy out up to `n` bytes from the front. Returns the slice.
    pub fn copy_out(&mut self, n: usize) -> Vec<u8> {
        let take = n.min(self.data.len());
        let out: Vec<u8> = self.data.drain(..take).collect();
        out
    }

    /// Peek at up to `n` bytes without consuming them.
    pub fn peek(&self, n: usize) -> &[u8] {
        let take = n.min(self.data.len());
        &self.data[..take]
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// True if capacity is reached.
    pub fn is_full(&self) -> bool {
        self.data.len() >= self.capacity
    }

    /// Fill with a repeated byte value.
    pub fn fill(&mut self, byte: u8) {
        let n = self.remaining();
        for _ in 0..n {
            self.data.push(byte);
        }
    }

    /// Overwrite byte at position; returns false if out of range.
    pub fn set_byte(&mut self, pos: usize, byte: u8) -> bool {
        if pos < self.data.len() {
            self.data[pos] = byte;
            true
        } else {
            false
        }
    }

    pub fn get_byte(&self, pos: usize) -> Option<u8> {
        self.data.get(pos).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty() {
        let buf = CopyBuffer::new(64);
        assert!(buf.is_empty());
        assert_eq!(buf.capacity(), 64);
        assert_eq!(buf.remaining(), 64);
    }

    #[test]
    fn copy_in_basic() {
        let mut buf = CopyBuffer::new(8);
        let n = buf.copy_in(&[1, 2, 3, 4]);
        assert_eq!(n, 4);
        assert_eq!(buf.len(), 4);
    }

    #[test]
    fn copy_in_clamped_to_capacity() {
        let mut buf = CopyBuffer::new(4);
        let n = buf.copy_in(&[0u8; 10]);
        assert_eq!(n, 4);
        assert!(buf.is_full());
    }

    #[test]
    fn copy_out_removes_front() {
        let mut buf = CopyBuffer::new(16);
        buf.copy_in(&[10, 20, 30, 40]);
        let out = buf.copy_out(2);
        assert_eq!(out, vec![10, 20]);
        assert_eq!(buf.len(), 2);
    }

    #[test]
    fn peek_does_not_consume() {
        let mut buf = CopyBuffer::new(16);
        buf.copy_in(&[1, 2, 3]);
        let peeked = buf.peek(2);
        assert_eq!(peeked, &[1, 2]);
        assert_eq!(buf.len(), 3);
    }

    #[test]
    fn fill_to_capacity() {
        let mut buf = CopyBuffer::new(4);
        buf.fill(0xFF);
        assert!(buf.is_full());
        assert_eq!(buf.as_slice(), &[0xFF, 0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn set_and_get_byte() {
        let mut buf = CopyBuffer::new(8);
        buf.copy_in(&[0u8; 4]);
        assert!(buf.set_byte(2, 99));
        assert_eq!(buf.get_byte(2), Some(99));
        assert!(!buf.set_byte(100, 0));
    }

    #[test]
    fn clear_resets() {
        let mut buf = CopyBuffer::new(8);
        buf.copy_in(&[1, 2, 3]);
        buf.clear();
        assert!(buf.is_empty());
        assert_eq!(buf.remaining(), 8);
    }

    #[test]
    fn remaining_decreases() {
        let mut buf = CopyBuffer::new(10);
        buf.copy_in(&[0u8; 7]);
        assert_eq!(buf.remaining(), 3);
    }

    #[test]
    fn copy_out_partial_when_short() {
        let mut buf = CopyBuffer::new(8);
        buf.copy_in(&[5, 6]);
        let out = buf.copy_out(10);
        assert_eq!(out, vec![5, 6]);
        assert!(buf.is_empty());
    }
}
