// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A reusable scratch buffer to avoid repeated heap allocation in hot paths.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ScratchBuffer {
    data: Vec<u8>,
    high_water: usize,
}

#[allow(dead_code)]
impl ScratchBuffer {
    pub fn new() -> Self {
        Self { data: Vec::new(), high_water: 0 }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self { data: Vec::with_capacity(cap), high_water: 0 }
    }

    pub fn reserve(&mut self, size: usize) {
        if self.data.len() < size {
            self.data.resize(size, 0);
        }
        if size > self.high_water {
            self.high_water = size;
        }
    }

    pub fn as_slice(&self, len: usize) -> &[u8] {
        let end = len.min(self.data.len());
        &self.data[..end]
    }

    pub fn as_mut_slice(&mut self, len: usize) -> &mut [u8] {
        self.reserve(len);
        &mut self.data[..len]
    }

    pub fn write_at(&mut self, offset: usize, bytes: &[u8]) {
        let end = offset + bytes.len();
        self.reserve(end);
        self.data[offset..end].copy_from_slice(bytes);
    }

    pub fn read_at(&self, offset: usize, len: usize) -> Option<&[u8]> {
        if offset + len <= self.data.len() {
            Some(&self.data[offset..offset + len])
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        for b in &mut self.data {
            *b = 0;
        }
    }

    pub fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit();
    }

    pub fn allocated_bytes(&self) -> usize {
        self.data.capacity()
    }

    pub fn used_bytes(&self) -> usize {
        self.data.len()
    }

    pub fn high_water_mark(&self) -> usize {
        self.high_water
    }

    pub fn fill(&mut self, len: usize, byte: u8) {
        self.reserve(len);
        for b in &mut self.data[..len] {
            *b = byte;
        }
    }
}

impl Default for ScratchBuffer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reserve_and_slice() {
        let mut buf = ScratchBuffer::new();
        buf.reserve(64);
        assert!(buf.used_bytes() >= 64);
        let s = buf.as_slice(10);
        assert_eq!(s.len(), 10);
    }

    #[test]
    fn test_write_and_read() {
        let mut buf = ScratchBuffer::new();
        buf.write_at(0, &[1, 2, 3, 4]);
        assert_eq!(buf.read_at(0, 4), Some([1, 2, 3, 4].as_slice()));
    }

    #[test]
    fn test_read_out_of_bounds() {
        let buf = ScratchBuffer::new();
        assert_eq!(buf.read_at(0, 1), None);
    }

    #[test]
    fn test_clear() {
        let mut buf = ScratchBuffer::new();
        buf.write_at(0, &[0xFF, 0xFF]);
        buf.clear();
        assert_eq!(buf.read_at(0, 2), Some([0, 0].as_slice()));
    }

    #[test]
    fn test_high_water_mark() {
        let mut buf = ScratchBuffer::new();
        buf.reserve(100);
        buf.reserve(50);
        assert_eq!(buf.high_water_mark(), 100);
    }

    #[test]
    fn test_fill() {
        let mut buf = ScratchBuffer::new();
        buf.fill(4, 0xAB);
        assert_eq!(buf.read_at(0, 4), Some([0xAB, 0xAB, 0xAB, 0xAB].as_slice()));
    }

    #[test]
    fn test_as_mut_slice() {
        let mut buf = ScratchBuffer::new();
        let s = buf.as_mut_slice(4);
        s[0] = 42;
        assert_eq!(buf.read_at(0, 1), Some([42].as_slice()));
    }

    #[test]
    fn test_with_capacity() {
        let buf = ScratchBuffer::with_capacity(128);
        assert!(buf.allocated_bytes() >= 128 || buf.allocated_bytes() == 0);
    }

    #[test]
    fn test_write_extends() {
        let mut buf = ScratchBuffer::new();
        buf.write_at(10, &[1, 2]);
        assert_eq!(buf.read_at(10, 2), Some([1, 2].as_slice()));
    }

    #[test]
    fn test_default() {
        let buf = ScratchBuffer::default();
        assert_eq!(buf.used_bytes(), 0);
    }
}
