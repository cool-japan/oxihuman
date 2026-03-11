// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Apache Thrift binary protocol encoding stub.

/// Thrift field types.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ThriftType {
    Stop = 0,
    Bool = 2,
    Byte = 3,
    Double = 4,
    I16 = 6,
    I32 = 8,
    I64 = 10,
    String = 11,
    Struct = 12,
    Map = 13,
    Set = 14,
    List = 15,
}

/// A Thrift binary encoder.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ThriftEncoder {
    pub buf: Vec<u8>,
}

impl ThriftEncoder {
    #[allow(dead_code)]
    pub fn new() -> Self {
        ThriftEncoder::default()
    }

    /// Write a field header (type + field id).
    #[allow(dead_code)]
    pub fn write_field_begin(&mut self, ftype: ThriftType, field_id: i16) {
        self.buf.push(ftype as u8);
        self.buf.extend_from_slice(&field_id.to_be_bytes());
    }

    /// Write field stop marker.
    #[allow(dead_code)]
    pub fn write_field_stop(&mut self) {
        self.buf.push(ThriftType::Stop as u8);
    }

    /// Write an i32 value.
    #[allow(dead_code)]
    pub fn write_i32(&mut self, val: i32) {
        self.buf.extend_from_slice(&val.to_be_bytes());
    }

    /// Write an i64 value.
    #[allow(dead_code)]
    pub fn write_i64(&mut self, val: i64) {
        self.buf.extend_from_slice(&val.to_be_bytes());
    }

    /// Write a bool value.
    #[allow(dead_code)]
    pub fn write_bool(&mut self, val: bool) {
        self.buf.push(if val { 1 } else { 0 });
    }

    /// Write a string (length-prefixed).
    #[allow(dead_code)]
    pub fn write_string(&mut self, s: &str) {
        self.buf.extend_from_slice(&(s.len() as i32).to_be_bytes());
        self.buf.extend_from_slice(s.as_bytes());
    }

    /// Write a double.
    #[allow(dead_code)]
    pub fn write_double(&mut self, val: f64) {
        self.buf.extend_from_slice(&val.to_bits().to_be_bytes());
    }

    /// Byte length.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Is empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Get encoded bytes.
    #[allow(dead_code)]
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let enc = ThriftEncoder::new();
        assert!(enc.is_empty());
    }

    #[test]
    fn write_field_begin_three_bytes() {
        let mut enc = ThriftEncoder::new();
        enc.write_field_begin(ThriftType::I32, 1);
        assert_eq!(enc.len(), 3);
    }

    #[test]
    fn write_field_stop_one_byte() {
        let mut enc = ThriftEncoder::new();
        enc.write_field_stop();
        assert_eq!(enc.buf[0], 0);
    }

    #[test]
    fn write_i32_four_bytes() {
        let mut enc = ThriftEncoder::new();
        enc.write_i32(42);
        assert_eq!(enc.len(), 4);
    }

    #[test]
    fn write_i64_eight_bytes() {
        let mut enc = ThriftEncoder::new();
        enc.write_i64(1_000_000);
        assert_eq!(enc.len(), 8);
    }

    #[test]
    fn write_bool_one_byte() {
        let mut enc = ThriftEncoder::new();
        enc.write_bool(true);
        assert_eq!(enc.buf[0], 1);
    }

    #[test]
    fn write_string_length_prefixed() {
        let mut enc = ThriftEncoder::new();
        enc.write_string("hi");
        assert_eq!(enc.len(), 6);
    }

    #[test]
    fn write_double_eight_bytes() {
        let mut enc = ThriftEncoder::new();
        enc.write_double(std::f64::consts::PI);
        assert_eq!(enc.len(), 8);
    }

    #[test]
    fn as_bytes_matches_buf() {
        let mut enc = ThriftEncoder::new();
        enc.write_i32(1);
        assert_eq!(enc.as_bytes(), &enc.buf[..]);
    }

    #[test]
    fn full_field_round_trip() {
        let mut enc = ThriftEncoder::new();
        enc.write_field_begin(ThriftType::I32, 1);
        enc.write_i32(99);
        enc.write_field_stop();
        assert_eq!(enc.len(), 8);
    }
}
