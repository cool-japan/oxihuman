// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Depth image export (16-bit PNG stub).

/// 16-bit depth image buffer.
#[allow(dead_code)]
pub struct DepthImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u16>,
    pub min_depth: f32,
    pub max_depth: f32,
}

/// Create a new depth image.
#[allow(dead_code)]
pub fn new_depth_image(width: u32, height: u32, min_depth: f32, max_depth: f32) -> DepthImage {
    DepthImage {
        width,
        height,
        pixels: vec![0u16; (width * height) as usize],
        min_depth,
        max_depth,
    }
}

/// Set a pixel value (float depth -> u16).
#[allow(dead_code)]
pub fn set_depth_pixel(img: &mut DepthImage, x: u32, y: u32, depth: f32) {
    let idx = (y * img.width + x) as usize;
    if idx < img.pixels.len() {
        let normalized = ((depth - img.min_depth) / (img.max_depth - img.min_depth)).clamp(0.0, 1.0);
        img.pixels[idx] = (normalized * 65535.0) as u16;
    }
}

/// Get depth float from pixel.
#[allow(dead_code)]
pub fn get_depth_float(img: &DepthImage, x: u32, y: u32) -> f32 {
    let idx = (y * img.width + x) as usize;
    if idx >= img.pixels.len() { return 0.0; }
    let normalized = img.pixels[idx] as f32 / 65535.0;
    img.min_depth + normalized * (img.max_depth - img.min_depth)
}

/// Pixel count.
#[allow(dead_code)]
pub fn depth_image_pixel_count(img: &DepthImage) -> usize {
    (img.width * img.height) as usize
}

/// Average depth value (float).
#[allow(dead_code)]
pub fn depth_image_avg(img: &DepthImage) -> f32 {
    if img.pixels.is_empty() { return 0.0; }
    let sum: u64 = img.pixels.iter().map(|&p| p as u64).sum();
    let normalized = sum as f32 / (img.pixels.len() as f32 * 65535.0);
    img.min_depth + normalized * (img.max_depth - img.min_depth)
}

/// Build a PNG stub header (8-byte PNG signature + IHDR-like stub).
#[allow(dead_code)]
pub fn build_png_stub_header(img: &DepthImage) -> Vec<u8> {
    let mut buf = vec![137u8, 80, 78, 71, 13, 10, 26, 10];
    buf.extend_from_slice(&img.width.to_be_bytes());
    buf.extend_from_slice(&img.height.to_be_bytes());
    buf.push(16);
    buf
}

/// Validate depth image.
#[allow(dead_code)]
pub fn validate_depth_image(img: &DepthImage) -> bool {
    img.pixels.len() == (img.width * img.height) as usize
        && img.max_depth > img.min_depth
}

/// Export raw 16-bit pixels as big-endian bytes.
#[allow(dead_code)]
pub fn export_depth_raw_bytes(img: &DepthImage) -> Vec<u8> {
    let mut buf = Vec::with_capacity(img.pixels.len() * 2);
    for &p in &img.pixels {
        buf.extend_from_slice(&p.to_be_bytes());
    }
    buf
}

/// Normalize depth image (remap to full u16 range).
#[allow(dead_code)]
pub fn normalize_depth_image(img: &mut DepthImage) {
    let mn = *img.pixels.iter().min().unwrap_or(&0);
    let mx = *img.pixels.iter().max().unwrap_or(&0);
    if mx == mn { return; }
    let range = (mx - mn) as u32;
    for p in img.pixels.iter_mut() {
        *p = ((*p - mn) as u32 * 65535 / range) as u16;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_image_zero_pixels() {
        let img = new_depth_image(4, 4, 0.0, 10.0);
        assert!(img.pixels.iter().all(|&p| p == 0));
    }

    #[test]
    fn pixel_count_correct() {
        let img = new_depth_image(8, 6, 0.0, 5.0);
        assert_eq!(depth_image_pixel_count(&img), 48);
    }

    #[test]
    fn set_and_get_depth() {
        let mut img = new_depth_image(4, 4, 0.0, 10.0);
        set_depth_pixel(&mut img, 0, 0, 5.0);
        let d = get_depth_float(&img, 0, 0);
        assert!((d - 5.0).abs() < 0.01);
    }

    #[test]
    fn validate_passes() {
        let img = new_depth_image(4, 4, 0.0, 5.0);
        assert!(validate_depth_image(&img));
    }

    #[test]
    fn validate_fails_wrong_range() {
        let img = new_depth_image(4, 4, 5.0, 0.0);
        assert!(!validate_depth_image(&img));
    }

    #[test]
    fn png_header_starts_with_signature() {
        let img = new_depth_image(4, 4, 0.0, 5.0);
        let h = build_png_stub_header(&img);
        assert_eq!(&h[..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
    }

    #[test]
    fn raw_bytes_length() {
        let img = new_depth_image(4, 4, 0.0, 5.0);
        let b = export_depth_raw_bytes(&img);
        assert_eq!(b.len(), 32);
    }

    #[test]
    fn normalize_increases_range() {
        let mut img = new_depth_image(2, 1, 0.0, 10.0);
        img.pixels[0] = 100;
        img.pixels[1] = 200;
        normalize_depth_image(&mut img);
        assert_eq!(img.pixels[1], 65535);
    }

    #[test]
    fn avg_depth_all_zero() {
        let img = new_depth_image(4, 4, 0.0, 10.0);
        assert_eq!(depth_image_avg(&img), 0.0);
    }
}
