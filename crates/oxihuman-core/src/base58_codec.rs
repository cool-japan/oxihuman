// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Base58 encode/decode (Bitcoin alphabet).

const ALPHABET: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

/// Encode bytes to Base58.
pub fn base58_encode(data: &[u8]) -> String {
    /* count leading zeros */
    let leading_zeros = data.iter().take_while(|&&b| b == 0).count();
    let mut digits: Vec<u8> = Vec::new();
    for &byte in data {
        let mut carry = byte as u32;
        for d in digits.iter_mut() {
            carry += (*d as u32) << 8;
            *d = (carry % 58) as u8;
            carry /= 58;
        }
        while carry > 0 {
            digits.push((carry % 58) as u8);
            carry /= 58;
        }
    }
    let mut out = String::with_capacity(leading_zeros + digits.len());
    for _ in 0..leading_zeros {
        out.push('1');
    }
    for &d in digits.iter().rev() {
        out.push(ALPHABET[d as usize] as char);
    }
    out
}

/// Decode a Base58 string back to bytes.
pub fn base58_decode(s: &str) -> Result<Vec<u8>, String> {
    /* build lookup table */
    let mut lookup = [0i16; 256];
    lookup.iter_mut().for_each(|v| *v = -1);
    for (i, &c) in ALPHABET.iter().enumerate() {
        lookup[c as usize] = i as i16;
    }

    let leading_ones = s.chars().take_while(|&c| c == '1').count();
    let mut bytes: Vec<u8> = Vec::new();

    for ch in s.chars() {
        let v = lookup[ch as usize];
        if v < 0 {
            return Err(format!("base58: invalid character '{}'", ch));
        }
        let mut carry = v as u32;
        for b in bytes.iter_mut() {
            carry += (*b as u32) * 58;
            *b = (carry & 0xFF) as u8;
            carry >>= 8;
        }
        while carry > 0 {
            bytes.push((carry & 0xFF) as u8);
            carry >>= 8;
        }
    }

    let mut out = vec![0u8; leading_ones];
    for b in bytes.iter().rev() {
        out.push(*b);
    }
    Ok(out)
}

/// Verify that a Base58 string contains only valid characters.
pub fn base58_is_valid(s: &str) -> bool {
    s.chars().all(|c| ALPHABET.contains(&(c as u8)))
}

/// Return the expected encoded length (estimate).
pub fn base58_encoded_len_estimate(byte_len: usize) -> usize {
    (byte_len * 138 / 100) + 1
}

/// Verify round-trip encode/decode.
pub fn base58_roundtrip_ok(data: &[u8]) -> bool {
    base58_decode(&base58_encode(data))
        .map(|d| d == data)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_known() {
        /* Bitcoin address leading '1' for zero byte */
        let encoded = base58_encode(&[0]);
        assert_eq!(encoded, "1");
    }

    #[test]
    fn test_roundtrip_hello() {
        /* basic round-trip */
        assert!(base58_roundtrip_ok(b"hello"));
    }

    #[test]
    fn test_roundtrip_empty() {
        /* empty input round-trip */
        assert!(base58_roundtrip_ok(&[]));
    }

    #[test]
    fn test_roundtrip_binary() {
        /* binary data round-trip */
        let data = vec![0u8, 1, 127, 255, 0, 42];
        assert!(base58_roundtrip_ok(&data));
    }

    #[test]
    fn test_is_valid_true() {
        /* valid characters pass */
        assert!(base58_is_valid("123456789ABC"));
    }

    #[test]
    fn test_is_valid_false() {
        /* '0', 'O', 'I', 'l' are not in Base58 */
        assert!(!base58_is_valid("0"));
    }

    #[test]
    fn test_decode_invalid_char() {
        /* invalid char returns error */
        assert!(base58_decode("!invalid!").is_err());
    }

    #[test]
    fn test_encoded_len_estimate() {
        /* estimate is positive */
        assert!(base58_encoded_len_estimate(10) > 0);
    }

    #[test]
    fn test_leading_zeros() {
        /* leading zero bytes map to '1' prefix */
        let enc = base58_encode(&[0, 0, 1]);
        assert!(enc.starts_with("11"));
    }
}
