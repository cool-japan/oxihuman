// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Snappy block-format compressor and decompressor.
//!
//! Wire format:
//! - Varint-encoded (LEB128) uncompressed length
//! - Followed by elements:
//!   - Literal  (tag & 0x03 == 0x00): upper 6 bits encodes length-1 if ≤ 59, or 59+extra_bytes
//!   - Copy-1   (tag & 0x03 == 0x01): 2-byte copy, offset ≤ 2047, length 4..11
//!   - Copy-2   (tag & 0x03 == 0x02): 3-byte copy, offset ≤ 65535, length 1..64
//!   - Copy-4   (tag & 0x03 == 0x03): 5-byte copy (rarely used, not emitted here)

const HASH_TABLE_BITS: usize = 15;
const HASH_TABLE_SIZE: usize = 1 << HASH_TABLE_BITS; // 32768
const MIN_MATCH: usize = 4;

/// Configuration for the Snappy compressor.
#[derive(Debug, Clone, Default)]
pub struct SnappyConfig {
    pub verify_checksum: bool,
}

/// Snappy compressor.
#[derive(Debug, Clone)]
pub struct SnappyCompressor {
    pub config: SnappyConfig,
}

impl SnappyCompressor {
    pub fn new(config: SnappyConfig) -> Self {
        Self { config }
    }

    pub fn default_compressor() -> Self {
        Self::new(SnappyConfig {
            verify_checksum: false,
        })
    }
}

/// Encode a varint (LEB128) into `out`.
fn write_varint(out: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if value == 0 {
            break;
        }
    }
}

/// Decode a varint (LEB128) from `data` starting at `pos`.
/// Returns (value, new_pos).
fn read_varint(data: &[u8], mut pos: usize) -> Result<(u64, usize), String> {
    let mut value: u64 = 0;
    let mut shift: u32 = 0;
    loop {
        if pos >= data.len() {
            return Err("snappy: truncated varint".to_string());
        }
        let byte = data[pos];
        pos += 1;
        if shift >= 63 && (byte & 0x7F) > 1 {
            return Err("snappy: varint overflow".to_string());
        }
        value |= ((byte & 0x7F) as u64) << shift;
        shift += 7;
        if byte & 0x80 == 0 {
            break;
        }
        if shift > 63 {
            return Err("snappy: varint too long".to_string());
        }
    }
    Ok((value, pos))
}

/// Snappy-specific 4-byte hash.
#[inline]
fn snappy_hash(data: &[u8], pos: usize) -> usize {
    if pos + 4 > data.len() {
        return 0;
    }
    let v = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
    let h = v.wrapping_mul(0x1E35A7BD);
    (h >> (32 - HASH_TABLE_BITS)) as usize
}

/// Find match length between two positions.
#[inline]
fn match_len(data: &[u8], mut a: usize, mut b: usize, limit: usize) -> usize {
    let start = a;
    while a < limit && b < limit && data[a] == data[b] {
        a += 1;
        b += 1;
    }
    a - start
}

/// Emit a Snappy literal element.
fn emit_literal(out: &mut Vec<u8>, literals: &[u8]) {
    if literals.is_empty() {
        return;
    }
    let n = literals.len();
    if n <= 60 {
        // Upper 6 bits = (n-1), lower 2 bits = 0x00
        out.push(((n - 1) as u8) << 2);
    } else if n <= 256 {
        // 60 extra bytes → value 59 in upper 6 bits (meaning 1 extra byte follows)
        out.push(0xF0); // (59 << 2) | 0x00 = 236 = 0xEC... wait:
                        // Actually: upper 6 bits = 60 means 1 extra length byte follows (0-indexed: 59+1)
                        // snappy spec: if upper 6 bits = 60, one extra byte; 61 → two bytes; etc.
                        // Correct: upper 6 bits = 60 → tag|0x00, n-1 in 1 byte
                        // Let's redo: tag byte = ((60) << 2) | 0x00 = 240 = 0xF0
                        // then n-1 as 1 LE byte
        out.pop(); // remove the premature push
        out.push(60u8 << 2); // = 0xF0
        out.push((n - 1) as u8);
    } else if n <= 65536 {
        // upper 6 bits = 61 → 2 extra bytes
        out.push(61u8 << 2); // = 0xF4
        let nm1 = (n - 1) as u16;
        out.extend_from_slice(&nm1.to_le_bytes());
    } else {
        // upper 6 bits = 62 → 3 extra bytes (covers up to 16MB)
        out.push(62u8 << 2); // = 0xF8
        let nm1 = (n - 1) as u32;
        out.extend_from_slice(&nm1.to_le_bytes()[..3]);
    }
    out.extend_from_slice(literals);
}

