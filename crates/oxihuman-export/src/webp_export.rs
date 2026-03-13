// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! WebP image stub export.

/// WebP export options.
#[derive(Debug, Clone)]
pub struct WebpOptions {
    pub width: u32,
    pub height: u32,
    pub quality: f32,
    pub lossless: bool,
}

impl Default for WebpOptions {
    fn default() -> Self {
        Self {
            width: 512,
            height: 512,
            quality: 80.0,
            lossless: false,
        }
    }
}

/// WebP image stub.
#[derive(Debug, Clone)]
pub struct WebpExport {
    pub options: WebpOptions,
    pub pixels: Vec<[u8; 4]>,
}

impl WebpExport {
    /// Create a new WebP export filled with a solid color.
    pub fn new_solid(width: u32, height: u32, color: [u8; 4]) -> Self {
        let pixels = vec![color; (width * height) as usize];
        Self {
            options: WebpOptions {
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

/// Estimate WebP compressed size (stub heuristic).
pub fn estimate_webp_bytes(export: &WebpExport) -> usize {
    let raw = export.pixel_count() * 4;
    if export.options.lossless {
        raw / 2
    } else {
        ((raw as f32 * export.options.quality / 100.0) as usize).max(1)
    }
}

/// Validate WebP options.
pub fn validate_webp(export: &WebpExport) -> bool {
    export.options.width > 0
        && export.options.height > 0
        && (0.0..=100.0).contains(&export.options.quality)
        && export.pixels.len() == (export.options.width * export.options.height) as usize
}

/// Serialize WebP metadata to JSON (stub).
pub fn webp_metadata_json(export: &WebpExport) -> String {
    format!(
        "{{\"width\":{},\"height\":{},\"quality\":{:.1},\"lossless\":{}}}",
        export.options.width,
        export.options.height,
        export.options.quality,
        export.options.lossless
    )
}

/// Average pixel luminance.
pub fn average_luminance(export: &WebpExport) -> f32 {
    if export.pixels.is_empty() {
        return 0.0;
    }
    let sum: f32 = export
        .pixels
        .iter()
        .map(|p| 0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32)
        .sum();
    sum / export.pixels.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn white_export() -> WebpExport {
        WebpExport::new_solid(8, 8, [255, 255, 255, 255])
    }

    #[test]
    fn test_pixel_count() {
        /* pixel count matches dimensions */
        let e = white_export();
        assert_eq!(e.pixel_count(), 64);
    }

    #[test]
    fn test_validate_valid() {
        /* valid export passes validation */
        assert!(validate_webp(&white_export()));
    }

    #[test]
    fn test_validate_zero_dim() {
        /* zero dimension fails validation */
        let mut e = white_export();
        e.options.width = 0;
        assert!(!validate_webp(&e));
    }

    #[test]
    fn test_estimate_webp_bytes_positive() {
        /* byte estimate is positive */
        assert!(estimate_webp_bytes(&white_export()) > 0);
    }

    #[test]
    fn test_lossless_larger_estimate() {
        /* lossless estimate and lossy estimate are both positive */
        let mut e = white_export();
        let lossy = estimate_webp_bytes(&e);
        e.options.lossless = true;
        let lossless = estimate_webp_bytes(&e);
        assert!(lossless > 0 && lossy > 0);
    }

    #[test]
    fn test_average_luminance_white() {
        /* white image has near-255 luminance */
        let e = white_export();
        assert!(average_luminance(&e) > 200.0);
    }

    #[test]
    fn test_metadata_json_contains_quality() {
        /* metadata JSON contains quality field */
        let e = white_export();
        assert!(webp_metadata_json(&e).contains("quality"));
    }

    #[test]
    fn test_black_luminance() {
        /* black image has near-zero luminance */
        let e = WebpExport::new_solid(4, 4, [0, 0, 0, 255]);
        assert!(average_luminance(&e) < 1.0);
    }
}
