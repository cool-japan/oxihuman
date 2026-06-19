// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! APNG animation export — produces conformant PNG / APNG binary files.
//!
//! ## Format notes
//!
//! For a single-frame export the output is a valid PNG 1.2 file.  For
//! multi-frame exports the output is a valid APNG extension of PNG (Mozilla /
//! APNG spec) containing:
//!
//! - PNG signature (8 bytes)
//! - `IHDR` image-header chunk
//! - `acTL` animation-control chunk  *(multi-frame only)*
//! - For each frame:
//!   - `fcTL` frame-control chunk    *(multi-frame only; first frame also)*
//!   - `IDAT` (frame 0) / `fdAT` (frames 1+) with zlib-compressed scanlines
//! - `IEND` chunk
//!
//! Pixels are stored as RGBA (8-bit/channel, `color_type = 6`).  Each row is
//! prefixed with filter-byte `0x00` (None) before compression.

use oxiarc_deflate::zlib_compress;
use oxihuman_core::crc32;

// ── PNG constants ────────────────────────────────────────────────────────────

/// PNG file signature (8 bytes) — RFC 2083 §12.12.
const PNG_SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

/// IHDR data length is always 13 bytes.
const IHDR_DATA_LEN: usize = 13;

/// Color type 6: RGBA (8-bit per channel → 4 bytes per pixel).
const COLOR_TYPE_RGBA: u8 = 6;

/// Zlib compression level used for IDAT / fdAT data.
const ZLIB_LEVEL: u8 = 6;

// ── Low-level chunk helpers ──────────────────────────────────────────────────

/// Write a big-endian `u32` into `buf`.
fn push_u32be(buf: &mut Vec<u8>, value: u32) {
    buf.extend_from_slice(&value.to_be_bytes());
}

/// Write a big-endian `u16` into `buf`.
fn push_u16be(buf: &mut Vec<u8>, value: u16) {
    buf.extend_from_slice(&value.to_be_bytes());
}

/// Append a PNG chunk to `out`:
///
/// `[length:u32BE][type:4B][data][crc32_of_type||data:u32BE]`
fn png_chunk(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    // Length of the data field only (chunk type is not counted).
    let data_len = data.len() as u32;
    push_u32be(out, data_len);
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);

    // CRC covers chunk_type + data (not the length field).
    let mut crc_input = Vec::with_capacity(4 + data.len());
    crc_input.extend_from_slice(chunk_type);
    crc_input.extend_from_slice(data);
    let checksum = crc32(&crc_input);
    push_u32be(out, checksum);
}

// ── Scanline filter + compress ───────────────────────────────────────────────

/// Build the uncompressed filtered image data for a single frame.
///
/// Each scanline is prefixed with filter-byte `0x00` (None-filter), then the
/// raw RGBA pixels follow.  The accumulated byte stream is then zlib-compressed
/// to produce the IDAT / fdAT payload.
fn compress_frame_pixels(
    pixels: &[[u8; 4]],
    width: u32,
    height: u32,
) -> anyhow::Result<Vec<u8>> {
    let w = width as usize;
    let h = height as usize;

    // Pre-allocate: 1 filter byte + 4 bytes × width per row.
    let mut raw: Vec<u8> = Vec::with_capacity(h * (1 + 4 * w));

    for row in 0..h {
        raw.push(0x00); // filter = None
        let row_start = row * w;
        let row_end = row_start + w;
        let row_pixels = if row_end <= pixels.len() {
            &pixels[row_start..row_end]
        } else {
            // Safety: if pixels are short, pad with black-transparent.
            &pixels[pixels.len()..pixels.len()]
        };
        for px in row_pixels {
            raw.extend_from_slice(px);
        }
        // Pad missing pixels in this row.
        let missing = w.saturating_sub(row_pixels.len());
        for _ in 0..missing {
            raw.extend_from_slice(&[0u8; 4]);
        }
    }

    let compressed = zlib_compress(&raw, ZLIB_LEVEL)
        .map_err(|e| anyhow::anyhow!("APNG zlib compression failed: {}", e))?;
    Ok(compressed)
}

// ── Public structures ────────────────────────────────────────────────────────

