// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A view into a contiguous byte buffer with offset and length.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BufferSlice {
    data: Vec<u8>,
    offset: usize,
    len: usize,
}

#[allow(dead_code)]
impl BufferSlice {
    pub fn new(data: Vec<u8>) -> Self {
        let len = data.len();
        Self {
            data,
            offset: 0,
            len,
        }
    }

    pub fn from_range(data: Vec<u8>, offset: usize, len: usize) -> Option<Self> {
        if offset + len > data.len() {
            return None;
        }
        Some(Self { data, offset, len })
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data[self.offset..self.offset + self.len]
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn get(&self, idx: usize) -> Option<u8> {
        if idx < self.len {
            Some(self.data[self.offset + idx])
        } else {
            None
        }
    }

    pub fn sub_slice(&self, start: usize, length: usize) -> Option<BufferSlice> {
        if start + length > self.len {
            return None;
        }
        Some(BufferSlice {
            data: self.data.clone(),
            offset: self.offset + start,
            len: length,
        })
    }

    pub fn read_u8(&self, pos: usize) -> Option<u8> {
        self.get(pos)
    }

    pub fn read_u16_le(&self, pos: usize) -> Option<u16> {
        if pos + 2 > self.len {
            return None;
        }
        let base = self.offset + pos;
        Some(u16::from_le_bytes([self.data[base], self.data[base + 1]]))
    }

    pub fn read_u32_le(&self, pos: usize) -> Option<u32> {
        if pos + 4 > self.len {
            return None;
        }
        let base = self.offset + pos;
        Some(u32::from_le_bytes([
            self.data[base],
            self.data[base + 1],
            self.data[base + 2],
            self.data[base + 3],
        ]))
    }

    pub fn read_f32_le(&self, pos: usize) -> Option<f32> {
        self.read_u32_le(pos).map(f32::from_bits)
    }

    pub fn total_capacity(&self) -> usize {
        self.data.len()
    }

    pub fn copy_to_vec(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }

    pub fn starts_with(&self, prefix: &[u8]) -> bool {
        self.as_bytes().starts_with(prefix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let bs = BufferSlice::new(vec![1, 2, 3]);
        assert_eq!(bs.len(), 3);
        assert_eq!(bs.offset(), 0);
    }

    #[test]
    fn test_from_range() {
        let bs = BufferSlice::from_range(vec![10, 20, 30, 40], 1, 2).expect("should succeed");
        assert_eq!(bs.as_bytes(), &[20, 30]);
    }

    #[test]
    fn test_from_range_invalid() {
        assert!(BufferSlice::from_range(vec![1, 2], 1, 5).is_none());
    }

    #[test]
    fn test_get() {
        let bs = BufferSlice::new(vec![5, 6, 7]);
        assert_eq!(bs.get(1), Some(6));
        assert!(bs.get(10).is_none());
    }

    #[test]
    fn test_sub_slice() {
        let bs = BufferSlice::new(vec![1, 2, 3, 4, 5]);
        let sub = bs.sub_slice(2, 2).expect("should succeed");
        assert_eq!(sub.as_bytes(), &[3, 4]);
    }

    #[test]
    fn test_read_u16_le() {
        let bs = BufferSlice::new(vec![0x01, 0x02]);
        assert_eq!(bs.read_u16_le(0), Some(0x0201));
    }

    #[test]
    fn test_read_u32_le() {
        let bs = BufferSlice::new(vec![0x01, 0x00, 0x00, 0x00]);
        assert_eq!(bs.read_u32_le(0), Some(1));
    }

    #[test]
    fn test_read_f32_le() {
        let val: f32 = 1.5;
        let bytes = val.to_le_bytes().to_vec();
        let bs = BufferSlice::new(bytes);
        let read = bs.read_f32_le(0).expect("should succeed");
        assert!((read - val).abs() < f32::EPSILON);
    }

    #[test]
    fn test_is_empty() {
        let bs = BufferSlice::new(vec![]);
        assert!(bs.is_empty());
    }

    #[test]
    fn test_starts_with() {
        let bs = BufferSlice::new(vec![0xAB, 0xCD, 0xEF]);
        assert!(bs.starts_with(&[0xAB, 0xCD]));
        assert!(!bs.starts_with(&[0xCD]));
    }
}
