// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! MSB-first bit writer/reader for AV1 syntax elements (headers, etc.).

/// Bit writer that emits bits MSB-first into a byte buffer.
pub struct BitWriter {
    buf: Vec<u8>,
    /// Number of bits written into the current (last) byte (0..=7).
    /// When 0, we are at a byte boundary and the next write starts a new byte.
    bit_pos: u8,
}

impl BitWriter {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            bit_pos: 0,
        }
    }

    /// Write a single bit (must be 0 or 1).
    pub fn write_bit(&mut self, bit: u8) {
        debug_assert!(bit <= 1, "bit must be 0 or 1");
        if self.bit_pos == 0 {
            self.buf.push(0);
        }
        let byte = self.buf.last_mut().expect("buf is non-empty after push");
        // bit_pos counts bits already written; place the new bit at position (7 - bit_pos).
        *byte |= (bit & 1) << (7 - self.bit_pos);
        self.bit_pos = (self.bit_pos + 1) & 7;
    }

    /// Write `n` bits from `val`, MSB first.  `n` must be in 0..=64.
    pub fn write_bits(&mut self, val: u64, n: u32) {
        debug_assert!(n <= 64, "n must be <= 64");
        for i in (0..n).rev() {
            let bit = ((val >> i) & 1) as u8;
            self.write_bit(bit);
        }
    }

    /// Write one byte MSB-first (8 bits).
    pub fn write_u8(&mut self, v: u8) {
        self.write_bits(v as u64, 8);
    }

    /// Write a big-endian u16 (16 bits, MSB first).
    pub fn write_u16(&mut self, v: u16) {
        self.write_bits(v as u64, 16);
    }

    /// Write a big-endian u32 (32 bits, MSB first).
    pub fn write_u32(&mut self, v: u32) {
        self.write_bits(v as u64, 32);
    }

    /// Pad the current byte with zero-bits until aligned to the next byte boundary.
    pub fn byte_align(&mut self) {
        if self.bit_pos != 0 {
            // Remaining bits in current byte are already zero (from the push(0) in write_bit).
            self.bit_pos = 0;
        }
    }

    /// Consume the writer and return the underlying byte vector.
    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }

    /// Return a slice of the bytes written so far.
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }

    /// Number of *bytes* in the buffer (the last byte may be partially filled).
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
}

impl Default for BitWriter {
    fn default() -> Self {
        Self::new()
    }
}

/// Bit reader that consumes bits MSB-first from a byte slice.
pub struct BitReader<'a> {
    data: &'a [u8],
    /// Current byte index.
    byte_pos: usize,
    /// Bits already consumed from `data[byte_pos]` (0..=7).
    bit_pos: u8,
}

