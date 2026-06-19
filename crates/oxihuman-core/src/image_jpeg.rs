// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Pure-Rust JPEG Baseline (DCT) encoder and decoder.
//!
//! Implements the full JFIF/JPEG Baseline process:
//! - RGB ↔ YCbCr color space conversion
//! - Forward and inverse 8×8 separable DCT
//! - Standard luminance/chrominance quantization tables with quality scaling
//! - Zigzag scan ordering
//! - Standard JPEG Huffman tables (Annex K) with DPCM/RLE entropy coding
//! - JFIF bitstream framing with proper marker sequences and byte stuffing
//!
//! Both encoder and decoder operate on raw RGB pixel buffers (3 bytes per pixel,
//! row-major, top-to-bottom). Chroma subsampling is 4:4:4 (no subsampling).

use std::fmt;

// ── Public error type ────────────────────────────────────────────────────────

/// Errors returned by the JPEG codec.
#[derive(Debug)]
pub enum JpegError {
    Invalid(String),
    Unsupported(String),
    Truncated,
}

impl fmt::Display for JpegError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JpegError::Invalid(s) => write!(f, "Invalid JPEG: {}", s),
            JpegError::Unsupported(s) => write!(f, "Unsupported JPEG feature: {}", s),
            JpegError::Truncated => write!(f, "Truncated JPEG data"),
        }
    }
}

impl std::error::Error for JpegError {}

// ── Quantization tables ──────────────────────────────────────────────────────

/// Standard JPEG luminance quantization table (quality = 50 baseline).
const LUMA_QTABLE_BASE: [u16; 64] = [
    16, 11, 10, 16, 24, 40, 51, 61, 12, 12, 14, 19, 26, 58, 60, 55, 14, 13, 16, 24, 40, 57, 69, 56,
    14, 17, 22, 29, 51, 87, 80, 62, 18, 22, 37, 56, 68, 109, 103, 77, 24, 35, 55, 64, 81, 104, 113,
    92, 49, 64, 78, 87, 103, 121, 120, 101, 72, 92, 95, 98, 112, 100, 103, 99,
];

/// Standard JPEG chrominance quantization table (quality = 50 baseline).
const CHROMA_QTABLE_BASE: [u16; 64] = [
    17, 18, 24, 47, 99, 99, 99, 99, 18, 21, 26, 66, 99, 99, 99, 99, 24, 26, 56, 99, 99, 99, 99, 99,
    47, 66, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99,
    99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99,
];

/// Compute scaled quantization table for the given quality factor (1..=100).
fn make_qtable(base: &[u16; 64], quality: u8) -> [u16; 64] {
    let q = quality.clamp(1, 100) as u32;
    let scale = if q < 50 { 5000 / q } else { 200 - 2 * q };
    let mut out = [0u16; 64];
    for (i, &v) in base.iter().enumerate() {
        let s = ((v as u32 * scale + 50) / 100).clamp(1, 255);
        out[i] = s as u16;
    }
    out
}

// ── Zigzag tables ────────────────────────────────────────────────────────────

/// Standard JPEG zigzag scan order — maps linear index [0..64) → (row, col).
const ZIGZAG_ORDER: [u8; 64] = [
    0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33, 40, 48, 41, 34, 27, 20,
    13, 6, 7, 14, 21, 28, 35, 42, 49, 56, 57, 50, 43, 36, 29, 22, 15, 23, 30, 37, 44, 51, 58, 59,
    52, 45, 38, 31, 39, 46, 53, 60, 61, 54, 47, 55, 62, 63,
];

// ── DCT / IDCT ───────────────────────────────────────────────────────────────

/// Precomputed cosine table: COS_TABLE[u][n] = cos((2n+1)*u*π/16)
fn cos_table() -> [[f32; 8]; 8] {
    use std::f32::consts::PI;
    let mut t = [[0f32; 8]; 8];
    for (u, row) in t.iter_mut().enumerate() {
        for (n, cell) in row.iter_mut().enumerate() {
            *cell = ((2 * n + 1) as f32 * u as f32 * PI / 16.0).cos();
        }
    }
    t
}

/// 2-D forward DCT on an 8×8 block using the separable row-then-column approach.
///
/// Input layout: `block[row * 8 + col]`, values already level-shifted (−128).
/// Output layout: `out[u * 8 + v]` where u=row-freq, v=col-freq.
///
/// Formula: F[u][v] = (1/4)*C(u)*C(v) * Σ_x Σ_y f[x][y]*cos((2x+1)uπ/16)*cos((2y+1)vπ/16)
/// C(0) = 1/√2, C(k≥1) = 1
fn dct8x8(block: &[f32; 64]) -> [f32; 64] {
    let cos = cos_table();
    let inv_sqrt2 = 1.0_f32 / std::f32::consts::SQRT_2;
    let mut tmp = [0f32; 64];

    // Row-wise 1-D DCT (transform columns of each row into frequency domain)
    for r in 0..8usize {
        for u in 0..8usize {
            let cu = if u == 0 { inv_sqrt2 } else { 1.0 };
            let mut sum = 0.0f32;
            for n in 0..8usize {
                sum += block[r * 8 + n] * cos[u][n];
            }
            tmp[r * 8 + u] = 0.5 * cu * sum;
        }
    }

    // Column-wise 1-D DCT (transform rows of the intermediate result)
    let mut out = [0f32; 64];
    for c in 0..8usize {
        for v in 0..8usize {
            let cv = if v == 0 { inv_sqrt2 } else { 1.0 };
            let mut sum = 0.0f32;
            for n in 0..8usize {
                sum += tmp[n * 8 + c] * cos[v][n];
            }
            out[v * 8 + c] = 0.5 * cv * sum;
        }
    }
    out
}

