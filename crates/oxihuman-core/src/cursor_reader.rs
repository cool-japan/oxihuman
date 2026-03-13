// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! A zero-copy cursor for reading bytes sequentially from a slice.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CursorReader<'a> {
    data: &'a [u8],
    pos: usize,
}

#[allow(dead_code)]
impl<'a> CursorReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn is_eof(&self) -> bool {
        self.pos >= self.data.len()
    }

    pub fn read_u8(&mut self) -> Option<u8> {
        if self.pos < self.data.len() {
            let v = self.data[self.pos];
            self.pos += 1;
            Some(v)
        } else {
            None
        }
    }

    pub fn read_u16_le(&mut self) -> Option<u16> {
        if self.pos + 2 <= self.data.len() {
            let v = u16::from_le_bytes([self.data[self.pos], self.data[self.pos + 1]]);
            self.pos += 2;
            Some(v)
        } else {
            None
        }
    }

    pub fn read_u32_le(&mut self) -> Option<u32> {
        if self.pos + 4 <= self.data.len() {
            let bytes: [u8; 4] = self.data[self.pos..self.pos + 4].try_into().ok()?;
            self.pos += 4;
            Some(u32::from_le_bytes(bytes))
        } else {
            None
        }
    }

    pub fn read_f32_le(&mut self) -> Option<f32> {
        self.read_u32_le().map(f32::from_bits)
    }

    pub fn read_bytes(&mut self, n: usize) -> Option<&'a [u8]> {
        if self.pos + n <= self.data.len() {
            let slice = &self.data[self.pos..self.pos + n];
            self.pos += n;
            Some(slice)
        } else {
            None
        }
    }

    pub fn skip(&mut self, n: usize) -> bool {
        if self.pos + n <= self.data.len() {
            self.pos += n;
            true
        } else {
            false
        }
    }

    pub fn seek(&mut self, pos: usize) -> bool {
        if pos <= self.data.len() {
            self.pos = pos;
            true
        } else {
            false
        }
    }

    pub fn peek_u8(&self) -> Option<u8> {
        if self.pos < self.data.len() {
            Some(self.data[self.pos])
        } else {
            None
        }
    }

    pub fn total_len(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_u8() {
        let mut c = CursorReader::new(&[0xAB, 0xCD]);
        assert_eq!(c.read_u8(), Some(0xAB));
        assert_eq!(c.read_u8(), Some(0xCD));
        assert_eq!(c.read_u8(), None);
    }

    #[test]
    fn test_read_u16_le() {
        let mut c = CursorReader::new(&[0x01, 0x02]);
        assert_eq!(c.read_u16_le(), Some(0x0201));
    }

    #[test]
    fn test_read_u32_le() {
        let mut c = CursorReader::new(&[0x78, 0x56, 0x34, 0x12]);
        assert_eq!(c.read_u32_le(), Some(0x1234_5678));
    }

    #[test]
    fn test_read_f32_le() {
        let bits = 1.0f32.to_bits().to_le_bytes();
        let mut c = CursorReader::new(&bits);
        let v = c.read_f32_le().expect("should succeed");
        assert!((v - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_read_bytes() {
        let mut c = CursorReader::new(b"hello world");
        assert_eq!(c.read_bytes(5), Some(b"hello".as_slice()));
    }

    #[test]
    fn test_remaining_and_eof() {
        let mut c = CursorReader::new(&[1, 2, 3]);
        assert_eq!(c.remaining(), 3);
        c.read_u8();
        assert_eq!(c.remaining(), 2);
        c.skip(2);
        assert!(c.is_eof());
    }

    #[test]
    fn test_seek() {
        let mut c = CursorReader::new(&[10, 20, 30]);
        assert!(c.seek(2));
        assert_eq!(c.read_u8(), Some(30));
    }

    #[test]
    fn test_peek() {
        let c = CursorReader::new(&[42]);
        assert_eq!(c.peek_u8(), Some(42));
        assert_eq!(c.position(), 0);
    }

    #[test]
    fn test_skip_past_end() {
        let mut c = CursorReader::new(&[1]);
        assert!(!c.skip(100));
    }

    #[test]
    fn test_total_len() {
        let c = CursorReader::new(&[1, 2, 3, 4, 5]);
        assert_eq!(c.total_len(), 5);
    }
}