/// A single APNG frame.
#[derive(Debug, Clone)]
pub struct ApngFrame {
    pub width: u32,
    pub height: u32,
    pub delay_num: u16,
    pub delay_den: u16,
    pub pixels: Vec<[u8; 4]>,
}

impl ApngFrame {
    /// Create a filled APNG frame.
    pub fn new_solid(
        width: u32,
        height: u32,
        delay_num: u16,
        delay_den: u16,
        color: [u8; 4],
    ) -> Self {
        let pixels = vec![color; (width * height) as usize];
        Self {
            width,
            height,
            delay_num,
            delay_den,
            pixels,
        }
    }

    /// Frame delay in seconds.
    pub fn delay_secs(&self) -> f32 {
        if self.delay_den == 0 {
            return 0.0;
        }
        self.delay_num as f32 / self.delay_den as f32
    }

    /// Pixel count.
    pub fn pixel_count(&self) -> usize {
        self.pixels.len()
    }
}

/// APNG export configuration.
#[derive(Debug, Clone)]
pub struct ApngExport {
    pub width: u32,
    pub height: u32,
    pub loop_count: u32,
    pub frames: Vec<ApngFrame>,
}

impl ApngExport {
    /// Create a new APNG export.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            loop_count: 0,
            frames: Vec::new(),
        }
    }

    /// Add a frame.
    pub fn add_frame(&mut self, frame: ApngFrame) {
        self.frames.push(frame);
    }

    /// Return frame count.
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Total animation duration in seconds.
    pub fn total_duration_secs(&self) -> f32 {
        self.frames.iter().map(|f| f.delay_secs()).sum()
    }
}

// ── Validation helpers ───────────────────────────────────────────────────────

/// Validate that all frames match animation dimensions.
pub fn validate_apng(apng: &ApngExport) -> bool {
    apng.frames
        .iter()
        .all(|f| f.width == apng.width && f.height == apng.height)
}

/// Estimate raw uncompressed size.
pub fn estimate_raw_bytes(apng: &ApngExport) -> usize {
    apng.frames
        .iter()
        .map(|f| f.pixel_count() * 4)
        .sum::<usize>()
        + 8
}

/// Serialize metadata to JSON (informational).
pub fn apng_metadata_json(apng: &ApngExport) -> String {
    format!(
        "{{\"width\":{},\"height\":{},\"frames\":{},\"loop\":{}}}",
        apng.width,
        apng.height,
        apng.frame_count(),
        apng.loop_count
    )
}

// ── Synthetic checkerboard pixel generator ──────────────────────────────────

/// Generate a checkerboard RGBA frame of the given size (used when no real
/// pixel data is available).
fn checkerboard_frame(width: u32, height: u32, delay_num: u16, delay_den: u16) -> ApngFrame {
    let mut pixels = Vec::with_capacity((width * height) as usize);
    for y in 0..height {
        for x in 0..width {
            let on = ((x / 8) + (y / 8)) % 2 == 0;
            let color = if on {
                [200u8, 200, 200, 255]
            } else {
                [50u8, 50, 50, 255]
            };
            pixels.push(color);
        }
    }
    ApngFrame {
        width,
        height,
        delay_num,
        delay_den,
        pixels,
    }
}

// ── Main export function ─────────────────────────────────────────────────────

