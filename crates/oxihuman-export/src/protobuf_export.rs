// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Protocol Buffers binary wire-format encoding.

/// Protobuf wire types.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireType {
    Varint = 0,
    Bit64 = 1,
    LenDelimited = 2,
    Bit32 = 5,
}

/// Encode a protobuf tag (field number + wire type).
#[allow(dead_code)]
pub fn encode_tag(field: u32, wire: WireType) -> Vec<u8> {
    let tag = (field << 3) | wire as u32;
    encode_varint(tag as u64)
}

/// Encode an unsigned varint.
#[allow(dead_code)]
pub fn encode_varint(mut val: u64) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let byte = (val & 0x7F) as u8;
        val >>= 7;
        if val == 0 {
            out.push(byte);
            break;
        } else {
            out.push(byte | 0x80);
        }
    }
    out
}

/// Zigzag-encode a signed int (for sint32/sint64).
#[allow(dead_code)]
pub fn zigzag32(val: i32) -> u32 {
    ((val << 1) ^ (val >> 31)) as u32
}

/// Zigzag-encode a signed int64.
#[allow(dead_code)]
pub fn zigzag64(val: i64) -> u64 {
    ((val << 1) ^ (val >> 63)) as u64
}

/// A protobuf encoder.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ProtoEncoder {
    pub buf: Vec<u8>,
}

impl ProtoEncoder {
    #[allow(dead_code)]
    pub fn new() -> Self {
        ProtoEncoder::default()
    }

    #[allow(dead_code)]
    pub fn write_varint_field(&mut self, field: u32, val: u64) {
        self.buf.extend(encode_tag(field, WireType::Varint));
        self.buf.extend(encode_varint(val));
    }

    #[allow(dead_code)]
    pub fn write_fixed64_field(&mut self, field: u32, val: u64) {
        self.buf.extend(encode_tag(field, WireType::Bit64));
        self.buf.extend_from_slice(&val.to_le_bytes());
    }

    #[allow(dead_code)]
    pub fn write_fixed32_field(&mut self, field: u32, val: u32) {
        self.buf.extend(encode_tag(field, WireType::Bit32));
        self.buf.extend_from_slice(&val.to_le_bytes());
    }

    #[allow(dead_code)]
    pub fn write_bytes_field(&mut self, field: u32, data: &[u8]) {
        self.buf.extend(encode_tag(field, WireType::LenDelimited));
        self.buf.extend(encode_varint(data.len() as u64));
        self.buf.extend_from_slice(data);
    }

    #[allow(dead_code)]
    pub fn write_string_field(&mut self, field: u32, s: &str) {
        self.write_bytes_field(field, s.as_bytes());
    }

    #[allow(dead_code)]
    pub fn write_f32_field(&mut self, field: u32, val: f32) {
        self.write_fixed32_field(field, val.to_bits());
    }

    #[allow(dead_code)]
    pub fn write_f64_field(&mut self, field: u32, val: f64) {
        self.write_fixed64_field(field, val.to_bits());
    }

    #[allow(dead_code)]
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varint_small_single_byte() {
        let b = encode_varint(1);
        assert_eq!(b, vec![0x01]);
    }

    #[test]
    fn varint_128_two_bytes() {
        let b = encode_varint(128);
        assert_eq!(b.len(), 2);
    }

    #[test]
    fn tag_field1_varint() {
        let t = encode_tag(1, WireType::Varint);
        assert_eq!(t, vec![0x08]);
    }

    #[test]
    fn zigzag32_positive() {
        assert_eq!(zigzag32(1), 2);
    }

    #[test]
    fn zigzag32_negative() {
        assert_eq!(zigzag32(-1), 1);
    }

    #[test]
    fn zigzag64_zero() {
        assert_eq!(zigzag64(0), 0);
    }

    #[test]
    fn write_varint_field() {
        let mut enc = ProtoEncoder::new();
        enc.write_varint_field(1, 42);
        assert!(!enc.is_empty());
    }

    #[test]
    fn write_string_field_contains_bytes() {
        let mut enc = ProtoEncoder::new();
        enc.write_string_field(1, "hi");
        let s = &enc.buf;
        assert!(s.windows(2).any(|w| w == b"hi"));
    }

    #[test]
    fn write_f32_field_nine_bytes() {
        let mut enc = ProtoEncoder::new();
        enc.write_f32_field(1, 1.0);
        assert_eq!(enc.len(), 5);
    }

    #[test]
    fn write_f64_field_ten_bytes() {
        let mut enc = ProtoEncoder::new();
        enc.write_f64_field(1, 1.0);
        assert_eq!(enc.len(), 9);
    }
}