impl<'a> BitReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            byte_pos: 0,
            bit_pos: 0,
        }
    }

    /// Read one bit, returning `Some(0)` or `Some(1)`, or `None` if exhausted.
    pub fn read_bit(&mut self) -> Option<u8> {
        if self.byte_pos >= self.data.len() {
            return None;
        }
        let byte = self.data[self.byte_pos];
        let bit = (byte >> (7 - self.bit_pos)) & 1;
        self.bit_pos += 1;
        if self.bit_pos == 8 {
            self.bit_pos = 0;
            self.byte_pos += 1;
        }
        Some(bit)
    }

    /// Read `n` bits MSB-first (n in 0..=64).  Returns `None` if not enough data.
    pub fn read_bits(&mut self, n: u32) -> Option<u64> {
        debug_assert!(n <= 64, "n must be <= 64");
        let mut val: u64 = 0;
        for _ in 0..n {
            let bit = self.read_bit()? as u64;
            val = (val << 1) | bit;
        }
        Some(val)
    }

    /// Read one byte (8 bits MSB-first).
    pub fn read_u8(&mut self) -> Option<u8> {
        self.read_bits(8).map(|v| v as u8)
    }

    /// Read a big-endian u16 (16 bits).
    pub fn read_u16(&mut self) -> Option<u16> {
        self.read_bits(16).map(|v| v as u16)
    }

    /// Read a big-endian u32 (32 bits).
    pub fn read_u32(&mut self) -> Option<u32> {
        self.read_bits(32).map(|v| v as u32)
    }

    /// Advance to the next byte boundary, discarding the remaining bits in the current byte.
    pub fn byte_align(&mut self) {
        if self.bit_pos != 0 {
            self.bit_pos = 0;
            self.byte_pos += 1;
        }
    }

    /// Number of whole bytes remaining (not counting the current partially-consumed byte).
    pub fn bytes_remaining(&self) -> usize {
        if self.byte_pos >= self.data.len() {
            0
        } else {
            self.data.len() - self.byte_pos - if self.bit_pos > 0 { 1 } else { 0 }
        }
    }

    /// Total number of bits consumed so far (byte_pos * 8 + bit_pos).
    pub fn position_bits(&self) -> usize {
        self.byte_pos * 8 + self.bit_pos as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_read_single_bit() {
        let mut w = BitWriter::new();
        w.write_bit(1);
        w.write_bit(0);
        w.write_bit(1);
        w.byte_align();
        let bytes = w.into_bytes();
        // 1 0 1 0_0000 = 0b10100000 = 0xA0
        assert_eq!(bytes, &[0b10100000]);
    }

    #[test]
    fn test_write_read_bits_round_trip() {
        let cases: &[(u64, u32)] = &[
            (0, 1),
            (1, 1),
            (0b101, 3),
            (0xFF, 8),
            (0x1234, 16),
            (0xDEADBEEF, 32),
            (0x0102_0304_0506_0708, 64),
            (0, 0),
            (127, 7),
        ];
        for &(val, n) in cases {
            let mut w = BitWriter::new();
            w.write_bits(val, n);
            w.byte_align();
            let bytes = w.into_bytes();
            let mut r = BitReader::new(&bytes);
            let got = r.read_bits(n).expect("should read back");
            assert_eq!(
                got, val,
                "round-trip failed for val={val:#x} n={n}"
            );
        }
    }

    #[test]
    fn test_write_u8_read_u8() {
        let mut w = BitWriter::new();
        w.write_u8(0xAB);
        w.write_u8(0xCD);
        let bytes = w.into_bytes();
        assert_eq!(bytes, &[0xAB, 0xCD]);
        let mut r = BitReader::new(&bytes);
        assert_eq!(r.read_u8(), Some(0xAB));
        assert_eq!(r.read_u8(), Some(0xCD));
        assert_eq!(r.read_u8(), None);
    }

    #[test]
    fn test_write_u16_u32_be() {
        let mut w = BitWriter::new();
        w.write_u16(0x1234);
        w.write_u32(0xDEADBEEF);
        let bytes = w.into_bytes();
        assert_eq!(bytes, &[0x12, 0x34, 0xDE, 0xAD, 0xBE, 0xEF]);
        let mut r = BitReader::new(&bytes);
        assert_eq!(r.read_u16(), Some(0x1234));
        assert_eq!(r.read_u32(), Some(0xDEADBEEF));
    }

    #[test]
    fn test_byte_align_writer_reader() {
        let mut w = BitWriter::new();
        // Write 3 bits then align, then another full byte.
        w.write_bits(0b101, 3);
        w.byte_align();
        w.write_u8(0xFF);
        let bytes = w.into_bytes();
        // byte 0: 101_00000 = 0xA0; byte 1: 0xFF
        assert_eq!(bytes, &[0b10100000, 0xFF]);

        let mut r = BitReader::new(&bytes);
        let v = r.read_bits(3).unwrap();
        assert_eq!(v, 0b101);
        r.byte_align();
        assert_eq!(r.position_bits(), 8);
        assert_eq!(r.read_u8(), Some(0xFF));
    }

    #[test]
    fn test_position_bits_tracking() {
        let data = [0xAB, 0xCD];
        let mut r = BitReader::new(&data);
        assert_eq!(r.position_bits(), 0);
        r.read_bit();
        assert_eq!(r.position_bits(), 1);
        r.read_bits(7).unwrap();
        assert_eq!(r.position_bits(), 8);
        r.read_u8().unwrap();
        assert_eq!(r.position_bits(), 16);
    }

    #[test]
    fn test_is_empty_len() {
        let w = BitWriter::new();
        assert!(w.is_empty());
        assert_eq!(w.len(), 0);
        let mut w = BitWriter::new();
        w.write_bit(1);
        assert!(!w.is_empty());
        assert_eq!(w.len(), 1);
    }
}