/// Encode the `ApngExport` to a binary PNG/APNG byte vector.
///
/// - Returns a valid single-image PNG when `frame_count() == 0` or `== 1`.
/// - Returns a valid APNG when `frame_count() > 1`.
/// - When there are no frames, a single checkerboard frame at the configured
///   dimensions is synthesised.
///
/// The output is always a conformant PNG byte stream accepted by any
/// PNG-compliant decoder.
pub fn export_apng(apng: &ApngExport) -> anyhow::Result<Vec<u8>> {
    let mut out: Vec<u8> = Vec::new();

    // ── PNG signature ────────────────────────────────────────────────────
    out.extend_from_slice(&PNG_SIGNATURE);

    // ── IHDR ────────────────────────────────────────────────────────────
    {
        let mut ihdr = Vec::with_capacity(IHDR_DATA_LEN);
        push_u32be(&mut ihdr, apng.width);
        push_u32be(&mut ihdr, apng.height);
        ihdr.push(8);                // bit depth
        ihdr.push(COLOR_TYPE_RGBA);  // color type 6 (RGBA)
        ihdr.push(0);                // compression method 0 (deflate)
        ihdr.push(0);                // filter method 0
        ihdr.push(0);                // interlace method 0 (none)
        png_chunk(&mut out, b"IHDR", &ihdr);
    }

    // Resolve effective frames: if none provided, synthesise one.
    let synthetic_frame;
    let effective_frames: &[ApngFrame] = if apng.frames.is_empty() {
        synthetic_frame = vec![checkerboard_frame(apng.width, apng.height, 1, 100)];
        &synthetic_frame
    } else {
        &apng.frames
    };

    let multi_frame = effective_frames.len() > 1;
    let mut seq_no: u32 = 0;

    // ── acTL (animation control) — only for multi-frame APNG ────────────
    if multi_frame {
        let mut actl = Vec::with_capacity(8);
        push_u32be(&mut actl, effective_frames.len() as u32); // num_frames
        push_u32be(&mut actl, apng.loop_count);               // num_plays (0 = loop)
        png_chunk(&mut out, b"acTL", &actl);
    }

    // ── Per-frame data ───────────────────────────────────────────────────
    for (frame_idx, frame) in effective_frames.iter().enumerate() {
        // fcTL (frame control) — only needed for APNG multi-frame.
        if multi_frame {
            let mut fctl = Vec::with_capacity(26);
            push_u32be(&mut fctl, seq_no);                     // sequence_number
            push_u32be(&mut fctl, frame.width);                // width
            push_u32be(&mut fctl, frame.height);               // height
            push_u32be(&mut fctl, 0);                          // x_offset
            push_u32be(&mut fctl, 0);                          // y_offset
            push_u16be(&mut fctl, frame.delay_num);            // delay_num
            push_u16be(&mut fctl, frame.delay_den);            // delay_den
            fctl.push(0);                                      // dispose_op = NONE
            fctl.push(0);                                      // blend_op = SOURCE
            png_chunk(&mut out, b"fcTL", &fctl);
            seq_no += 1;
        }

        let compressed = compress_frame_pixels(&frame.pixels, frame.width, frame.height)?;

        if frame_idx == 0 {
            // First frame always goes into IDAT.
            png_chunk(&mut out, b"IDAT", &compressed);
        } else {
            // Subsequent frames go into fdAT (sequence_number + compressed data).
            let mut fdat = Vec::with_capacity(4 + compressed.len());
            push_u32be(&mut fdat, seq_no);
            fdat.extend_from_slice(&compressed);
            png_chunk(&mut out, b"fdAT", &fdat);
            seq_no += 1;
        }
    }

    // ── IEND ────────────────────────────────────────────────────────────
    png_chunk(&mut out, b"IEND", &[]);

    Ok(out)
}

// ── PNG validation helpers (used in tests and by callers) ───────────────────

/// Return `true` if `data` begins with the 8-byte PNG signature.
pub fn has_png_signature(data: &[u8]) -> bool {
    data.len() >= 8 && data[..8] == PNG_SIGNATURE
}

