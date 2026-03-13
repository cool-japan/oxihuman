// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A zero-copy byte reader that tracks a cursor position.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ByteReader<'a> {
    data: &'a [u8],
    pos: usize,
}

#[allow(dead_code)]
impl<'a> ByteReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    pub fn is_exhausted(&self) -> bool {
        self.pos >= self.data.len()
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn total_len(&self) -> usize {
        self.data.len()
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
            let arr: [u8; 4] = [
                self.data[self.pos],
                self.data[self.pos + 1],
                self.data[self.pos + 2],
                self.data[self.pos + 3],
            ];
            self.pos += 4;
            Some(u32::from_le_bytes(arr))
        } else {
            None
        }
    }

    pub fn read_f32_le(&mut self) -> Option<f32> {
        self.read_u32_le().map(f32::from_bits)
    }

    pub fn read_bytes(&mut self, count: usize) -> Option<&'a [u8]> {
        if self.pos + count <= self.data.len() {
            let slice = &self.data[self.pos..self.pos + count];
            self.pos += count;
            Some(slice)
        } else {
            None
        }
    }

    pub fn skip(&mut self, count: usize) -> bool {
        if self.pos + count <= self.data.len() {
            self.pos += count;
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.pos = 0;
    }

    pub fn peek_u8(&self) -> Option<u8> {
        self.data.get(self.pos).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_reader() {
        let data = [1u8, 2, 3];
        let r = ByteReader::new(&data);
        assert_eq!(r.remaining(), 3);
        assert_eq!(r.position(), 0);
    }

    #[test]
    fn test_read_u8() {
        let data = [0xAB, 0xCD];
        let mut r = ByteReader::new(&data);
        assert_eq!(r.read_u8(), Some(0xAB));
        assert_eq!(r.read_u8(), Some(0xCD));
        assert!(r.read_u8().is_none());
    }

    #[test]
    fn test_read_u16_le() {
        let data = [0x01, 0x02];
        let mut r = ByteReader::new(&data);
        assert_eq!(r.read_u16_le(), Some(0x0201));
    }

    #[test]
    fn test_read_u32_le() {
        let data = [0x78, 0x56, 0x34, 0x12];
        let mut r = ByteReader::new(&data);
        assert_eq!(r.read_u32_le(), Some(0x12345678));
    }

    #[test]
    fn test_read_f32_le() {
        let val: f32 = 1.0;
        let bytes = val.to_le_bytes();
        let mut r = ByteReader::new(&bytes);
        let result = r.read_f32_le().expect("should succeed");
        assert!((result - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_read_bytes() {
        let data = [10, 20, 30, 40];
        let mut r = ByteReader::new(&data);
        assert_eq!(r.read_bytes(2), Some([10u8, 20].as_slice()));
        assert_eq!(r.remaining(), 2);
    }

    #[test]
    fn test_skip() {
        let data = [1, 2, 3, 4];
        let mut r = ByteReader::new(&data);
        assert!(r.skip(2));
        assert_eq!(r.read_u8(), Some(3));
    }

    #[test]
    fn test_reset() {
        let data = [1, 2, 3];
        let mut r = ByteReader::new(&data);
        r.read_u8();
        r.reset();
        assert_eq!(r.position(), 0);
    }

    #[test]
    fn test_peek_u8() {
        let data = [99];
        let r = ByteReader::new(&data);
        assert_eq!(r.peek_u8(), Some(99));
        assert_eq!(r.position(), 0);
    }

    #[test]
    fn test_is_exhausted() {
        let data = [1];
        let mut r = ByteReader::new(&data);
        assert!(!r.is_exhausted());
        r.read_u8();
        assert!(r.is_exhausted());
    }
}
