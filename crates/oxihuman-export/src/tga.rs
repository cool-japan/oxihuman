// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

use std::fs;
use std::io::Read;
use std::path::Path;

/// A simple RGBA image buffer for TGA export.
pub struct TgaImage {
    pub pixels: Vec<u8>, // raw RGBA bytes (4 bytes per pixel)
    pub width: usize,
    pub height: usize,
}

impl TgaImage {
    /// Create a new image filled with a solid color.
    pub fn new(width: usize, height: usize, fill: [u8; 4]) -> Self {
        let mut pixels = Vec::with_capacity(width * height * 4);
        for _ in 0..(width * height) {
            pixels.extend_from_slice(&fill);
        }
        Self {
            pixels,
            width,
            height,
        }
    }

    /// Build from float RGB pixels (values in [0.0, 1.0]).
    pub fn from_rgb_f32(pixels: &[[f32; 3]], width: usize, height: usize) -> Self {
        let mut rgba = Vec::with_capacity(width * height * 4);
        for p in pixels {
            let r = (p[0] * 255.0).clamp(0.0, 255.0) as u8;
            let g = (p[1] * 255.0).clamp(0.0, 255.0) as u8;
            let b = (p[2] * 255.0).clamp(0.0, 255.0) as u8;
            rgba.extend_from_slice(&[r, g, b, 255]);
        }
        Self {
            pixels: rgba,
            width,
            height,
        }
    }

    /// Build from float RGBA pixels (values in [0.0, 1.0]).
    pub fn from_rgba_f32(pixels: &[[f32; 4]], width: usize, height: usize) -> Self {
        let mut rgba = Vec::with_capacity(width * height * 4);
        for p in pixels {
            let r = (p[0] * 255.0).clamp(0.0, 255.0) as u8;
            let g = (p[1] * 255.0).clamp(0.0, 255.0) as u8;
            let b = (p[2] * 255.0).clamp(0.0, 255.0) as u8;
            let a = (p[3] * 255.0).clamp(0.0, 255.0) as u8;
            rgba.extend_from_slice(&[r, g, b, a]);
        }
        Self {
            pixels: rgba,
            width,
            height,
        }
    }

    /// Get a pixel at (x, y).
    pub fn get_pixel(&self, x: usize, y: usize) -> [u8; 4] {
        let i = 4 * (y * self.width + x);
        [
            self.pixels[i],
            self.pixels[i + 1],
            self.pixels[i + 2],
            self.pixels[i + 3],
        ]
    }

    /// Set a pixel at (x, y).
    pub fn set_pixel(&mut self, x: usize, y: usize, rgba: [u8; 4]) {
        let i = 4 * (y * self.width + x);
        self.pixels[i..i + 4].copy_from_slice(&rgba);
    }

    /// Return total number of pixels.
    pub fn pixel_count(&self) -> usize {
        self.width * self.height
    }

    /// Convert to TGA bytes (RGB 24-bit, bottom-left origin).
    pub fn to_tga_bytes_rgb(&self) -> Vec<u8> {
        let w = self.width as u16;
        let h = self.height as u16;
        let mut out = Vec::with_capacity(18 + self.width * self.height * 3);

        // 18-byte header
        out.extend_from_slice(&[
            0, // [0]  ID length
            0, // [1]  color map type
            2, // [2]  image type: uncompressed true-color
            0, 0, // [3-4]  color map origin
            0, 0, // [5-6]  color map length
            0, // [7]  color map entry size
            0, 0, // [8-9]  x origin
            0, 0, // [10-11] y origin
        ]);
        out.extend_from_slice(&w.to_le_bytes()); // [12-13] width
        out.extend_from_slice(&h.to_le_bytes()); // [14-15] height
        out.push(24); // [16] bits per pixel
        out.push(0); // [17] image descriptor: bottom-left origin

        // Pixel data — bottom row first (TGA default)
        for row in (0..self.height).rev() {
            for col in 0..self.width {
                let px = self.get_pixel(col, row);
                out.push(px[2]); // B
                out.push(px[1]); // G
                out.push(px[0]); // R
            }
        }
        out
    }

    /// Convert to TGA bytes (RGBA 32-bit, bottom-left origin).
    pub fn to_tga_bytes_rgba(&self) -> Vec<u8> {
        let w = self.width as u16;
        let h = self.height as u16;
        let mut out = Vec::with_capacity(18 + self.width * self.height * 4);

        // 18-byte header
        out.extend_from_slice(&[
            0, // [0]  ID length
            0, // [1]  color map type
            2, // [2]  image type: uncompressed true-color
            0, 0, // [3-4]  color map origin
            0, 0, // [5-6]  color map length
            0, // [7]  color map entry size
            0, 0, // [8-9]  x origin
            0, 0, // [10-11] y origin
        ]);
        out.extend_from_slice(&w.to_le_bytes()); // [12-13] width
        out.extend_from_slice(&h.to_le_bytes()); // [14-15] height
        out.push(32); // [16] bits per pixel
        out.push(8); // [17] image descriptor: 8 bits of alpha, bottom-left origin

        // Pixel data — bottom row first
        for row in (0..self.height).rev() {
            for col in 0..self.width {
                let px = self.get_pixel(col, row);
                out.push(px[2]); // B
                out.push(px[1]); // G
                out.push(px[0]); // R
                out.push(px[3]); // A
            }
        }
        out
    }

