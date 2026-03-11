// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Protobuf varint encode/decode stub.

/// Error type for varint operations.
#[derive(Debug, Clone, PartialEq)]
pub enum VarintError {
    BufferTooShort,
    Overflow,
    TrailingBytes,
}

impl std::fmt::Display for VarintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BufferTooShort => write!(f, "buffer too short for varint"),
            Self::Overflow => write!(f, "varint overflows u64"),
            Self::TrailingBytes => write!(f, "unexpected trailing bytes"),
        }
    }
}

/// Encode a `u64` as a protobuf varint into a byte buffer.
pub fn encode_varint(mut value: u64, buf: &mut Vec<u8>) {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        buf.push(byte);
        if value == 0 {
            break;
        }
    }
}

/// Decode a protobuf varint from a byte slice.
/// Returns `(decoded_value, bytes_consumed)`.
pub fn decode_varint(buf: &[u8]) -> Result<(u64, usize), VarintError> {
    let mut result: u64 = 0;
    let mut shift = 0u32;
    for (i, &byte) in buf.iter().enumerate() {
        if shift >= 64 {
            return Err(VarintError::Overflow);
        }
        result |= ((byte & 0x7F) as u64) << shift;
        if byte & 0x80 == 0 {
            return Ok((result, i + 1));
        }
        shift += 7;
    }
    Err(VarintError::BufferTooShort)
}

/// Encode a `i64` using zigzag encoding then varint.
pub fn encode_zigzag(value: i64, buf: &mut Vec<u8>) {
    let zz = ((value << 1) ^ (value >> 63)) as u64;
    encode_varint(zz, buf);
}

/// Decode a zigzag-encoded varint from a byte slice.
pub fn decode_zigzag(buf: &[u8]) -> Result<(i64, usize), VarintError> {
    let (zz, n) = decode_varint(buf)?;
    let value = ((zz >> 1) as i64) ^ (-((zz & 1) as i64));
    Ok((value, n))
}

/// Return the number of bytes needed to encode a varint.
pub fn varint_size(mut value: u64) -> usize {
    let mut n = 1;
    loop {
        value >>= 7;
        if value == 0 {
            break;
        }
        n += 1;
    }
    n
}

/// Encode then immediately decode, verifying the roundtrip.
pub fn varint_roundtrip_ok(value: u64) -> bool {
    let mut buf = vec![];
    encode_varint(value, &mut buf);
    decode_varint(&buf)
        .map(|(v, _)| v == value)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_zero() {
        /* zero encodes to single 0x00 byte */
        let mut buf = vec![];
        encode_varint(0, &mut buf);
        assert_eq!(buf, &[0x00]);
    }

    #[test]
    fn test_encode_one() {
        /* 1 encodes to 0x01 */
        let mut buf = vec![];
        encode_varint(1, &mut buf);
        assert_eq!(buf, &[0x01]);
    }

    #[test]
    fn test_decode_single_byte() {
        /* single-byte varint */
        let (v, n) = decode_varint(&[0x05]).unwrap();
        assert_eq!(v, 5);
        assert_eq!(n, 1);
    }

    #[test]
    fn test_roundtrip_small() {
        /* small value roundtrip */
        assert!(varint_roundtrip_ok(42));
    }

    #[test]
    fn test_roundtrip_large() {
        /* large value roundtrip */
        assert!(varint_roundtrip_ok(u64::MAX));
    }

    #[test]
    fn test_varint_size_one_byte() {
        /* values 0..=127 fit in one byte */
        assert_eq!(varint_size(127), 1);
    }

    #[test]
    fn test_varint_size_two_bytes() {
        /* 128 requires two bytes */
        assert_eq!(varint_size(128), 2);
    }

    #[test]
    fn test_zigzag_positive() {
        /* positive zigzag roundtrip */
        let mut buf = vec![];
        encode_zigzag(100, &mut buf);
        let (v, _) = decode_zigzag(&buf).unwrap();
        assert_eq!(v, 100);
    }

    #[test]
    fn test_zigzag_negative() {
        /* negative zigzag roundtrip */
        let mut buf = vec![];
        encode_zigzag(-50, &mut buf);
        let (v, _) = decode_zigzag(&buf).unwrap();
        assert_eq!(v, -50);
    }

    #[test]
    fn test_buffer_too_short() {
        /* truncated varint returns error */
        assert_eq!(
            decode_varint(&[0x80]).unwrap_err(),
            VarintError::BufferTooShort
        );
    }
}
