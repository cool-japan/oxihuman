#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Bit-level writer for building packed byte streams.

/// Writes individual bits into a byte buffer.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BitWriter {
    bytes: Vec<u8>,
    current: u8,
    bit_pos: u8, // 0..8, bits written into current byte
}

#[allow(dead_code)]
pub fn new_bit_writer() -> BitWriter {
    BitWriter {
        bytes: Vec::new(),
        current: 0,
        bit_pos: 0,
    }
}

#[allow(dead_code)]
pub fn write_bit(w: &mut BitWriter, bit: bool) {
    if bit {
        w.current |= 1 << (7 - w.bit_pos);
    }
    w.bit_pos += 1;
    if w.bit_pos == 8 {
        w.bytes.push(w.current);
        w.current = 0;
        w.bit_pos = 0;
    }
}

#[allow(dead_code)]
pub fn write_bits(w: &mut BitWriter, value: u32, count: u8) {
    for i in (0..count).rev() {
        write_bit(w, (value >> i) & 1 != 0);
    }
}

#[allow(dead_code)]
pub fn flush_bits(w: &mut BitWriter) {
    if w.bit_pos > 0 {
        w.bytes.push(w.current);
        w.current = 0;
        w.bit_pos = 0;
    }
}

#[allow(dead_code)]
pub fn bit_count(w: &BitWriter) -> usize {
    w.bytes.len() * 8 + w.bit_pos as usize
}

#[allow(dead_code)]
pub fn byte_count(w: &BitWriter) -> usize {
    w.bytes.len() + if w.bit_pos > 0 { 1 } else { 0 }
}

#[allow(dead_code)]
pub fn to_bytes(w: &mut BitWriter) -> Vec<u8> {
    flush_bits(w);
    w.bytes.clone()
}

#[allow(dead_code)]
pub fn reset_bit_writer(w: &mut BitWriter) {
    w.bytes.clear();
    w.current = 0;
    w.bit_pos = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let w = new_bit_writer();
        assert_eq!(bit_count(&w), 0);
        assert_eq!(byte_count(&w), 0);
    }

    #[test]
    fn test_write_single_bit() {
        let mut w = new_bit_writer();
        write_bit(&mut w, true);
        assert_eq!(bit_count(&w), 1);
    }

    #[test]
    fn test_write_byte() {
        let mut w = new_bit_writer();
        write_bits(&mut w, 0xFF, 8);
        assert_eq!(bit_count(&w), 8);
        assert_eq!(byte_count(&w), 1);
    }

    #[test]
    fn test_flush() {
        let mut w = new_bit_writer();
        write_bit(&mut w, true);
        flush_bits(&mut w);
        assert_eq!(w.bytes.len(), 1);
        assert_eq!(w.bytes[0], 0x80);
    }

    #[test]
    fn test_to_bytes() {
        let mut w = new_bit_writer();
        write_bits(&mut w, 0b1010, 4);
        let b = to_bytes(&mut w);
        assert_eq!(b.len(), 1);
        assert_eq!(b[0], 0b10100000);
    }

    #[test]
    fn test_reset() {
        let mut w = new_bit_writer();
        write_bits(&mut w, 0xFF, 8);
        reset_bit_writer(&mut w);
        assert_eq!(bit_count(&w), 0);
    }

    #[test]
    fn test_multi_byte() {
        let mut w = new_bit_writer();
        write_bits(&mut w, 0xABCD, 16);
        let b = to_bytes(&mut w);
        assert_eq!(b.len(), 2);
        assert_eq!(b[0], 0xAB);
        assert_eq!(b[1], 0xCD);
    }

    #[test]
    fn test_byte_count_partial() {
        let mut w = new_bit_writer();
        write_bit(&mut w, false);
        assert_eq!(byte_count(&w), 1); // partial byte counts as 1
    }

    #[test]
    fn test_write_zero_bits() {
        let mut w = new_bit_writer();
        write_bits(&mut w, 0, 0);
        assert_eq!(bit_count(&w), 0);
    }

    #[test]
    fn test_sequential_writes() {
        let mut w = new_bit_writer();
        write_bits(&mut w, 0b11, 2);
        write_bits(&mut w, 0b00, 2);
        write_bits(&mut w, 0b11, 2);
        write_bits(&mut w, 0b00, 2);
        let b = to_bytes(&mut w);
        assert_eq!(b[0], 0b11001100);
    }
}
