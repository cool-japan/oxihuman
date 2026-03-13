#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

/// A stub thumbnail export (no actual rendering).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ThumbnailExport {
    pub width: u32,
    pub height: u32,
    /// RGBA pixel data.
    pub pixels: Vec<u8>,
}

/// Create a stub thumbnail filled with a solid color.
#[allow(dead_code)]
pub fn export_thumbnail_stub(width: u32, height: u32, r: u8, g: u8, b: u8, a: u8) -> ThumbnailExport {
    let count = (width * height) as usize;
    let mut pixels = Vec::with_capacity(count * 4);
    for _ in 0..count {
        pixels.push(r);
        pixels.push(g);
        pixels.push(b);
        pixels.push(a);
    }
    ThumbnailExport { width, height, pixels }
}

/// Return the thumbnail width.
#[allow(dead_code)]
pub fn thumbnail_width(t: &ThumbnailExport) -> u32 {
    t.width
}

/// Return the thumbnail height.
#[allow(dead_code)]
pub fn thumbnail_height(t: &ThumbnailExport) -> u32 {
    t.height
}

/// Return the total pixel count.
#[allow(dead_code)]
pub fn thumbnail_pixel_count(t: &ThumbnailExport) -> u32 {
    t.width * t.height
}

/// Return a clone of the RGBA data.
#[allow(dead_code)]
pub fn thumbnail_to_rgba(t: &ThumbnailExport) -> Vec<u8> {
    t.pixels.clone()
}

/// Convert to grayscale (single channel).
#[allow(dead_code)]
pub fn thumbnail_to_grayscale(t: &ThumbnailExport) -> Vec<u8> {
    let count = (t.width * t.height) as usize;
    let mut gray = Vec::with_capacity(count);
    for i in 0..count {
        let r = t.pixels[i * 4] as f32;
        let g = t.pixels[i * 4 + 1] as f32;
        let b = t.pixels[i * 4 + 2] as f32;
        gray.push((0.299 * r + 0.587 * g + 0.114 * b) as u8);
    }
    gray
}

/// Return the aspect ratio (width / height).
#[allow(dead_code)]
pub fn thumbnail_aspect_ratio(t: &ThumbnailExport) -> f32 {
    if t.height == 0 {
        return 0.0;
    }
    t.width as f32 / t.height as f32
}

/// Return the byte size of the RGBA export.
#[allow(dead_code)]
pub fn thumbnail_export_size(t: &ThumbnailExport) -> usize {
    t.pixels.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> ThumbnailExport {
        export_thumbnail_stub(4, 2, 128, 64, 32, 255)
    }

    #[test]
    fn test_stub_creation() {
        let t = sample();
        assert_eq!(t.width, 4);
        assert_eq!(t.height, 2);
    }

    #[test]
    fn test_width() {
        assert_eq!(thumbnail_width(&sample()), 4);
    }

    #[test]
    fn test_height() {
        assert_eq!(thumbnail_height(&sample()), 2);
    }

    #[test]
    fn test_pixel_count() {
        assert_eq!(thumbnail_pixel_count(&sample()), 8);
    }

    #[test]
    fn test_to_rgba() {
        let rgba = thumbnail_to_rgba(&sample());
        assert_eq!(rgba.len(), 32); // 4*2*4
        assert_eq!(rgba[0], 128);
    }

    #[test]
    fn test_to_grayscale() {
        let g = thumbnail_to_grayscale(&sample());
        assert_eq!(g.len(), 8);
        assert!(g[0] > 0);
    }

    #[test]
    fn test_aspect_ratio() {
        let ar = thumbnail_aspect_ratio(&sample());
        assert!((ar - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_export_size() {
        assert_eq!(thumbnail_export_size(&sample()), 32);
    }

    #[test]
    fn test_square() {
        let t = export_thumbnail_stub(8, 8, 0, 0, 0, 255);
        assert!((thumbnail_aspect_ratio(&t) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_zero_height() {
        let t = ThumbnailExport { width: 4, height: 0, pixels: vec![] };
        assert!((thumbnail_aspect_ratio(&t)).abs() < 1e-5);
    }
}