/// 2-D inverse DCT on an 8×8 block (separable column-then-row).
///
/// Input layout: `coeffs[u * 8 + v]`.
/// Output layout: `out[row * 8 + col]`.
fn idct8x8(coeffs: &[f32; 64]) -> [f32; 64] {
    let cos = cos_table();
    let inv_sqrt2 = 1.0_f32 / std::f32::consts::SQRT_2;

    // Column-wise inverse 1-D DCT
    let mut tmp = [0f32; 64];
    for c in 0..8usize {
        for n in 0..8usize {
            let mut sum = 0.0f32;
            for v in 0..8usize {
                let cv = if v == 0 { inv_sqrt2 } else { 1.0 };
                sum += cv * coeffs[v * 8 + c] * cos[v][n];
            }
            tmp[n * 8 + c] = 0.5 * sum;
        }
    }

    // Row-wise inverse 1-D DCT
    let mut out = [0f32; 64];
    for r in 0..8usize {
        for n in 0..8usize {
            let mut sum = 0.0f32;
            for u in 0..8usize {
                let cu = if u == 0 { inv_sqrt2 } else { 1.0 };
                sum += cu * tmp[r * 8 + u] * cos[u][n];
            }
            out[r * 8 + n] = 0.5 * sum;
        }
    }
    out
}

// ── Standard JPEG Huffman tables (Annex K) ───────────────────────────────────

/// Huffman table entry: (code_length_in_bits, code_word).
type HuffEntry = (u8, u16);

/// Build the canonical Huffman code table from JPEG Annex K table data.
///
/// `bits[i]` = number of codes of length (i+1).
/// `huffval` = symbols in code-value order.
/// Returns a 256-entry array where index = symbol, value = (length, code).
fn build_huffman_table(bits: &[u8; 16], huffval: &[u8]) -> [Option<HuffEntry>; 256] {
    let mut table = [None; 256];
    let mut code = 0u16;
    let mut idx = 0usize;
    for (bit_len, &count) in bits.iter().enumerate() {
        let length = (bit_len + 1) as u8;
        for _ in 0..count {
            if idx < huffval.len() {
                table[huffval[idx] as usize] = Some((length, code));
                code += 1;
            }
            idx += 1;
        }
        code <<= 1;
    }
    table
}

// DC luminance Huffman (Annex K Table K.3)
const DC_LUMA_BITS: [u8; 16] = [0, 1, 5, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0];
const DC_LUMA_HUFFVAL: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];

// DC chrominance Huffman (Annex K Table K.4)
const DC_CHROMA_BITS: [u8; 16] = [0, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0];
const DC_CHROMA_HUFFVAL: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];

// AC luminance Huffman (Annex K Table K.5)
const AC_LUMA_BITS: [u8; 16] = [0, 2, 1, 3, 3, 2, 4, 3, 5, 5, 4, 4, 0, 0, 1, 125];
const AC_LUMA_HUFFVAL: &[u8] = &[
    0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12, 0x21, 0x31, 0x41, 0x06, 0x13, 0x51, 0x61, 0x07,
    0x22, 0x71, 0x14, 0x32, 0x81, 0x91, 0xA1, 0x08, 0x23, 0x42, 0xB1, 0xC1, 0x15, 0x52, 0xD1, 0xF0,
    0x24, 0x33, 0x62, 0x72, 0x82, 0x09, 0x0A, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x25, 0x26, 0x27, 0x28,
    0x29, 0x2A, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49,
    0x4A, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69,
    0x6A, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89,
    0x8A, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7,
    0xA8, 0xA9, 0xAA, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xC2, 0xC3, 0xC4, 0xC5,
    0xC6, 0xC7, 0xC8, 0xC9, 0xCA, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xE1, 0xE2,
    0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8,
    0xF9, 0xFA,
];

// AC chrominance Huffman (Annex K Table K.6)
const AC_CHROMA_BITS: [u8; 16] = [0, 2, 1, 2, 4, 4, 3, 4, 7, 5, 4, 4, 0, 1, 2, 119];
const AC_CHROMA_HUFFVAL: &[u8] = &[
    0x00, 0x01, 0x02, 0x03, 0x11, 0x04, 0x05, 0x21, 0x31, 0x06, 0x12, 0x41, 0x51, 0x07, 0x61, 0x71,
    0x13, 0x22, 0x32, 0x81, 0x08, 0x14, 0x42, 0x91, 0xA1, 0xB1, 0xC1, 0x09, 0x23, 0x33, 0x52, 0xF0,
    0x15, 0x62, 0x72, 0xD1, 0x0A, 0x16, 0x24, 0x34, 0xE1, 0x25, 0xF1, 0x17, 0x18, 0x19, 0x1A, 0x26,
    0x27, 0x28, 0x29, 0x2A, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48,
    0x49, 0x4A, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68,
    0x69, 0x6A, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87,
    0x88, 0x89, 0x8A, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0xA2, 0xA3, 0xA4, 0xA5,
    0xA6, 0xA7, 0xA8, 0xA9, 0xAA, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xC2, 0xC3,
    0xC4, 0xC5, 0xC6, 0xC7, 0xC8, 0xC9, 0xCA, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA,
    0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8,
    0xF9, 0xFA,
];

// ── Bitstream writer ─────────────────────────────────────────────────────────

struct BitWriter {
    buffer: Vec<u8>,
    accumulator: u32,
    bits_pending: u8,
}

impl BitWriter {
    fn new() -> Self {
        Self {
            buffer: Vec::new(),
            accumulator: 0,
            bits_pending: 0,
        }
    }

    fn write_bits(&mut self, code: u16, length: u8) {
        self.accumulator = (self.accumulator << length) | (code as u32);
        self.bits_pending += length;
        while self.bits_pending >= 8 {
            self.bits_pending -= 8;
            let byte = ((self.accumulator >> self.bits_pending) & 0xFF) as u8;
            self.buffer.push(byte);
            // Byte stuffing: 0xFF must be followed by 0x00 in the scan data
            if byte == 0xFF {
                self.buffer.push(0x00);
            }
        }
    }