    /// Create a solid color image.
    pub fn solid_color(width: usize, height: usize, color: [u8; 4]) -> Self {
        Self::new(width, height, color)
    }

    /// Create a horizontal gradient image (lerp from→to along x axis).
    pub fn gradient(width: usize, height: usize, from: [u8; 4], to: [u8; 4]) -> Self {
        let mut img = Self::new(width, height, [0, 0, 0, 255]);
        for y in 0..height {
            for x in 0..width {
                let t = if width <= 1 {
                    0.0f32
                } else {
                    x as f32 / (width - 1) as f32
                };
                let r = (from[0] as f32 + t * (to[0] as f32 - from[0] as f32)) as u8;
                let g = (from[1] as f32 + t * (to[1] as f32 - from[1] as f32)) as u8;
                let b = (from[2] as f32 + t * (to[2] as f32 - from[2] as f32)) as u8;
                let a = (from[3] as f32 + t * (to[3] as f32 - from[3] as f32)) as u8;
                img.set_pixel(x, y, [r, g, b, a]);
            }
        }
        img
    }

    /// Create a checkerboard pattern.
    pub fn checkerboard(width: usize, height: usize, size: usize, a: [u8; 4], b: [u8; 4]) -> Self {
        let mut img = Self::new(width, height, [0, 0, 0, 255]);
        let tile = size.max(1);
        for y in 0..height {
            for x in 0..width {
                let color = if ((x / tile) + (y / tile)).is_multiple_of(2) {
                    a
                } else {
                    b
                };
                img.set_pixel(x, y, color);
            }
        }
        img
    }
}

// ---------------------------------------------------------------------------
// File I/O helpers
// ---------------------------------------------------------------------------

/// Write a TGA file (RGB 24-bit).
pub fn export_tga_rgb(image: &TgaImage, path: &Path) -> anyhow::Result<()> {
    let data = image.to_tga_bytes_rgb();
    fs::write(path, data)?;
    Ok(())
}

/// Write a TGA file (RGBA 32-bit).
pub fn export_tga_rgba(image: &TgaImage, path: &Path) -> anyhow::Result<()> {
    let data = image.to_tga_bytes_rgba();
    fs::write(path, data)?;
    Ok(())
}

/// Export float RGB pixels as a TGA file.
pub fn export_float_rgb_tga(
    pixels: &[[f32; 3]],
    width: usize,
    height: usize,
    path: &Path,
) -> anyhow::Result<()> {
    let image = TgaImage::from_rgb_f32(pixels, width, height);
    export_tga_rgb(&image, path)
}

/// Export float RGBA pixels as a TGA file.
pub fn export_float_rgba_tga(
    pixels: &[[f32; 4]],
    width: usize,
    height: usize,
    path: &Path,
) -> anyhow::Result<()> {
    let image = TgaImage::from_rgba_f32(pixels, width, height);
    export_tga_rgba(&image, path)
}

/// Read only the header of a TGA file. Returns (width, height, bits_per_pixel).
pub fn read_tga_header(path: &Path) -> anyhow::Result<(usize, usize, u8)> {
    let mut file = fs::File::open(path)?;
    let mut header = [0u8; 18];
    file.read_exact(&mut header)?;
    let width = u16::from_le_bytes([header[12], header[13]]) as usize;
    let height = u16::from_le_bytes([header[14], header[15]]) as usize;
    let bpp = header[16];
    Ok((width, height, bpp))
}

