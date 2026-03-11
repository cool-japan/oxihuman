// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Hex encode/decode utilities.

/// Encode bytes as a lowercase hex string.
pub fn hex_encode(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Encode bytes as an uppercase hex string.
pub fn hex_encode_upper(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02X}", b)).collect()
}

/// Decode a hex string (lowercase or uppercase) back to bytes.
pub fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    if !s.len().is_multiple_of(2) {
        return Err("hex: odd-length string".to_string());
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let hi = hex_nibble(bytes[i])?;
        let lo = hex_nibble(bytes[i + 1])?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Ok(out)
}

fn hex_nibble(b: u8) -> Result<u8, String> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(format!("hex: invalid character '{}'", b as char)),
    }
}

/// Return whether a string is valid hex (even length, valid chars).
pub fn hex_is_valid(s: &str) -> bool {
    s.len().is_multiple_of(2) && s.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Verify round-trip encode/decode.
pub fn hex_roundtrip_ok(data: &[u8]) -> bool {
    hex_decode(&hex_encode(data))
        .map(|d| d == data)
        .unwrap_or(false)
}

/// Strip optional `0x` prefix from a hex string.
pub fn hex_strip_prefix(s: &str) -> &str {
    s.strip_prefix("0x")
        .or_else(|| s.strip_prefix("0X"))
        .unwrap_or(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_known() {
        /* known encoding */
        assert_eq!(hex_encode(&[0x0F, 0xA0]), "0fa0");
    }

    #[test]
    fn test_encode_upper() {
        /* uppercase variant */
        assert_eq!(hex_encode_upper(&[0xAB, 0xCD]), "ABCD");
    }

    #[test]
    fn test_roundtrip_hello() {
        /* round-trip check */
        assert!(hex_roundtrip_ok(b"hello"));
    }

    #[test]
    fn test_roundtrip_empty() {
        /* empty round-trip */
        assert!(hex_roundtrip_ok(&[]));
    }

    #[test]
    fn test_decode_valid() {
        /* decode known hex */
        assert_eq!(hex_decode("deadbeef").unwrap(), &[0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn test_decode_odd_length() {
        /* odd length returns error */
        assert!(hex_decode("abc").is_err());
    }

    #[test]
    fn test_decode_invalid_char() {
        /* invalid char returns error */
        assert!(hex_decode("zz").is_err());
    }

    #[test]
    fn test_is_valid() {
        /* valid and invalid examples */
        assert!(hex_is_valid("ff00"));
        assert!(!hex_is_valid("gg"));
    }

    #[test]
    fn test_strip_prefix() {
        /* strip 0x prefix */
        assert_eq!(hex_strip_prefix("0xff"), "ff");
        assert_eq!(hex_strip_prefix("ff"), "ff");
    }
}
