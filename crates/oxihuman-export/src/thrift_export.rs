// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
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

// ─── Thrift Compact Protocol ─────────────────────────────────────────────────

/// Compact-protocol type codes.
#[allow(dead_code)]
pub(crate) mod compact_type {
    pub const BOOLEAN_TRUE: u8 = 1;
    pub const BOOLEAN_FALSE: u8 = 2;
    pub const BYTE: u8 = 3;
    pub const I16: u8 = 4;
    pub const I32: u8 = 5;
    pub const I64: u8 = 6;
    pub const DOUBLE: u8 = 7;
    pub const BINARY: u8 = 8;
    pub const LIST: u8 = 9;
    pub const SET: u8 = 10;
    pub const MAP: u8 = 11;
    pub const STRUCT: u8 = 12;
}

/// An Apache Thrift Compact Protocol encoder.
///
/// Implements the compact binary wire format used by Parquet metadata and
/// other Thrift-based protocols.  Encoding rules:
/// - Integers are zigzag-encoded then written as varints.
/// - Doubles are 8-byte little-endian IEEE 754.
/// - Strings/binary have a varint length prefix then raw bytes.
/// - Field headers use delta encoding when the delta fits in 4 bits.
/// - Structs begin implicitly and end with a 0x00 stop byte.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ThriftCompactEncoder {
    buf: Vec<u8>,
    /// Stack of `last_field_id` values for nested structs.
    field_stack: Vec<i16>,
    last_field: i16,
}

impl ThriftCompactEncoder {
    /// Create a new encoder.
    #[allow(dead_code)]
    pub fn new() -> Self {
        ThriftCompactEncoder {
            buf: Vec::new(),
            field_stack: Vec::new(),
            last_field: 0,
        }
    }

    /// Begin a new struct scope, pushing the current `last_field_id` onto the
    /// stack and resetting it to 0.
    #[allow(dead_code)]
    pub fn begin_struct(&mut self) {
        self.field_stack.push(self.last_field);
        self.last_field = 0;
    }

    /// End the current struct scope: write the 0x00 stop byte and pop the
    /// saved `last_field_id` from the stack.
    #[allow(dead_code)]
    pub fn end_struct(&mut self) {
        self.buf.push(0x00);
        if let Some(prev) = self.field_stack.pop() {
            self.last_field = prev;
        } else {
            self.last_field = 0;
        }
    }

    /// Write a boolean field.
    #[allow(dead_code)]
    pub fn write_bool_field(&mut self, field_id: i16, value: bool) {
        let ctype = if value {
            compact_type::BOOLEAN_TRUE
        } else {
            compact_type::BOOLEAN_FALSE
        };
        self.write_field_header(field_id, ctype);
        // Boolean value is embedded in the type nibble — no extra byte.
    }

    /// Write an i32 field (zigzag varint).
    #[allow(dead_code)]
    pub fn write_i32_field(&mut self, field_id: i16, value: i32) {
        self.write_field_header(field_id, compact_type::I32);
        let zz = Self::zigzag32(value);
        self.write_varint(zz as u64);
    }

    /// Write an i64 field (zigzag varint).
    #[allow(dead_code)]
    pub fn write_i64_field(&mut self, field_id: i16, value: i64) {
        self.write_field_header(field_id, compact_type::I64);
        let zz = Self::zigzag64(value);
        self.write_varint(zz);
    }

    /// Write a double field (8-byte little-endian IEEE 754).
    #[allow(dead_code)]
    pub fn write_double_field(&mut self, field_id: i16, value: f64) {
        self.write_field_header(field_id, compact_type::DOUBLE);
        self.buf.extend_from_slice(&value.to_bits().to_le_bytes());
    }

    /// Write a binary / string field (varint length then raw bytes).
    #[allow(dead_code)]
    pub fn write_string_field(&mut self, field_id: i16, value: &[u8]) {
        self.write_field_header(field_id, compact_type::BINARY);
        self.write_varint(value.len() as u64);
        self.buf.extend_from_slice(value);
    }

