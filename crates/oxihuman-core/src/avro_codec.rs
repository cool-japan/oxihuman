// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Apache Avro codec stub.

/// An Avro primitive type.
#[derive(Debug, Clone, PartialEq)]
pub enum AvroType {
    Null,
    Boolean,
    Int,
    Long,
    Float,
    Double,
    Bytes,
    String,
    Record {
        name: String,
        fields: Vec<AvroField>,
    },
    Array {
        items: Box<AvroType>,
    },
    Union(Vec<AvroType>),
}

/// A record field in an Avro schema.
#[derive(Debug, Clone, PartialEq)]
pub struct AvroField {
    pub name: String,
    pub schema: AvroType,
}

/// An Avro value.
#[derive(Debug, Clone, PartialEq)]
pub enum AvroValue {
    Null,
    Boolean(bool),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Bytes(Vec<u8>),
    String(String),
    Array(Vec<AvroValue>),
    Record(Vec<(String, AvroValue)>),
}

/// Avro codec error.
#[derive(Debug, Clone, PartialEq)]
pub enum AvroError {
    SchemaMismatch,
    UnexpectedEnd,
    InvalidEncoding,
}

impl std::fmt::Display for AvroError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SchemaMismatch => write!(f, "Avro schema mismatch"),
            Self::UnexpectedEnd => write!(f, "unexpected end of Avro buffer"),
            Self::InvalidEncoding => write!(f, "invalid Avro encoding"),
        }
    }
}

/// Encode an Avro long using zigzag + varint.
pub fn encode_long(value: i64, buf: &mut Vec<u8>) {
    let zz = ((value << 1) ^ (value >> 63)) as u64;
    let mut n = zz;
    loop {
        let byte = (n & 0x7F) as u8;
        n >>= 7;
        if n == 0 {
            buf.push(byte);
            break;
        }
        buf.push(byte | 0x80);
    }
}

/// Decode an Avro zigzag long.
pub fn decode_long(buf: &[u8]) -> Result<(i64, usize), AvroError> {
    let mut n: u64 = 0;
    let mut shift = 0u32;
    for (i, &b) in buf.iter().enumerate() {
        n |= ((b & 0x7F) as u64) << shift;
        if b & 0x80 == 0 {
            let value = ((n >> 1) as i64) ^ (-((n & 1) as i64));
            return Ok((value, i + 1));
        }
        shift += 7;
        if shift >= 64 {
            return Err(AvroError::InvalidEncoding);
        }
    }
    Err(AvroError::UnexpectedEnd)
}

/// Encode an Avro bytes field (length-prefixed).
pub fn encode_bytes(data: &[u8], buf: &mut Vec<u8>) {
    encode_long(data.len() as i64, buf);
    buf.extend_from_slice(data);
}

/// Return the number of fields in an Avro record value.
pub fn record_field_count(val: &AvroValue) -> usize {
    if let AvroValue::Record(fields) = val {
        fields.len()
    } else {
        0
    }
}

/// Return `true` if the Avro type is a union.
pub fn is_union(t: &AvroType) -> bool {
    matches!(t, AvroType::Union(_))
}

/// Return the name of an Avro record type, or `None`.
pub fn type_name(t: &AvroType) -> Option<&str> {
    if let AvroType::Record { name, .. } = t {
        Some(name.as_str())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_zero() {
        /* zero encodes to 0x00 */
        let mut buf = vec![];
        encode_long(0, &mut buf);
        assert_eq!(buf, &[0x00]);
    }

    #[test]
    fn test_encode_negative_one() {
        /* -1 encodes to 0x01 in zigzag */
        let mut buf = vec![];
        encode_long(-1, &mut buf);
        assert_eq!(buf, &[0x01]);
    }

    #[test]
    fn test_decode_zero() {
        /* decode 0x00 gives 0 */
        let (v, n) = decode_long(&[0x00]).expect("should succeed");
        assert_eq!(v, 0);
        assert_eq!(n, 1);
    }

    #[test]
    fn test_roundtrip_positive() {
        /* positive value roundtrip */
        let mut buf = vec![];
        encode_long(12345, &mut buf);
        let (v, _) = decode_long(&buf).expect("should succeed");
        assert_eq!(v, 12345);
    }

    #[test]
    fn test_roundtrip_negative() {
        /* negative value roundtrip */
        let mut buf = vec![];
        encode_long(-999, &mut buf);
        let (v, _) = decode_long(&buf).expect("should succeed");
        assert_eq!(v, -999);
    }

    #[test]
    fn test_encode_bytes() {
        /* bytes field encodes length then data */
        let mut buf = vec![];
        encode_bytes(&[1, 2, 3], &mut buf);
        assert!(buf.len() >= 4);
    }

    #[test]
    fn test_record_field_count() {
        /* field count for record value */
        let v = AvroValue::Record(vec![
            ("a".to_string(), AvroValue::Int(1)),
            ("b".to_string(), AvroValue::Boolean(false)),
        ]);
        assert_eq!(record_field_count(&v), 2);
    }

    #[test]
    fn test_is_union_true() {
        /* union type detected */
        let t = AvroType::Union(vec![AvroType::Null, AvroType::String]);
        assert!(is_union(&t));
    }

    #[test]
    fn test_type_name() {
        /* type_name returns record name */
        let t = AvroType::Record {
            name: "Foo".to_string(),
            fields: vec![],
        };
        assert_eq!(type_name(&t), Some("Foo"));
    }

    #[test]
    fn test_unexpected_end() {
        /* truncated buffer returns error */
        assert_eq!(decode_long(&[0x80]).unwrap_err(), AvroError::UnexpectedEnd);
    }
}
