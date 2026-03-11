// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! BMP image stub export.

/// BMP bit depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BmpBitDepth {
    Bpp24,
    Bpp32,
}

impl BmpBitDepth {
    /// Bytes per pixel.
    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            Self::Bpp24 => 3,
            Self::Bpp32 => 4,
        }
    }
}

/// BMP image stub.
#[derive(Debug, Clone)]
pub struct BmpExport {
    pub width: u32,
    pub height: u32,
    pub bit_depth: BmpBitDepth,
    pub pixels: Vec<[u8; 4]>,
}

impl BmpExport {
    /// Create BMP filled with solid color.
    pub fn new_solid(width: u32, height: u32, bit_depth: BmpBitDepth, color: [u8; 4]) -> Self {
        let pixels = vec![color; (width * height) as usize];
        Self {
            width,
            height,
            bit_depth,
            pixels,
        }
    }

    /// Pixel count.
    pub fn pixel_count(&self) -> usize {
        self.pixels.len()
    }
}

/// Build BMP header bytes (stub).
pub fn build_bmp_header(export: &BmpExport) -> Vec<u8> {
    let bytes_per_pixel = export.bit_depth.bytes_per_pixel();
    let row_size = (export.width as usize * bytes_per_pixel).div_ceil(4) * 4;
    let pixel_data_size = row_size * export.height as usize;
    let file_size = 54 + pixel_data_size;
    let mut header = vec![0u8; 54];
    header[0] = b'B';
    header[1] = b'M';
    header[2..6].copy_from_slice(&(file_size as u32).to_le_bytes());
    header[10] = 54;
    header[14] = 40;
    header[18..22].copy_from_slice(&export.width.to_le_bytes());
    header[22..26].copy_from_slice(&export.height.to_le_bytes());
    header[26] = 1;
    header[28] = (bytes_per_pixel * 8) as u8;
    header
}

/// Validate BMP export.
pub fn validate_bmp(export: &BmpExport) -> bool {
    export.width > 0
        && export.height > 0
        && export.pixels.len() == (export.width * export.height) as usize
}

/// Estimate BMP file size.
pub fn estimate_bmp_bytes(export: &BmpExport) -> usize {
    let bytes_per_pixel = export.bit_depth.bytes_per_pixel();
    let row_size = (export.width as usize * bytes_per_pixel).div_ceil(4) * 4;
    54 + row_size * export.height as usize
}

/// Average pixel brightness (0-255).
pub fn average_brightness(export: &BmpExport) -> f32 {
    if export.pixels.is_empty() {
        return 0.0;
    }
    let sum: f32 = export
        .pixels
        .iter()
        .map(|p| (p[0] as f32 + p[1] as f32 + p[2] as f32) / 3.0)
        .sum();
    sum / export.pixels.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> BmpExport {
        BmpExport::new_solid(8, 8, BmpBitDepth::Bpp24, [100, 150, 200, 255])
    }

    #[test]
    fn test_pixel_count() {
        /* pixel count matches dimensions */
        assert_eq!(sample().pixel_count(), 64);
    }

    #[test]
    fn test_validate_valid() {
        /* valid export passes */
        assert!(validate_bmp(&sample()));
    }

    #[test]
    fn test_bmp_header_magic() {
        /* BMP header starts with BM */
        let h = build_bmp_header(&sample());
        assert_eq!(h[0], b'B');
        assert_eq!(h[1], b'M');
    }

    #[test]
    fn test_estimate_bytes_positive() {
        /* estimated size is positive */
        assert!(estimate_bmp_bytes(&sample()) > 0);
    }

    #[test]
    fn test_average_brightness() {
        /* average brightness of grey pixel is correct */
        let e = BmpExport::new_solid(2, 2, BmpBitDepth::Bpp24, [100, 100, 100, 255]);
        assert!((average_brightness(&e) - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_bpp24_bytes_per_pixel() {
        /* 24-bit has 3 bytes per pixel */
        assert_eq!(BmpBitDepth::Bpp24.bytes_per_pixel(), 3);
    }

    #[test]
    fn test_bpp32_estimate_larger() {
        /* 32-bit estimate is larger than 24-bit */
        let e24 = BmpExport::new_solid(8, 8, BmpBitDepth::Bpp24, [0; 4]);
        let e32 = BmpExport::new_solid(8, 8, BmpBitDepth::Bpp32, [0; 4]);
        assert!(estimate_bmp_bytes(&e32) > estimate_bmp_bytes(&e24));
    }

    #[test]
    fn test_validate_zero_dim() {
        /* zero dimension fails validation */
        let mut e = sample();
        e.width = 0;
        assert!(!validate_bmp(&e));
    }
}
