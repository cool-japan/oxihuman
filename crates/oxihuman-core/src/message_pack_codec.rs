// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! MessagePack encode/decode stub.

/// A MessagePack value.
#[derive(Debug, Clone, PartialEq)]
pub enum MsgValue {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Bin(Vec<u8>),
    Array(Vec<MsgValue>),
    Map(Vec<(MsgValue, MsgValue)>),
}

/// MessagePack codec error.
#[derive(Debug, Clone, PartialEq)]
pub enum MsgError {
    UnexpectedEnd,
    UnknownFormat(u8),
    InvalidUtf8,
    LengthOverflow,
}

impl std::fmt::Display for MsgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedEnd => write!(f, "unexpected end of buffer"),
            Self::UnknownFormat(b) => write!(f, "unknown format byte: 0x{b:02x}"),
            Self::InvalidUtf8 => write!(f, "invalid UTF-8 in string"),
            Self::LengthOverflow => write!(f, "length field overflows usize"),
        }
    }
}

/// Encode a `MsgValue` into a byte buffer (stub: minimal subset).
pub fn encode(val: &MsgValue, buf: &mut Vec<u8>) {
    match val {
        MsgValue::Nil => buf.push(0xc0),
        MsgValue::Bool(true) => buf.push(0xc3),
        MsgValue::Bool(false) => buf.push(0xc2),
        MsgValue::Int(i) => {
            if (0..=127).contains(i) {
                buf.push(*i as u8);
            } else {
                buf.push(0xd3);
                buf.extend_from_slice(&i.to_be_bytes());
            }
        }
        MsgValue::Float(f) => {
            buf.push(0xcb);
            buf.extend_from_slice(&f.to_bits().to_be_bytes());
        }
        MsgValue::Str(s) => {
            let bytes = s.as_bytes();
            if bytes.len() <= 31 {
                buf.push(0xa0 | bytes.len() as u8);
            } else {
                buf.push(0xd9);
                buf.push(bytes.len() as u8);
            }
            buf.extend_from_slice(bytes);
        }
        MsgValue::Bin(b) => {
            buf.push(0xc4);
            buf.push(b.len() as u8);
            buf.extend_from_slice(b);
        }
        MsgValue::Array(arr) => {
            buf.push(0x90 | arr.len().min(15) as u8);
            for item in arr {
                encode(item, buf);
            }
        }
        MsgValue::Map(map) => {
            buf.push(0x80 | map.len().min(15) as u8);
            for (k, v) in map {
                encode(k, buf);
                encode(v, buf);
            }
        }
    }
}

/// Return the encoded size of a value (stub).
pub fn encoded_size(val: &MsgValue) -> usize {
    match val {
        MsgValue::Nil | MsgValue::Bool(_) => 1,
        MsgValue::Int(i) => {
            if (0..=127).contains(i) {
                1
            } else {
                9
            }
        }
        MsgValue::Float(_) => 9,
        MsgValue::Str(s) => 1 + s.len(),
        MsgValue::Bin(b) => 2 + b.len(),
        MsgValue::Array(a) => 1 + a.iter().map(encoded_size).sum::<usize>(),
        MsgValue::Map(m) => {
            1 + m
                .iter()
                .map(|(k, v)| encoded_size(k) + encoded_size(v))
                .sum::<usize>()
        }
    }
}

/// Return `true` if two encoded buffers represent equal values.
pub fn buffers_equal(a: &[u8], b: &[u8]) -> bool {
    a == b
}

/// Count the number of top-level items in an array value.
pub fn array_len(val: &MsgValue) -> usize {
    if let MsgValue::Array(arr) = val {
        arr.len()
    } else {
        0
    }
}

/// Return `true` if the value is `Nil`.
pub fn is_nil(val: &MsgValue) -> bool {
    matches!(val, MsgValue::Nil)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_nil() {
        /* nil encodes to 0xc0 */
        let mut buf = vec![];
        encode(&MsgValue::Nil, &mut buf);
        assert_eq!(buf, &[0xc0]);
    }

    #[test]
    fn test_encode_bool_true() {
        /* true encodes to 0xc3 */
        let mut buf = vec![];
        encode(&MsgValue::Bool(true), &mut buf);
        assert_eq!(buf, &[0xc3]);
    }

    #[test]
    fn test_encode_bool_false() {
        /* false encodes to 0xc2 */
        let mut buf = vec![];
        encode(&MsgValue::Bool(false), &mut buf);
        assert_eq!(buf, &[0xc2]);
    }

    #[test]
    fn test_encode_positive_fixint() {
        /* small int uses positive fixint */
        let mut buf = vec![];
        encode(&MsgValue::Int(42), &mut buf);
        assert_eq!(buf[0], 42);
    }

    #[test]
    fn test_encode_str() {
        /* short string uses fixstr format */
        let mut buf = vec![];
        encode(&MsgValue::Str("hi".to_string()), &mut buf);
        assert_eq!(buf[0], 0xa0 | 2);
    }

    #[test]
    fn test_is_nil() {
        /* nil detection */
        assert!(is_nil(&MsgValue::Nil));
        assert!(!is_nil(&MsgValue::Bool(false)));
    }

    #[test]
    fn test_array_len() {
        /* array_len counts items */
        let v = MsgValue::Array(vec![MsgValue::Nil, MsgValue::Bool(true)]);
        assert_eq!(array_len(&v), 2);
    }

    #[test]
    fn test_encoded_size_nil() {
        /* nil size is 1 */
        assert_eq!(encoded_size(&MsgValue::Nil), 1);
    }

    #[test]
    fn test_buffers_equal() {
        /* identical buffers are equal */
        assert!(buffers_equal(&[1, 2, 3], &[1, 2, 3]));
        assert!(!buffers_equal(&[1], &[2]));
    }

    #[test]
    fn test_encode_bin() {
        /* binary value includes header + data */
        let mut buf = vec![];
        encode(&MsgValue::Bin(vec![0xDE, 0xAD]), &mut buf);
        assert_eq!(buf[0], 0xc4);
        assert_eq!(buf[1], 2);
    }
}
