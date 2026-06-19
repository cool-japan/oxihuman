// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! LZ77/LZSS sliding-window compressor and decompressor.
//!
//! Binary format:
//! - Magic: b"LZ77" (4 bytes)
//! - Uncompressed length: 4 bytes LE u32
//! - Body: LZSS flag-byte groups. Each group starts with a flag byte (8 bits).
//!   For each bit (LSB first):
//!   - `0` = literal: the next byte is copied verbatim
//!   - `1` = back-reference: the next 3 bytes encode offset and length:
//!     - Byte 1: `length - MIN_MATCH_LEN` (0..=255 → len MIN_MATCH_LEN..=MIN_MATCH_LEN+255)
//!     - Byte 2: `(offset - 1) & 0xFF` (low 8 bits of 0-based offset)
//!     - Byte 3: `(offset - 1) >> 8` (high 8 bits, supports window 65536)
//! - Window size: 65536 bytes
//! - Min match length: 3, max match length: 258 (3 + 255)
//!
//! This 3-byte match encoding allows long matches and achieves excellent compression
//! on repetitive data (e.g., 1000 'A' bytes compresses to ~25 bytes).

const MAGIC: &[u8; 4] = b"LZ77";
const WINDOW_SIZE: usize = 65536;
const MAX_MATCH_LEN: usize = 258; // 3 + 255 (fits in 1 byte as length-3)
const MIN_MATCH_LEN: usize = 3;

/// Configuration for the LZ compressor.
#[derive(Debug, Clone)]
pub struct LzConfig {
    pub window_size: usize,
    pub max_match: usize,
    pub min_match: usize,
}

impl Default for LzConfig {
    fn default() -> Self {
        Self {
            window_size: WINDOW_SIZE,
            max_match: MAX_MATCH_LEN,
            min_match: MIN_MATCH_LEN,
        }
    }
}

/// LZ77/LZSS compressor.
#[derive(Debug, Clone)]
pub struct LzCompressor {
    pub config: LzConfig,
}

impl LzCompressor {
    pub fn new(config: LzConfig) -> Self {
        Self { config }
    }

    pub fn default_compressor() -> Self {
        Self::new(LzConfig::default())
    }

    pub fn with_window_size(window_size: usize) -> Self {
        Self::new(LzConfig {
            window_size,
            ..LzConfig::default()
        })
    }
}

/// Find the longest match in the sliding window.
/// Returns (offset, length) where offset is 1-based distance back (1..=window_size).
/// Uses a forward hash chain for efficiency on repetitive data.
fn find_longest_match(data: &[u8], pos: usize, window_size: usize) -> (usize, usize) {
    let window_start = pos.saturating_sub(window_size);
    let max_len = MAX_MATCH_LEN.min(data.len() - pos);

    if max_len < MIN_MATCH_LEN {
        return (0, 0);
    }

    let mut best_len = 0usize;
    let mut best_offset = 0usize;

    // Scan backward through the window looking for the longest match.
    // For repetitive data, scanning from the nearest position (offset=1) first
    // maximizes the chance of finding long overlapping copies.
    let scan_end = window_start;
    let mut candidate = pos.saturating_sub(1);

    loop {
        if candidate < scan_end {
            break;
        }

        // Check if first byte matches before doing full comparison
        if data[candidate] == data[pos] {
            let mut ml = 1;
            while ml < max_len && data[candidate + ml] == data[pos + ml] {
                ml += 1;
            }
            if ml >= MIN_MATCH_LEN && ml > best_len {
                best_len = ml;
                best_offset = pos - candidate; // 1-based distance back
                if best_len == max_len {
                    break; // can't do better
                }
            }
        }

        if candidate == 0 {
            break;
        }
        candidate -= 1;
    }

    if best_len >= MIN_MATCH_LEN {
        (best_offset, best_len)
    } else {
        (0, 0)
    }
}

