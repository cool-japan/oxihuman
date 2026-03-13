#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Bit-level reader for packed byte streams.

/// Reads individual bits from a byte buffer.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BitReader {
    data: Vec<u8>,
    byte_idx: usize,
    bit_idx: u8, // 0..8
}

#[allow(dead_code)]
pub fn new_bit_reader(data: &[u8]) -> BitReader {
    BitReader {
        data: data.to_vec(),
        byte_idx: 0,
        bit_idx: 0,
    }
}

#[allow(dead_code)]
pub fn read_bit(r: &mut BitReader) -> Option<bool> {
    if r.byte_idx >= r.data.len() {
        return None;
    }
    let bit = (r.data[r.byte_idx] >> (7 - r.bit_idx)) & 1 != 0;
    r.bit_idx += 1;
    if r.bit_idx == 8 {
        r.bit_idx = 0;
        r.byte_idx += 1;
    }
    Some(bit)
}

#[allow(dead_code)]
pub fn read_bits(r: &mut BitReader, count: u8) -> Option<u32> {
    let mut val = 0u32;
    for _ in 0..count {
        let b = read_bit(r)?;
        val = (val << 1) | (b as u32);
    }
    Some(val)
}

#[allow(dead_code)]
pub fn bits_remaining(r: &BitReader) -> usize {
    if r.byte_idx >= r.data.len() {
        return 0;
    }
    (r.data.len() - r.byte_idx) * 8 - r.bit_idx as usize
}

#[allow(dead_code)]
pub fn bytes_consumed(r: &BitReader) -> usize {
    r.byte_idx
}

#[allow(dead_code)]
pub fn peek_bit(r: &BitReader) -> Option<bool> {
    if r.byte_idx >= r.data.len() {
        return None;
    }
    Some((r.data[r.byte_idx] >> (7 - r.bit_idx)) & 1 != 0)
}

#[allow(dead_code)]
pub fn is_exhausted(r: &BitReader) -> bool {
    r.byte_idx >= r.data.len()
}

#[allow(dead_code)]
pub fn reader_position(r: &BitReader) -> usize {
    r.byte_idx * 8 + r.bit_idx as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_reader() {
        let r = new_bit_reader(&[0xFF]);
        assert_eq!(bits_remaining(&r), 8);
    }

    #[test]
    fn test_read_bit() {
        let mut r = new_bit_reader(&[0b10000000]);
        assert_eq!(read_bit(&mut r), Some(true));
        assert_eq!(read_bit(&mut r), Some(false));
    }

    #[test]
    fn test_read_bits() {
        let mut r = new_bit_reader(&[0b11010000]);
        assert_eq!(read_bits(&mut r, 4), Some(0b1101));
    }

    #[test]
    fn test_exhausted() {
        let mut r = new_bit_reader(&[0x00]);
        for _ in 0..8 {
            read_bit(&mut r);
        }
        assert!(is_exhausted(&r));
        assert_eq!(read_bit(&mut r), None);
    }

    #[test]
    fn test_peek_no_consume() {
        let r = new_bit_reader(&[0b10000000]);
        assert_eq!(peek_bit(&r), Some(true));
        assert_eq!(bits_remaining(&r), 8);
    }

    #[test]
    fn test_bytes_consumed() {
        let mut r = new_bit_reader(&[0x00, 0x00]);
        for _ in 0..8 {
            read_bit(&mut r);
        }
        assert_eq!(bytes_consumed(&r), 1);
    }

    #[test]
    fn test_position() {
        let mut r = new_bit_reader(&[0x00]);
        read_bit(&mut r);
        read_bit(&mut r);
        assert_eq!(reader_position(&r), 2);
    }

    #[test]
    fn test_empty_reader() {
        let r = new_bit_reader(&[]);
        assert!(is_exhausted(&r));
        assert_eq!(bits_remaining(&r), 0);
    }

    #[test]
    fn test_read_full_byte() {
        let mut r = new_bit_reader(&[0xAB]);
        assert_eq!(read_bits(&mut r, 8), Some(0xAB));
    }

    #[test]
    fn test_multi_byte_read() {
        let mut r = new_bit_reader(&[0xAB, 0xCD]);
        assert_eq!(read_bits(&mut r, 16), Some(0xABCD));
    }
}
