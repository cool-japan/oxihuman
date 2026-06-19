// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Baseline TIFF encoder/decoder — uncompressed RGB, little-endian and big-endian read,
//! little-endian write.
//!
//! # Format
//!
//! * Header   : `b"II"` + `0x2A 0x00` (magic 42, LE) + IFD offset (u32 LE)
//! * IFD      : u16 entry-count, 12 bytes per entry, trailing 4-byte next-IFD (zero)
//! * Required tags: ImageWidth, ImageLength, BitsPerSample, Compression (=1),
//!   PhotometricInterpretation (=2/RGB), StripOffsets, SamplesPerPixel (=3),
//!   RowsPerStrip, StripByteCounts, XResolution (72/1), YResolution (72/1),
//!   ResolutionUnit (=2/inch)
//! * Pixels   : row-major top-to-bottom, RGB interleaved, uncompressed.

#![allow(dead_code)]

use super::image_codec::RawDecodeResult;

// ── Tag constants ─────────────────────────────────────────────────────────────

const TAG_IMAGE_WIDTH: u16 = 0x0100;
const TAG_IMAGE_LENGTH: u16 = 0x0101;
const TAG_BITS_PER_SAMPLE: u16 = 0x0102;
const TAG_COMPRESSION: u16 = 0x0103;
const TAG_PHOTOMETRIC: u16 = 0x0106;
const TAG_STRIP_OFFSETS: u16 = 0x0111;
const TAG_SAMPLES_PER_PIXEL: u16 = 0x0115;
const TAG_ROWS_PER_STRIP: u16 = 0x0116;
const TAG_STRIP_BYTE_COUNTS: u16 = 0x0117;
const TAG_X_RESOLUTION: u16 = 0x011A;
const TAG_Y_RESOLUTION: u16 = 0x011B;
const TAG_RESOLUTION_UNIT: u16 = 0x0128;

// ── TIFF field types ──────────────────────────────────────────────────────────

const TYPE_SHORT: u16 = 3;
const TYPE_LONG: u16 = 4;
const TYPE_RATIONAL: u16 = 5;

// ── Error ─────────────────────────────────────────────────────────────────────

/// Errors that can occur during TIFF encode/decode.
#[derive(Debug, thiserror::Error)]
pub enum TiffError {
    #[error("Invalid TIFF: {0}")]
    Invalid(String),
    #[error("Unsupported TIFF feature: {0}")]
    Unsupported(String),
    #[error("Truncated data")]
    Truncated,
}

// ── Encoder ───────────────────────────────────────────────────────────────────

