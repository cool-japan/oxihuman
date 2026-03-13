// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! TIFF image stub export.

/// TIFF compression type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TiffCompression {
    None,
    Lzw,
    Deflate,
}

impl TiffCompression {
    /// Return TIFF compression tag value.
    pub fn tag_value(&self) -> u16 {
        match self {
            Self::None => 1,
            Self::Lzw => 5,
            Self::Deflate => 8,
        }
    }
}

/// TIFF export options.
#[derive(Debug, Clone)]
pub struct TiffOptions {
    pub width: u32,
    pub height: u32,
    pub bits_per_sample: u8,
    pub samples_per_pixel: u8,
    pub compression: TiffCompression,
}

impl Default for TiffOptions {
    fn default() -> Self {
        Self {
            width: 512,
            height: 512,
            bits_per_sample: 8,
            samples_per_pixel: 4,
            compression: TiffCompression::None,
        }
    }
}

/// TIFF image stub.
#[derive(Debug, Clone)]
pub struct TiffExport {
    pub options: TiffOptions,
    pub pixels: Vec<u8>,
}

impl TiffExport {
    /// Create TIFF export filled with solid RGBA color.
    pub fn new_solid(width: u32, height: u32, color: [u8; 4]) -> Self {
        let n = (width * height) as usize;
        let mut pixels = Vec::with_capacity(n * 4);
        for _ in 0..n {
            pixels.extend_from_slice(&color);
        }
        Self {
            options: TiffOptions {
                width,
                height,
                ..Default::default()
            },
            pixels,
        }
    }

    /// Pixel count.
    pub fn pixel_count(&self) -> usize {
        (self.options.width * self.options.height) as usize
    }

    /// Raw byte count of pixel data.
    pub fn raw_bytes(&self) -> usize {
        self.pixels.len()
    }
}

/// Estimate TIFF file size including header (stub).
pub fn estimate_tiff_bytes(export: &TiffExport) -> usize {
    let raw = export.raw_bytes();
    match export.options.compression {
        TiffCompression::None => raw + 512,
        TiffCompression::Lzw => raw / 3 + 512,
        TiffCompression::Deflate => raw / 4 + 512,
    }
}

/// Validate TIFF options.
pub fn validate_tiff(export: &TiffExport) -> bool {
    export.options.width > 0
        && export.options.height > 0
        && export.options.bits_per_sample > 0
        && export.options.samples_per_pixel > 0
}

/// Serialize TIFF metadata to JSON (stub).
pub fn tiff_metadata_json(export: &TiffExport) -> String {
    format!(
        "{{\"width\":{},\"height\":{},\"bps\":{},\"spp\":{},\"compression\":{}}}",
        export.options.width,
        export.options.height,
        export.options.bits_per_sample,
        export.options.samples_per_pixel,
        export.options.compression.tag_value()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> TiffExport {
        TiffExport::new_solid(8, 8, [200, 150, 100, 255])
    }

    #[test]
    fn test_pixel_count() {
        /* pixel count matches dimensions */
        assert_eq!(sample().pixel_count(), 64);
    }

    #[test]
    fn test_raw_bytes() {
        /* raw bytes = pixel_count * 4 for RGBA */
        assert_eq!(sample().raw_bytes(), 256);
    }

    #[test]
    fn test_validate_valid() {
        /* valid export passes */
        assert!(validate_tiff(&sample()));
    }

    #[test]
    fn test_compression_tag() {
        /* compression tags are distinct */
        assert_ne!(
            TiffCompression::None.tag_value(),
            TiffCompression::Lzw.tag_value()
        );
    }

    #[test]
    fn test_estimate_bytes_none_largest() {
        /* no-compression estimate is larger than compressed */
        let mut e = sample();
        let none_size = estimate_tiff_bytes(&e);
        e.options.compression = TiffCompression::Deflate;
        let deflate_size = estimate_tiff_bytes(&e);
        assert!(none_size > deflate_size);
    }

    #[test]
    fn test_metadata_json_has_bps() {
        /* metadata JSON contains bps field */
        assert!(tiff_metadata_json(&sample()).contains("bps"));
    }

    #[test]
    fn test_validate_zero_dim() {
        /* zero dimension fails validation */
        let mut e = sample();
        e.options.width = 0;
        assert!(!validate_tiff(&e));
    }

    #[test]
    fn test_default_options() {
        /* default options are valid */
        let opts = TiffOptions::default();
        assert_eq!(opts.bits_per_sample, 8);
    }
}
