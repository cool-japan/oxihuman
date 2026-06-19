// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! GIF89a encoder and decoder with proper LZW compression.
//!
//! This module implements a complete GIF89a codec in pure Rust:
//! - Encoder: RGB pixels → GIF89a bytes, using median-cut color quantization
//!   (≤256 colors) and variable-width LSB-first LZW compression.
//! - Decoder: GIF89a / GIF87a bytes → RGB pixels, handling extension blocks,
//!   local color tables, and the standard LZW special cases.

use super::image_codec::RawDecodeResult;
use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────────────────────
// Error type
// ─────────────────────────────────────────────────────────────────────────────

/// Errors that can occur during GIF encode / decode operations.
#[derive(Debug, thiserror::Error)]
pub enum GifError {
    /// The data is structurally invalid.
    #[error("Invalid GIF: {0}")]
    Invalid(String),
    /// The byte stream ended prematurely.
    #[error("Truncated GIF data")]
    Truncated,
    /// An error in the LZW layer.
    #[error("LZW error: {0}")]
    LzwError(String),
}

// ─────────────────────────────────────────────────────────────────────────────
// Colour quantisation — median-cut algorithm
// ─────────────────────────────────────────────────────────────────────────────

/// One bucket of pixels used during median-cut quantisation.
struct Bucket {
    /// Flat list of (R, G, B) triples belonging to this bucket.
    pixels: Vec<[u8; 3]>,
}

impl Bucket {
    fn new(pixels: Vec<[u8; 3]>) -> Self {
        Self { pixels }
    }

    /// Range of each channel.
    fn channel_ranges(&self) -> ([u8; 3], [u8; 3]) {
        let mut min = [255u8; 3];
        let mut max = [0u8; 3];
        for p in &self.pixels {
            for c in 0..3 {
                if p[c] < min[c] {
                    min[c] = p[c];
                }
                if p[c] > max[c] {
                    max[c] = p[c];
                }
            }
        }
        (min, max)
    }

    /// Widest channel index.
    fn widest_channel(&self) -> usize {
        let (min, max) = self.channel_ranges();
        let ranges = [
            max[0].saturating_sub(min[0]),
            max[1].saturating_sub(min[1]),
            max[2].saturating_sub(min[2]),
        ];
        if ranges[0] >= ranges[1] && ranges[0] >= ranges[2] {
            0
        } else if ranges[1] >= ranges[2] {
            1
        } else {
            2
        }
    }

    /// Split the bucket along the median of the widest channel.
    /// Returns `None` if the bucket cannot be split (single pixel or all same).
    fn split(mut self) -> Option<(Bucket, Bucket)> {
        if self.pixels.len() < 2 {
            return None;
        }
        let ch = self.widest_channel();
        self.pixels.sort_unstable_by_key(|p| p[ch]);
        let mid = self.pixels.len() / 2;
        // Require that the two halves actually differ in the split channel,
        // otherwise there is no real division to be made.
        if self.pixels[0][ch] == self.pixels[self.pixels.len() - 1][ch] {
            return None;
        }
        let right = self.pixels.split_off(mid);
        Some((Bucket::new(self.pixels), Bucket::new(right)))
    }

    /// Representative colour: the unweighted average of all pixels.
    fn representative(&self) -> [u8; 3] {
        if self.pixels.is_empty() {
            return [0, 0, 0];
        }
        let mut sum = [0u64; 3];
        for p in &self.pixels {
            sum[0] += p[0] as u64;
            sum[1] += p[1] as u64;
            sum[2] += p[2] as u64;
        }
        let n = self.pixels.len() as u64;
        [(sum[0] / n) as u8, (sum[1] / n) as u8, (sum[2] / n) as u8]
    }
}