/// Validate that a file is an uncompressed TGA (image type byte == 2).
pub fn validate_tga(path: &Path) -> anyhow::Result<bool> {
    let mut file = fs::File::open(path)?;
    let mut header = [0u8; 18];
    file.read_exact(&mut header)?;
    Ok(header[2] == 2)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn tmp(name: &str) -> PathBuf {
        PathBuf::from(format!("/tmp/{name}"))
    }

    #[test]
    fn test_tga_image_new() {
        let img = TgaImage::new(4, 4, [255, 0, 0, 255]);
        assert_eq!(img.width, 4);
        assert_eq!(img.height, 4);
        assert_eq!(img.pixels.len(), 4 * 4 * 4);
        assert_eq!(&img.pixels[0..4], &[255, 0, 0, 255]);
    }

    #[test]
    fn test_tga_image_get_set_pixel() {
        let mut img = TgaImage::new(8, 8, [0, 0, 0, 255]);
        img.set_pixel(3, 5, [10, 20, 30, 40]);
        let px = img.get_pixel(3, 5);
        assert_eq!(px, [10, 20, 30, 40]);
    }

    #[test]
    fn test_tga_to_bytes_rgb_header() {
        let img = TgaImage::new(16, 32, [0, 0, 0, 255]);
        let bytes = img.to_tga_bytes_rgb();
        assert!(bytes.len() >= 18);
        assert_eq!(bytes[0], 0); // ID length
        assert_eq!(bytes[1], 0); // color map type
        assert_eq!(bytes[2], 2); // image type: uncompressed
                                 // width
        let w = u16::from_le_bytes([bytes[12], bytes[13]]);
        assert_eq!(w, 16);
        // height
        let h = u16::from_le_bytes([bytes[14], bytes[15]]);
        assert_eq!(h, 32);
        assert_eq!(bytes[16], 24); // bpp
        assert_eq!(bytes[17], 0); // descriptor
        assert_eq!(bytes.len(), 18 + 16 * 32 * 3);
    }

    #[test]
    fn test_tga_to_bytes_rgba_header() {
        let img = TgaImage::new(8, 8, [0, 0, 0, 255]);
        let bytes = img.to_tga_bytes_rgba();
        assert_eq!(bytes[2], 2); // image type
        assert_eq!(bytes[16], 32); // bpp
        assert_eq!(bytes[17], 8); // descriptor (alpha depth)
        assert_eq!(bytes.len(), 18 + 8 * 8 * 4);
    }

    #[test]
    fn test_tga_pixel_order_bgr() {
        // Single red pixel => TGA RGB should have B=0, G=0, R=255
        let mut img = TgaImage::new(1, 1, [0, 0, 0, 255]);
        img.set_pixel(0, 0, [255, 0, 0, 255]); // red
        let bytes = img.to_tga_bytes_rgb();
        // Pixel starts at offset 18
        assert_eq!(bytes[18], 0); // B
        assert_eq!(bytes[19], 0); // G
        assert_eq!(bytes[20], 255); // R
    }

    #[test]
    fn test_solid_color() {
        let color = [128u8, 64, 32, 200];
        let img = TgaImage::solid_color(10, 10, color);
        for y in 0..10 {
            for x in 0..10 {
                assert_eq!(img.get_pixel(x, y), color);
            }
        }
    }

    #[test]
    fn test_gradient() {
        let from = [0u8, 0, 0, 255];
        let to = [255u8, 0, 0, 255];
        let img = TgaImage::gradient(11, 1, from, to);
        // First pixel should match 'from'
        assert_eq!(img.get_pixel(0, 0)[0], 0);
        // Last pixel should match 'to'
        assert_eq!(img.get_pixel(10, 0)[0], 255);
    }

    #[test]
    fn test_checkerboard() {
        let a = [255u8, 255, 255, 255];
        let b = [0u8, 0, 0, 255];
        let img = TgaImage::checkerboard(4, 4, 2, a, b);
        // (0,0) in tile 0+0=0 => color a
        assert_eq!(img.get_pixel(0, 0), a);
        // (2,0) in tile 1+0=1 => color b
        assert_eq!(img.get_pixel(2, 0), b);
        // (2,2) in tile 1+1=2 => color a
        assert_eq!(img.get_pixel(2, 2), a);
    }

    #[test]
    fn test_from_rgb_f32() {
        let pixels = vec![[1.0f32, 0.5, 0.0]];
        let img = TgaImage::from_rgb_f32(&pixels, 1, 1);
        let px = img.get_pixel(0, 0);
        assert_eq!(px[0], 255);
        assert!((px[1] as i32 - 127).abs() <= 1);
        assert_eq!(px[2], 0);
        assert_eq!(px[3], 255); // alpha filled to 255
    }

    #[test]
    fn test_export_tga_rgb() {
        let path = tmp("test_export_tga_rgb.tga");
        let img = TgaImage::solid_color(16, 16, [255, 128, 64, 255]);
        export_tga_rgb(&img, &path).unwrap();
        let data = fs::read(&path).unwrap();
        assert_eq!(data.len(), 18 + 16 * 16 * 3);
        assert_eq!(data[2], 2);
        assert_eq!(data[16], 24);
    }

    #[test]
    fn test_export_tga_rgba() {
        let path = tmp("test_export_tga_rgba.tga");
        let img = TgaImage::solid_color(8, 8, [0, 255, 0, 128]);
        export_tga_rgba(&img, &path).unwrap();
        let data = fs::read(&path).unwrap();
        assert_eq!(data.len(), 18 + 8 * 8 * 4);
        assert_eq!(data[16], 32);
    }

    #[test]
    fn test_validate_tga() {
        let path = tmp("test_validate_tga.tga");
        let img = TgaImage::solid_color(4, 4, [255, 0, 0, 255]);
        export_tga_rgb(&img, &path).unwrap();
        let ok = validate_tga(&path).unwrap();
        assert!(ok);
    }

    #[test]
    fn test_read_tga_header() {
        let path = tmp("test_read_tga_header.tga");
        let img = TgaImage::solid_color(32, 64, [0, 0, 255, 255]);
        export_tga_rgb(&img, &path).unwrap();
        let (w, h, bpp) = read_tga_header(&path).unwrap();
        assert_eq!(w, 32);
        assert_eq!(h, 64);
        assert_eq!(bpp, 24);
    }
}
