// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Image encode/decode stub providing format detection, header construction, and byte-size utilities.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Bmp,
    Tga,
    Hdr,
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

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DecodeResult {
    pub header: ImageHeader,
    pub pixel_count: usize,
    pub byte_size: usize,
}

#[allow(dead_code)]
pub fn default_encode_config(fmt: ImageFormat) -> EncodeConfig {
    EncodeConfig {
        quality: 90,
        format: fmt,
    }
}

/// Returns dummy encoded bytes as a stub (4-byte magic header).
#[allow(dead_code)]
pub fn encode_stub(header: &ImageHeader, _pixels: &[u8], cfg: &EncodeConfig) -> Vec<u8> {
    let fmt_byte = match cfg.format {
        ImageFormat::Png => 0x89u8,
        ImageFormat::Jpeg => 0xFFu8,
        ImageFormat::Bmp => 0x42u8,
        ImageFormat::Tga => 0x00u8,
        ImageFormat::Hdr => 0x23u8,
    };
    vec![fmt_byte, 0x00, (header.width & 0xFF) as u8, (header.height & 0xFF) as u8]
}

/// Returns `None` for empty data, otherwise `Some(DecodeResult)` with a dummy header.
#[allow(dead_code)]
pub fn decode_stub(data: &[u8]) -> Option<DecodeResult> {
    if data.is_empty() {
        return None;
    }
    let header = ImageHeader {
        width: 1,
        height: 1,
        format: ImageFormat::Png,
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
        let bytes = encode_stub(&header, &[], &cfg);
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
        let dr = result.unwrap();
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
}
