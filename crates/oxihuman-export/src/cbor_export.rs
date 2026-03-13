// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! CBOR (Concise Binary Object Representation) encoding stub.

/// CBOR major types.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CborMajor {
    Uint = 0,
    NInt = 1,
    Bytes = 2,
    Text = 3,
    Array = 4,
    Map = 5,
    Tag = 6,
    Simple = 7,
}

/// Encode a CBOR header byte.
#[allow(dead_code)]
pub fn cbor_header(major: CborMajor, additional: u8) -> u8 {
    ((major as u8) << 5) | (additional & 0x1F)
}

/// Encode an unsigned integer as CBOR.
#[allow(dead_code)]
pub fn encode_uint(val: u64) -> Vec<u8> {
    cbor_encode_head(CborMajor::Uint, val)
}

/// Encode a text string as CBOR.
#[allow(dead_code)]
pub fn encode_text(s: &str) -> Vec<u8> {
    let mut out = cbor_encode_head(CborMajor::Text, s.len() as u64);
    out.extend_from_slice(s.as_bytes());
    out
}

/// Encode a byte string as CBOR.
#[allow(dead_code)]
pub fn encode_bytes(data: &[u8]) -> Vec<u8> {
    let mut out = cbor_encode_head(CborMajor::Bytes, data.len() as u64);
    out.extend_from_slice(data);
    out
}

/// Encode CBOR array header with n elements.
#[allow(dead_code)]
pub fn encode_array_header(n: usize) -> Vec<u8> {
    cbor_encode_head(CborMajor::Array, n as u64)
}

/// Encode CBOR map header with n pairs.
#[allow(dead_code)]
pub fn encode_map_header(n: usize) -> Vec<u8> {
    cbor_encode_head(CborMajor::Map, n as u64)
}

/// Encode a CBOR boolean.
#[allow(dead_code)]
pub fn encode_bool(val: bool) -> Vec<u8> {
    vec![cbor_header(CborMajor::Simple, if val { 21 } else { 20 })]
}

/// Encode CBOR null.
#[allow(dead_code)]
pub fn encode_null() -> Vec<u8> {
    vec![cbor_header(CborMajor::Simple, 22)]
}

/// Encode a 32-bit float as CBOR (major 7, additional 26).
#[allow(dead_code)]
pub fn encode_f32(val: f32) -> Vec<u8> {
    let mut out = vec![cbor_header(CborMajor::Simple, 26)];
    out.extend_from_slice(&val.to_bits().to_be_bytes());
    out
}

/// Byte length of a CBOR-encoded uint.
#[allow(dead_code)]
pub fn uint_byte_len(val: u64) -> usize {
    encode_uint(val).len()
}

fn cbor_encode_head(major: CborMajor, val: u64) -> Vec<u8> {
    if val <= 23 {
        vec![cbor_header(major, val as u8)]
    } else if val <= 0xFF {
        vec![cbor_header(major, 24), val as u8]
    } else if val <= 0xFFFF {
        let bytes = (val as u16).to_be_bytes();
        vec![cbor_header(major, 25), bytes[0], bytes[1]]
    } else if val <= 0xFFFF_FFFF {
        let bytes = (val as u32).to_be_bytes();
        let mut out = vec![cbor_header(major, 26)];
        out.extend_from_slice(&bytes);
        out
    } else {
        let bytes = val.to_be_bytes();
        let mut out = vec![cbor_header(major, 27)];
        out.extend_from_slice(&bytes);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_uint_small() {
        let b = encode_uint(0);
        assert_eq!(b.len(), 1);
        assert_eq!(b[0], 0x00);
    }

    #[test]
    fn encode_uint_23() {
        let b = encode_uint(23);
        assert_eq!(b.len(), 1);
        assert_eq!(b[0], 23);
    }

    #[test]
    fn encode_uint_24() {
        let b = encode_uint(24);
        assert_eq!(b.len(), 2);
        assert_eq!(b[0], cbor_header(CborMajor::Uint, 24));
    }

    #[test]
    fn encode_text_hello() {
        let b = encode_text("hi");
        assert_eq!(b.len(), 3);
        assert_eq!(b[1..], b"hi"[..]);
    }

    #[test]
    fn encode_bytes_nonempty() {
        let b = encode_bytes(&[0xDE, 0xAD]);
        assert_eq!(b.len(), 3);
    }

    #[test]
    fn encode_bool_true() {
        let b = encode_bool(true);
        assert_eq!(b[0], cbor_header(CborMajor::Simple, 21));
    }

    #[test]
    fn encode_bool_false() {
        let b = encode_bool(false);
        assert_eq!(b[0], cbor_header(CborMajor::Simple, 20));
    }

    #[test]
    fn encode_null_one_byte() {
        let b = encode_null();
        assert_eq!(b.len(), 1);
    }

    #[test]
    fn encode_f32_five_bytes() {
        let b = encode_f32(1.0);
        assert_eq!(b.len(), 5);
    }

    #[test]
    fn array_header_correct() {
        let b = encode_array_header(3);
        assert_eq!(b[0], cbor_header(CborMajor::Array, 3));
    }
}
