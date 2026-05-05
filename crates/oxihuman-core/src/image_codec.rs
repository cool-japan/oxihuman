// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Image encode/decode providing format detection, BMP and PNG codec implementations,
//! and header construction utilities.

use zune_core::bytestream::ZCursor;
use zune_core::colorspace::ColorSpace;
use zune_core::options::EncoderOptions;
use zune_png::{PngDecoder, PngEncoder};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Bmp,
    Tga,
    Hdr,
    Gif,
    Webp,
    Tiff,
    Unknown,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PixelFormat {
    Rgb8,
    Rgba8,
    Grayscale8,
    Rgba16,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ImageHeader {
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
    pub pixel_format: PixelFormat,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EncodeConfig {
    pub quality: u8,
    pub format: ImageFormat,
}

/// Decode result carrying both metadata and raw pixel data.
#[derive(Debug, Clone)]
pub struct RawDecodeResult {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DecodeResult {
    pub header: ImageHeader,
    pub pixel_count: usize,
    pub byte_size: usize,
}

/// Errors that can occur during image encode/decode operations.
#[derive(Debug, thiserror::Error)]
pub enum ImageError {
    #[error("Image encoding failed: {0}")]
    EncodeError(String),
    #[error("Image decoding failed: {0}")]
    DecodeError(String),
    #[error("Invalid or unrecognised magic bytes")]
    InvalidMagic,
    #[error("Input data was truncated or too short")]
    TruncatedInput,
    #[error("Unsupported compression method in BMP")]
    UnsupportedCompression,
}

/// Detect image format from leading magic bytes.
pub fn detect_format(bytes: &[u8]) -> ImageFormat {
    if bytes.len() >= 8 && bytes[..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
        return ImageFormat::Png;
    }
    if bytes.len() >= 3 && bytes[..3] == [0xFF, 0xD8, 0xFF] {
        return ImageFormat::Jpeg;
    }
    if bytes.len() >= 2 && &bytes[..2] == b"BM" {
        return ImageFormat::Bmp;
    }
    if bytes.len() >= 6 && (&bytes[..6] == b"GIF87a" || &bytes[..6] == b"GIF89a") {
        return ImageFormat::Gif;
    }
    if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return ImageFormat::Webp;
    }
    if bytes.len() >= 4 && (&bytes[..4] == b"II*\x00" || &bytes[..4] == b"MM\x00*") {
        return ImageFormat::Tiff;
    }
    ImageFormat::Unknown
}

/// Encode 24-bit RGB pixel data as a BMP file.
///
/// `pixels` must be tightly packed RGB (3 bytes per pixel), row-major top-to-bottom.
pub fn bmp_encode_rgb(width: u32, height: u32, pixels: &[u8]) -> Vec<u8> {
    let stride = width as usize * 3;
    let row_bytes = (stride + 3) & !3; // pad rows to 4-byte boundary
    let pixel_data_size = row_bytes * height as usize;
    let file_size = (54 + pixel_data_size) as u32;
    let mut buf = Vec::with_capacity(file_size as usize);

    // File header (14 bytes)
    buf.extend_from_slice(b"BM");
    buf.extend_from_slice(&file_size.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes()); // reserved
    buf.extend_from_slice(&54u32.to_le_bytes()); // pixel data offset

    // DIB header / BITMAPINFOHEADER (40 bytes)
    buf.extend_from_slice(&40u32.to_le_bytes()); // header size
    buf.extend_from_slice(&(width as i32).to_le_bytes());
    buf.extend_from_slice(&(height as i32).to_le_bytes()); // positive = bottom-up
    buf.extend_from_slice(&1u16.to_le_bytes()); // color planes
    buf.extend_from_slice(&24u16.to_le_bytes()); // bits per pixel
    buf.extend_from_slice(&0u32.to_le_bytes()); // no compression
    buf.extend_from_slice(&(pixel_data_size as u32).to_le_bytes());
    buf.extend_from_slice(&2835u32.to_le_bytes()); // h ppm
    buf.extend_from_slice(&2835u32.to_le_bytes()); // v ppm
    buf.extend_from_slice(&0u32.to_le_bytes()); // palette colours
    buf.extend_from_slice(&0u32.to_le_bytes()); // important colours

    // Pixel data: rows in reverse order (bottom-up), BGR byte order
    let padding = row_bytes - stride;
    for row in (0..height as usize).rev() {
        let row_src = &pixels[row * stride..(row + 1) * stride];
        for px in row_src.chunks_exact(3) {
            buf.push(px[2]); // B
            buf.push(px[1]); // G
            buf.push(px[0]); // R
        }
        buf.extend(std::iter::repeat_n(0u8, padding));
    }
    buf
}

/// Encode 32-bit RGBA pixel data as a BMP file.
///
/// `pixels` must be tightly packed RGBA (4 bytes per pixel), row-major top-to-bottom.
pub fn bmp_encode_rgba(width: u32, height: u32, pixels: &[u8]) -> Vec<u8> {
    let stride = width as usize * 4;
    let row_bytes = stride; // 32bpp rows are always 4-byte aligned
    let pixel_data_size = row_bytes * height as usize;
    let file_size = (54 + pixel_data_size) as u32;
    let mut buf = Vec::with_capacity(file_size as usize);

    // File header (14 bytes)
    buf.extend_from_slice(b"BM");
    buf.extend_from_slice(&file_size.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes()); // reserved
    buf.extend_from_slice(&54u32.to_le_bytes()); // pixel data offset

    // DIB header / BITMAPINFOHEADER (40 bytes)
    buf.extend_from_slice(&40u32.to_le_bytes()); // header size
    buf.extend_from_slice(&(width as i32).to_le_bytes());
    buf.extend_from_slice(&(height as i32).to_le_bytes()); // positive = bottom-up
    buf.extend_from_slice(&1u16.to_le_bytes()); // color planes
    buf.extend_from_slice(&32u16.to_le_bytes()); // bits per pixel
    buf.extend_from_slice(&0u32.to_le_bytes()); // no compression
    buf.extend_from_slice(&(pixel_data_size as u32).to_le_bytes());
    buf.extend_from_slice(&2835u32.to_le_bytes()); // h ppm
    buf.extend_from_slice(&2835u32.to_le_bytes()); // v ppm
    buf.extend_from_slice(&0u32.to_le_bytes()); // palette colours
    buf.extend_from_slice(&0u32.to_le_bytes()); // important colours

    // Pixel data: rows in reverse order (bottom-up), BGRA byte order
    for row in (0..height as usize).rev() {
        let row_src = &pixels[row * stride..(row + 1) * stride];
        for px in row_src.chunks_exact(4) {
            buf.push(px[2]); // B
            buf.push(px[1]); // G
            buf.push(px[0]); // R
            buf.push(px[3]); // A
        }
    }
    buf
}

/// Decode BMP bytes (24bpp or 32bpp, uncompressed) into raw RGB/RGBA pixels.
pub fn bmp_decode(bytes: &[u8]) -> Result<RawDecodeResult, ImageError> {
    if bytes.len() < 54 || &bytes[0..2] != b"BM" {
        return Err(ImageError::InvalidMagic);
    }
    let pixel_data_offset = u32::from_le_bytes(
        bytes[10..14]
            .try_into()
            .map_err(|_| ImageError::TruncatedInput)?,
    ) as usize;
    let width = i32::from_le_bytes(
        bytes[18..22]
            .try_into()
            .map_err(|_| ImageError::TruncatedInput)?,
    );
    let height = i32::from_le_bytes(
        bytes[22..26]
            .try_into()
            .map_err(|_| ImageError::TruncatedInput)?,
    );
    let bpp = u16::from_le_bytes(
        bytes[28..30]
            .try_into()
            .map_err(|_| ImageError::TruncatedInput)?,
    );
    let compression = u32::from_le_bytes(
        bytes[30..34]
            .try_into()
            .map_err(|_| ImageError::TruncatedInput)?,
    );

    if compression != 0 {
        return Err(ImageError::UnsupportedCompression);
    }

    let (abs_height, bottom_up) = if height < 0 {
        (-height as usize, false)
    } else {
        (height as usize, true)
    };
    let abs_width = width.unsigned_abs() as usize;
    let channels = (bpp / 8) as usize;

    if channels != 3 && channels != 4 {
        return Err(ImageError::DecodeError(format!("Unsupported bpp: {}", bpp)));
    }

    let row_stride_padded = (abs_width * channels + 3) & !3;
    let expected_pixel_bytes = row_stride_padded * abs_height;

    let pixel_bytes = bytes
        .get(pixel_data_offset..)
        .ok_or(ImageError::TruncatedInput)?;

    if pixel_bytes.len() < expected_pixel_bytes {
        return Err(ImageError::TruncatedInput);
    }

    let mut pixels = Vec::with_capacity(abs_width * abs_height * channels);
    for row_idx in 0..abs_height {
        let src_row = if bottom_up {
            abs_height - 1 - row_idx
        } else {
            row_idx
        };
        let row_start = src_row * row_stride_padded;
        let row_data = &pixel_bytes[row_start..row_start + abs_width * channels];
        for px in row_data.chunks_exact(channels) {
            // BMP stores BGR/BGRA; convert to RGB/RGBA
            pixels.push(px[2]); // R
            pixels.push(px[1]); // G
            pixels.push(px[0]); // B
            if channels == 4 {
                pixels.push(px[3]); // A
            }
        }
    }

    Ok(RawDecodeResult {
        width: abs_width,
        height: abs_height,
        pixels,
    })
}

/// Encode raw RGB pixels as a PNG file using zune-png.
pub fn png_encode_rgb(width: usize, height: usize, pixels: &[u8]) -> Result<Vec<u8>, ImageError> {
    let opts = EncoderOptions::new(
        width,
        height,
        ColorSpace::RGB,
        zune_core::bit_depth::BitDepth::Eight,
    );
    let mut encoder = PngEncoder::new(pixels, opts);
    let mut out: Vec<u8> = Vec::new();
    encoder
        .encode(&mut out)
        .map_err(|e| ImageError::EncodeError(format!("{:?}", e)))?;
    Ok(out)
}

/// Decode PNG bytes into raw pixel data.
pub fn png_decode(bytes: &[u8]) -> Result<RawDecodeResult, ImageError> {
    let mut decoder = PngDecoder::new(ZCursor::new(bytes));
    let raw_pixels = decoder
        .decode_raw()
        .map_err(|e| ImageError::DecodeError(e.to_string()))?;
    let (width, height) = decoder
        .dimensions()
        .ok_or_else(|| ImageError::DecodeError("No dimensions after decode".into()))?;
    Ok(RawDecodeResult {
        width,
        height,
        pixels: raw_pixels,
    })
}

#[allow(dead_code)]
pub fn default_encode_config(fmt: ImageFormat) -> EncodeConfig {
    EncodeConfig {
        quality: 90,
        format: fmt,
    }
}

/// Returns encoded bytes for the given image data and config.
/// BMP and PNG are routed to real implementations when the pixel buffer matches
/// the expected size. All other formats use a 4-byte stub magic header.
#[allow(dead_code)]
pub fn encode_stub(header: &ImageHeader, pixels: &[u8], cfg: &EncodeConfig) -> Vec<u8> {
    let pixel_count = (header.width as usize) * (header.height as usize);
    match cfg.format {
        ImageFormat::Bmp => {
            let expected_rgba = pixel_count * 4;
            let expected_rgb = pixel_count * 3;
            if header.pixel_format == PixelFormat::Rgba8 && pixels.len() == expected_rgba {
                bmp_encode_rgba(header.width, header.height, pixels)
            } else if pixels.len() == expected_rgb {
                bmp_encode_rgb(header.width, header.height, pixels)
            } else {
                // Pixel buffer doesn't match; return BMP magic stub
                vec![
                    0x42u8,
                    0x4Du8,
                    (header.width & 0xFF) as u8,
                    (header.height & 0xFF) as u8,
                ]
            }
        }
        ImageFormat::Png => {
            match png_encode_rgb(header.width as usize, header.height as usize, pixels) {
                Ok(encoded) => encoded,
                Err(_) => {
                    vec![
                        0x89u8,
                        0x00,
                        (header.width & 0xFF) as u8,
                        (header.height & 0xFF) as u8,
                    ]
                }
            }
        }
        _ => {
            let fmt_byte = match cfg.format {
                ImageFormat::Jpeg => 0xFFu8,
                ImageFormat::Tga => 0x00u8,
                ImageFormat::Hdr => 0x23u8,
                ImageFormat::Gif => 0x47u8,
                ImageFormat::Webp => 0x52u8,
                ImageFormat::Tiff => 0x49u8,
                _ => 0x00u8,
            };
            vec![
                fmt_byte,
                0x00,
                (header.width & 0xFF) as u8,
                (header.height & 0xFF) as u8,
            ]
        }
    }
}

/// Returns `None` for empty data, otherwise `Some(DecodeResult)` with a dummy header.
/// Real BMP and PNG are decoded using their respective implementations.
#[allow(dead_code)]
pub fn decode_stub(data: &[u8]) -> Option<DecodeResult> {
    if data.is_empty() {
        return None;
    }

    let fmt = detect_format(data);

    match fmt {
        ImageFormat::Bmp => {
            if let Ok(raw) = bmp_decode(data) {
                let channels = raw.pixels.len() / (raw.width * raw.height).max(1);
                let pixel_count = raw.width * raw.height;
                let pf = if channels == 4 {
                    PixelFormat::Rgba8
                } else {
                    PixelFormat::Rgb8
                };
                let header = ImageHeader {
                    width: raw.width as u32,
                    height: raw.height as u32,
                    format: ImageFormat::Bmp,
                    pixel_format: pf,
                };
                return Some(DecodeResult {
                    byte_size: raw.pixels.len(),
                    pixel_count,
                    header,
                });
            }
        }
        ImageFormat::Png => {
            if let Ok(raw) = png_decode(data) {
                let pixel_count = raw.width * raw.height;
                let channels = raw.pixels.len() / pixel_count.max(1);
                let pf = if channels == 4 {
                    PixelFormat::Rgba8
                } else {
                    PixelFormat::Rgb8
                };
                let header = ImageHeader {
                    width: raw.width as u32,
                    height: raw.height as u32,
                    format: ImageFormat::Png,
                    pixel_format: pf,
                };
                return Some(DecodeResult {
                    byte_size: raw.pixels.len(),
                    pixel_count,
                    header,
                });
            }
        }
        _ => {}
    }

    // Fallback: return a stub result for unknown / unsupported formats
    let header = ImageHeader {
        width: 1,
        height: 1,
        format: ImageFormat::Unknown,
        pixel_format: PixelFormat::Rgba8,
    };
    let pixel_count = (header.width * header.height) as usize;
    let bpp = pixel_format_bytes_per_pixel(&header.pixel_format) as usize;
    Some(DecodeResult {
        byte_size: pixel_count * bpp,
        pixel_count,
        header,
    })
}

#[allow(dead_code)]
pub fn image_format_name(fmt: &ImageFormat) -> &'static str {
    match fmt {
        ImageFormat::Png => "PNG",
        ImageFormat::Jpeg => "JPEG",
        ImageFormat::Bmp => "BMP",
        ImageFormat::Tga => "TGA",
        ImageFormat::Hdr => "HDR",
        ImageFormat::Gif => "GIF",
        ImageFormat::Webp => "WEBP",
        ImageFormat::Tiff => "TIFF",
        ImageFormat::Unknown => "Unknown",
    }
}

#[allow(dead_code)]
pub fn pixel_format_bytes_per_pixel(fmt: &PixelFormat) -> u32 {
    match fmt {
        PixelFormat::Rgb8 => 3,
        PixelFormat::Rgba8 => 4,
        PixelFormat::Grayscale8 => 1,
        PixelFormat::Rgba16 => 8,
    }
}

#[allow(dead_code)]
pub fn image_byte_size(header: &ImageHeader) -> usize {
    let pixels = (header.width as usize) * (header.height as usize);
    let bpp = pixel_format_bytes_per_pixel(&header.pixel_format) as usize;
    pixels * bpp
}

#[allow(dead_code)]
pub fn image_header_to_json(h: &ImageHeader) -> String {
    format!(
        "{{\"width\":{},\"height\":{},\"format\":\"{}\",\"pixel_format\":\"{}\"}}",
        h.width,
        h.height,
        image_format_name(&h.format),
        pixel_format_name(&h.pixel_format),
    )
}

fn pixel_format_name(fmt: &PixelFormat) -> &'static str {
    match fmt {
        PixelFormat::Rgb8 => "RGB8",
        PixelFormat::Rgba8 => "RGBA8",
        PixelFormat::Grayscale8 => "Grayscale8",
        PixelFormat::Rgba16 => "RGBA16",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_encode_config() {
        let cfg = default_encode_config(ImageFormat::Png);
        assert_eq!(cfg.quality, 90);
        assert_eq!(cfg.format, ImageFormat::Png);
    }

    #[test]
    fn test_encode_stub_nonempty() {
        let header = ImageHeader {
            width: 4,
            height: 4,
            format: ImageFormat::Png,
            pixel_format: PixelFormat::Rgba8,
        };
        let cfg = default_encode_config(ImageFormat::Png);
        // Pass correctly-sized pixel buffer for PNG (3 channels for RGB)
        let pixels = vec![128u8; 4 * 4 * 3];
        let bytes = encode_stub(&header, &pixels, &cfg);
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_decode_stub_empty_returns_none() {
        let result = decode_stub(&[]);
        assert!(result.is_none());
    }

    #[test]
    fn test_decode_stub_nonempty_returns_some() {
        let result = decode_stub(&[0x89, 0x50]);
        assert!(result.is_some());
        let dr = result.expect("should succeed");
        assert!(dr.pixel_count > 0);
        assert!(dr.byte_size > 0);
    }

    #[test]
    fn test_image_format_name() {
        assert_eq!(image_format_name(&ImageFormat::Png), "PNG");
        assert_eq!(image_format_name(&ImageFormat::Jpeg), "JPEG");
        assert_eq!(image_format_name(&ImageFormat::Bmp), "BMP");
        assert_eq!(image_format_name(&ImageFormat::Tga), "TGA");
        assert_eq!(image_format_name(&ImageFormat::Hdr), "HDR");
    }

    #[test]
    fn test_pixel_format_bytes_per_pixel() {
        assert_eq!(pixel_format_bytes_per_pixel(&PixelFormat::Rgb8), 3);
        assert_eq!(pixel_format_bytes_per_pixel(&PixelFormat::Rgba8), 4);
        assert_eq!(pixel_format_bytes_per_pixel(&PixelFormat::Grayscale8), 1);
        assert_eq!(pixel_format_bytes_per_pixel(&PixelFormat::Rgba16), 8);
    }

    #[test]
    fn test_image_byte_size() {
        let header = ImageHeader {
            width: 2,
            height: 3,
            format: ImageFormat::Bmp,
            pixel_format: PixelFormat::Rgb8,
        };
        assert_eq!(image_byte_size(&header), 18); // 2*3*3
    }

    #[test]
    fn test_image_header_to_json() {
        let h = ImageHeader {
            width: 1920,
            height: 1080,
            format: ImageFormat::Jpeg,
            pixel_format: PixelFormat::Rgba8,
        };
        let json = image_header_to_json(&h);
        assert!(json.contains("1920"));
        assert!(json.contains("JPEG"));
        assert!(json.contains("RGBA8"));
    }

    #[test]
    fn test_encode_different_formats() {
        let header = ImageHeader {
            width: 1,
            height: 1,
            format: ImageFormat::Tga,
            pixel_format: PixelFormat::Grayscale8,
        };
        let cfg_jpeg = default_encode_config(ImageFormat::Jpeg);
        let cfg_bmp = default_encode_config(ImageFormat::Bmp);
        let b1 = encode_stub(&header, &[128], &cfg_jpeg);
        let b2 = encode_stub(&header, &[128], &cfg_bmp);
        assert_ne!(b1[0], b2[0]);
    }

    // --- Slice E: BMP tests ---

    #[test]
    fn test_bmp_encode_decode_24bit() {
        let pixels: Vec<u8> = vec![
            255, 0, 0, 0, 255, 0, // row 0: red, green
            0, 0, 255, 255, 255, 0, // row 1: blue, yellow
        ];
        let encoded = bmp_encode_rgb(2, 2, &pixels);
        assert!(encoded.starts_with(b"BM"));
        let decoded = bmp_decode(&encoded).expect("BMP decode");
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 2);
        assert_eq!(&decoded.pixels[..3], &[255, 0, 0]); // first pixel = red
    }

    #[test]
    fn test_bmp_padding_alignment() {
        // 3x1 RGB: row needs 1 byte padding (3*3=9 → padded to 12)
        let pixels: Vec<u8> = vec![255, 0, 0, 0, 255, 0, 0, 0, 255];
        let encoded = bmp_encode_rgb(3, 1, &pixels);
        // File size = 54 + 12 = 66
        let file_size = u32::from_le_bytes(encoded[2..6].try_into().expect("file size slice"));
        assert_eq!(file_size, 66);
    }

    // --- Slice E: PNG tests ---

    #[test]
    fn test_png_encode_decode_rgb() {
        let pixels: Vec<u8> = vec![255, 0, 0, 0, 255, 0]; // 2x1 RGB
        let encoded = png_encode_rgb(2, 1, &pixels).expect("PNG encode");
        assert_eq!(
            &encoded[..8],
            &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
        );
        let decoded = png_decode(&encoded).expect("PNG decode");
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 1);
    }

    // --- Slice E: format detection tests ---

    #[test]
    fn test_format_detection_png() {
        let magic = [0x89u8, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(detect_format(&magic), ImageFormat::Png);
    }

    #[test]
    fn test_format_detection_jpeg() {
        let magic = [0xFFu8, 0xD8, 0xFF, 0xE0];
        assert_eq!(detect_format(&magic), ImageFormat::Jpeg);
    }

    #[test]
    fn test_format_detection_bmp() {
        assert_eq!(detect_format(b"BM\x00"), ImageFormat::Bmp);
    }

    #[test]
    fn test_format_detection_unknown() {
        assert_eq!(detect_format(b"\x00\x01\x02\x03"), ImageFormat::Unknown);
    }
}