/// Quantise `pixels` (tightly-packed RGB) to at most `max_colors` palette entries.
///
/// Returns `(palette, index_map)` where `palette` is the list of representative
/// colours and `index_map[i]` gives the palette index for pixel `i`.
fn median_cut_quantize(pixels: &[[u8; 3]], max_colors: usize) -> (Vec<[u8; 3]>, Vec<u8>) {
    if pixels.is_empty() {
        return (vec![[0, 0, 0]], vec![]);
    }

    // Seed with one bucket holding every pixel.
    let mut buckets: Vec<Bucket> = vec![Bucket::new(pixels.to_vec())];

    // Split until we reach max_colors or no more splits are possible.
    while buckets.len() < max_colors {
        // Pick the bucket with the largest range (most "spreadable").
        let pick = buckets
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let (mn, mx) = b.channel_ranges();
                let spread = (mx[0].saturating_sub(mn[0]) as u32)
                    + (mx[1].saturating_sub(mn[1]) as u32)
                    + (mx[2].saturating_sub(mn[2]) as u32);
                (spread, i)
            })
            .max_by_key(|&(s, _)| s);

        let idx = match pick {
            Some((0, _)) | None => break, // No splittable bucket.
            Some((_, i)) => i,
        };

        let bucket = buckets.remove(idx);
        match bucket.split() {
            Some((a, b)) => {
                buckets.push(a);
                buckets.push(b);
            }
            None => {
                // Put it back — it can't be split.
                buckets.insert(idx, Bucket::new(vec![]));
                break;
            }
        }
    }

    // Build the palette from bucket representatives.
    let palette: Vec<[u8; 3]> = buckets.iter().map(|b| b.representative()).collect();

    // Assign each pixel to the nearest palette entry (Euclidean² distance).
    let index_map: Vec<u8> = pixels
        .iter()
        .map(|p| nearest_palette_index(p, &palette))
        .collect();

    (palette, index_map)
}

/// Return the index of the palette entry closest to `pixel` in Euclidean² space.
fn nearest_palette_index(pixel: &[u8; 3], palette: &[[u8; 3]]) -> u8 {
    let mut best_idx = 0usize;
    let mut best_dist = u32::MAX;
    for (i, entry) in palette.iter().enumerate() {
        let dr = pixel[0] as i32 - entry[0] as i32;
        let dg = pixel[1] as i32 - entry[1] as i32;
        let db = pixel[2] as i32 - entry[2] as i32;
        let dist = (dr * dr + dg * dg + db * db) as u32;
        if dist < best_dist {
            best_dist = dist;
            best_idx = i;
            if dist == 0 {
                break;
            }
        }
    }
    best_idx as u8
}

// ─────────────────────────────────────────────────────────────────────────────
// LZW compressor (GIF variant — LSB-first bit packing)
// ─────────────────────────────────────────────────────────────────────────────

/// Compress `indices` using GIF LZW with the given `min_code_size`.
///
/// Returns the raw bit-stream bytes (not yet wrapped in sub-blocks).
fn lzw_compress(indices: &[u8], min_code_size: u8) -> Vec<u8> {
    let clear_code = 1u32 << min_code_size;
    let end_code = clear_code + 1;
    let initial_next = end_code + 1;

    // The code table maps (prefix_code, suffix_byte) → new_code.
    let mut table: HashMap<(u32, u8), u32> = HashMap::new();
    let mut next_code = initial_next;
    let mut code_width = min_code_size as u32 + 1;
    // Threshold at which we increase code_width.
    let mut next_threshold = 1u32 << code_width;

    let mut bit_buf = 0u64; // Bit accumulator (LSB-first).
    let mut bit_len = 0u32; // Number of valid bits in bit_buf.
    let mut out: Vec<u8> = Vec::new();

    /// Flush `bits` bits from bit_buf into `out` (LSB first).
    fn emit(code: u32, bits: u32, buf: &mut u64, len: &mut u32, out: &mut Vec<u8>) {
        *buf |= (code as u64) << *len;
        *len += bits;
        while *len >= 8 {
            out.push((*buf & 0xFF) as u8);
            *buf >>= 8;
            *len -= 8;
        }
    }

    // Helper to reset the code table to initial state.
    macro_rules! reset_table {
        () => {{
            table.clear();
            next_code = initial_next;
            code_width = min_code_size as u32 + 1;
            next_threshold = 1u32 << code_width;
        }};
    }

    // Emit the initial Clear code.
    emit(clear_code, code_width, &mut bit_buf, &mut bit_len, &mut out);

    if indices.is_empty() {
        emit(end_code, code_width, &mut bit_buf, &mut bit_len, &mut out);
        // Flush remaining bits.
        if bit_len > 0 {
            out.push((bit_buf & 0xFF) as u8);
        }
        return out;
    }

    let mut string_code = indices[0] as u32;

    for &byte in &indices[1..] {
        let key = (string_code, byte);
        match table.get(&key) {
            Some(&existing) => {
                string_code = existing;
            }
            None => {
                // Emit the current string code.
                emit(
                    string_code,
                    code_width,
                    &mut bit_buf,
                    &mut bit_len,
                    &mut out,
                );

                if next_code < 4096 {
                    table.insert(key, next_code);
                    next_code += 1;
                    if next_code > next_threshold && code_width < 12 {
                        code_width += 1;
                        next_threshold = 1u32 << code_width;
                    }
                } else {
                    // Table full: emit Clear and reset.
                    emit(clear_code, code_width, &mut bit_buf, &mut bit_len, &mut out);
                    reset_table!();
                }

                string_code = byte as u32;
            }
        }
    }

    // Emit the remaining string.
    emit(
        string_code,
        code_width,
        &mut bit_buf,
        &mut bit_len,
        &mut out,
    );
    // Emit End code.
    emit(end_code, code_width, &mut bit_buf, &mut bit_len, &mut out);
    // Flush any remaining bits.
    if bit_len > 0 {
        out.push((bit_buf & 0xFF) as u8);
    }

    out
}

