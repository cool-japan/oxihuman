// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Pixel art / posterize render mode stub.

/// Palette quantization mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QuantizeMode {
    NearestNeighbor,
    OrderedDither,
    ErrorDiffusion,
}

/// Pixel art view configuration.
#[derive(Debug, Clone)]
pub struct PixelArtView {
    pub pixel_size: u32,
    pub palette_colors: u32,
    pub quantize_mode: QuantizeMode,
    pub outline: bool,
    pub enabled: bool,
}

impl PixelArtView {
    pub fn new() -> Self {
        PixelArtView {
            pixel_size: 8,
            palette_colors: 16,
            quantize_mode: QuantizeMode::NearestNeighbor,
            outline: false,
            enabled: true,
        }
    }
}

impl Default for PixelArtView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new pixel art view.
pub fn new_pixel_art_view() -> PixelArtView {
    PixelArtView::new()
}

/// Set pixel block size.
pub fn pav_set_pixel_size(view: &mut PixelArtView, size: u32) {
    view.pixel_size = size.clamp(1, 64);
}

/// Set palette color count.
pub fn pav_set_palette_colors(view: &mut PixelArtView, colors: u32) {
    view.palette_colors = colors.clamp(2, 256);
}

/// Set quantization mode.
pub fn pav_set_quantize_mode(view: &mut PixelArtView, mode: QuantizeMode) {
    view.quantize_mode = mode;
}

/// Toggle pixel outline.
pub fn pav_set_outline(view: &mut PixelArtView, outline: bool) {
    view.outline = outline;
}

/// Enable or disable.
pub fn pav_set_enabled(view: &mut PixelArtView, enabled: bool) {
    view.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn pav_to_json(view: &PixelArtView) -> String {
    let mode = match view.quantize_mode {
        QuantizeMode::NearestNeighbor => "nearest",
        QuantizeMode::OrderedDither => "ordered_dither",
        QuantizeMode::ErrorDiffusion => "error_diffusion",
    };
    format!(
        r#"{{"pixel_size":{},"palette_colors":{},"quantize_mode":"{}","outline":{},"enabled":{}}}"#,
        view.pixel_size, view.palette_colors, mode, view.outline, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_pixel_size() {
        let v = new_pixel_art_view();
        assert_eq!(v.pixel_size, 8 /* default pixel size must be 8 */);
    }

    #[test]
    fn test_set_pixel_size() {
        let mut v = new_pixel_art_view();
        pav_set_pixel_size(&mut v, 16);
        assert_eq!(v.pixel_size, 16 /* pixel size must be set */);
    }

    #[test]
    fn test_pixel_size_clamped() {
        let mut v = new_pixel_art_view();
        pav_set_pixel_size(&mut v, 0);
        assert_eq!(v.pixel_size, 1 /* pixel size clamped to minimum 1 */);
    }

    #[test]
    fn test_set_palette_colors() {
        let mut v = new_pixel_art_view();
        pav_set_palette_colors(&mut v, 32);
        assert_eq!(v.palette_colors, 32 /* palette colors must be set */);
    }

    #[test]
    fn test_palette_colors_clamped() {
        let mut v = new_pixel_art_view();
        pav_set_palette_colors(&mut v, 1024);
        assert_eq!(
            v.palette_colors,
            256 /* palette colors clamped to 256 */
        );
    }

    #[test]
    fn test_set_quantize_mode() {
        let mut v = new_pixel_art_view();
        pav_set_quantize_mode(&mut v, QuantizeMode::OrderedDither);
        assert_eq!(
            v.quantize_mode,
            QuantizeMode::OrderedDither /* quantize mode must be set */
        );
    }

    #[test]
    fn test_set_outline() {
        let mut v = new_pixel_art_view();
        pav_set_outline(&mut v, true);
        assert!(v.outline /* outline must be enabled */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_pixel_art_view();
        pav_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_pixel_size() {
        let v = new_pixel_art_view();
        let j = pav_to_json(&v);
        assert!(j.contains("\"pixel_size\"") /* JSON must contain pixel_size */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_pixel_art_view();
        assert!(v.enabled /* must be enabled by default */);
    }
}
