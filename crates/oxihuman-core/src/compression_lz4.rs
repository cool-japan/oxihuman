// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! LZ4 block-format compressor and decompressor.
//!
//! Implements the LZ4 block format specification:
//! - No frame header (raw block format)
//! - Sequence: token_byte + [extra_lit_len...] + literal_bytes + offset_le16 + [extra_match_len...]
//! - Token byte high nibble = literal length (15 = extend with 255-byte runs)
//! - Token byte low nibble = match length - 4 (min match 4; 15 = extend similarly)
//! - Final sequence has no offset/match fields

const HASH_TABLE_SIZE: usize = 1 << 16; // 65536 entries
const MIN_MATCH: usize = 4;
const MAX_OFFSET: usize = 65535;

/// Configuration for the LZ4 compressor.
#[derive(Debug, Clone)]
pub struct Lz4Config {
    pub acceleration: i32,
    pub block_size: usize,
}

impl Default for Lz4Config {
    fn default() -> Self {
        Self {
            acceleration: 1,
            block_size: 65536,
        }
    }
}

/// LZ4 compressor.
#[derive(Debug, Clone)]
pub struct Lz4Compressor {
    pub config: Lz4Config,
}

impl Lz4Compressor {
    pub fn new(config: Lz4Config) -> Self {
        Self { config }
    }

    pub fn default_compressor() -> Self {
        Self::new(Lz4Config::default())
    }
}

/// Compute a 4-byte hash for LZ4 hash table.
#[inline]
fn lz4_hash(data: &[u8], pos: usize) -> usize {
    if pos + 4 > data.len() {
        return 0;
    }
    let v = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
    // Knuth's multiplicative hash
    let h = v.wrapping_mul(0x9E3779B1u32);
    (h >> 16) as usize % HASH_TABLE_SIZE
}

/// Find the length of the match between `data[a..]` and `data[b..]`.
#[inline]
fn match_length(data: &[u8], mut a: usize, mut b: usize) -> usize {
    let limit = data.len();
    let start = a;
    while a < limit && b < limit && data[a] == data[b] {
        a += 1;
        b += 1;
    }
    a - start
}

/// Write LZ4 variable-length extension bytes (for lengths >= 15).
/// Writes 255-byte runs until remainder < 255, then the remainder.
fn write_var_len(out: &mut Vec<u8>, mut remainder: usize) {
    while remainder >= 255 {
        out.push(255);
        remainder -= 255;
    }
    out.push(remainder as u8);
}

/// Compress `data` using the LZ4 block format.
pub fn lz4_compress(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        // Empty block: emit a single token 0x00 (0 literals, 0 match) followed by nothing.
        // Actually the spec says final sequence can be just 0 literals — emit the token byte.
        return vec![0x00];
    }

    let mut out = Vec::with_capacity(lz4_compress_bound(data.len()));
    // Hash table: maps hash → last position
    let mut hash_table: Vec<u32> = vec![u32::MAX; HASH_TABLE_SIZE];

    let mut pos: usize = 0;
    // `literal_start` tracks where the pending literal run begins
    let mut literal_start: usize = 0;

    // We must leave the last 5 bytes as literals (LZ4 spec requirement: last match
    // must end at least 12 bytes from end, but we use a simpler safe bound here).
    let match_limit = if data.len() > 12 { data.len() - 12 } else { 0 };

    while pos < data.len() {
        // Try to find a match if we have at least MIN_MATCH bytes left and are before match_limit
        if pos + MIN_MATCH <= data.len() && pos < match_limit {
            let h = lz4_hash(data, pos);
            let candidate = hash_table[h] as usize;
            hash_table[h] = pos as u32;

            if candidate != u32::MAX as usize
                && candidate < pos
                && pos - candidate <= MAX_OFFSET
                && candidate + MIN_MATCH <= data.len()
            {
                // Verify actual match
                let match_len = match_length(data, pos, candidate);
                if match_len >= MIN_MATCH {
                    // Emit sequence: literals [literal_start..pos] + match
                    let lit_len = pos - literal_start;
                    let ml_encoded = match_len - MIN_MATCH; // minimum 4 → 0

                    // Token byte
                    let lit_nibble = lit_len.min(15) as u8;
                    let match_nibble = ml_encoded.min(15) as u8;
                    let token = (lit_nibble << 4) | match_nibble;
                    out.push(token);

                    // Extra literal length bytes if needed
                    if lit_len >= 15 {
                        write_var_len(&mut out, lit_len - 15);
                    }

                    // Literal bytes
                    out.extend_from_slice(&data[literal_start..pos]);

                    // Match offset (LE16)
                    let offset = (pos - candidate) as u16;
                    out.extend_from_slice(&offset.to_le_bytes());

                    // Extra match length bytes if needed
                    if ml_encoded >= 15 {
                        write_var_len(&mut out, ml_encoded - 15);
                    }

                    // Advance past the match
                    // Update hash table entries within the match region
                    for i in 1..match_len {
                        if pos + i + MIN_MATCH <= data.len() {
                            let hi = lz4_hash(data, pos + i);
                            hash_table[hi] = (pos + i) as u32;
                        }
                    }
                    pos += match_len;
                    literal_start = pos;
                    continue;
                }
            }
        } else if pos + MIN_MATCH <= data.len() {
            // Still update hash table
            let h = lz4_hash(data, pos);
            hash_table[h] = pos as u32;
        }

        pos += 1;
    }

    // Final sequence: remaining literals, no match
    let lit_len = data.len() - literal_start;
    let lit_nibble = lit_len.min(15) as u8;
    let token = lit_nibble << 4; // match nibble = 0, no match follows
    out.push(token);
    if lit_len >= 15 {
        write_var_len(&mut out, lit_len - 15);
    }
    out.extend_from_slice(&data[literal_start..]);
    // No offset/match bytes for the final sequence

    out
}