/// Encode raw RGB24 pixels (row-major, top-to-bottom) as a little-endian baseline TIFF file.
///
/// `pixels` must be exactly `width * height * 3` bytes.
pub fn tiff_encode_rgb(width: u32, height: u32, pixels: &[u8]) -> Result<Vec<u8>, TiffError> {
    let expected = (width as usize) * (height as usize) * 3;
    if pixels.len() != expected {
        return Err(TiffError::Invalid(format!(
            "pixel buffer length {} != expected {}",
            pixels.len(),
            expected
        )));
    }

    // Layout plan (all offsets from start of file, LE):
    //   0x00 : 8-byte TIFF header
    //   0x08 : IFD  (2 + 12*12 + 4 = 150 bytes)
    //   0x9E : extra data (BitsPerSample u16[3]=6 bytes, XRes rational=8 bytes, YRes rational=8 bytes)
    //   0xB4 : pixel data

    const IFD_ENTRY_COUNT: u16 = 12;
    const IFD_OFFSET: u32 = 8;
    const IFD_SIZE: u32 = 2 + 12 * (IFD_ENTRY_COUNT as u32) + 4; // 150
    const EXTRA_OFFSET: u32 = IFD_OFFSET + IFD_SIZE; // 158  (0x9E)
    const BPS_OFFSET: u32 = EXTRA_OFFSET; // 3 × u16 = 6 bytes
    const XRES_OFFSET: u32 = EXTRA_OFFSET + 6; // 2 × u32 = 8 bytes
    const YRES_OFFSET: u32 = XRES_OFFSET + 8; // 2 × u32 = 8 bytes
    const PIXEL_OFFSET: u32 = YRES_OFFSET + 8; // pixel data starts here

    let pixel_byte_count: u32 = width * height * 3;
    let total_size = (PIXEL_OFFSET as usize) + (pixel_byte_count as usize);

    let mut buf: Vec<u8> = Vec::with_capacity(total_size);

    // ── TIFF header (8 bytes) ─────────────────────────────────────────────────
    buf.extend_from_slice(b"II"); // little-endian marker
    buf.extend_from_slice(&42u16.to_le_bytes()); // magic
    buf.extend_from_slice(&IFD_OFFSET.to_le_bytes()); // IFD offset = 8

    // ── IFD ──────────────────────────────────────────────────────────────────
    buf.extend_from_slice(&IFD_ENTRY_COUNT.to_le_bytes());

    // Helper: write a 12-byte IFD entry
    let write_entry = |out: &mut Vec<u8>, tag: u16, typ: u16, count: u32, val: u32| {
        out.extend_from_slice(&tag.to_le_bytes());
        out.extend_from_slice(&typ.to_le_bytes());
        out.extend_from_slice(&count.to_le_bytes());
        out.extend_from_slice(&val.to_le_bytes());
    };

    // Tags must be in ascending tag-number order (TIFF spec §2).
    write_entry(&mut buf, TAG_IMAGE_WIDTH, TYPE_LONG, 1, width);
    write_entry(&mut buf, TAG_IMAGE_LENGTH, TYPE_LONG, 1, height);
    write_entry(&mut buf, TAG_BITS_PER_SAMPLE, TYPE_SHORT, 3, BPS_OFFSET); // offset to [8,8,8]
    write_entry(&mut buf, TAG_COMPRESSION, TYPE_SHORT, 1, 1); // no compression
    write_entry(&mut buf, TAG_PHOTOMETRIC, TYPE_SHORT, 1, 2); // RGB
    write_entry(&mut buf, TAG_STRIP_OFFSETS, TYPE_LONG, 1, PIXEL_OFFSET);
    write_entry(&mut buf, TAG_SAMPLES_PER_PIXEL, TYPE_SHORT, 1, 3);
    write_entry(&mut buf, TAG_ROWS_PER_STRIP, TYPE_LONG, 1, height);
    write_entry(
        &mut buf,
        TAG_STRIP_BYTE_COUNTS,
        TYPE_LONG,
        1,
        pixel_byte_count,
    );
    write_entry(&mut buf, TAG_X_RESOLUTION, TYPE_RATIONAL, 1, XRES_OFFSET);
    write_entry(&mut buf, TAG_Y_RESOLUTION, TYPE_RATIONAL, 1, YRES_OFFSET);
    write_entry(&mut buf, TAG_RESOLUTION_UNIT, TYPE_SHORT, 1, 2); // inch

    // Next IFD offset = 0 (no more IFDs)
    buf.extend_from_slice(&0u32.to_le_bytes());

    // ── Extra data ───────────────────────────────────────────────────────────
    // BitsPerSample: [8, 8, 8] as u16 LE
    buf.extend_from_slice(&8u16.to_le_bytes());
    buf.extend_from_slice(&8u16.to_le_bytes());
    buf.extend_from_slice(&8u16.to_le_bytes());

    // XResolution: 72/1 as two u32 LE (numerator, denominator)
    buf.extend_from_slice(&72u32.to_le_bytes());
    buf.extend_from_slice(&1u32.to_le_bytes());

    // YResolution: 72/1
    buf.extend_from_slice(&72u32.to_le_bytes());
    buf.extend_from_slice(&1u32.to_le_bytes());

    // ── Pixel data ───────────────────────────────────────────────────────────
    buf.extend_from_slice(pixels);

    debug_assert_eq!(buf.len(), total_size);
    Ok(buf)
}

// ── Decoder ───────────────────────────────────────────────────────────────────

