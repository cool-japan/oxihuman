// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Sequential byte writer with big/little endian support.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ByteWriter {
    data: Vec<u8>,
    little_endian: bool,
}

#[allow(dead_code)]
impl ByteWriter {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            little_endian: true,
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            data: Vec::with_capacity(cap),
            little_endian: true,
        }
    }

    pub fn set_little_endian(&mut self, le: bool) {
        self.little_endian = le;
    }

    pub fn write_u8(&mut self, val: u8) {
        self.data.push(val);
    }

    pub fn write_u16(&mut self, val: u16) {
        let bytes = if self.little_endian {
            val.to_le_bytes()
        } else {
            val.to_be_bytes()
        };
        self.data.extend_from_slice(&bytes);
    }

    pub fn write_u32(&mut self, val: u32) {
        let bytes = if self.little_endian {
            val.to_le_bytes()
        } else {
            val.to_be_bytes()
        };
        self.data.extend_from_slice(&bytes);
    }

    pub fn write_f32(&mut self, val: f32) {
        let bytes = if self.little_endian {
            val.to_le_bytes()
        } else {
            val.to_be_bytes()
        };
        self.data.extend_from_slice(&bytes);
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    pub fn write_string(&mut self, s: &str) {
        self.write_u32(s.len() as u32);
        self.data.extend_from_slice(s.as_bytes());
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}

impl Default for ByteWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let w = ByteWriter::new();
        assert!(w.is_empty());
    }

    #[test]
    fn test_write_u8() {
        let mut w = ByteWriter::new();
        w.write_u8(0xFF);
        assert_eq!(w.len(), 1);
        assert_eq!(w.as_bytes()[0], 0xFF);
    }

    #[test]
    fn test_write_u16_le() {
        let mut w = ByteWriter::new();
        w.write_u16(0x0102);
        assert_eq!(w.as_bytes(), &[0x02, 0x01]);
    }

    #[test]
    fn test_write_u16_be() {
        let mut w = ByteWriter::new();
        w.set_little_endian(false);
        w.write_u16(0x0102);
        assert_eq!(w.as_bytes(), &[0x01, 0x02]);
    }

    #[test]
    fn test_write_u32() {
        let mut w = ByteWriter::new();
        w.write_u32(1);
        assert_eq!(w.len(), 4);
    }

    #[test]
    fn test_write_f32() {
        let mut w = ByteWriter::new();
        w.write_f32(1.0);
        assert_eq!(w.len(), 4);
        let bytes = w.as_bytes();
        let val = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        assert!((val - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_write_bytes() {
        let mut w = ByteWriter::new();
        w.write_bytes(&[1, 2, 3]);
        assert_eq!(w.len(), 3);
    }

    #[test]
    fn test_write_string() {
        let mut w = ByteWriter::new();
        w.write_string("hi");
        assert_eq!(w.len(), 6); // 4 bytes len + 2 bytes
    }

    #[test]
    fn test_clear() {
        let mut w = ByteWriter::new();
        w.write_u8(1);
        w.clear();
        assert!(w.is_empty());
    }

    #[test]
    fn test_into_vec() {
        let mut w = ByteWriter::new();
        w.write_u8(42);
        let v = w.into_vec();
        assert_eq!(v, vec![42]);
    }
}