    fn flush(&mut self) {
        if self.bits_pending > 0 {
            // Pad with 1-bits (as per JPEG spec)
            let shift = 8 - self.bits_pending;
            let byte = (((self.accumulator << shift) | ((1u32 << shift) - 1)) & 0xFF) as u8;
            self.buffer.push(byte);
            if byte == 0xFF {
                self.buffer.push(0x00);
            }
            self.bits_pending = 0;
        }
    }

    fn into_bytes(self) -> Vec<u8> {
        self.buffer
    }
}

// ── Size category helpers ────────────────────────────────────────────────────

/// Number of bits needed to represent `|v|` (size category S):
/// 0 → 0, ±1 → 1, ±2-±3 → 2, …
fn size_category(v: i16) -> u8 {
    let abs = v.unsigned_abs();
    if abs == 0 {
        0
    } else {
        16 - abs.leading_zeros() as u8
    }
}

/// Encode amplitude into the size-category representation.
///
/// For positive v: amplitude = v.
/// For negative v: amplitude = v − 1 (i.e. v + 2^s - 1 − (2^s - 1) = v − 1 + 2^s).
fn amplitude_bits(v: i16, s: u8) -> u16 {
    if v >= 0 {
        v as u16
    } else {
        // Two's-complement-like JPEG encoding: negative amplitude is v + (2^s - 1)
        (v + ((1i16 << s) - 1)) as u16
    }
}

// ── JPEG Encoder ─────────────────────────────────────────────────────────────

