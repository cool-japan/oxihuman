#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Encoding utilities: hex, base64 (simple), URL encoding.

/// Encoding type marker.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Encoding {
    Hex,
    Base64,
    Url,
}

const HEX_CHARS: &[u8] = b"0123456789abcdef";

/// Encode bytes as a lowercase hex string.
#[allow(dead_code)]
pub fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX_CHARS[(b >> 4) as usize] as char);
        out.push(HEX_CHARS[(b & 0xF) as usize] as char);
    }
    out
}

/// Decode a lowercase hex string into bytes. Returns `None` on invalid input.
#[allow(dead_code)]
pub fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    let chars: Vec<char> = s.chars().collect();
    let mut out = Vec::with_capacity(s.len() / 2);
    let mut i = 0;
    while i + 1 < chars.len() {
        let hi = hex_digit(chars[i])?;
        let lo = hex_digit(chars[i + 1])?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Some(out)
}

fn hex_digit(c: char) -> Option<u8> {
    match c {
        '0'..='9' => Some(c as u8 - b'0'),
        'a'..='f' => Some(c as u8 - b'a' + 10),
        'A'..='F' => Some(c as u8 - b'A' + 10),
        _ => None,
    }
}

const B64_TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Simple base64 encoder (standard alphabet, with padding).
#[allow(dead_code)]
pub fn base64_encode_simple(bytes: &[u8]) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i + 2 < bytes.len() {
        let b0 = bytes[i];
        let b1 = bytes[i + 1];
        let b2 = bytes[i + 2];
        out.push(B64_TABLE[(b0 >> 2) as usize] as char);
        out.push(B64_TABLE[((b0 & 3) << 4 | b1 >> 4) as usize] as char);
        out.push(B64_TABLE[((b1 & 0xF) << 2 | b2 >> 6) as usize] as char);
        out.push(B64_TABLE[(b2 & 0x3F) as usize] as char);
        i += 3;
    }
    let rem = bytes.len() - i;
    if rem == 1 {
        let b0 = bytes[i];
        out.push(B64_TABLE[(b0 >> 2) as usize] as char);
        out.push(B64_TABLE[((b0 & 3) << 4) as usize] as char);
        out.push_str("==");
    } else if rem == 2 {
        let b0 = bytes[i];
        let b1 = bytes[i + 1];
        out.push(B64_TABLE[(b0 >> 2) as usize] as char);
        out.push(B64_TABLE[((b0 & 3) << 4 | b1 >> 4) as usize] as char);
        out.push(B64_TABLE[((b1 & 0xF) << 2) as usize] as char);
        out.push('=');
    }
    out
}

/// Simple base64 decoder. Returns `None` on invalid input.
#[allow(dead_code)]
pub fn base64_decode_simple(s: &str) -> Option<Vec<u8>> {
    let s = s.trim_end_matches('=');
    let decode_char = |c: char| -> Option<u8> {
        match c {
            'A'..='Z' => Some(c as u8 - b'A'),
            'a'..='z' => Some(c as u8 - b'a' + 26),
            '0'..='9' => Some(c as u8 - b'0' + 52),
            '+' => Some(62),
            '/' => Some(63),
            _ => None,
        }
    };
    let chars: Vec<u8> = s.chars().map(decode_char).collect::<Option<Vec<_>>>()?;
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < chars.len() {
        let b0 = chars[i];
        let b1 = chars[i + 1];
        out.push((b0 << 2) | (b1 >> 4));
        if i + 2 < chars.len() {
            let b2 = chars[i + 2];
            out.push((b1 << 4) | (b2 >> 2));
            if i + 3 < chars.len() {
                let b3 = chars[i + 3];
                out.push((b2 << 6) | b3);
            }
        }
        i += 4;
    }
    Some(out)
}

/// Full RFC 3986 percent-encoder.
///
/// Unreserved characters (A–Z a–z 0–9 `-` `_` `.` `~`) are passed through
/// unchanged; every other byte is encoded as `%XX` (uppercase hex).
///
/// The function name is kept as `url_encode_stub` for API compatibility.
#[allow(dead_code)]
pub fn url_encode_stub(s: &str) -> String {
    // Upper-case hex digits for percent-encoding (RFC 3986 §2.1 recommends
    // upper case for normalisation).
    const HEX_UPPER: &[u8] = b"0123456789ABCDEF";
    let mut out = String::with_capacity(s.len() * 3);
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => {
                out.push('%');
                out.push(HEX_UPPER[(byte >> 4) as usize] as char);
                out.push(HEX_UPPER[(byte & 0xF) as usize] as char);
            }
        }
    }
    out
}

