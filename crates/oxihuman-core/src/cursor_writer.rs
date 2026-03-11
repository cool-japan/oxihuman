// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A byte-buffer writer with a movable cursor position.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CursorWriter {
    buffer: Vec<u8>,
    position: usize,
}

impl Default for CursorWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl CursorWriter {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            position: 0,
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(cap),
            position: 0,
        }
    }

    pub fn write_u8(&mut self, v: u8) {
        self.ensure_capacity(1);
        self.buffer[self.position] = v;
        self.position += 1;
    }

    pub fn write_u16_le(&mut self, v: u16) {
        let bytes = v.to_le_bytes();
        self.ensure_capacity(2);
        self.buffer[self.position] = bytes[0];
        self.buffer[self.position + 1] = bytes[1];
        self.position += 2;
    }

    pub fn write_u32_le(&mut self, v: u32) {
        let bytes = v.to_le_bytes();
        self.ensure_capacity(4);
        self.buffer[self.position..self.position + 4].copy_from_slice(&bytes);
        self.position += 4;
    }

    pub fn write_f32_le(&mut self, v: f32) {
        self.write_u32_le(v.to_bits());
    }

    pub fn write_bytes(&mut self, data: &[u8]) {
        self.ensure_capacity(data.len());
        self.buffer[self.position..self.position + data.len()].copy_from_slice(data);
        self.position += data.len();
    }

    fn ensure_capacity(&mut self, additional: usize) {
        let needed = self.position + additional;
        if needed > self.buffer.len() {
            self.buffer.resize(needed, 0);
        }
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn set_position(&mut self, pos: usize) {
        self.position = pos;
    }

    pub fn seek_start(&mut self) {
        self.position = 0;
    }

    pub fn seek_end(&mut self) {
        self.position = self.buffer.len();
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.buffer
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.buffer
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.position = 0;
    }

    pub fn remaining(&self) -> usize {
        self.buffer.len().saturating_sub(self.position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let cw = CursorWriter::new();
        assert!(cw.is_empty());
        assert_eq!(cw.position(), 0);
    }

    #[test]
    fn test_write_u8() {
        let mut cw = CursorWriter::new();
        cw.write_u8(0xAB);
        assert_eq!(cw.as_bytes(), &[0xAB]);
        assert_eq!(cw.position(), 1);
    }

    #[test]
    fn test_write_u16_le() {
        let mut cw = CursorWriter::new();
        cw.write_u16_le(0x0102);
        assert_eq!(cw.as_bytes(), &[0x02, 0x01]);
    }

    #[test]
    fn test_write_u32_le() {
        let mut cw = CursorWriter::new();
        cw.write_u32_le(1);
        assert_eq!(cw.as_bytes(), &[1, 0, 0, 0]);
    }

    #[test]
    fn test_write_f32_le() {
        let mut cw = CursorWriter::new();
        let val: f32 = 1.0;
        cw.write_f32_le(val);
        let bits = u32::from_le_bytes([
            cw.as_bytes()[0],
            cw.as_bytes()[1],
            cw.as_bytes()[2],
            cw.as_bytes()[3],
        ]);
        assert!((f32::from_bits(bits) - val).abs() < f32::EPSILON);
    }

    #[test]
    fn test_write_bytes() {
        let mut cw = CursorWriter::new();
        cw.write_bytes(&[1, 2, 3]);
        assert_eq!(cw.as_bytes(), &[1, 2, 3]);
    }

    #[test]
    fn test_seek() {
        let mut cw = CursorWriter::new();
        cw.write_bytes(&[1, 2, 3]);
        cw.seek_start();
        assert_eq!(cw.position(), 0);
        cw.seek_end();
        assert_eq!(cw.position(), 3);
    }

    #[test]
    fn test_overwrite() {
        let mut cw = CursorWriter::new();
        cw.write_bytes(&[0, 0, 0]);
        cw.set_position(1);
        cw.write_u8(0xFF);
        assert_eq!(cw.as_bytes(), &[0, 0xFF, 0]);
    }

    #[test]
    fn test_clear() {
        let mut cw = CursorWriter::new();
        cw.write_u8(1);
        cw.clear();
        assert!(cw.is_empty());
        assert_eq!(cw.position(), 0);
    }

    #[test]
    fn test_remaining() {
        let mut cw = CursorWriter::new();
        cw.write_bytes(&[1, 2, 3, 4]);
        cw.set_position(1);
        assert_eq!(cw.remaining(), 3);
    }
}
