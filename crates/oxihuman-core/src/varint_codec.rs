// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Variable-length integer encoding (LEB128 style).

#![allow(dead_code)]

/// Encode an unsigned 32-bit integer using LEB128 variable-length encoding.
#[allow(dead_code)]
pub fn encode_varint_u32(mut val: u32) -> Vec<u8> {
    let mut out = Vec::with_capacity(5);
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

/// Decode an unsigned 32-bit integer from LEB128 bytes.
/// Returns `(value, bytes_consumed)` or `None` on error.
#[allow(dead_code)]
pub fn decode_varint_u32(data: &[u8]) -> Option<(u32, usize)> {
    let mut result: u32 = 0;
    let mut shift = 0u32;
    for (i, &byte) in data.iter().enumerate() {
        if shift >= 35 {
            return None;
        }
        result |= ((byte & 0x7F) as u32) << shift;
        shift += 7;
        if byte & 0x80 == 0 {
            return Some((result, i + 1));
        }
    }
    None
}

/// Encode a signed 32-bit integer using zigzag + LEB128 encoding.
#[allow(dead_code)]
pub fn encode_varint_i32(val: i32) -> Vec<u8> {
    let zigzag = ((val << 1) ^ (val >> 31)) as u32;
    encode_varint_u32(zigzag)
}

/// Decode a signed 32-bit integer from zigzag + LEB128 bytes.
#[allow(dead_code)]
pub fn decode_varint_i32(data: &[u8]) -> Option<(i32, usize)> {
    let (zigzag, n) = decode_varint_u32(data)?;
    let val = ((zigzag >> 1) as i32) ^ (-((zigzag & 1) as i32));
    Some((val, n))
}

/// Return the number of bytes needed to encode `val` as a varint.
#[allow(dead_code)]
pub fn varint_byte_size(val: u32) -> usize {
    match val {
        0..=127 => 1,
        128..=16383 => 2,
        16384..=2097151 => 3,
        2097152..=268435455 => 4,
        _ => 5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_zero() {
        let enc = encode_varint_u32(0);
        assert_eq!(enc, vec![0]);
    }

    #[test]
    fn encode_one_byte_max() {
        let enc = encode_varint_u32(127);
        assert_eq!(enc, vec![127]);
    }

    #[test]
    fn encode_two_bytes() {
        let enc = encode_varint_u32(128);
        assert_eq!(enc, vec![0x80, 0x01]);
    }

    #[test]
    fn decode_zero() {
        let (val, n) = decode_varint_u32(&[0]).expect("should succeed");
        assert_eq!(val, 0);
        assert_eq!(n, 1);
    }

    #[test]
    fn roundtrip_u32_small() {
        for v in [0u32, 1, 63, 127, 128, 300, 16383, 16384, 0xFFFF] {
            let enc = encode_varint_u32(v);
            let (dec, _) = decode_varint_u32(&enc).expect("should succeed");
            assert_eq!(dec, v);
        }
    }

    #[test]
    fn roundtrip_u32_large() {
        let v = u32::MAX;
        let enc = encode_varint_u32(v);
        let (dec, _) = decode_varint_u32(&enc).expect("should succeed");
        assert_eq!(dec, v);
    }

    #[test]
    fn roundtrip_i32_positive() {
        for v in [0i32, 1, 100, 32767, i32::MAX] {
            let enc = encode_varint_i32(v);
            let (dec, _) = decode_varint_i32(&enc).expect("should succeed");
            assert_eq!(dec, v);
        }
    }

    #[test]
    fn roundtrip_i32_negative() {
        for v in [-1i32, -100, -32768, i32::MIN] {
            let enc = encode_varint_i32(v);
            let (dec, _) = decode_varint_i32(&enc).expect("should succeed");
            assert_eq!(dec, v);
        }
    }

    #[test]
    fn varint_byte_size_correct() {
        assert_eq!(varint_byte_size(0), 1);
        assert_eq!(varint_byte_size(127), 1);
        assert_eq!(varint_byte_size(128), 2);
        assert_eq!(varint_byte_size(16383), 2);
        assert_eq!(varint_byte_size(16384), 3);
    }

    #[test]
    fn decode_empty_returns_none() {
        assert!(decode_varint_u32(&[]).is_none());
    }

    #[test]
    fn decode_truncated_returns_none() {
        // A byte with continuation bit set but no next byte
        assert!(decode_varint_u32(&[0x80]).is_none());
    }
}