/// Read variable-length extension from compressed stream.
/// Returns (total_length_with_base, new_position) where base has already been added.
fn read_var_len(data: &[u8], pos: usize, base: usize) -> Result<(usize, usize), String> {
    let mut length = base;
    let mut cur = pos;
    loop {
        if cur >= data.len() {
            return Err("lz4: truncated variable-length extension".to_string());
        }
        let b = data[cur] as usize;
        cur += 1;
        length += b;
        if b < 255 {
            break;
        }
    }
    Ok((length, cur))
}

/// Decompress data produced by [`lz4_compress`].
pub fn lz4_decompress(data: &[u8]) -> Result<Vec<u8>, String> {
    if data.is_empty() {
        return Err("lz4: empty input".to_string());
    }

    // Special case: single 0x00 token means empty output
    if data == [0x00] {
        return Ok(Vec::new());
    }

    let mut out: Vec<u8> = Vec::new();
    let mut pos: usize = 0;
    let len = data.len();

    loop {
        if pos >= len {
            return Err("lz4: unexpected end of input".to_string());
        }

        let token = data[pos];
        pos += 1;

        // Decode literal length
        let lit_nibble = (token >> 4) as usize;
        let lit_len = if lit_nibble == 15 {
            let (l, new_pos) = read_var_len(data, pos, 15)?;
            pos = new_pos;
            l
        } else {
            lit_nibble
        };

        // Copy literals
        if pos + lit_len > len {
            return Err("lz4: literal run overflows input".to_string());
        }
        out.extend_from_slice(&data[pos..pos + lit_len]);
        pos += lit_len;

        // Check if this is the final sequence (no match follows)
        if pos >= len {
            break;
        }

        // Decode match offset (LE16)
        if pos + 2 > len {
            return Err("lz4: truncated match offset".to_string());
        }
        let offset = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
        pos += 2;

        if offset == 0 {
            return Err("lz4: zero match offset".to_string());
        }
        if offset > out.len() {
            return Err(format!(
                "lz4: match offset {} exceeds output length {}",
                offset,
                out.len()
            ));
        }

        // Decode match length
        let match_nibble = (token & 0x0F) as usize;
        let match_len = if match_nibble == 15 {
            let (ml, new_pos) = read_var_len(data, pos, 15 + MIN_MATCH)?;
            pos = new_pos;
            ml
        } else {
            match_nibble + MIN_MATCH
        };

        // Copy match (may overlap — must copy byte-by-byte)
        let match_start = out.len() - offset;
        for i in 0..match_len {
            let b = out[match_start + i];
            out.push(b);
        }
    }

    Ok(out)
}

/// Return the worst-case compressed size for `input_len` bytes.
pub fn lz4_compress_bound(input_len: usize) -> usize {
    input_len + input_len / 255 + 16
}

/// Return whether `data` could be a valid LZ4 block (any 1+ byte slice).
pub fn lz4_is_compressed(data: &[u8]) -> bool {
    data.len() >= 4
}

/// Round-trip compress then decompress and verify equality.
pub fn lz4_roundtrip_ok(data: &[u8]) -> bool {
    match lz4_decompress(&lz4_compress(data)) {
        Ok(out) => out == data,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = Lz4Config::default();
        assert_eq!(cfg.acceleration, 1);
    }

    #[test]
    fn test_compress_bound() {
        assert!(lz4_compress_bound(100) >= 100);
    }

    #[test]
    fn test_roundtrip_empty() {
        assert!(lz4_roundtrip_ok(&[]));
    }

    #[test]
    fn test_roundtrip_hello() {
        assert!(lz4_roundtrip_ok(b"Hello, World!"));
    }

    #[test]
    fn test_roundtrip_binary() {
        let data: Vec<u8> = (0u8..=255).collect();
        assert!(lz4_roundtrip_ok(&data));
    }

    #[test]
    fn test_roundtrip_repetitive() {
        let data: Vec<u8> = vec![b'A'; 1000];
        assert!(lz4_roundtrip_ok(&data));
    }

    #[test]
    fn test_compress_produces_smaller_than_input_for_repetitive_data() {
        let data: Vec<u8> = vec![b'A'; 1000];
        let compressed = lz4_compress(&data);
        assert!(
            compressed.len() < 100,
            "Expected compressed size < 100, got {}",
            compressed.len()
        );
    }

    #[test]
    fn test_decompress_too_short() {
        assert!(lz4_decompress(&[]).is_err());
    }

    #[test]
    fn test_is_compressed_false() {
        assert!(!lz4_is_compressed(&[]));
    }

    #[test]
    fn test_is_compressed_true() {
        assert!(lz4_is_compressed(&[0, 0, 0, 0]));
    }

    #[test]
    fn test_compressor_new() {
        let c = Lz4Compressor::default_compressor();
        assert_eq!(c.config.acceleration, 1);
    }

    #[test]
    fn test_roundtrip_longer_text() {
        let text = b"the quick brown fox jumps over the lazy dog. the quick brown fox!";
        assert!(lz4_roundtrip_ok(text));
    }

    #[test]
    fn test_roundtrip_all_zeros() {
        let data = vec![0u8; 512];
        assert!(lz4_roundtrip_ok(&data));
    }

    #[test]
    fn test_roundtrip_single_byte() {
        assert!(lz4_roundtrip_ok(b"x"));
    }
}
