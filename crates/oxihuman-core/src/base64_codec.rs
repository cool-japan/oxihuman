// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Base64 encode/decode (standard alphabet, no external deps).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Base64Config {
    pub url_safe: bool,
}

#[allow(dead_code)]
pub fn default_base64_config() -> Base64Config {
    Base64Config { url_safe: false }
}

const STANDARD: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const URL_SAFE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

#[allow(dead_code)]
fn alphabet(url_safe: bool) -> &'static [u8; 64] {
    if url_safe {
        URL_SAFE
    } else {
        STANDARD
    }
}

#[allow(dead_code)]
pub fn base64_encode(bytes: &[u8]) -> String {
    base64_encode_with_config(bytes, false)
}

#[allow(dead_code)]
fn base64_encode_with_config(bytes: &[u8], url_safe: bool) -> String {
    let table = alphabet(url_safe);
    let mut out = Vec::with_capacity(base64_encoded_len(bytes.len()));
    let mut i = 0;
    while i + 3 <= bytes.len() {
        let b0 = bytes[i] as u32;
        let b1 = bytes[i + 1] as u32;
        let b2 = bytes[i + 2] as u32;
        out.push(table[((b0 >> 2) & 0x3f) as usize]);
        out.push(table[((b0 << 4 | b1 >> 4) & 0x3f) as usize]);
        out.push(table[((b1 << 2 | b2 >> 6) & 0x3f) as usize]);
        out.push(table[(b2 & 0x3f) as usize]);
        i += 3;
    }
    let rem = bytes.len() - i;
    if rem == 1 {
        let b0 = bytes[i] as u32;
        out.push(table[((b0 >> 2) & 0x3f) as usize]);
        out.push(table[((b0 << 4) & 0x3f) as usize]);
        out.push(b'=');
        out.push(b'=');
    } else if rem == 2 {
        let b0 = bytes[i] as u32;
        let b1 = bytes[i + 1] as u32;
        out.push(table[((b0 >> 2) & 0x3f) as usize]);
        out.push(table[((b0 << 4 | b1 >> 4) & 0x3f) as usize]);
        out.push(table[((b1 << 2) & 0x3f) as usize]);
        out.push(b'=');
    }
    // SAFETY: all bytes are valid ASCII
    unsafe { String::from_utf8_unchecked(out) }
}

#[allow(dead_code)]
pub fn base64_decode(s: &str) -> Result<Vec<u8>, String> {
    let s = s.trim_end_matches('=');
    let mut out = Vec::with_capacity(base64_decoded_len(s.len() + (4 - s.len() % 4) % 4));
    let table: [i8; 256] = {
        let mut t = [-1i8; 256];
        for (i, &b) in STANDARD.iter().enumerate() {
            t[b as usize] = i as i8;
        }
        // Also support url-safe chars
        t[b'-' as usize] = 62;
        t[b'_' as usize] = 63;
        t
    };
    let bytes = s.as_bytes();
    let mut i = 0;
    while i + 4 <= bytes.len() {
        let v: Vec<i8> = (0..4).map(|j| table[bytes[i + j] as usize]).collect();
        if v.iter().any(|&x| x < 0) {
            return Err("invalid base64 character".to_string());
        }
        let (c0, c1, c2, c3) = (v[0] as u8, v[1] as u8, v[2] as u8, v[3] as u8);
        out.push((c0 << 2) | (c1 >> 4));
        out.push((c1 << 4) | (c2 >> 2));
        out.push((c2 << 6) | c3);
        i += 4;
    }
    let rem = bytes.len() - i;
    if rem == 2 {
        let (c0, c1) = (table[bytes[i] as usize], table[bytes[i + 1] as usize]);
        if c0 < 0 || c1 < 0 {
            return Err("invalid base64 character".to_string());
        }
        out.push(((c0 as u8) << 2) | ((c1 as u8) >> 4));
    } else if rem == 3 {
        let (c0, c1, c2) = (
            table[bytes[i] as usize],
            table[bytes[i + 1] as usize],
            table[bytes[i + 2] as usize],
        );
        if c0 < 0 || c1 < 0 || c2 < 0 {
            return Err("invalid base64 character".to_string());
        }
        out.push(((c0 as u8) << 2) | ((c1 as u8) >> 4));
        out.push(((c1 as u8) << 4) | ((c2 as u8) >> 2));
    } else if rem == 1 {
        return Err("invalid base64 length".to_string());
    }
    Ok(out)
}

#[allow(dead_code)]
pub fn base64_encode_str(s: &str) -> String {
    base64_encode(s.as_bytes())
}

#[allow(dead_code)]
pub fn base64_is_valid(s: &str) -> bool {
    base64_decode(s).is_ok()
}

#[allow(dead_code)]
pub fn base64_encoded_len(byte_len: usize) -> usize {
    byte_len.div_ceil(3) * 4
}

#[allow(dead_code)]
pub fn base64_decoded_len(encoded_len: usize) -> usize {
    (encoded_len / 4) * 3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_base64_config();
        assert!(!cfg.url_safe);
    }

    #[test]
    fn test_encode_empty() {
        assert_eq!(base64_encode(b""), "");
    }

    #[test]
    fn test_encode_hello() {
        assert_eq!(base64_encode(b"hello"), "aGVsbG8=");
    }

    #[test]
    fn test_roundtrip_ascii() {
        let original = b"Hello, World!";
        let encoded = base64_encode(original);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_roundtrip_binary() {
        let original: Vec<u8> = (0u8..=255).collect();
        let encoded = base64_encode(&original);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_encode_str() {
        let s = "test";
        let encoded = base64_encode_str(s);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, s.as_bytes());
    }

    #[test]
    fn test_is_valid() {
        assert!(base64_is_valid("aGVsbG8="));
        assert!(!base64_is_valid("!!!invalid!!!"));
    }

    #[test]
    fn test_encoded_len() {
        assert_eq!(base64_encoded_len(3), 4);
        assert_eq!(base64_encoded_len(4), 8);
    }

    #[test]
    fn test_decoded_len() {
        assert_eq!(base64_decoded_len(4), 3);
        assert_eq!(base64_decoded_len(8), 6);
    }
}
