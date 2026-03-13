// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub fn hex_encode_new(data: &[u8]) -> String {
    let mut s = String::with_capacity(data.len() * 2);
    for &b in data {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

pub fn hex_encode_upper_new(data: &[u8]) -> String {
    let mut s = String::with_capacity(data.len() * 2);
    for &b in data {
        s.push_str(&format!("{:02X}", b));
    }
    s
}

pub fn hex_is_valid_new(s: &str) -> bool {
    s.len().is_multiple_of(2) && s.chars().all(|c| c.is_ascii_hexdigit())
}

pub fn hex_byte_count_new(hex_str: &str) -> usize {
    hex_str.len() / 2
}

pub fn hex_decode_new(s: &str) -> Result<Vec<u8>, &'static str> {
    if !hex_is_valid_new(s) {
        return Err("invalid hex");
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let b = s.as_bytes();
    let mut i = 0;
    while i + 1 < b.len() {
        let hi = hex_nibble(b[i])?;
        let lo = hex_nibble(b[i + 1])?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Ok(out)
}

fn hex_nibble(c: u8) -> Result<u8, &'static str> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err("invalid hex char"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_known() {
        /* known bytes encode correctly */
        assert_eq!(hex_encode_new(&[0xDE, 0xAD]), "dead");
    }

    #[test]
    fn encode_upper_known() {
        /* upper-case variant */
        assert_eq!(hex_encode_upper_new(&[0xBE, 0xEF]), "BEEF");
    }

    #[test]
    fn decode_roundtrip() {
        /* encode then decode restores original */
        let data = b"oxihuman";
        assert_eq!(
            hex_decode_new(&hex_encode_new(data)).expect("should succeed"),
            data
        );
    }

    #[test]
    fn is_valid_true() {
        /* even-length all-hex chars passes */
        assert!(hex_is_valid_new("deadbeef"));
    }

    #[test]
    fn is_valid_false_odd_len() {
        /* odd length fails */
        assert!(!hex_is_valid_new("abc"));
    }

    #[test]
    fn byte_count() {
        /* byte count is half the string length */
        assert_eq!(hex_byte_count_new("deadbeef"), 4);
    }
}