/// Decode a TIFF file (baseline, uncompressed, 8-bit RGB) into raw RGB24 pixels.
///
/// Supports both little-endian (`II`) and big-endian (`MM`) TIFF files.
pub fn tiff_decode(bytes: &[u8]) -> Result<RawDecodeResult, TiffError> {
    if bytes.len() < 8 {
        return Err(TiffError::Truncated);
    }

    // Determine byte order from the first two bytes.
    let little_endian = match &bytes[0..2] {
        b"II" => true,
        b"MM" => false,
        _ => {
            return Err(TiffError::Invalid(format!(
                "unrecognised byte-order marker: {:02X} {:02X}",
                bytes[0], bytes[1]
            )))
        }
    };

    let read_u16 = |off: usize| -> Result<u16, TiffError> {
        bytes
            .get(off..off + 2)
            .and_then(|s| s.try_into().ok())
            .map(|a: [u8; 2]| {
                if little_endian {
                    u16::from_le_bytes(a)
                } else {
                    u16::from_be_bytes(a)
                }
            })
            .ok_or(TiffError::Truncated)
    };

    let read_u32 = |off: usize| -> Result<u32, TiffError> {
        bytes
            .get(off..off + 4)
            .and_then(|s| s.try_into().ok())
            .map(|a: [u8; 4]| {
                if little_endian {
                    u32::from_le_bytes(a)
                } else {
                    u32::from_be_bytes(a)
                }
            })
            .ok_or(TiffError::Truncated)
    };

    // Validate magic number (42).
    let magic = read_u16(2)?;
    if magic != 42 {
        return Err(TiffError::Invalid(format!("bad TIFF magic: {}", magic)));
    }

    let ifd_offset = read_u32(4)? as usize;

    // ── Parse IFD entries ────────────────────────────────────────────────────
    let entry_count = read_u16(ifd_offset)? as usize;
    let entries_start = ifd_offset + 2;
    if bytes.len() < entries_start + entry_count * 12 {
        return Err(TiffError::Truncated);
    }

    // Per-entry reader: (tag, type, count, value_or_offset)
    let read_entry = |n: usize| -> Result<(u16, u16, u32, u32), TiffError> {
        let base = entries_start + n * 12;
        let tag = read_u16(base)?;
        let typ = read_u16(base + 2)?;
        let count = read_u32(base + 4)?;
        let val = read_u32(base + 8)?;
        Ok((tag, typ, count, val))
    };

    // Collect values we need.
    let mut width: Option<u32> = None;
    let mut height: Option<u32> = None;
    let mut compression: Option<u32> = None;
    let mut samples_per_pixel: Option<u32> = None;
    let mut strip_offsets: Option<u32> = None;
    let mut strip_byte_counts: Option<u32> = None;
    let mut bits_per_sample_info: Option<(u16, u32, u32)> = None; // (type, count, val_or_offset)

    for i in 0..entry_count {
        let (tag, typ, count, val) = read_entry(i)?;
        match tag {
            TAG_IMAGE_WIDTH => width = Some(val),
            TAG_IMAGE_LENGTH => height = Some(val),
            TAG_BITS_PER_SAMPLE => bits_per_sample_info = Some((typ, count, val)),
            TAG_COMPRESSION => compression = Some(val),
            TAG_STRIP_OFFSETS => strip_offsets = Some(val),
            TAG_SAMPLES_PER_PIXEL => samples_per_pixel = Some(val),
            TAG_STRIP_BYTE_COUNTS => strip_byte_counts = Some(val),
            _ => {}
        }
    }

    let width = width.ok_or_else(|| TiffError::Invalid("missing ImageWidth".into()))? as usize;
    let height = height.ok_or_else(|| TiffError::Invalid("missing ImageLength".into()))? as usize;
    let compression =
        compression.ok_or_else(|| TiffError::Invalid("missing Compression".into()))?;
    let spp =
        samples_per_pixel.ok_or_else(|| TiffError::Invalid("missing SamplesPerPixel".into()))?;
    let strip_offset =
        strip_offsets.ok_or_else(|| TiffError::Invalid("missing StripOffsets".into()))? as usize;
    let strip_bytes = strip_byte_counts
        .ok_or_else(|| TiffError::Invalid("missing StripByteCounts".into()))?
        as usize;

    if compression != 1 {
        return Err(TiffError::Unsupported(format!(
            "compression={} (only 1/no-compression supported)",
            compression
        )));
    }
    if spp != 3 {
        return Err(TiffError::Unsupported(format!(
            "SamplesPerPixel={} (only 3/RGB supported)",
            spp
        )));
    }

    // Validate BitsPerSample: must be 8,8,8
    if let Some((bps_type, bps_count, bps_val)) = bits_per_sample_info {
        if bps_count != 3 {
            return Err(TiffError::Unsupported(format!(
                "BitsPerSample count={} (expected 3)",
                bps_count
            )));
        }
        // When count=3 the value field holds an *offset* to 3 u16 values.
        // Validate each channel is 8-bit.
        if bps_type == TYPE_SHORT {
            let off = bps_val as usize;
            if bytes.len() < off + 6 {
                return Err(TiffError::Truncated);
            }
            let b0 = read_u16(off)?;
            let b1 = read_u16(off + 2)?;
            let b2 = read_u16(off + 4)?;
            if b0 != 8 || b1 != 8 || b2 != 8 {
                return Err(TiffError::Unsupported(format!(
                    "BitsPerSample=[{},{},{}] (only 8,8,8 supported)",
                    b0, b1, b2
                )));
            }
        }
    }

    // Validate pixel buffer bounds.
    let expected_bytes = width * height * 3;
    if strip_bytes < expected_bytes {
        return Err(TiffError::Invalid(format!(
            "StripByteCounts={} < expected={}",
            strip_bytes, expected_bytes
        )));
    }
    if bytes.len() < strip_offset + expected_bytes {
        return Err(TiffError::Truncated);
    }

    let pixels = bytes[strip_offset..strip_offset + expected_bytes].to_vec();

    Ok(RawDecodeResult {
        width,
        height,
        pixels,
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Generate an 8×8 gradient test image (RGB24, row-major).
    fn make_gradient(w: usize, h: usize) -> Vec<u8> {
        let mut pixels = Vec::with_capacity(w * h * 3);
        for y in 0..h {
            for x in 0..w {
                pixels.push(((x * 255) / w.max(1)) as u8); // R increases left→right
                pixels.push(((y * 255) / h.max(1)) as u8); // G increases top→bottom
                pixels.push(128u8); // B constant
            }
        }
        pixels
    }

    #[test]
    fn test_tiff_header_le() {
        let pixels = vec![200u8; 4 * 4 * 3];
        let encoded = tiff_encode_rgb(4, 4, &pixels).expect("encode must succeed");
        assert!(
            encoded.starts_with(b"II\x2A\x00"),
            "must start with II (LE) + magic 42"
        );
    }

    #[test]
    fn test_tiff_roundtrip_exact() {
        let pixels: Vec<u8> = (0u8..=255u8).cycle().take(4 * 4 * 3).collect();
        let encoded = tiff_encode_rgb(4, 4, &pixels).expect("encode");
        let decoded = tiff_decode(&encoded).expect("decode");
        assert_eq!(decoded.width, 4);
        assert_eq!(decoded.height, 4);
        assert_eq!(
            decoded.pixels, pixels,
            "round-tripped pixels must be identical"
        );
    }

    #[test]
    fn test_tiff_dimensions() {
        let pixels = vec![0u8; 10 * 6 * 3];
        let encoded = tiff_encode_rgb(10, 6, &pixels).expect("encode");
        let decoded = tiff_decode(&encoded).expect("decode");
        assert_eq!(decoded.width, 10);
        assert_eq!(decoded.height, 6);
    }

    #[test]
    fn test_tiff_decode_invalid_returns_error() {
        let result = tiff_decode(b"not a tiff file at all 12345678");
        assert!(result.is_err(), "invalid data must return an error");
    }

    #[test]
    fn test_tiff_roundtrip_gradient() {
        let src = make_gradient(8, 8);
        let encoded = tiff_encode_rgb(8, 8, &src).expect("encode");
        let decoded = tiff_decode(&encoded).expect("decode");
        assert_eq!(decoded.width, 8);
        assert_eq!(decoded.height, 8);
        assert_eq!(decoded.pixels, src, "gradient round-trip must be exact");
    }

    #[test]
    fn test_tiff_wrong_pixel_buffer_size_returns_error() {
        // Too few pixels for 4×4 image
        let result = tiff_encode_rgb(4, 4, &[0u8; 10]);
        assert!(result.is_err());
    }

    #[test]
    fn test_tiff_truncated_returns_error() {
        let pixels = vec![0u8; 4 * 4 * 3];
        let encoded = tiff_encode_rgb(4, 4, &pixels).expect("encode");
        // Truncate severely
        let result = tiff_decode(&encoded[..6]);
        assert!(result.is_err());
    }
}