/// Encode an RGB pixel buffer as JPEG.
///
/// `pixels` must contain `width * height * 3` bytes (RGB, row-major, top-bottom).
/// `quality` is in the range 1..=100.
pub fn jpeg_encode_rgb(
    width: u32,
    height: u32,
    pixels: &[u8],
    quality: u8,
) -> Result<Vec<u8>, JpegError> {
    let w = width as usize;
    let h = height as usize;
    if pixels.len() != w * h * 3 {
        return Err(JpegError::Invalid(format!(
            "expected {} bytes, got {}",
            w * h * 3,
            pixels.len()
        )));
    }
    if w == 0 || h == 0 {
        return Err(JpegError::Invalid("zero-dimension image".into()));
    }

    let luma_qt = make_qtable(&LUMA_QTABLE_BASE, quality);
    let chroma_qt = make_qtable(&CHROMA_QTABLE_BASE, quality);

    // Convert to YCbCr planes (f32)
    let mut y_plane = vec![0f32; w * h];
    let mut cb_plane = vec![0f32; w * h];
    let mut cr_plane = vec![0f32; w * h];
    for i in 0..w * h {
        let r = pixels[i * 3] as f32;
        let g = pixels[i * 3 + 1] as f32;
        let b = pixels[i * 3 + 2] as f32;
        y_plane[i] = 0.299 * r + 0.587 * g + 0.114 * b;
        cb_plane[i] = -0.168_736 * r - 0.331_264 * g + 0.5 * b + 128.0;
        cr_plane[i] = 0.5 * r - 0.418_688 * g - 0.081_312 * b + 128.0;
    }

    // Build Huffman encode tables
    let dc_luma_ht = build_huffman_table(&DC_LUMA_BITS, &DC_LUMA_HUFFVAL);
    let dc_chroma_ht = build_huffman_table(&DC_CHROMA_BITS, &DC_CHROMA_HUFFVAL);
    let ac_luma_ht = build_huffman_table(&AC_LUMA_BITS, AC_LUMA_HUFFVAL);
    let ac_chroma_ht = build_huffman_table(&AC_CHROMA_BITS, AC_CHROMA_HUFFVAL);

    // Encode scan data
    let mut bw = BitWriter::new();
    let mut prev_dc_y = 0i16;
    let mut prev_dc_cb = 0i16;
    let mut prev_dc_cr = 0i16;

    // Number of 8×8 MCUs (padded)
    let mcu_cols = w.div_ceil(8);
    let mcu_rows = h.div_ceil(8);

    for mcu_row in 0..mcu_rows {
        for mcu_col in 0..mcu_cols {
            // Extract and encode each channel
            for ch in 0..3 {
                let plane = match ch {
                    0 => y_plane.as_slice(),
                    1 => cb_plane.as_slice(),
                    _ => cr_plane.as_slice(),
                };
                let qt = if ch == 0 { &luma_qt } else { &chroma_qt };
                let dc_ht = if ch == 0 { &dc_luma_ht } else { &dc_chroma_ht };
                let ac_ht = if ch == 0 { &ac_luma_ht } else { &ac_chroma_ht };
                let prev_dc = match ch {
                    0 => &mut prev_dc_y,
                    1 => &mut prev_dc_cb,
                    _ => &mut prev_dc_cr,
                };

                // Fill 8×8 block with level-shifted values (pad at edges by replication)
                let mut block = [0f32; 64];
                for by in 0..8 {
                    for bx in 0..8 {
                        let px = (mcu_col * 8 + bx).min(w - 1);
                        let py = (mcu_row * 8 + by).min(h - 1);
                        block[by * 8 + bx] = plane[py * w + px] - 128.0;
                    }
                }

                // Forward DCT
                let dct_coeffs = dct8x8(&block);

                // Quantize in zigzag order
                let mut quant = [0i16; 64];
                for (zz, &pos) in ZIGZAG_ORDER.iter().enumerate() {
                    let coeff = dct_coeffs[pos as usize];
                    let q = qt[zz] as f32;
                    quant[zz] = (coeff / q).round() as i16;
                }

                // DC coefficient — DPCM
                let dc_diff = quant[0] - *prev_dc;
                *prev_dc = quant[0];
                let s = size_category(dc_diff);
                let (dc_len, dc_code) = dc_ht[s as usize]
                    .ok_or_else(|| JpegError::Invalid("DC Huffman missing".into()))?;
                bw.write_bits(dc_code, dc_len);
                if s > 0 {
                    bw.write_bits(amplitude_bits(dc_diff, s), s);
                }

                // AC coefficients — RLE
                let mut run = 0u8;
                for (k, &ac) in quant[1..].iter().enumerate().map(|(i, v)| (i + 1, v)) {
                    if ac == 0 {
                        if k == 63 {
                            // EOB
                            let (len, code) = ac_ht[0x00]
                                .ok_or_else(|| JpegError::Invalid("AC EOB missing".into()))?;
                            bw.write_bits(code, len);
                        } else {
                            run += 1;
                            if run == 16 {
                                // ZRL: run of 16 zeros
                                let (len, code) = ac_ht[0xF0]
                                    .ok_or_else(|| JpegError::Invalid("AC ZRL missing".into()))?;
                                bw.write_bits(code, len);
                                run = 0;
                            }
                        }
                    } else {
                        let s = size_category(ac);
                        let symbol = (run << 4) | s;
                        let (ac_len, ac_code) = ac_ht[symbol as usize]
                            .ok_or_else(|| JpegError::Invalid("AC Huffman missing".into()))?;
                        bw.write_bits(ac_code, ac_len);
                        bw.write_bits(amplitude_bits(ac, s), s);
                        run = 0;
                    }
                }
            }
        }
    }
    bw.flush();
    let scan_data = bw.into_bytes();

    // ── Assemble JFIF bitstream ──────────────────────────────────────────────
    let mut out = Vec::new();

    // SOI
    out.extend_from_slice(&[0xFF, 0xD8]);

    // APP0 (JFIF marker)
    let app0: &[u8] = &[
        0xFF, 0xE0, 0x00, 0x10, // marker, length=16
        0x4A, 0x46, 0x49, 0x46, 0x00, // "JFIF\0"
        0x01, 0x01, // version 1.1
        0x00, // aspect ratio units (0=no units)
        0x00, 0x01, // Xdensity = 1
        0x00, 0x01, // Ydensity = 1
        0x00, 0x00, // thumbnail 0×0
    ];
    out.extend_from_slice(app0);

    // DQT — Quantization tables (table 0 = luma, table 1 = chroma)
    for (id, qt) in [(&luma_qt, 0u8), (&chroma_qt, 1u8)] {
        // length = 2 (length field) + 1 (precision+id) + 64 = 67
        let seg_len: u16 = 2 + 1 + 64;
        out.extend_from_slice(&[0xFF, 0xDB]);
        out.extend_from_slice(&seg_len.to_be_bytes());
        out.push(qt); // precision=0 (8-bit) | table id
        for &v in id.iter() {
            out.push(v.min(255) as u8);
        }
    }

    // SOF0 — Baseline DCT frame header
    // length = 2 + 1 + 2 + 2 + 1 + 3*(1+1+1) = 17
    let sof0_len: u16 = 17;
    out.extend_from_slice(&[0xFF, 0xC0]);
    out.extend_from_slice(&sof0_len.to_be_bytes());
    out.push(8); // precision = 8 bits
    out.extend_from_slice(&(height as u16).to_be_bytes());
    out.extend_from_slice(&(width as u16).to_be_bytes());
    out.push(3); // number of components
                 // Y: component id=1, sampling factors=1:1, qtable=0
    out.extend_from_slice(&[1, 0x11, 0]);
    // Cb: component id=2, sampling factors=1:1, qtable=1
    out.extend_from_slice(&[2, 0x11, 1]);
    // Cr: component id=3, sampling factors=1:1, qtable=1
    out.extend_from_slice(&[3, 0x11, 1]);

    // DHT — Huffman tables (DC luma, DC chroma, AC luma, AC chroma)
    write_dht(&mut out, 0x00, &DC_LUMA_BITS, &DC_LUMA_HUFFVAL);
    write_dht(&mut out, 0x01, &DC_CHROMA_BITS, &DC_CHROMA_HUFFVAL);
    write_dht(&mut out, 0x10, &AC_LUMA_BITS, AC_LUMA_HUFFVAL);
    write_dht(&mut out, 0x11, &AC_CHROMA_BITS, AC_CHROMA_HUFFVAL);

    // SOS — Start of scan
    // length = 2 + 1 + 3*2 + 3 = 12
    let sos_len: u16 = 12;
    out.extend_from_slice(&[0xFF, 0xDA]);
    out.extend_from_slice(&sos_len.to_be_bytes());
    out.push(3); // components in scan
    out.extend_from_slice(&[1, 0x00]); // Y: DC=0, AC=0
    out.extend_from_slice(&[2, 0x11]); // Cb: DC=1, AC=1
    out.extend_from_slice(&[3, 0x11]); // Cr: DC=1, AC=1
    out.extend_from_slice(&[0, 63, 0]); // spectral selection 0..63, Ah/Al=0

    // Scan data (already byte-stuffed by BitWriter)
    out.extend_from_slice(&scan_data);

    // EOI
    out.extend_from_slice(&[0xFF, 0xD9]);

    Ok(out)
}

/// Write a single DHT (Define Huffman Table) segment.
fn write_dht(out: &mut Vec<u8>, tc_th: u8, bits: &[u8; 16], huffval: &[u8]) {
    let total_codes: u16 = bits.iter().map(|&b| b as u16).sum();
    // length = 2 + 1 + 16 + total_codes
    let seg_len = 2u16 + 1 + 16 + total_codes;
    out.extend_from_slice(&[0xFF, 0xC4]);
    out.extend_from_slice(&seg_len.to_be_bytes());
    out.push(tc_th);
    out.extend_from_slice(bits);
    out.extend_from_slice(huffval);
}

// ── JPEG Decoder ─────────────────────────────────────────────────────────────