    /// Write a list-field header.
    ///
    /// `element_type` should be one of the `compact_type::*` constants.
    /// After this call the caller must write `count` elements directly.
    #[allow(dead_code)]
    pub fn write_list_field_header(&mut self, field_id: i16, element_type: u8, count: usize) {
        self.write_field_header(field_id, compact_type::LIST);
        if count < 15 {
            self.buf.push(((count as u8) << 4) | element_type);
        } else {
            self.buf.push(0xf0 | element_type);
            self.write_varint(count as u64);
        }
    }

    /// Write a nested-struct field header (type = STRUCT, 0x0C) and enter the
    /// struct scope so that subsequent field writes belong to the nested struct.
    #[allow(dead_code)]
    pub fn write_struct_field_begin(&mut self, field_id: i16) {
        self.write_field_header(field_id, compact_type::STRUCT);
        self.field_stack.push(self.last_field);
        self.last_field = 0;
    }

    /// Write raw bytes directly into the output buffer (used for embedding
    /// pre-serialised sub-structs).
    #[allow(dead_code)]
    pub fn write_raw(&mut self, data: &[u8]) {
        self.buf.extend_from_slice(data);
    }

    /// Consume the encoder and return its byte buffer.
    #[allow(dead_code)]
    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }

    /// Borrow the current byte buffer.
    #[allow(dead_code)]
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }

    // ── Public helpers needed by sibling modules ──────────────────────────────

    /// Public wrapper around `zigzag32` for use in sibling modules.
    #[allow(dead_code)]
    #[inline]
    pub fn zigzag32_pub(n: i32) -> u32 {
        Self::zigzag32(n)
    }

    /// Encode a single varint value and return it as a `Vec<u8>`.
    #[allow(dead_code)]
    pub fn encode_varint_pub(v: u64) -> Vec<u8> {
        let mut enc = ThriftCompactEncoder::new();
        enc.write_varint(v);
        enc.into_bytes()
    }

    // ── Private helpers ──────────────────────────────────────────────────────

    /// Encode a field header using delta compression when possible.
    ///
    /// If `field_id - last_field` fits in [1..=15], the header is a single
    /// byte `(delta << 4) | compact_type`.  Otherwise a 0x00 byte is written
    /// followed by the zigzag-encoded i16 field id as a varint.
    pub fn write_field_header(&mut self, field_id: i16, compact_type: u8) {
        let delta = field_id.wrapping_sub(self.last_field);
        if (1..=15).contains(&delta) {
            self.buf.push(((delta as u8) << 4) | compact_type);
        } else {
            self.buf.push(compact_type);
            let zz = Self::zigzag32(field_id as i32);
            self.write_varint(zz as u64);
        }
        self.last_field = field_id;
    }

    /// Write a variable-length integer in little-endian 7-bit groups.
    fn write_varint(&mut self, mut v: u64) {
        loop {
            let byte = (v & 0x7f) as u8;
            v >>= 7;
            if v == 0 {
                self.buf.push(byte);
                break;
            }
            self.buf.push(byte | 0x80);
        }
    }

    /// Zigzag-encode an i32: maps signed integers to unsigned so that small
    /// absolute values produce small varints.
    #[inline]
    fn zigzag32(n: i32) -> u32 {
        ((n << 1) ^ (n >> 31)) as u32
    }

    /// Zigzag-encode an i64.
    #[inline]
    fn zigzag64(n: i64) -> u64 {
        ((n << 1) ^ (n >> 63)) as u64
    }
}

impl Default for ThriftCompactEncoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod compact_tests {
    use super::*;

    /// An empty struct must encode as exactly the single stop byte 0x00.
    #[test]
    fn empty_struct_is_stop_byte() {
        let mut enc = ThriftCompactEncoder::new();
        enc.begin_struct();
        enc.end_struct();
        assert_eq!(enc.as_bytes(), &[0x00]);
    }

    /// Field 1, i32 value 0.
    /// Header: delta=1, type=I32=5 → (1<<4)|5 = 0x15.
    /// Zigzag(0) = 0 → varint = 0x00.
    #[test]
    fn i32_field_1_value_0() {
        let mut enc = ThriftCompactEncoder::new();
        enc.begin_struct();
        enc.write_i32_field(1, 0);
        enc.end_struct();
        // [0x15, 0x00, 0x00]
        let b = enc.as_bytes();
        assert_eq!(b[0], 0x15, "field header byte");
        assert_eq!(b[1], 0x00, "zigzag varint for 0");
        assert_eq!(b[2], 0x00, "stop byte");
    }

