// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! CBOR encode/decode stub.

/// Major type codes used in CBOR.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CborMajor {
    Uint = 0,
    Negint = 1,
    Bstr = 2,
    Tstr = 3,
    Array = 4,
    Map = 5,
    Tag = 6,
    Float = 7,
}

/// A CBOR value.
#[derive(Debug, Clone, PartialEq)]
pub enum CborValue {
    Uint(u64),
    Negint(i64),
    Bstr(Vec<u8>),
    Tstr(String),
    Array(Vec<CborValue>),
    Map(Vec<(CborValue, CborValue)>),
    Bool(bool),
    Null,
    Float(f64),
}

/// CBOR codec error.
#[derive(Debug, Clone, PartialEq)]
pub enum CborError {
    UnexpectedEnd,
    InvalidMajor(u8),
    InvalidUtf8,
    NotImplemented(&'static str),
}

impl std::fmt::Display for CborError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedEnd => write!(f, "unexpected end of CBOR input"),
            Self::InvalidMajor(b) => write!(f, "invalid major type: {b}"),
            Self::InvalidUtf8 => write!(f, "invalid UTF-8 in tstr"),
            Self::NotImplemented(s) => write!(f, "not implemented: {s}"),
        }
    }
}

/// Encode a `CborValue` to bytes (stub).
pub fn encode_cbor(val: &CborValue, buf: &mut Vec<u8>) {
    match val {
        CborValue::Uint(n) => encode_uint(*n, 0, buf),
        CborValue::Negint(n) => {
            let encoded = (-1 - n) as u64;
            encode_uint(encoded, 1, buf);
        }
        CborValue::Bstr(b) => {
            encode_uint(b.len() as u64, 2, buf);
            buf.extend_from_slice(b);
        }
        CborValue::Tstr(s) => {
            let bytes = s.as_bytes();
            encode_uint(bytes.len() as u64, 3, buf);
            buf.extend_from_slice(bytes);
        }
        CborValue::Array(arr) => {
            encode_uint(arr.len() as u64, 4, buf);
            for item in arr {
                encode_cbor(item, buf);
            }
        }
        CborValue::Map(map) => {
            encode_uint(map.len() as u64, 5, buf);
            for (k, v) in map {
                encode_cbor(k, buf);
                encode_cbor(v, buf);
            }
        }
        CborValue::Bool(true) => buf.push(0xf5),
        CborValue::Bool(false) => buf.push(0xf4),
        CborValue::Null => buf.push(0xf6),
        CborValue::Float(f) => {
            buf.push(0xfb);
            buf.extend_from_slice(&f.to_bits().to_be_bytes());
        }
    }
}

fn encode_uint(n: u64, major: u8, buf: &mut Vec<u8>) {
    let mt = major << 5;
    if n <= 23 {
        buf.push(mt | n as u8);
    } else if n <= 0xFF {
        buf.push(mt | 24);
        buf.push(n as u8);
    } else if n <= 0xFFFF {
        buf.push(mt | 25);
        buf.extend_from_slice(&(n as u16).to_be_bytes());
    } else if n <= 0xFFFF_FFFF {
        buf.push(mt | 26);
        buf.extend_from_slice(&(n as u32).to_be_bytes());
    } else {
        buf.push(mt | 27);
        buf.extend_from_slice(&n.to_be_bytes());
    }
}

/// Return the byte length of the encoded form.
pub fn cbor_encoded_len(val: &CborValue) -> usize {
    let mut buf = vec![];
    encode_cbor(val, &mut buf);
    buf.len()
}

/// Return `true` if the value is null.
pub fn cbor_is_null(val: &CborValue) -> bool {
    matches!(val, CborValue::Null)
}

/// Return the major type of a CBOR value.
pub fn major_of(val: &CborValue) -> CborMajor {
    match val {
        CborValue::Uint(_) => CborMajor::Uint,
        CborValue::Negint(_) => CborMajor::Negint,
        CborValue::Bstr(_) => CborMajor::Bstr,
        CborValue::Tstr(_) => CborMajor::Tstr,
        CborValue::Array(_) => CborMajor::Array,
        CborValue::Map(_) => CborMajor::Map,
        CborValue::Bool(_) | CborValue::Null | CborValue::Float(_) => CborMajor::Float,
    }
}

/// Count items in a CBOR array value.
pub fn cbor_array_len(val: &CborValue) -> usize {
    if let CborValue::Array(a) = val {
        a.len()
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_null() {
        /* null encodes to 0xf6 */
        let mut buf = vec![];
        encode_cbor(&CborValue::Null, &mut buf);
        assert_eq!(buf, &[0xf6]);
    }

    #[test]
    fn test_encode_bool_true() {
        /* true encodes to 0xf5 */
        let mut buf = vec![];
        encode_cbor(&CborValue::Bool(true), &mut buf);
        assert_eq!(buf, &[0xf5]);
    }

    #[test]
    fn test_encode_uint_small() {
        /* small uint uses single byte */
        let mut buf = vec![];
        encode_cbor(&CborValue::Uint(10), &mut buf);
        assert_eq!(buf, &[10]);
    }

    #[test]
    fn test_encode_tstr() {
        /* text string encodes correctly */
        let mut buf = vec![];
        encode_cbor(&CborValue::Tstr("a".to_string()), &mut buf);
        assert_eq!(buf[0], 0x61); /* major 3, length 1 */
        assert_eq!(buf[1], b'a');
    }

    #[test]
    fn test_cbor_is_null() {
        /* null detection */
        assert!(cbor_is_null(&CborValue::Null));
        assert!(!cbor_is_null(&CborValue::Bool(false)));
    }

    #[test]
    fn test_major_of_uint() {
        /* uint major type */
        assert_eq!(major_of(&CborValue::Uint(0)), CborMajor::Uint);
    }

    #[test]
    fn test_array_len() {
        /* cbor_array_len counts correctly */
        let v = CborValue::Array(vec![CborValue::Null, CborValue::Null]);
        assert_eq!(cbor_array_len(&v), 2);
    }

    #[test]
    fn test_encoded_len_float() {
        /* float is 9 bytes */
        assert_eq!(cbor_encoded_len(&CborValue::Float(0.0)), 9);
    }

    #[test]
    fn test_encode_negint() {
        /* negative integer encodes with major type 1 */
        let mut buf = vec![];
        encode_cbor(&CborValue::Negint(-1), &mut buf);
        assert_eq!(buf[0] >> 5, 1); /* major type 1 */
    }

    #[test]
    fn test_encode_bstr() {
        /* binary string encodes with major type 2 */
        let mut buf = vec![];
        encode_cbor(&CborValue::Bstr(vec![0xAB]), &mut buf);
        assert_eq!(buf[0] >> 5, 2); /* major type 2 */
    }
}