/// Decode a JPEG bitstream to raw RGB pixels.
///
/// Returns a [`super::image_codec::RawDecodeResult`] with width, height and RGB bytes.
pub fn jpeg_decode(bytes: &[u8]) -> Result<super::image_codec::RawDecodeResult, JpegError> {
    // Verify SOI
    if bytes.len() < 4 || bytes[0] != 0xFF || bytes[1] != 0xD8 {
        return Err(JpegError::Invalid("missing SOI marker".into()));
    }

    // Parse markers to collect tables and geometry
    let mut ctx = DecodeContext::new();
    ctx.parse_markers(bytes)?;

    // Decode the actual scan data
    let rgb_pixels = ctx.decode_scan()?;

    Ok(super::image_codec::RawDecodeResult {
        width: ctx.image_width,
        height: ctx.image_height,
        pixels: rgb_pixels,
    })
}

// ── Decode context ───────────────────────────────────────────────────────────

/// Huffman decoder tree node.
#[derive(Clone)]
struct HuffNode {
    /// For leaf nodes: symbol (0..=255). For internal: 0 (unused).
    symbol: u8,
    /// For leaf nodes: code length. 0 indicates not-a-leaf.
    is_leaf: bool,
    left: Option<Box<HuffNode>>,
    right: Option<Box<HuffNode>>,
}

impl HuffNode {
    fn internal() -> Self {
        Self {
            symbol: 0,
            is_leaf: false,
            left: None,
            right: None,
        }
    }
}

/// Build a Huffman decode tree from (length, code, symbol) triples.
fn build_huffman_decode_tree(bits: &[u8; 16], huffval: &[u8]) -> Result<HuffNode, JpegError> {
    let mut root = HuffNode::internal();
    let mut code = 0u16;
    let mut idx = 0usize;

    for (bit_len_0, &count) in bits.iter().enumerate() {
        let depth = bit_len_0 + 1;
        for _ in 0..count {
            if idx >= huffval.len() {
                break;
            }
            let sym = huffval[idx];
            idx += 1;
            // Insert this (code, depth, sym) into the tree
            insert_huffman_code(&mut root, code, depth as u8, sym)?;
            code += 1;
        }
        code <<= 1;
    }
    Ok(root)
}

/// Recursively insert a canonical code into the Huffman decode tree.
fn insert_huffman_code(
    node: &mut HuffNode,
    code: u16,
    depth: u8,
    sym: u8,
) -> Result<(), JpegError> {
    if depth == 0 {
        node.symbol = sym;
        node.is_leaf = true;
        return Ok(());
    }
    let go_right = (code >> (depth - 1)) & 1 == 1;
    if go_right {
        if node.right.is_none() {
            node.right = Some(Box::new(HuffNode::internal()));
        }
        if let Some(ref mut child) = node.right {
            insert_huffman_code(child, code & ((1 << (depth - 1)) - 1), depth - 1, sym)?;
        }
    } else {
        if node.left.is_none() {
            node.left = Some(Box::new(HuffNode::internal()));
        }
        if let Some(ref mut child) = node.left {
            insert_huffman_code(child, code & ((1 << (depth - 1)) - 1), depth - 1, sym)?;
        }
    }
    Ok(())
}

/// JPEG component metadata from SOF0.
/// `h_samp` and `v_samp` are parsed for completeness; this codec operates in 4:4:4 mode.
#[derive(Clone, Default)]
struct Component {
    id: u8,
    #[allow(dead_code)]
    h_samp: u8,
    #[allow(dead_code)]
    v_samp: u8,
    qt_id: u8,
    dc_ht_id: u8,
    ac_ht_id: u8,
}

/// Accumulated decode state.
struct DecodeContext {
    image_width: usize,
    image_height: usize,
    /// Sample precision in bits; parsed from SOF0 but fixed at 8 for baseline JPEG.
    #[allow(dead_code)]
    precision: u8,
    components: Vec<Component>,
    /// qtables[id] = 64-entry table
    qtables: [[u16; 64]; 4],
    /// Tracks which qtable slots were populated.
    #[allow(dead_code)]
    qtable_present: [bool; 4],
    /// dc_huffman_trees[id], ac_huffman_trees[id]
    dc_trees: [Option<HuffNode>; 4],
    ac_trees: [Option<HuffNode>; 4],
    /// Raw scan data bytes (after removing byte stuffing)
    scan_data: Vec<u8>,
}

impl DecodeContext {
    fn new() -> Self {
        Self {
            image_width: 0,
            image_height: 0,
            precision: 8,
            components: Vec::new(),
            qtables: [[0u16; 64]; 4],
            qtable_present: [false; 4],
            dc_trees: [None, None, None, None],
            ac_trees: [None, None, None, None],
            scan_data: Vec::new(),
        }
    }

