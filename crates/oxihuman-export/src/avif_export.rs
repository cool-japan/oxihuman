// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! AVIF image stub export.

/// AVIF quality preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvifPreset {
    Fast,
    Balanced,
    Best,
}

impl AvifPreset {
    /// Return the quantizer value for this preset (lower = better quality).
    pub fn quantizer(&self) -> u8 {
        match self {
            Self::Fast => 40,
            Self::Balanced => 24,
            Self::Best => 10,
        }
    }
}

/// AVIF export options.
#[derive(Debug, Clone)]
pub struct AvifOptions {
    pub width: u32,
    pub height: u32,
    pub preset: AvifPreset,
    pub alpha: bool,
}

impl Default for AvifOptions {
    fn default() -> Self {
        Self {
            width: 512,
            height: 512,
            preset: AvifPreset::Balanced,
            alpha: true,
        }
    }
}

/// AVIF image stub.
#[derive(Debug, Clone)]
pub struct AvifExport {
    pub options: AvifOptions,
    pub pixels: Vec<[u8; 4]>,
}

impl AvifExport {
    /// Create a new AVIF export filled with solid color.
    pub fn new_solid(width: u32, height: u32, color: [u8; 4]) -> Self {
        let pixels = vec![color; (width * height) as usize];
        Self {
            options: AvifOptions {
                width,
                height,
                ..Default::default()
            },
            pixels,
        }
    }

    /// Pixel count.
    pub fn pixel_count(&self) -> usize {
        self.pixels.len()
    }
}

/// Estimate AVIF file size (stub).
pub fn estimate_avif_bytes(export: &AvifExport) -> usize {
    let raw = export.pixel_count() * 3;
    let q = export.options.preset.quantizer() as usize;
    (raw * q / 64).max(256)
}

/// Validate AVIF export.
pub fn validate_avif(export: &AvifExport) -> bool {
    export.options.width > 0
        && export.options.height > 0
        && export.pixels.len() == (export.options.width * export.options.height) as usize
}

/// Serialize AVIF metadata to JSON (stub).
pub fn avif_metadata_json(export: &AvifExport) -> String {
    format!(
        "{{\"width\":{},\"height\":{},\"quantizer\":{},\"alpha\":{}}}",
        export.options.width,
        export.options.height,
        export.options.preset.quantizer(),
        export.options.alpha
    )
}

/// Check if all pixels are fully opaque.
pub fn all_opaque(export: &AvifExport) -> bool {
    export.pixels.iter().all(|p| p[3] == 255)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> AvifExport {
        AvifExport::new_solid(8, 8, [128, 64, 32, 255])
    }

    #[test]
    fn test_pixel_count() {
        /* pixel count matches dimensions */
        assert_eq!(sample().pixel_count(), 64);
    }

    #[test]
    fn test_validate_valid() {
        /* valid export passes */
        assert!(validate_avif(&sample()));
    }

    #[test]
    fn test_preset_quantizer() {
        /* each preset has distinct quantizer */
        assert!(AvifPreset::Best.quantizer() < AvifPreset::Balanced.quantizer());
        assert!(AvifPreset::Balanced.quantizer() < AvifPreset::Fast.quantizer());
    }

    #[test]
    fn test_estimate_bytes_positive() {
        /* byte estimate is positive */
        assert!(estimate_avif_bytes(&sample()) > 0);
    }

    #[test]
    fn test_metadata_json_has_quantizer() {
        /* metadata JSON contains quantizer */
        assert!(avif_metadata_json(&sample()).contains("quantizer"));
    }

    #[test]
    fn test_all_opaque_true() {
        /* solid opaque image is all opaque */
        assert!(all_opaque(&sample()));
    }

    #[test]
    fn test_all_opaque_false() {
        /* image with transparent pixel is not all opaque */
        let mut e = sample();
        e.pixels[0][3] = 0;
        assert!(!all_opaque(&e));
    }

    #[test]
    fn test_validate_zero_dim() {
        /* zero dimension fails validation */
        let mut e = sample();
        e.options.height = 0;
        assert!(!validate_avif(&e));
    }
}