    /// A string field must have a varint length prefix followed by the raw
    /// UTF-8 bytes.
    #[test]
    fn string_field_has_length_prefix() {
        let mut enc = ThriftCompactEncoder::new();
        enc.begin_struct();
        enc.write_string_field(1, b"hi");
        enc.end_struct();
        let b = enc.as_bytes();
        // Header byte (0x18 = (1<<4)|8), then varint(2)=0x02, then 'h','i', then 0x00
        assert_eq!(b[0], (1u8 << 4) | compact_type::BINARY);
        assert_eq!(b[1], 2u8); // varint length 2
        assert_eq!(b[2], b'h');
        assert_eq!(b[3], b'i');
        assert_eq!(b[4], 0x00);
    }

    /// Verify that a known struct round-trips: field 1 = i32(42), field 2 = i64(1).
    #[test]
    fn known_struct_round_trip() {
        let mut enc = ThriftCompactEncoder::new();
        enc.begin_struct();
        enc.write_i32_field(1, 42);
        enc.write_i64_field(2, 1);
        enc.end_struct();
        let b = enc.into_bytes();
        // Must end with stop byte
        assert_eq!(*b.last().unwrap(), 0x00);
        // Must start with field-1 header (delta=1, type=I32=5) → 0x15
        assert_eq!(b[0], 0x15);
        // zigzag(42) = 84 = 0x54 (single varint byte)
        assert_eq!(b[1], 84u8);
        // field-2 header: delta=1, type=I64=6 → 0x16
        assert_eq!(b[2], 0x16);
        // zigzag(1) = 2 = 0x02
        assert_eq!(b[3], 2u8);
    }

    /// Double field must produce 8 bytes of LE IEEE-754 bits after the header.
    #[test]
    fn double_field_le_ieee754() {
        let mut enc = ThriftCompactEncoder::new();
        enc.begin_struct();
        enc.write_double_field(1, 1.0_f64);
        enc.end_struct();
        let b = enc.as_bytes();
        // 1.0f64 bits = 0x3FF0000000000000 LE → [0,0,0,0,0,0,0xF0,0x3F]
        let expected_le = 1.0_f64.to_bits().to_le_bytes();
        assert_eq!(&b[1..9], &expected_le);
    }

    /// Nested struct round-trip: outer field 5 is a struct, inner has field 1.
    #[test]
    fn nested_struct_round_trip() {
        let mut enc = ThriftCompactEncoder::new();
        enc.begin_struct();
        enc.write_struct_field_begin(5);
        enc.write_i32_field(1, 7);
        enc.end_struct(); // end inner
        enc.end_struct(); // end outer
        let b = enc.into_bytes();
        assert_eq!(*b.last().unwrap(), 0x00);
        // outer field 5 header (non-delta since 5 fits in delta from 0): delta=5, type=STRUCT=12 → (5<<4)|12 = 0x5C
        assert_eq!(b[0], (5u8 << 4) | compact_type::STRUCT);
    }

    /// List field header encoding for a small count.
    #[test]
    fn list_field_header_small_count() {
        let mut enc = ThriftCompactEncoder::new();
        enc.begin_struct();
        enc.write_list_field_header(1, compact_type::STRUCT, 3);
        enc.end_struct();
        let b = enc.as_bytes();
        // field header: delta=1, type=LIST=9 → 0x19
        assert_eq!(b[0], (1u8 << 4) | compact_type::LIST);
        // list header: count=3 < 15, element_type=STRUCT=12 → (3<<4)|12 = 0x3C
        assert_eq!(b[1], (3u8 << 4) | compact_type::STRUCT);
    }

    /// Boolean true field.
    #[test]
    fn bool_true_field() {
        let mut enc = ThriftCompactEncoder::new();
        enc.begin_struct();
        enc.write_bool_field(1, true);
        enc.end_struct();
        let b = enc.as_bytes();
        // Boolean value encoded in type nibble, no extra byte.
        // Header: delta=1, type=BOOL_TRUE=1 → (1<<4)|1 = 0x11
        assert_eq!(b[0], (1u8 << 4) | compact_type::BOOLEAN_TRUE);
        // Next byte is stop byte
        assert_eq!(b[1], 0x00);
    }
}
