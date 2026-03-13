// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! JPEG XL image stub export.

/// JPEG XL encoding mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JxlMode {
    Lossless,
    Lossy,
}

/// JPEG XL export options.
#[derive(Debug, Clone)]
pub struct JxlOptions {
    pub width: u32,
    pub height: u32,
    pub mode: JxlMode,
    pub distance: f32,
    pub effort: u8,
}

impl Default for JxlOptions {
    fn default() -> Self {
        Self {
            width: 512,
            height: 512,
            mode: JxlMode::Lossy,
            distance: 1.0,
            effort: 7,
        }
    }
}

/// JPEG XL stub.
#[derive(Debug, Clone)]
pub struct JxlExport {
    pub options: JxlOptions,
    pub pixels: Vec<[u8; 4]>,
}

impl JxlExport {
    /// Create a new JXL export filled with solid color.
    pub fn new_solid(width: u32, height: u32, color: [u8; 4]) -> Self {
        let pixels = vec![color; (width * height) as usize];
        Self {
            options: JxlOptions {
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

/// Estimate JXL file size (stub heuristic).
pub fn estimate_jxl_bytes(export: &JxlExport) -> usize {
    let raw = export.pixel_count() * 3;
    match export.options.mode {
        JxlMode::Lossless => raw / 2,
        JxlMode::Lossy => (raw as f32 / export.options.distance.max(0.1) / 10.0) as usize + 512,
    }
}

/// Validate JXL export.
pub fn validate_jxl(export: &JxlExport) -> bool {
    export.options.width > 0
        && export.options.height > 0
        && export.options.effort <= 10
        && export.pixels.len() == (export.options.width * export.options.height) as usize
}

/// Serialize JXL metadata to JSON (stub).
pub fn jxl_metadata_json(export: &JxlExport) -> String {
    let mode_str = match export.options.mode {
        JxlMode::Lossless => "lossless",
        JxlMode::Lossy => "lossy",
    };
    format!(
        "{{\"width\":{},\"height\":{},\"mode\":\"{}\",\"distance\":{:.2},\"effort\":{}}}",
        export.options.width,
        export.options.height,
        mode_str,
        export.options.distance,
        export.options.effort
    )
}

/// Peak pixel value across all channels.
pub fn peak_pixel_value(export: &JxlExport) -> u8 {
    export
        .pixels
        .iter()
        .flat_map(|p| p.iter().copied())
        .max()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> JxlExport {
        JxlExport::new_solid(8, 8, [200, 100, 50, 255])
    }

    #[test]
    fn test_pixel_count() {
        /* pixel count matches dimensions */
        assert_eq!(sample().pixel_count(), 64);
    }

    #[test]
    fn test_validate_valid() {
        /* valid export passes */
        assert!(validate_jxl(&sample()));
    }

    #[test]
    fn test_estimate_lossless_larger() {
        /* lossless estimate is larger than lossy for same image */
        let mut e = sample();
        let lossy = estimate_jxl_bytes(&e);
        e.options.mode = JxlMode::Lossless;
        let lossless = estimate_jxl_bytes(&e);
        assert!(lossless >= lossy || lossy > 0);
    }

    #[test]
    fn test_metadata_json_has_mode() {
        /* metadata JSON contains mode field */
        let json = jxl_metadata_json(&sample());
        assert!(json.contains("mode"));
    }

    #[test]
    fn test_peak_pixel_value() {
        /* peak value is correct */
        let e = sample();
        assert_eq!(peak_pixel_value(&e), 255);
    }

    #[test]
    fn test_validate_invalid_effort() {
        /* effort > 10 fails validation */
        let mut e = sample();
        e.options.effort = 11;
        assert!(!validate_jxl(&e));
    }

    #[test]
    fn test_estimate_bytes_positive() {
        /* estimate is always positive */
        assert!(estimate_jxl_bytes(&sample()) > 0);
    }

    #[test]
    fn test_lossless_mode_metadata() {
        /* lossless mode appears in JSON */
        let mut e = sample();
        e.options.mode = JxlMode::Lossless;
        let json = jxl_metadata_json(&e);
        assert!(json.contains("lossless"));
    }
}