/// Wrap `lzw_data` in GIF sub-blocks (length-prefixed chunks of ≤255 bytes).
fn wrap_in_sub_blocks(lzw_data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    for chunk in lzw_data.chunks(255) {
        out.push(chunk.len() as u8);
        out.extend_from_slice(chunk);
    }
    out.push(0); // Block terminator.
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// Public encoder
// ─────────────────────────────────────────────────────────────────────────────

/// Encode a raw RGB pixel buffer as a GIF89a byte stream.
///
/// `pixels` must contain exactly `width * height * 3` bytes in row-major,
/// top-to-bottom, R-G-B order.
pub fn gif_encode_rgb(width: u32, height: u32, pixels: &[u8]) -> Result<Vec<u8>, GifError> {
    let pixel_count = (width as usize) * (height as usize);
    if pixels.len() < pixel_count * 3 {
        return Err(GifError::Invalid(format!(
            "pixel buffer too short: expected {} bytes, got {}",
            pixel_count * 3,
            pixels.len()
        )));
    }
    if width == 0 || height == 0 {
        return Err(GifError::Invalid("width and height must be > 0".into()));
    }

    // Reinterpret the flat u8 slice as an array of RGB triples.
    let rgb_pixels: Vec<[u8; 3]> = pixels
        .chunks_exact(3)
        .take(pixel_count)
        .map(|c| [c[0], c[1], c[2]])
        .collect();

    // Quantise to ≤256 colours.
    let max_colors = 256usize;
    let (mut palette, index_map) = median_cut_quantize(&rgb_pixels, max_colors);

    // GIF requires the palette length to be a power of two.  Find the smallest
    // power-of-two size ≥ palette.len() that is also ≥ 2.
    let palette_len = palette.len().max(2);
    let depth = {
        let mut d = 1u8;
        while (1usize << d) < palette_len {
            d += 1;
        }
        d // actual bit depth; palette contains 2^depth entries
    };
    // Pad palette to exactly 2^depth entries.
    while palette.len() < (1 << depth) {
        palette.push([0, 0, 0]);
    }

    let gct_size_field = depth - 1; // field value n means 2^(n+1) entries
    let packed_byte: u8 = 0x80 |                       // Global Color Table flag
        ((depth - 1) << 4) |         // colour resolution (depth - 1)
        gct_size_field; // GCT size field

    let min_code_size = depth.max(2); // GIF spec: minimum is 2

    let mut out: Vec<u8> = Vec::new();

    // ── Header ────────────────────────────────────────────────────────────────
    out.extend_from_slice(b"GIF89a");

    // ── Logical Screen Descriptor ─────────────────────────────────────────────
    out.extend_from_slice(&(width as u16).to_le_bytes());
    out.extend_from_slice(&(height as u16).to_le_bytes());
    out.push(packed_byte);
    out.push(0); // Background colour index.
    out.push(0); // Pixel aspect ratio.

    // ── Global Color Table ────────────────────────────────────────────────────
    for entry in &palette {
        out.push(entry[0]);
        out.push(entry[1]);
        out.push(entry[2]);
    }

    // ── Image Descriptor ──────────────────────────────────────────────────────
    out.push(0x2C); // Image Separator.
    out.extend_from_slice(&0u16.to_le_bytes()); // Left.
    out.extend_from_slice(&0u16.to_le_bytes()); // Top.
    out.extend_from_slice(&(width as u16).to_le_bytes());
    out.extend_from_slice(&(height as u16).to_le_bytes());
    out.push(0x00); // Packed: no local table, not interlaced.

    // ── LZW Minimum Code Size ─────────────────────────────────────────────────
    out.push(min_code_size);

    // ── Image Data ────────────────────────────────────────────────────────────
    let lzw_bytes = lzw_compress(&index_map, min_code_size);
    let sub_blocks = wrap_in_sub_blocks(&lzw_bytes);
    out.extend_from_slice(&sub_blocks);

    // ── Trailer ───────────────────────────────────────────────────────────────
    out.push(0x3B);

    Ok(out)
}

// ─────────────────────────────────────────────────────────────────────────────
// LZW decompressor (GIF variant — LSB-first bit reading)
// ─────────────────────────────────────────────────────────────────────────────

/// Bit-level reader that extracts variable-width codes (LSB-first) from a byte
/// slice.
struct BitReader<'a> {
    data: &'a [u8],
    pos: usize, // byte position
    bit_buf: u32,
    bit_len: u32,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            bit_buf: 0,
            bit_len: 0,
        }
    }

    /// Read `n` bits LSB-first.  Returns `None` if the stream is exhausted.
    fn read_bits(&mut self, n: u32) -> Option<u32> {
        while self.bit_len < n {
            if self.pos >= self.data.len() {
                return None;
            }
            self.bit_buf |= (self.data[self.pos] as u32) << self.bit_len;
            self.pos += 1;
            self.bit_len += 8;
        }
        let mask = (1u32 << n) - 1;
        let val = self.bit_buf & mask;
        self.bit_buf >>= n;
        self.bit_len -= n;
        Some(val)
    }
}

