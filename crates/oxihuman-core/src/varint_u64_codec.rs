// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub fn varint_encoded_size_u64(v: u64) -> usize {
    if v == 0 {
        return 1;
    }
    let bits = 64 - v.leading_zeros() as usize;
    bits.div_ceil(7)
}

pub fn varint_encode_u64(v: u64) -> Vec<u8> {
    let mut val = v;
    let mut out = Vec::with_capacity(varint_encoded_size_u64(v));
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

pub fn varint_decode_u64(data: &[u8]) -> Option<(u64, usize)> {
    let mut val = 0u64;
    let mut shift = 0usize;
    for (i, &b) in data.iter().enumerate() {
        val |= ((b & 0x7F) as u64) << shift;
        shift += 7;
        if b & 0x80 == 0 {
            return Some((val, i + 1));
        }
        if shift >= 64 {
            return None;
        }
    }
    None
}

fn zigzag_encode_i64(v: i64) -> u64 {
    ((v << 1) ^ (v >> 63)) as u64
}

fn zigzag_decode_i64(v: u64) -> i64 {
    ((v >> 1) as i64) ^ (-((v & 1) as i64))
}

pub fn varint_encode_i64(v: i64) -> Vec<u8> {
    varint_encode_u64(zigzag_encode_i64(v))
}

pub fn varint_decode_i64(data: &[u8]) -> Option<(i64, usize)> {
    let (u, n) = varint_decode_u64(data)?;
    Some((zigzag_decode_i64(u), n))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_zero_u64() {
        /* zero encodes to single byte 0x00 */
        assert_eq!(varint_encode_u64(0), vec![0]);
    }

    #[test]
    fn encode_small_u64() {
        /* small value fits in one byte */
        assert_eq!(varint_encode_u64(127), vec![127]);
    }

    #[test]
    fn encode_large_roundtrip_u64() {
        /* large value roundtrip */
        let v = 300u64;
        let enc = varint_encode_u64(v);
        let (dec, _) = varint_decode_u64(&enc).unwrap();
        assert_eq!(dec, v);
    }

    #[test]
    fn decode_consumes_correct_bytes_u64() {
        /* decode returns correct byte count consumed */
        let enc = varint_encode_u64(300);
        let (_, n) = varint_decode_u64(&enc).unwrap();
        assert_eq!(n, enc.len());
    }

    #[test]
    fn signed_positive_roundtrip_i64() {
        /* positive i64 roundtrip */
        let v = 42i64;
        let enc = varint_encode_i64(v);
        let (dec, _) = varint_decode_i64(&enc).unwrap();
        assert_eq!(dec, v);
    }

    #[test]
    fn signed_negative_roundtrip_i64() {
        /* negative i64 roundtrip via zigzag */
        let v = -1i64;
        let enc = varint_encode_i64(v);
        let (dec, _) = varint_decode_i64(&enc).unwrap();
        assert_eq!(dec, v);
    }

    #[test]
    fn encoded_size_u64() {
        /* size estimate is correct */
        assert_eq!(varint_encoded_size_u64(128), 2);
        assert_eq!(varint_encoded_size_u64(0), 1);
    }
}