/// Emit a Snappy copy element.
fn emit_copy(out: &mut Vec<u8>, offset: usize, length: usize) {
    // Try Copy-1 (offset ≤ 2047, length 4..11)
    if offset < 2048 && (4..=11).contains(&length) {
        // tag: lower 2 bits = 01, bits 2..4 = offset[8..10], bits 5..7 = length-4
        let len_bits = ((length - 4) as u8) << 2; // bits [4:2]
        let off_high = ((offset >> 8) & 0x07) as u8; // bits [2:0] of offset high
        let tag = 0x01 | len_bits | (off_high << 5);
        out.push(tag);
        out.push((offset & 0xFF) as u8);
        return;
    }

    // Use Copy-2 (offset ≤ 65535, length 1..64)
    // Emit in chunks of up to 64 if length > 64
    let remaining = length;
    let off = offset;
    if remaining > 0 {
        let chunk = remaining.min(64);
        // tag: lower 2 bits = 10, upper 6 bits = chunk - 1
        let tag = (((chunk - 1) as u8) << 2) | 0x02;
        out.push(tag);
        out.extend_from_slice(&(off as u16).to_le_bytes());
        // subsequent copies use offset relative to where we are in output,
        // but for simplicity we only do one chunk since offset stays fixed
        // The offset remains the same for all chunks (pointing back to original match)
        // Actually offset grows by chunk each iteration, but since we already output
        // the literals, the pattern repeats — keep same offset
        let _ = off; // suppress warning; off unchanged for repeated pattern
    }
}

/// Compress `data` using the Snappy block format.
pub fn snappy_compress(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(snappy_max_compressed_length(data.len()));

    // Write uncompressed length as varint
    write_varint(&mut out, data.len() as u64);

    if data.is_empty() {
        return out;
    }

    let mut hash_table: Vec<u32> = vec![u32::MAX; HASH_TABLE_SIZE];

    let mut pos: usize = 0;
    let mut lit_start: usize = 0;

    let match_limit = if data.len() > 15 { data.len() - 15 } else { 0 };

    while pos < data.len() {
        if pos + MIN_MATCH <= data.len() && pos < match_limit {
            let h = snappy_hash(data, pos);
            let candidate = hash_table[h] as usize;
            hash_table[h] = pos as u32;

            if candidate != u32::MAX as usize
                && candidate < pos
                && pos - candidate <= 65535
                && candidate + MIN_MATCH <= data.len()
            {
                let ml = match_len(data, pos, candidate, data.len());
                if ml >= MIN_MATCH {
                    // Flush pending literals
                    if pos > lit_start {
                        emit_literal(&mut out, &data[lit_start..pos]);
                    }

                    let offset = pos - candidate;
                    // Emit copies, possibly in multiple chunks for long matches
                    let mut emitted = 0;
                    while emitted < ml {
                        let chunk = (ml - emitted).min(64);
                        emit_copy(&mut out, offset, chunk);
                        emitted += chunk;
                    }

                    // Update hash table for positions inside the match
                    for i in 1..ml {
                        if pos + i + MIN_MATCH <= data.len() {
                            let hi = snappy_hash(data, pos + i);
                            hash_table[hi] = (pos + i) as u32;
                        }
                    }
                    pos += ml;
                    lit_start = pos;
                    continue;
                }
            }
        } else if pos + MIN_MATCH <= data.len() {
            let h = snappy_hash(data, pos);
            hash_table[h] = pos as u32;
        }

        pos += 1;
    }

    // Flush remaining literals
    if lit_start < data.len() {
        emit_literal(&mut out, &data[lit_start..]);
    }

    out
}