    fn parse_markers(&mut self, bytes: &[u8]) -> Result<(), JpegError> {
        let mut pos = 2; // skip SOI
        while pos + 1 < bytes.len() {
            if bytes[pos] != 0xFF {
                return Err(JpegError::Invalid(format!(
                    "expected 0xFF marker at position {}",
                    pos
                )));
            }
            let marker = bytes[pos + 1];
            pos += 2;

            match marker {
                0xD8 => {}     // SOI (shouldn't appear again)
                0xD9 => break, // EOI
                0xE0..=0xEF => {
                    // APPn — skip
                    let seg_len = read_u16_be(bytes, pos)? as usize;
                    pos += seg_len;
                }
                0xDB => {
                    // DQT
                    let seg_len = read_u16_be(bytes, pos)? as usize;
                    let seg_end = pos + seg_len;
                    let mut cur = pos + 2;
                    while cur < seg_end {
                        let pq_tq = *bytes.get(cur).ok_or(JpegError::Truncated)?;
                        let precision = (pq_tq >> 4) & 0x0F;
                        let table_id = (pq_tq & 0x0F) as usize;
                        cur += 1;
                        if precision == 0 {
                            // 8-bit precision
                            if cur + 64 > bytes.len() {
                                return Err(JpegError::Truncated);
                            }
                            for i in 0..64 {
                                self.qtables[table_id][i] = bytes[cur + i] as u16;
                            }
                            cur += 64;
                        } else {
                            // 16-bit precision
                            if cur + 128 > bytes.len() {
                                return Err(JpegError::Truncated);
                            }
                            for i in 0..64 {
                                self.qtables[table_id][i] = u16::from_be_bytes([
                                    bytes[cur + i * 2],
                                    bytes[cur + i * 2 + 1],
                                ]);
                            }
                            cur += 128;
                        }
                        self.qtable_present[table_id] = true;
                    }
                    pos = seg_end;
                }
                0xC0 | 0xC1 => {
                    // SOF0 or SOF1 (baseline / extended sequential)
                    let seg_len = read_u16_be(bytes, pos)? as usize;
                    let seg = bytes.get(pos..pos + seg_len).ok_or(JpegError::Truncated)?;
                    self.precision = seg[2];
                    self.image_height = u16::from_be_bytes([seg[3], seg[4]]) as usize;
                    self.image_width = u16::from_be_bytes([seg[5], seg[6]]) as usize;
                    let ncomp = seg[7] as usize;
                    self.components = Vec::with_capacity(ncomp);
                    for i in 0..ncomp {
                        let base = 8 + i * 3;
                        let comp = Component {
                            id: seg[base],
                            h_samp: (seg[base + 1] >> 4) & 0x0F,
                            v_samp: seg[base + 1] & 0x0F,
                            qt_id: seg[base + 2],
                            dc_ht_id: 0,
                            ac_ht_id: 0,
                        };
                        self.components.push(comp);
                    }
                    pos += seg_len;
                }
                0xC4 => {
                    // DHT
                    let seg_len = read_u16_be(bytes, pos)? as usize;
                    let seg_end = pos + seg_len;
                    let mut cur = pos + 2;
                    while cur < seg_end {
                        let tc_th = *bytes.get(cur).ok_or(JpegError::Truncated)?;
                        let tc = (tc_th >> 4) & 0x0F; // 0=DC, 1=AC
                        let th = (tc_th & 0x0F) as usize;
                        cur += 1;
                        if cur + 16 > bytes.len() {
                            return Err(JpegError::Truncated);
                        }
                        let mut bits = [0u8; 16];
                        bits.copy_from_slice(&bytes[cur..cur + 16]);
                        cur += 16;
                        let total_codes: usize = bits.iter().map(|&b| b as usize).sum();
                        if cur + total_codes > bytes.len() {
                            return Err(JpegError::Truncated);
                        }
                        let huffval = &bytes[cur..cur + total_codes];
                        cur += total_codes;
                        let tree = build_huffman_decode_tree(&bits, huffval)?;
                        if tc == 0 {
                            self.dc_trees[th] = Some(tree);
                        } else {
                            self.ac_trees[th] = Some(tree);
                        }
                    }
                    pos = seg_end;
                }
                0xDA => {
                    // SOS — read scan header, then collect remaining bytes
                    let seg_len = read_u16_be(bytes, pos)? as usize;
                    let seg_end = pos + seg_len;
                    let seg = bytes.get(pos..seg_end).ok_or(JpegError::Truncated)?;
                    let ncomp = seg[2] as usize;
                    for i in 0..ncomp {
                        let comp_id = seg[3 + i * 2];
                        let ht_byte = seg[3 + i * 2 + 1];
                        let dc_id = ht_byte >> 4;
                        let ac_id = ht_byte & 0x0F;
                        // Match to component list by id
                        for comp in &mut self.components {
                            if comp.id == comp_id {
                                comp.dc_ht_id = dc_id;
                                comp.ac_ht_id = ac_id;
                            }
                        }
                    }
                    pos = seg_end;
                    // Collect entropy-coded data (up to next unescaped 0xFF marker)
                    self.scan_data = collect_scan_data(bytes, pos)?;
                    break;
                }
                0xFE | 0xC2..=0xC3 | 0xC5..=0xCF => {
                    // COM, progressive / other SOF variants — skip
                    if pos + 2 > bytes.len() {
                        return Err(JpegError::Truncated);
                    }
                    let seg_len = read_u16_be(bytes, pos)? as usize;
                    if marker >= 0xC2 {
                        return Err(JpegError::Unsupported(format!(
                            "non-baseline SOF marker 0x{:02X}",
                            marker
                        )));
                    }
                    pos += seg_len;
                }
                0xDD => {
                    // DRI — restart interval (skip for now, we don't use restart markers)
                    let seg_len = read_u16_be(bytes, pos)? as usize;
                    pos += seg_len;
                }
                0xD0..=0xD7 => {
                    // RST markers — no length field
                }
                _ => {
                    // Unknown marker — skip using length field if present
                    if pos + 2 <= bytes.len() {
                        let seg_len = read_u16_be(bytes, pos)? as usize;
                        pos += seg_len;
                    } else {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    fn decode_scan(&self) -> Result<Vec<u8>, JpegError> {
        if self.image_width == 0 || self.image_height == 0 {
            return Err(JpegError::Invalid("SOF0 not found".into()));
        }
        if self.components.len() != 3 {
            return Err(JpegError::Unsupported(format!(
                "{} components (only 3-component YCbCr supported)",
                self.components.len()
            )));
        }

        let w = self.image_width;
        let h = self.image_height;
        let mcu_cols = w.div_ceil(8);
        let mcu_rows = h.div_ceil(8);

        // Allocate output planes
        let mut y_plane = vec![0i32; mcu_cols * 8 * mcu_rows * 8];
        let mut cb_plane = vec![0i32; mcu_cols * 8 * mcu_rows * 8];
        let mut cr_plane = vec![0i32; mcu_cols * 8 * mcu_rows * 8];

        // Build a bitstream reader over scan_data
        let mut br = BitReader::new(&self.scan_data);
        let stride = mcu_cols * 8;

        let mut prev_dc = [0i32; 3];

        for mcu_row in 0..mcu_rows {
            for mcu_col in 0..mcu_cols {
                for (ch, comp) in self.components.iter().enumerate() {
                    let dc_tree = self.dc_trees[comp.dc_ht_id as usize]
                        .as_ref()
                        .ok_or_else(|| JpegError::Invalid("DC Huffman tree missing".into()))?;
                    let ac_tree = self.ac_trees[comp.ac_ht_id as usize]
                        .as_ref()
                        .ok_or_else(|| JpegError::Invalid("AC Huffman tree missing".into()))?;
                    let qt = &self.qtables[comp.qt_id as usize];

                    // Decode DC coefficient
                    let dc_size = decode_huffman_symbol(&mut br, dc_tree)? as usize;
                    let dc_diff = if dc_size == 0 {
                        0i32
                    } else {
                        let raw = br.read_bits(dc_size)?;
                        decode_signed_coeff(raw as i32, dc_size as u8)
                    };
                    prev_dc[ch] += dc_diff;
                    let dc_val = prev_dc[ch];

                    // Decode 63 AC coefficients
                    let mut zz_coeffs = [0i16; 64];
                    zz_coeffs[0] = dc_val as i16;
                    let mut k = 1usize;
                    while k < 64 {
                        let sym = decode_huffman_symbol(&mut br, ac_tree)?;
                        if sym == 0x00 {
                            // EOB
                            break;
                        }
                        if sym == 0xF0 {
                            // ZRL: 16 zeros
                            k += 16;
                            continue;
                        }
                        let run_len = ((sym >> 4) & 0x0F) as usize;
                        let ac_size = (sym & 0x0F) as usize;
                        k += run_len;
                        if k >= 64 {
                            break;
                        }
                        if ac_size > 0 {
                            let raw = br.read_bits(ac_size)?;
                            zz_coeffs[k] = decode_signed_coeff(raw as i32, ac_size as u8) as i16;
                        }
                        k += 1;
                    }

                    // Dequantize + inverse zigzag → 8×8 block in raster order
                    let mut coeffs = [0f32; 64];
                    for (zz, &pos) in ZIGZAG_ORDER.iter().enumerate() {
                        coeffs[pos as usize] = zz_coeffs[zz] as f32 * qt[zz] as f32;
                    }

                    // IDCT
                    let spatial = idct8x8(&coeffs);

                    // Place into output plane
                    let plane = match ch {
                        0 => &mut y_plane,
                        1 => &mut cb_plane,
                        _ => &mut cr_plane,
                    };
                    for by in 0..8 {
                        for bx in 0..8 {
                            let px = mcu_col * 8 + bx;
                            let py = mcu_row * 8 + by;
                            if px < stride && py < mcu_rows * 8 {
                                // Level shift + clamp
                                let val = (spatial[by * 8 + bx] + 128.0).round() as i32;
                                plane[py * stride + px] = val.clamp(0, 255);
                            }
                        }
                    }
                }
            }
        }

        // Convert YCbCr → RGB and pack into output buffer
        let mut pixels = vec![0u8; w * h * 3];
        for py in 0..h {
            for px in 0..w {
                let idx = py * stride + px;
                let y = y_plane[idx] as f32;
                let cb = cb_plane[idx] as f32 - 128.0;
                let cr = cr_plane[idx] as f32 - 128.0;

                let r = (y + 1.402 * cr).round() as i32;
                let g = (y - 0.344_136 * cb - 0.714_136 * cr).round() as i32;
                let b = (y + 1.772 * cb).round() as i32;

                let out_idx = (py * w + px) * 3;
                pixels[out_idx] = r.clamp(0, 255) as u8;
                pixels[out_idx + 1] = g.clamp(0, 255) as u8;
                pixels[out_idx + 2] = b.clamp(0, 255) as u8;
            }
        }

        Ok(pixels)
    }
}

// ── Bitstream reader ─────────────────────────────────────────────────────────

struct BitReader<'a> {
    data: &'a [u8],
    pos: usize,
    buffer: u32,
    bits_avail: u8,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            buffer: 0,
            bits_avail: 0,
        }
    }

    fn read_bits(&mut self, n: usize) -> Result<u32, JpegError> {
        while self.bits_avail < n as u8 {
            let byte = self.next_byte()?;
            self.buffer = (self.buffer << 8) | byte as u32;
            self.bits_avail += 8;
        }
        self.bits_avail -= n as u8;
        let val = (self.buffer >> self.bits_avail) & ((1 << n) - 1);
        Ok(val)
    }

    fn next_byte(&mut self) -> Result<u8, JpegError> {
        if self.pos >= self.data.len() {
            return Err(JpegError::Truncated);
        }
        let b = self.data[self.pos];
        self.pos += 1;
        Ok(b)
    }
}

/// Walk a Huffman tree one bit at a time to decode a symbol.
fn decode_huffman_symbol(br: &mut BitReader<'_>, tree: &HuffNode) -> Result<u8, JpegError> {
    let mut node = tree;
    loop {
        if node.is_leaf {
            return Ok(node.symbol);
        }
        let bit = br.read_bits(1)?;
        if bit == 0 {
            node = node
                .left
                .as_deref()
                .ok_or_else(|| JpegError::Invalid("Huffman tree: null left child".into()))?;
        } else {
            node = node
                .right
                .as_deref()
                .ok_or_else(|| JpegError::Invalid("Huffman tree: null right child".into()))?;
        }
    }
}

/// Decode a signed coefficient from its raw magnitude bits and size category.
fn decode_signed_coeff(raw: i32, size: u8) -> i32 {
    // If the MSB of the raw value is 0, the number is negative.
    // The encoded negative amplitude is: negative + (2^size - 1)
    let threshold = 1 << (size - 1);
    if raw < threshold {
        raw - ((1 << size) - 1)
    } else {
        raw
    }
}

// ── Scan data collector ───────────────────────────────────────────────────────

/// Extract entropy-coded data from the scan, removing byte stuffing (0xFF 0x00 → 0xFF).
fn collect_scan_data(bytes: &[u8], start: usize) -> Result<Vec<u8>, JpegError> {
    let mut data = Vec::new();
    let mut i = start;
    while i < bytes.len() {
        let b = bytes[i];
        if b == 0xFF {
            if i + 1 >= bytes.len() {
                break;
            }
            let next = bytes[i + 1];
            if next == 0x00 {
                // Byte stuffing — output 0xFF
                data.push(0xFF);
                i += 2;
            } else if next == 0xD9 {
                // EOI
                break;
            } else if (0xD0..=0xD7).contains(&next) {
                // RST marker — skip
                i += 2;
            } else {
                // Some other marker — stop
                break;
            }
        } else {
            data.push(b);
            i += 1;
        }
    }
    Ok(data)
}

// ── Utility ──────────────────────────────────────────────────────────────────

fn read_u16_be(bytes: &[u8], pos: usize) -> Result<u16, JpegError> {
    if pos + 2 > bytes.len() {
        return Err(JpegError::Truncated);
    }
    Ok(u16::from_be_bytes([bytes[pos], bytes[pos + 1]]))
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn solid_rgb(width: u32, height: u32, r: u8, g: u8, b: u8) -> Vec<u8> {
        let n = (width * height) as usize * 3;
        let mut buf = Vec::with_capacity(n);
        for _ in 0..(width * height) as usize {
            buf.push(r);
            buf.push(g);
            buf.push(b);
        }
        buf
    }

    #[test]
    fn test_jpeg_encode_returns_jfif_magic() {
        let pixels = solid_rgb(8, 8, 100, 150, 200);
        let encoded = jpeg_encode_rgb(8, 8, &pixels, 90).expect("encode failed");
        assert_eq!(&encoded[..2], &[0xFF, 0xD8]);
    }

    #[test]
    fn test_jpeg_encode_ends_with_eoi() {
        let pixels = solid_rgb(8, 8, 80, 80, 80);
        let encoded = jpeg_encode_rgb(8, 8, &pixels, 75).expect("encode failed");
        let n = encoded.len();
        assert!(n >= 2);
        assert_eq!(&encoded[n - 2..], &[0xFF, 0xD9]);
    }

    #[test]
    fn test_jpeg_roundtrip_8x8() {
        let pixels = solid_rgb(8, 8, 120, 80, 200);
        let encoded = jpeg_encode_rgb(8, 8, &pixels, 90).expect("encode");
        let decoded = jpeg_decode(&encoded).expect("decode");
        assert_eq!(decoded.width, 8);
        assert_eq!(decoded.height, 8);
        assert_eq!(decoded.pixels.len(), 8 * 8 * 3);
        for i in 0..8 * 8 {
            let dr = (decoded.pixels[i * 3] as i16 - 120i16).abs();
            let dg = (decoded.pixels[i * 3 + 1] as i16 - 80i16).abs();
            let db = (decoded.pixels[i * 3 + 2] as i16 - 200i16).abs();
            assert!(dr <= 8, "R channel error {} at pixel {}", dr, i);
            assert!(dg <= 8, "G channel error {} at pixel {}", dg, i);
            assert!(db <= 8, "B channel error {} at pixel {}", db, i);
        }
    }

    #[test]
    fn test_jpeg_roundtrip_gradient_16x16() {
        let w = 16usize;
        let h = 16usize;
        let mut pixels = vec![0u8; w * h * 3];
        for y in 0..h {
            for x in 0..w {
                let r = (x * 255 / (w - 1)) as u8;
                let idx = (y * w + x) * 3;
                pixels[idx] = r;
                pixels[idx + 1] = 100;
                pixels[idx + 2] = 50;
            }
        }
        let encoded = jpeg_encode_rgb(w as u32, h as u32, &pixels, 90).expect("encode");
        let decoded = jpeg_decode(&encoded).expect("decode");
        assert_eq!(decoded.width, w);
        assert_eq!(decoded.height, h);
        // Verify gradient is preserved: left column < right column (with tolerance)
        let left_r = decoded.pixels[0] as i16;
        let right_r = decoded.pixels[(w - 1) * 3] as i16;
        assert!(
            right_r > left_r,
            "gradient not preserved: left={} right={}",
            left_r,
            right_r
        );
        // Verify each pixel is within ±15 of original
        for y in 0..h {
            for x in 0..w {
                let orig_r = (x * 255 / (w - 1)) as i16;
                let decoded_r = decoded.pixels[(y * w + x) * 3] as i16;
                let err = (decoded_r - orig_r).abs();
                assert!(
                    err <= 15,
                    "R error {} at ({},{}) expected {}",
                    err,
                    x,
                    y,
                    orig_r
                );
            }
        }
    }

    #[test]
    fn test_jpeg_invalid_magic_returns_error() {
        let result = jpeg_decode(&[0x00, 0x01, 0x02]);
        assert!(result.is_err());
    }

    #[test]
    fn test_jpeg_quality_50_smaller_than_quality_90() {
        let pixels = solid_rgb(16, 16, 128, 64, 32);
        let enc50 = jpeg_encode_rgb(16, 16, &pixels, 50).expect("encode q50");
        let enc90 = jpeg_encode_rgb(16, 16, &pixels, 90).expect("encode q90");
        assert!(
            enc50.len() < enc90.len(),
            "q50 size {} not < q90 size {}",
            enc50.len(),
            enc90.len()
        );
    }

    #[test]
    fn test_jpeg_decode_empty_returns_error() {
        let result = jpeg_decode(&[]);
        assert!(result.is_err());
    }
}