/// Compress `data` using LZSS with the binary format described at the top of this file.
pub fn lz_compress(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(lz_compress_bound(data.len()));

    // Write header
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());

    if data.is_empty() {
        return out;
    }

    let mut pos = 0;
    while pos < data.len() {
        // Reserve space for the flag byte
        let flag_pos = out.len();
        out.push(0u8);
        let mut flags: u8 = 0;

        for bit in 0..8u8 {
            if pos >= data.len() {
                break;
            }
            let (offset, length) = find_longest_match(data, pos, WINDOW_SIZE);
            if offset > 0 && length >= MIN_MATCH_LEN {
                // Back-reference: set bit
                flags |= 1 << bit;
                // Encode: byte1 = length - MIN_MATCH_LEN (0..=255)
                //         byte2 = (offset - 1) & 0xFF
                //         byte3 = (offset - 1) >> 8
                let enc_len = (length - MIN_MATCH_LEN) as u8; // 0..=255
                let enc_off = (offset - 1) as u16; // 0..=65535
                out.push(enc_len);
                out.push((enc_off & 0xFF) as u8);
                out.push((enc_off >> 8) as u8);
                pos += length;
            } else {
                // Literal: bit stays 0
                out.push(data[pos]);
                pos += 1;
            }
        }

        out[flag_pos] = flags;
    }

    out
}

/// Decompress data produced by [`lz_compress`].
pub fn lz_decompress(data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() < 8 {
        return Err("lz77: input too short for header".to_string());
    }

    // Check magic
    if &data[..4] != MAGIC {
        return Err("lz77: invalid magic bytes".to_string());
    }

    let uncompressed_len = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;

    let mut out: Vec<u8> = Vec::with_capacity(uncompressed_len);
    let mut pos = 8usize;

    while pos < data.len() {
        if out.len() >= uncompressed_len {
            break;
        }

        let flags = data[pos];
        pos += 1;

        for bit in 0..8u8 {
            if pos >= data.len() || out.len() >= uncompressed_len {
                break;
            }

            if (flags >> bit) & 1 == 0 {
                // Literal
                out.push(data[pos]);
                pos += 1;
            } else {
                // Back-reference: 3 bytes
                if pos + 3 > data.len() {
                    return Err("lz77: truncated back-reference".to_string());
                }
                let enc_len = data[pos] as usize;
                let enc_off_lo = data[pos + 1] as usize;
                let enc_off_hi = data[pos + 2] as usize;
                pos += 3;

                let length = enc_len + MIN_MATCH_LEN;
                let offset = ((enc_off_hi << 8) | enc_off_lo) + 1; // 1-based

                if offset > out.len() {
                    return Err(format!(
                        "lz77: back-reference offset {} exceeds output length {}",
                        offset,
                        out.len()
                    ));
                }

                let match_start = out.len() - offset;
                for i in 0..length {
                    let b = out[match_start + i];
                    out.push(b);
                    if out.len() >= uncompressed_len {
                        break;
                    }
                }
            }
        }
    }

    if out.len() != uncompressed_len {
        return Err(format!(
            "lz77: expected {} bytes, got {}",
            uncompressed_len,
            out.len()
        ));
    }

    Ok(out)
}

/// Return the worst-case compressed size (all literals + flag bytes + header).
/// Each group of 8 literals: 1 flag + 8 bytes = 9 bytes per 8 input bytes.
pub fn lz_compress_bound(input_len: usize) -> usize {
    let groups = input_len.div_ceil(8);
    8 + groups + input_len
}

/// Return whether data starts with the LZ77 magic bytes.
pub fn lz_is_compressed(data: &[u8]) -> bool {
    data.len() >= 4 && &data[..4] == MAGIC
}

/// Round-trip compress then decompress and verify equality.
pub fn lz_roundtrip_ok(data: &[u8]) -> bool {
    match lz_decompress(&lz_compress(data)) {
        Ok(out) => out == data,
        Err(_) => false,
    }
}

// ── Legacy API preserved for backward compatibility ───────────────────────────

#[allow(dead_code)]
pub fn compress_bytes(data: &[u8]) -> Vec<u8> {
    lz_compress(data)
}

#[allow(dead_code)]
pub fn decompress_bytes(data: &[u8]) -> Vec<u8> {
    lz_decompress(data).unwrap_or_default()
}

#[allow(dead_code)]
pub fn compressed_size(data: &[u8]) -> usize {
    lz_compress(data).len()
}