/// Concatenate all sub-block data bytes from a GIF image data section.
///
/// `bytes` must point to the start of the first sub-block (after the LZW
/// minimum code size byte).  Returns the concatenated payload and the number
/// of source bytes consumed (including the trailing zero-length block).
fn read_sub_blocks(bytes: &[u8]) -> Result<(Vec<u8>, usize), GifError> {
    let mut out: Vec<u8> = Vec::new();
    let mut pos = 0usize;
    loop {
        if pos >= bytes.len() {
            return Err(GifError::Truncated);
        }
        let block_len = bytes[pos] as usize;
        pos += 1;
        if block_len == 0 {
            break;
        }
        if pos + block_len > bytes.len() {
            return Err(GifError::Truncated);
        }
        out.extend_from_slice(&bytes[pos..pos + block_len]);
        pos += block_len;
    }
    Ok((out, pos))
}

/// GIF LZW decompressor.
///
/// Decompresses `data` using `min_code_size` and returns the index stream.
///
/// The code table uses a linked-list representation: each entry stores
/// (prefix_code, suffix_byte).  For the initial leaf codes prefix_code is
/// `u16::MAX` (sentinel meaning "no prefix").  Strings are reconstructed by
/// walking the chain backwards and then reversing.
///
/// Code-width growth rule (GIF spec / Netscape interpretation):
///   After emitting a code that fills the current range — i.e. after the table
///   size reaches `2^code_width` — the *next* code read uses `code_width + 1`
///   bits (up to a maximum of 12).
fn lzw_decompress(data: &[u8], min_code_size: u8) -> Result<Vec<u8>, GifError> {
    let clear_code = 1u16 << min_code_size;
    let end_code = clear_code + 1;
    let table_initial_size = (end_code + 1) as usize;

    // prefix[i] == u16::MAX  →  leaf (no prefix)
    let mut prefix: Vec<u16> = Vec::with_capacity(4096);
    let mut suffix: Vec<u8> = Vec::with_capacity(4096);

    // Seed: single-byte leaf codes 0..clear_code.
    for i in 0..(clear_code as usize) {
        prefix.push(u16::MAX);
        suffix.push(i as u8);
    }
    // clear_code slot
    prefix.push(u16::MAX);
    suffix.push(0);
    // end_code slot
    prefix.push(u16::MAX);
    suffix.push(0);

    let mut code_width = min_code_size as u32 + 1;

    let mut reader = BitReader::new(data);
    let mut output: Vec<u8> = Vec::new();

    // Reusable scratch buffer for string reconstruction.
    let mut scratch: Vec<u8> = Vec::new();

    // Walk the chain for `code` and write the string (reversed) into `scratch`.
    let walk_chain =
        |code: u16, pfx: &[u16], sfx: &[u8], scratch: &mut Vec<u8>| -> Result<(), GifError> {
            scratch.clear();
            let mut c = code;
            let mut depth = 0usize;
            loop {
                if (c as usize) >= pfx.len() {
                    return Err(GifError::LzwError(format!("code {} not in table", c)));
                }
                scratch.push(sfx[c as usize]);
                let p = pfx[c as usize];
                if p == u16::MAX {
                    break;
                }
                c = p;
                depth += 1;
                if depth > 4096 {
                    return Err(GifError::LzwError("cycle in LZW chain".into()));
                }
            }
            Ok(())
        };

    // Returns the first (root) byte for `code`.
    let first_byte = |code: u16, pfx: &[u16], sfx: &[u8]| -> u8 {
        let mut c = code;
        loop {
            let p = pfx[c as usize];
            if p == u16::MAX {
                return sfx[c as usize];
            }
            c = p;
        }
    };

    let mut prev_code: Option<u16> = None;
    // Track whether we are waiting for the first non-clear code.
    let mut after_clear = true;

    loop {
        let raw = reader.read_bits(code_width).ok_or(GifError::Truncated)?;
        let code = raw as u16;

        if code == end_code {
            break;
        }

        if code == clear_code {
            prefix.truncate(table_initial_size);
            suffix.truncate(table_initial_size);
            code_width = min_code_size as u32 + 1;
            prev_code = None;
            after_clear = true;
            continue;
        }

        // ── First code after a clear ──────────────────────────────────────────
        if after_clear {
            // The first code after Clear must be in the initial table.
            if (code as usize) >= table_initial_size {
                return Err(GifError::LzwError(format!(
                    "first code after clear ({}) exceeds initial table size ({})",
                    code, table_initial_size
                )));
            }
            output.push(suffix[code as usize]);
            prev_code = Some(code);
            after_clear = false;
            // No new entry is added for the first code.
            continue;
        }

        let prev = prev_code.ok_or_else(|| GifError::LzwError("no prev code".into()))?;
        let next_code = prefix.len() as u16;

        if (code as usize) < prefix.len() {
            // ── Known code path ───────────────────────────────────────────────
            // Decode the string for `code`.
            walk_chain(code, &prefix, &suffix, &mut scratch)?;
            scratch.reverse();

            // New table entry: prev + first_byte(code).
            let fb = scratch[0];
            if next_code < 4096 {
                prefix.push(prev);
                suffix.push(fb);
            }

            output.extend_from_slice(&scratch);
        } else if code == next_code {
            // ── Special case: code == next table slot ─────────────────────────
            // The string is prev_string + prev_string[0].
            let fb = first_byte(prev, &prefix, &suffix);

            if next_code < 4096 {
                prefix.push(prev);
                suffix.push(fb);
            }

            // Now decode the newly-added entry (which equals `prev` + `fb`).
            walk_chain(prev, &prefix, &suffix, &mut scratch)?;
            scratch.reverse();
            scratch.push(fb);

            output.extend_from_slice(&scratch);
        } else {
            return Err(GifError::LzwError(format!(
                "code {} is beyond next expected entry {}",
                code, next_code
            )));
        }

        prev_code = Some(code);

        // Grow code_width once the table has filled the current slot range.
        // The threshold is: when table size == 2^code_width (i.e. when the
        // table is exactly full), the *next* read uses code_width + 1.
        if code_width < 12 && prefix.len() == (1 << code_width) {
            code_width += 1;
        }
    }

    Ok(output)
}

