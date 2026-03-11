// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Apache Thrift codec stub.

/// Thrift type identifiers.
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

/// A Thrift value.
#[derive(Debug, Clone, PartialEq)]
pub enum ThriftValue {
    Bool(bool),
    Byte(u8),
    I16(i16),
    I32(i32),
    I64(i64),
    Double(f64),
    String(String),
    Struct(Vec<ThriftField>),
    List(ThriftType, Vec<ThriftValue>),
}

/// A Thrift struct field.
#[derive(Debug, Clone, PartialEq)]
pub struct ThriftField {
    pub field_id: i16,
    pub field_type: ThriftType,
    pub value: ThriftValue,
}

/// Thrift codec error.
#[derive(Debug, Clone, PartialEq)]
pub enum ThriftError {
    UnexpectedEnd,
    UnknownType(u8),
    InvalidFieldId,
}

impl std::fmt::Display for ThriftError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedEnd => write!(f, "unexpected end of Thrift buffer"),
            Self::UnknownType(t) => write!(f, "unknown Thrift type: {t}"),
            Self::InvalidFieldId => write!(f, "invalid Thrift field ID"),
        }
    }
}

/// Encode a Thrift `i32` in binary protocol (big-endian).
pub fn encode_i32(value: i32, buf: &mut Vec<u8>) {
    buf.extend_from_slice(&value.to_be_bytes());
}

/// Decode a Thrift `i32` from a big-endian byte slice.
pub fn decode_i32(buf: &[u8], offset: usize) -> Result<i32, ThriftError> {
    if offset + 4 > buf.len() {
        return Err(ThriftError::UnexpectedEnd);
    }
    let bytes: [u8; 4] = buf[offset..offset + 4].try_into().unwrap_or_default();
    Ok(i32::from_be_bytes(bytes))
}

/// Encode a Thrift string (i32 length prefix + UTF-8 bytes).
pub fn encode_string(s: &str, buf: &mut Vec<u8>) {
    encode_i32(s.len() as i32, buf);
    buf.extend_from_slice(s.as_bytes());
}

/// Encode a field header (type byte + i16 field ID).
pub fn encode_field_header(field_type: ThriftType, field_id: i16, buf: &mut Vec<u8>) {
    buf.push(field_type as u8);
    buf.extend_from_slice(&field_id.to_be_bytes());
}

/// Count fields in a Thrift struct value.
pub fn struct_field_count(val: &ThriftValue) -> usize {
    if let ThriftValue::Struct(fields) = val {
        fields.len()
    } else {
        0
    }
}

/// Return `true` if a value is a Thrift struct.
pub fn is_struct(val: &ThriftValue) -> bool {
    matches!(val, ThriftValue::Struct(_))
}

/// Return the Thrift type identifier for a value.
pub fn type_of(val: &ThriftValue) -> ThriftType {
    match val {
        ThriftValue::Bool(_) => ThriftType::Bool,
        ThriftValue::Byte(_) => ThriftType::Byte,
        ThriftValue::I16(_) => ThriftType::I16,
        ThriftValue::I32(_) => ThriftType::I32,
        ThriftValue::I64(_) => ThriftType::I64,
        ThriftValue::Double(_) => ThriftType::Double,
        ThriftValue::String(_) => ThriftType::String,
        ThriftValue::Struct(_) => ThriftType::Struct,
        ThriftValue::List(_, _) => ThriftType::List,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_i32_zero() {
        /* zero encodes to four 0x00 bytes */
        let mut buf = vec![];
        encode_i32(0, &mut buf);
        assert_eq!(buf, &[0, 0, 0, 0]);
    }

    #[test]
    fn test_encode_decode_i32_roundtrip() {
        /* encode then decode gives back original */
        let mut buf = vec![];
        encode_i32(12345, &mut buf);
        assert_eq!(decode_i32(&buf, 0).unwrap(), 12345);
    }

    #[test]
    fn test_decode_i32_short_buffer() {
        /* short buffer returns error */
        assert!(decode_i32(&[0, 0], 0).is_err());
    }

    #[test]
    fn test_encode_string() {
        /* string encoded with length prefix */
        let mut buf = vec![];
        encode_string("hi", &mut buf);
        assert_eq!(decode_i32(&buf, 0).unwrap(), 2);
        assert_eq!(&buf[4..], b"hi");
    }

    #[test]
    fn test_encode_field_header() {
        /* field header is 3 bytes */
        let mut buf = vec![];
        encode_field_header(ThriftType::I32, 1, &mut buf);
        assert_eq!(buf.len(), 3);
        assert_eq!(buf[0], ThriftType::I32 as u8);
    }

    #[test]
    fn test_struct_field_count() {
        /* field count for struct */
        let val = ThriftValue::Struct(vec![ThriftField {
            field_id: 1,
            field_type: ThriftType::I32,
            value: ThriftValue::I32(0),
        }]);
        assert_eq!(struct_field_count(&val), 1);
    }

    #[test]
    fn test_is_struct_true() {
        /* struct value detected */
        let val = ThriftValue::Struct(vec![]);
        assert!(is_struct(&val));
    }

    #[test]
    fn test_type_of_bool() {
        /* type_of Bool returns ThriftType::Bool */
        assert_eq!(type_of(&ThriftValue::Bool(true)), ThriftType::Bool);
    }

    #[test]
    fn test_type_of_string() {
        /* type_of String returns ThriftType::String */
        assert_eq!(
            type_of(&ThriftValue::String("x".to_string())),
            ThriftType::String
        );
    }

    #[test]
    fn test_decode_i32_offset() {
        /* decode at non-zero offset */
        let mut buf = vec![0u8; 4];
        buf.extend_from_slice(&42i32.to_be_bytes());
        assert_eq!(decode_i32(&buf, 4).unwrap(), 42);
    }
}