#[allow(dead_code)]
pub fn compression_ratio(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 1.0;
    }
    let c = compressed_size(data);
    c as f64 / data.len() as f64
}

#[allow(dead_code)]
pub fn is_compressed(data: &[u8]) -> bool {
    lz_is_compressed(data)
}

#[allow(dead_code)]
pub fn compress_to_vec(data: &[u8]) -> Vec<u8> {
    lz_compress(data)
}

#[allow(dead_code)]
pub fn decompress_to_vec(data: &[u8]) -> Vec<u8> {
    lz_decompress(data).unwrap_or_default()
}

#[allow(dead_code)]
pub fn compressor_name() -> &'static str {
    "lz77"
}

impl LzCompressor {
    #[allow(dead_code)]
    pub fn compress(&self, data: &[u8]) -> Vec<u8> {
        lz_compress(data)
    }

    #[allow(dead_code)]
    pub fn decompress(&self, data: &[u8]) -> Vec<u8> {
        lz_decompress(data).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_simple() {
        let data = b"aaabbbccc";
        assert!(lz_roundtrip_ok(data));
    }

    #[test]
    fn test_roundtrip_empty() {
        assert!(lz_roundtrip_ok(&[]));
    }

    #[test]
    fn test_single_byte() {
        assert!(lz_roundtrip_ok(b"x"));
    }

    #[test]
    fn test_roundtrip_hello() {
        assert!(lz_roundtrip_ok(b"Hello, World!"));
    }

    #[test]
    fn test_roundtrip_binary() {
        let data: Vec<u8> = (0u8..=255).collect();
        assert!(lz_roundtrip_ok(&data));
    }

    #[test]
    fn test_is_compressed() {
        let compressed = lz_compress(b"hello");
        assert!(lz_is_compressed(&compressed));
        assert!(!lz_is_compressed(b"hello"));
        assert!(!lz_is_compressed(&[]));
    }

    #[test]
    fn test_compress_bound() {
        assert!(lz_compress_bound(100) >= 100);
    }

    #[test]
    fn test_compress_repetitive_yields_smaller() {
        let data: Vec<u8> = vec![b'A'; 1000];
        let compressed = lz_compress(&data);
        // With max_match=258: 1000/258 ≈ 4 matches + 1 lit = 5 items, 1 group
        // = 8(header) + 1(flag) + 1(lit) + 4*3(matches) = 22 bytes, well < 100
        assert!(
            compressed.len() < 100,
            "Expected < 100 bytes, got {}",
            compressed.len()
        );
    }

    #[test]
    fn test_roundtrip_repetitive() {
        let data: Vec<u8> = vec![b'B'; 500];
        assert!(lz_roundtrip_ok(&data));
    }

    #[test]
    fn test_magic_in_header() {
        let out = lz_compress(b"test");
        assert_eq!(&out[..4], b"LZ77");
    }

    #[test]
    fn test_decompressed_length_in_header() {
        let data = b"hello world";
        let out = lz_compress(data);
        let stored_len = u32::from_le_bytes([out[4], out[5], out[6], out[7]]) as usize;
        assert_eq!(stored_len, data.len());
    }

    #[test]
    fn test_invalid_magic_errors() {
        let bad = b"XXXX\x05\x00\x00\x00hello";
        assert!(lz_decompress(bad).is_err());
    }

    #[test]
    fn test_struct_roundtrip() {
        let lz = LzCompressor::default_compressor();
        let data = b"hello hello";
        let c = lz.compress(data);
        let d = lz.decompress(&c);
        assert_eq!(d, data);
    }

    #[test]
    fn test_compressor_name() {
        assert_eq!(compressor_name(), "lz77");
    }

    #[test]
    fn test_longer_repetitive_text() {
        let text = b"abcabcabcabcabcabcabcabcabcabc";
        assert!(lz_roundtrip_ok(text));
    }

    #[test]
    fn test_roundtrip_large() {
        let data: Vec<u8> = (0..1000u32).map(|i| (i % 256) as u8).collect();
        assert!(lz_roundtrip_ok(&data));
    }
}