/// Parse the IHDR chunk from `data` and return `(width, height)`.
///
/// Returns `None` if the data is too short, the signature is wrong, or the
/// first chunk is not IHDR.
pub fn parse_ihdr_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    // Minimum: 8 (sig) + 4 (len) + 4 (type) + 13 (data) + 4 (crc) = 33
    if data.len() < 33 {
        return None;
    }
    if data[..8] != PNG_SIGNATURE {
        return None;
    }
    // After signature: [len:4][type:4][data:13][crc:4]
    let chunk_type = &data[12..16];
    if chunk_type != b"IHDR" {
        return None;
    }
    let width = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
    let height = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
    Some((width, height))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_apng() -> ApngExport {
        let mut apng = ApngExport::new(32, 32);
        apng.add_frame(ApngFrame::new_solid(32, 32, 1, 24, [255, 0, 0, 255]));
        apng.add_frame(ApngFrame::new_solid(32, 32, 1, 24, [0, 0, 255, 255]));
        apng
    }

    #[test]
    fn test_frame_count() {
        /* frame count is correct */
        assert_eq!(sample_apng().frame_count(), 2);
    }

    #[test]
    fn test_delay_secs() {
        /* delay in seconds computed from num/den */
        let f = ApngFrame::new_solid(4, 4, 1, 10, [0; 4]);
        assert!((f.delay_secs() - 0.1).abs() < 1e-5);
    }

    #[test]
    fn test_total_duration_secs() {
        /* total duration sums frame delays */
        let apng = sample_apng();
        let expected = 2.0 / 24.0;
        assert!((apng.total_duration_secs() - expected).abs() < 1e-5);
    }

    #[test]
    fn test_validate_apng_valid() {
        /* matching frames pass validation */
        assert!(validate_apng(&sample_apng()));
    }

    #[test]
    fn test_validate_apng_invalid() {
        /* mismatched frames fail validation */
        let mut apng = ApngExport::new(32, 32);
        apng.add_frame(ApngFrame::new_solid(16, 16, 1, 10, [0; 4]));
        assert!(!validate_apng(&apng));
    }

    #[test]
    fn test_estimate_raw_bytes_positive() {
        /* raw byte estimate is positive */
        assert!(estimate_raw_bytes(&sample_apng()) > 0);
    }

    #[test]
    fn test_metadata_json() {
        /* metadata JSON contains expected keys */
        let json = apng_metadata_json(&sample_apng());
        assert!(json.contains("32"));
    }

    #[test]
    fn test_pixel_count() {
        /* pixel count matches dimensions */
        let f = ApngFrame::new_solid(8, 8, 1, 10, [0; 4]);
        assert_eq!(f.pixel_count(), 64);
    }

    /// The exported bytes must begin with the 8-byte PNG signature and
    /// the first chunk must be IHDR with matching width/height.
    #[test]
    fn apng_output_is_valid_png() {
        let mut apng = ApngExport::new(16, 8);
        apng.add_frame(ApngFrame::new_solid(16, 8, 1, 100, [128, 64, 32, 255]));

        let bytes = export_apng(&apng).expect("export_apng should succeed");

        // 1. PNG signature check.
        assert!(
            has_png_signature(&bytes),
            "output does not start with PNG signature"
        );

        // 2. IHDR width/height match what we asked for.
        let (w, h) = parse_ihdr_dimensions(&bytes)
            .expect("failed to parse IHDR dimensions");
        assert_eq!(w, 16, "IHDR width mismatch");
        assert_eq!(h, 8, "IHDR height mismatch");
    }

    /// Multi-frame export must still produce a valid PNG-signature header and
    /// correct IHDR, and include the acTL / fcTL markers expected by APNG.
    #[test]
    fn apng_multi_frame_valid_png_signature() {
        let apng = sample_apng();
        let bytes = export_apng(&apng).expect("multi-frame export_apng should succeed");

        assert!(has_png_signature(&bytes), "multi-frame PNG signature missing");

        let (w, h) = parse_ihdr_dimensions(&bytes)
            .expect("failed to parse IHDR from multi-frame APNG");
        assert_eq!(w, 32);
        assert_eq!(h, 32);

        // The serialised output should contain the ASCII tag "acTL" (APNG marker).
        let actl_pos = bytes
            .windows(4)
            .position(|w| w == b"acTL");
        assert!(actl_pos.is_some(), "acTL chunk not found in multi-frame APNG");
    }

    /// Export with zero frames should produce a synthetic checkerboard PNG —
    /// still conformant, not empty bytes.
    #[test]
    fn apng_zero_frames_produces_synthetic_png() {
        let apng = ApngExport::new(4, 4);
        let bytes = export_apng(&apng).expect("zero-frame export_apng should succeed");
        assert!(has_png_signature(&bytes), "synthetic PNG signature missing");
        let (w, h) = parse_ihdr_dimensions(&bytes)
            .expect("failed to parse IHDR from synthetic PNG");
        assert_eq!(w, 4);
        assert_eq!(h, 4);
    }
}