/// Full RFC 3986 percent-decoder.
///
/// Converts `%XX` sequences (case-insensitive hex digits) back to their
/// original bytes; any sequence that cannot be decoded is passed through
/// verbatim (lenient mode).  The resulting byte vector is interpreted as
/// UTF-8 with replacement on invalid sequences.
///
/// The function name is kept as `url_decode_stub` for API compatibility.
#[allow(dead_code)]
pub fn url_decode_stub(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hi = hex_digit(bytes[i + 1] as char);
            let lo = hex_digit(bytes[i + 2] as char);
            if let (Some(h), Some(l)) = (hi, lo) {
                out.push((h << 4) | l);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Count UTF-8 characters (not bytes) in a string.
#[allow(dead_code)]
pub fn utf8_len_chars(s: &str) -> usize {
    s.chars().count()
}

/// Check if a byte slice is valid UTF-8.
#[allow(dead_code)]
pub fn encoding_is_valid_utf8(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_encode() {
        assert_eq!(hex_encode(&[0xDE, 0xAD, 0xBE, 0xEF]), "deadbeef");
        assert_eq!(hex_encode(&[]), "");
    }

    #[test]
    fn test_hex_decode() {
        assert_eq!(hex_decode("deadbeef"), Some(vec![0xDE, 0xAD, 0xBE, 0xEF]));
        assert_eq!(hex_decode(""), Some(vec![]));
        assert!(hex_decode("xyz").is_none());
        assert!(hex_decode("a").is_none());
    }

    #[test]
    fn test_hex_roundtrip() {
        let data = vec![0u8, 1, 127, 255];
        assert_eq!(hex_decode(&hex_encode(&data)).expect("should succeed"), data);
    }

    #[test]
    fn test_base64_encode() {
        assert_eq!(base64_encode_simple(b"Man"), "TWFu");
        assert_eq!(base64_encode_simple(b""), "");
    }

    #[test]
    fn test_base64_decode() {
        let enc = base64_encode_simple(b"hello");
        let dec = base64_decode_simple(&enc).expect("should succeed");
        assert_eq!(dec, b"hello");
    }

    #[test]
    fn test_url_encode_stub() {
        // Space → %20
        assert_eq!(url_encode_stub("hello world"), "hello%20world");
        // & and = are not unreserved; must be encoded.
        assert_eq!(url_encode_stub("a&b=c"), "a%26b%3Dc");
    }

    #[test]
    fn test_url_decode_stub() {
        // %20 → space
        assert_eq!(url_decode_stub("hello%20world"), "hello world");
        // roundtrip for encoded punctuation
        assert_eq!(url_decode_stub("a%26b%3Dc"), "a&b=c");
    }

    #[test]
    fn test_url_encode_unreserved_passthrough() {
        // Unreserved characters must not be encoded.
        let unreserved = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_.~";
        assert_eq!(url_encode_stub(unreserved), unreserved);
    }

    #[test]
    fn test_url_roundtrip() {
        let original = "https://example.com/path?q=hello world&lang=en";
        let encoded = url_encode_stub(original);
        let decoded = url_decode_stub(&encoded);
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_url_decode_invalid_percent_passthrough() {
        // A lone '%' with no valid hex digits should be passed through as-is.
        assert_eq!(url_decode_stub("100%pure"), "100%pure");
        // %GG is not valid hex, should be left verbatim.
        assert_eq!(url_decode_stub("%GG"), "%GG");
    }

    #[test]
    fn test_utf8_len_chars() {
        assert_eq!(utf8_len_chars("hello"), 5);
        assert_eq!(utf8_len_chars(""), 0);
    }

    #[test]
    fn test_encoding_is_valid_utf8() {
        assert!(encoding_is_valid_utf8(b"hello"));
        assert!(!encoding_is_valid_utf8(&[0xFF, 0xFE]));
    }
}
