// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! URL percent-encoding utilities.

/// Percent-encode a byte string (encodes all non-unreserved characters).
pub fn url_encode(s: &str) -> String {
    /* unreserved chars per RFC 3986 */
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push('%');
            out.push_str(&format!("{:02X}", b));
        }
    }
    out
}

/// Decode a percent-encoded string.
pub fn url_decode(s: &str) -> Result<String, String> {
    let mut out = Vec::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if i + 2 >= bytes.len() {
                return Err("url_decode: incomplete percent sequence".to_string());
            }
            let hi = hex_nibble(bytes[i + 1])?;
            let lo = hex_nibble(bytes[i + 2])?;
            out.push((hi << 4) | lo);
            i += 3;
        } else if bytes[i] == b'+' {
            /* '+' decodes to space in form encoding */
            out.push(b' ');
            i += 1;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).map_err(|e| format!("url_decode: utf8 error: {}", e))
}

fn hex_nibble(b: u8) -> Result<u8, String> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(format!("url_decode: invalid hex char '{}'", b as char)),
    }
}

/// Encode only the query-string component (space → `+`).
pub fn url_encode_query(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else if b == b' ' {
            out.push('+');
        } else {
            out.push('%');
            out.push_str(&format!("{:02X}", b));
        }
    }
    out
}

/// Return true if a string contains no characters requiring encoding.
pub fn url_is_safe(s: &str) -> bool {
    s.bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~'))
}

/// Verify round-trip encode/decode.
pub fn url_roundtrip_ok(s: &str) -> bool {
    url_decode(&url_encode(s)).map(|d| d == s).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_safe_chars() {
        /* safe chars not encoded */
        assert_eq!(url_encode("hello"), "hello");
    }

    #[test]
    fn test_encode_space() {
        /* space encodes to %20 */
        assert_eq!(url_encode(" "), "%20");
    }

    #[test]
    fn test_encode_special() {
        /* special chars encoded */
        let enc = url_encode("a&b=c");
        assert!(enc.contains('%'));
    }

    #[test]
    fn test_decode_percent() {
        /* percent sequence decoded */
        assert_eq!(url_decode("%41%42%43").unwrap(), "ABC");
    }

    #[test]
    fn test_decode_plus() {
        /* plus decoded to space */
        assert_eq!(url_decode("hello+world").unwrap(), "hello world");
    }

    #[test]
    fn test_roundtrip_unicode_bytes() {
        /* non-ASCII round-trips */
        let s = "caf\u{00E9}"; /* café */
        assert!(url_roundtrip_ok(s));
    }

    #[test]
    fn test_decode_invalid_hex() {
        /* invalid hex char errors */
        assert!(url_decode("%ZZ").is_err());
    }

    #[test]
    fn test_query_encode_space() {
        /* space → '+' in query encoding */
        assert_eq!(url_encode_query("hello world"), "hello+world");
    }

    #[test]
    fn test_is_safe_true() {
        /* alphanumeric is safe */
        assert!(url_is_safe("hello123"));
    }

    #[test]
    fn test_is_safe_false() {
        /* space is not safe */
        assert!(!url_is_safe("hello world"));
    }
}