/// Decompress data produced by [`snappy_compress`].
pub fn snappy_decompress(data: &[u8]) -> Result<Vec<u8>, String> {
    if data.is_empty() {
        return Err("snappy: empty input".to_string());
    }

    let (uncompressed_len, mut pos) = read_varint(data, 0)?;
    let uncompressed_len = uncompressed_len as usize;

    let mut out: Vec<u8> = Vec::with_capacity(uncompressed_len);

    while pos < data.len() {
        let tag = data[pos];
        let element_type = tag & 0x03;
        pos += 1;

        match element_type {
            0x00 => {
                // Literal
                let upper6 = (tag >> 2) as usize;
                let lit_len = if upper6 < 60 {
                    upper6 + 1
                } else {
                    let extra_bytes = upper6 - 59; // 1..=4
                    if pos + extra_bytes > data.len() {
                        return Err("snappy: truncated literal length".to_string());
                    }
                    let mut n: usize = 0;
                    for i in 0..extra_bytes {
                        n |= (data[pos + i] as usize) << (8 * i);
                    }
                    pos += extra_bytes;
                    n + 1
                };
                if pos + lit_len > data.len() {
                    return Err("snappy: literal overflows input".to_string());
                }
                out.extend_from_slice(&data[pos..pos + lit_len]);
                pos += lit_len;
            }
            0x01 => {
                // Copy-1: 2-byte element
                if pos >= data.len() {
                    return Err("snappy: truncated copy-1".to_string());
                }
                let off_low = data[pos] as usize;
                pos += 1;
                let len = ((tag >> 2) & 0x07) as usize + 4;
                let off_high = ((tag >> 5) & 0x07) as usize;
                let offset = (off_high << 8) | off_low;
                if offset == 0 || offset > out.len() {
                    return Err(format!(
                        "snappy: copy-1 invalid offset {} (output len {})",
                        offset,
                        out.len()
                    ));
                }
                let match_start = out.len() - offset;
                for i in 0..len {
                    let b = out[match_start + i];
                    out.push(b);
                }
            }
            0x02 => {
                // Copy-2: 3-byte element
                if pos + 2 > data.len() {
                    return Err("snappy: truncated copy-2".to_string());
                }
                let offset = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
                pos += 2;
                let len = ((tag >> 2) as usize) + 1;
                if offset == 0 || offset > out.len() {
                    return Err(format!(
                        "snappy: copy-2 invalid offset {} (output len {})",
                        offset,
                        out.len()
                    ));
                }
                let match_start = out.len() - offset;
                for i in 0..len {
                    let b = out[match_start + i];
                    out.push(b);
                }
            }
            0x03 => {
                // Copy-4: 5-byte element (not emitted by this encoder, but handle for completeness)
                if pos + 4 > data.len() {
                    return Err("snappy: truncated copy-4".to_string());
                }
                let offset =
                    u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                        as usize;
                pos += 4;
                let len = ((tag >> 2) as usize) + 1;
                if offset == 0 || offset > out.len() {
                    return Err(format!(
                        "snappy: copy-4 invalid offset {} (output len {})",
                        offset,
                        out.len()
                    ));
                }
                let match_start = out.len() - offset;
                for i in 0..len {
                    let b = out[match_start + i];
                    out.push(b);
                }
            }
            _ => unreachable!(),
        }
    }

    if out.len() != uncompressed_len {
        return Err(format!(
            "snappy: decompressed {} bytes, expected {}",
            out.len(),
            uncompressed_len
        ));
    }

    Ok(out)
}

/// Return the maximum output size for a given input length.
pub fn snappy_max_compressed_length(input_len: usize) -> usize {
    32 + input_len + input_len / 6
}

/// Return whether a byte slice could be a valid Snappy frame (has at least a varint).
pub fn snappy_validate_compressed_buffer(data: &[u8]) -> bool {
    data.len() >= 4
}

/// Verify round-trip integrity.
pub fn snappy_roundtrip_ok(data: &[u8]) -> bool {
    snappy_decompress(&snappy_compress(data))
        .map(|d| d == data)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = SnappyCompressor::default_compressor();
        assert!(!c.config.verify_checksum);
    }

    #[test]
    fn test_compress_empty() {
        let out = snappy_compress(&[]);
        // Just the varint 0
        assert!(!out.is_empty());
        assert!(snappy_roundtrip_ok(&[]));
    }

    #[test]
    fn test_roundtrip_ascii() {
        assert!(snappy_roundtrip_ok(b"snappy test string"));
    }

    #[test]
    fn test_roundtrip_binary() {
        let data: Vec<u8> = (0u8..=255).collect();
        assert!(snappy_roundtrip_ok(&data));
    }

    #[test]
    fn test_decompress_too_short() {
        assert!(snappy_decompress(&[]).is_err());
    }

    #[test]
    fn test_max_compressed_length() {
        assert!(snappy_max_compressed_length(100) > 100);
    }

    #[test]
    fn test_validate_valid() {
        assert!(snappy_validate_compressed_buffer(&[0, 0, 0, 0]));
    }

    #[test]
    fn test_validate_too_short() {
        assert!(!snappy_validate_compressed_buffer(&[0, 0, 0]));
    }

    #[test]
    fn test_new_config() {
        let c = SnappyCompressor::new(SnappyConfig {
            verify_checksum: true,
        });
        assert!(c.config.verify_checksum);
    }

    #[test]
    fn test_compress_repetitive_yields_smaller() {
        let data: Vec<u8> = vec![b'A'; 1000];
        let compressed = snappy_compress(&data);
        assert!(
            compressed.len() < 100,
            "Expected < 100 bytes, got {}",
            compressed.len()
        );
    }

    #[test]
    fn test_roundtrip_repetitive() {
        let data: Vec<u8> = vec![b'X'; 1000];
        assert!(snappy_roundtrip_ok(&data));
    }

    #[test]
    fn test_roundtrip_single_byte() {
        assert!(snappy_roundtrip_ok(b"z"));
    }

    #[test]
    fn test_roundtrip_longer_text() {
        let text = b"the quick brown fox jumps over the lazy dog. the quick brown fox!";
        assert!(snappy_roundtrip_ok(text));
    }
}