// ─────────────────────────────────────────────────────────────────────────────
// Public decoder
// ─────────────────────────────────────────────────────────────────────────────

/// Decode a GIF87a or GIF89a byte stream into raw RGB pixels.
///
/// Only the first frame is decoded.  The result always uses 3 bytes per pixel
/// (R, G, B) regardless of the original colour depth.
pub fn gif_decode(bytes: &[u8]) -> Result<RawDecodeResult, GifError> {
    if bytes.len() < 6 {
        return Err(GifError::Truncated);
    }

    // ── Header ────────────────────────────────────────────────────────────────
    if &bytes[..6] != b"GIF89a" && &bytes[..6] != b"GIF87a" {
        return Err(GifError::Invalid(format!(
            "bad GIF signature: {:?}",
            &bytes[..6.min(bytes.len())]
        )));
    }

    if bytes.len() < 13 {
        return Err(GifError::Truncated);
    }

    // ── Logical Screen Descriptor ─────────────────────────────────────────────
    let canvas_width = u16::from_le_bytes([bytes[6], bytes[7]]) as usize;
    let canvas_height = u16::from_le_bytes([bytes[8], bytes[9]]) as usize;
    let lsd_packed = bytes[10];
    let has_gct = (lsd_packed & 0x80) != 0;
    let gct_size_field = lsd_packed & 0x07;
    let gct_entries = 1usize << (gct_size_field as usize + 1);

    let mut pos = 13usize; // Skip LSD (7 bytes past header).

    // ── Global Colour Table ───────────────────────────────────────────────────
    let mut global_palette: Vec<[u8; 3]> = Vec::new();
    if has_gct {
        let gct_bytes = gct_entries * 3;
        if pos + gct_bytes > bytes.len() {
            return Err(GifError::Truncated);
        }
        for i in 0..gct_entries {
            global_palette.push([
                bytes[pos + i * 3],
                bytes[pos + i * 3 + 1],
                bytes[pos + i * 3 + 2],
            ]);
        }
        pos += gct_bytes;
    }

    // ── Walk blocks until we hit an Image Descriptor ──────────────────────────
    loop {
        if pos >= bytes.len() {
            return Err(GifError::Truncated);
        }
        match bytes[pos] {
            0x3B => {
                // Trailer before we found an image.
                return Err(GifError::Invalid("GIF contains no image frames".into()));
            }
            0x21 => {
                // Extension block.
                pos += 1;
                if pos >= bytes.len() {
                    return Err(GifError::Truncated);
                }
                pos += 1; // Skip extension label.
                          // Skip all sub-blocks.
                loop {
                    if pos >= bytes.len() {
                        return Err(GifError::Truncated);
                    }
                    let block_len = bytes[pos] as usize;
                    pos += 1;
                    if block_len == 0 {
                        break;
                    }
                    pos += block_len;
                    if pos > bytes.len() {
                        return Err(GifError::Truncated);
                    }
                }
            }
            0x2C => {
                // Image Descriptor.
                break;
            }
            other => {
                return Err(GifError::Invalid(format!(
                    "unexpected block byte 0x{:02X} at offset {}",
                    other, pos
                )));
            }
        }
    }

    // ── Image Descriptor ──────────────────────────────────────────────────────
    // pos now points at 0x2C.
    if pos + 10 > bytes.len() {
        return Err(GifError::Truncated);
    }
    let img_width = u16::from_le_bytes([bytes[pos + 5], bytes[pos + 6]]) as usize;
    let img_height = u16::from_le_bytes([bytes[pos + 7], bytes[pos + 8]]) as usize;
    let img_packed = bytes[pos + 9];
    let has_lct = (img_packed & 0x80) != 0;
    let lct_size_field = img_packed & 0x07;
    pos += 10; // Consume Image Descriptor.

    // ── Local Colour Table (if present) ───────────────────────────────────────
    let active_palette: Vec<[u8; 3]> = if has_lct {
        let lct_entries = 1usize << (lct_size_field as usize + 1);
        let lct_bytes = lct_entries * 3;
        if pos + lct_bytes > bytes.len() {
            return Err(GifError::Truncated);
        }
        let mut lct = Vec::with_capacity(lct_entries);
        for i in 0..lct_entries {
            lct.push([
                bytes[pos + i * 3],
                bytes[pos + i * 3 + 1],
                bytes[pos + i * 3 + 2],
            ]);
        }
        pos += lct_bytes;
        lct
    } else {
        global_palette
    };

    if active_palette.is_empty() {
        return Err(GifError::Invalid("no colour table available".into()));
    }

    // ── LZW Minimum Code Size ─────────────────────────────────────────────────
    if pos >= bytes.len() {
        return Err(GifError::Truncated);
    }
    let min_code_size = bytes[pos];
    pos += 1;

    if !(2..=11).contains(&min_code_size) {
        return Err(GifError::Invalid(format!(
            "invalid LZW minimum code size: {}",
            min_code_size
        )));
    }

    // ── Image Sub-blocks ──────────────────────────────────────────────────────
    if pos >= bytes.len() {
        return Err(GifError::Truncated);
    }
    let (lzw_data, _consumed) = read_sub_blocks(&bytes[pos..])?;

    // ── LZW Decompress ────────────────────────────────────────────────────────
    let indices = lzw_decompress(&lzw_data, min_code_size)?;

    // Use canvas dimensions if image dimensions are 0 (degenerate case).
    let out_width = if img_width == 0 {
        canvas_width
    } else {
        img_width
    };
    let out_height = if img_height == 0 {
        canvas_height
    } else {
        img_height
    };
    let pixel_count = out_width * out_height;

    // Map indices → RGB.
    let mut pixels: Vec<u8> = Vec::with_capacity(pixel_count * 3);
    for i in 0..pixel_count {
        let idx = if i < indices.len() {
            indices[i] as usize
        } else {
            0
        };
        let color = if idx < active_palette.len() {
            active_palette[idx]
        } else {
            [0, 0, 0]
        };
        pixels.push(color[0]);
        pixels.push(color[1]);
        pixels.push(color[2]);
    }

    Ok(RawDecodeResult {
        width: out_width,
        height: out_height,
        pixels,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a solid-colour `width × height` RGB pixel buffer.
    fn solid_rgb(width: u32, height: u32, r: u8, g: u8, b: u8) -> Vec<u8> {
        let n = (width * height) as usize;
        let mut buf = Vec::with_capacity(n * 3);
        for _ in 0..n {
            buf.push(r);
            buf.push(g);
            buf.push(b);
        }
        buf
    }

    /// Build a simple gradient `width × height` RGB pixel buffer.
    fn gradient_rgb(width: u32, height: u32) -> Vec<u8> {
        let mut buf = Vec::new();
        for y in 0..height {
            for x in 0..width {
                buf.push(((x * 255) / width.max(1)) as u8);
                buf.push(((y * 255) / height.max(1)) as u8);
                buf.push(128u8);
            }
        }
        buf
    }

    #[test]
    fn test_gif_header() {
        let pixels = solid_rgb(4, 4, 200, 10, 10);
        let gif = gif_encode_rgb(4, 4, &pixels).expect("encode");
        assert!(
            gif.starts_with(b"GIF89a"),
            "GIF must start with GIF89a signature"
        );
    }

    #[test]
    fn test_gif_trailer() {
        let pixels = solid_rgb(4, 4, 10, 200, 10);
        let gif = gif_encode_rgb(4, 4, &pixels).expect("encode");
        assert_eq!(
            *gif.last().expect("non-empty"),
            0x3Bu8,
            "GIF must end with 0x3B trailer"
        );
    }

    #[test]
    fn test_gif_roundtrip_solid_color() {
        let pixels = solid_rgb(4, 4, 220, 20, 20);
        let gif = gif_encode_rgb(4, 4, &pixels).expect("encode");
        let result = gif_decode(&gif).expect("decode");
        // At least half the pixels should be "reddish".
        let reddish = result
            .pixels
            .chunks_exact(3)
            .filter(|p| p[0] >= 200 && p[1] <= 50 && p[2] <= 50)
            .count();
        let total = result.width * result.height;
        assert!(
            reddish * 2 >= total,
            "expected reddish pixels: got {}/{} reddish",
            reddish,
            total
        );
    }

    #[test]
    fn test_gif_roundtrip_size() {
        let pixels = gradient_rgb(8, 8);
        let gif = gif_encode_rgb(8, 8, &pixels).expect("encode");
        let result = gif_decode(&gif).expect("decode");
        assert_eq!(result.width, 8);
        assert_eq!(result.height, 8);
    }

    #[test]
    fn test_gif_lzw_clear_code() {
        // The LZW stream for min_code_size=8 starts with the clear code (256 = 0x100).
        // In LSB-first 9-bit packing the first 9 bits are:
        //   0x100 = 0b1_0000_0000 → byte[0] = 0x00, bits [0..1] of byte[1] = 0b1
        // The stream is wrapped in sub-blocks; the first sub-block byte is the
        // length, then the data starts at byte[1] of the encoded output.
        let pixels = solid_rgb(2, 2, 100, 100, 100);
        let gif = gif_encode_rgb(2, 2, &pixels).expect("encode");

        // Locate the LZW minimum code size byte.
        // Offset: 6 (header) + 7 (LSD) + palette_bytes + 10 (image descriptor) + 1 (min code).
        // We search for 0x2C (image separator) to locate the image descriptor.
        let sep_pos = gif
            .iter()
            .position(|&b| b == 0x2C)
            .expect("image separator");
        let min_code_size_pos = sep_pos + 10;
        assert!(
            min_code_size_pos + 2 < gif.len(),
            "GIF too short for LZW data"
        );
        let min_code_size = gif[min_code_size_pos];
        let clear_code = 1u32 << min_code_size;

        // The first sub-block starts right after the min_code_size byte.
        let sub_block_start = min_code_size_pos + 1;
        assert!(
            sub_block_start < gif.len(),
            "No sub-block after min_code_size"
        );
        let sub_block_len = gif[sub_block_start] as usize;
        assert!(sub_block_len > 0, "First sub-block must be non-empty");

        // Extract the first sub-block's data bytes.
        let data_start = sub_block_start + 1;
        assert!(
            data_start + sub_block_len <= gif.len(),
            "Sub-block data truncated"
        );
        let lzw_data = &gif[data_start..data_start + sub_block_len];

        // Read the first `min_code_size + 1` bits LSB-first and verify they
        // equal the clear code.
        let code_width = min_code_size as u32 + 1;
        let mut bit_buf = 0u32;
        for (i, &byte) in lzw_data.iter().take(4).enumerate() {
            bit_buf |= (byte as u32) << (i * 8);
        }
        let mask = (1u32 << code_width) - 1;
        let first_code = bit_buf & mask;
        assert_eq!(
            first_code, clear_code,
            "First LZW code must be the clear code ({}); got {}",
            clear_code, first_code
        );
    }

    #[test]
    fn test_gif_decode_invalid_returns_error() {
        let bad = b"NOT_GIF";
        let result = gif_decode(bad);
        assert!(result.is_err(), "Expected error for invalid GIF magic");
    }

    #[test]
    fn test_gif_decode_empty_returns_error() {
        let result = gif_decode(&[]);
        assert!(result.is_err(), "Expected error for empty input");
    }

    #[test]
    fn test_gif_encode_dimensions_preserved() {
        let pixels = gradient_rgb(10, 6);
        let gif = gif_encode_rgb(10, 6, &pixels).expect("encode");
        // Logical Screen Descriptor width at bytes 6-7, height at bytes 8-9.
        let w = u16::from_le_bytes([gif[6], gif[7]]) as u32;
        let h = u16::from_le_bytes([gif[8], gif[9]]) as u32;
        assert_eq!(w, 10, "encoded width mismatch");
        assert_eq!(h, 6, "encoded height mismatch");
    }
}
