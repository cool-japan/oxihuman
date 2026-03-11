// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Texture data export to PPM and raw byte formats.
//!
//! Supports RGB8, RGBA8, and GrayF32 pixel formats with basic
//! resize (nearest-neighbour), vertical flip, and gamma correction.

// ── Types ─────────────────────────────────────────────────────────────────────

/// Pixel format carried by a [`TextureBuffer`].
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum PixelFormat {
    /// 3 bytes per pixel: red, green, blue (0–255).
    Rgb8,
    /// 4 bytes per pixel: red, green, blue, alpha (0–255).
    Rgba8,
    /// 4 bytes per pixel: single 32-bit float luminance.
    GrayF32,
}

/// Configuration for texture export operations.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TextureExportConfig {
    /// Gamma value applied during export (1.0 = no change).
    pub gamma: f32,
    /// Pixel format to target when encoding.
    pub format: PixelFormat,
    /// Whether to vertically flip the image before encoding.
    pub flip_on_export: bool,
}

/// A CPU-side texture buffer holding raw pixel data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TextureBuffer {
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    /// Raw pixel bytes laid out in row-major order.
    pub data: Vec<u8>,
}

// ── Type aliases ──────────────────────────────────────────────────────────────

/// An encoded image as raw bytes (PPM, raw, etc.).
pub type EncodedTexture = Vec<u8>;

// ── Config helpers ────────────────────────────────────────────────────────────

/// Create a default [`TextureExportConfig`] with gamma 1.0 and RGB8 format.
#[allow(dead_code)]
pub fn default_texture_config() -> TextureExportConfig {
    TextureExportConfig {
        gamma: 1.0,
        format: PixelFormat::Rgb8,
        flip_on_export: false,
    }
}

// ── Buffer construction ───────────────────────────────────────────────────────

/// Create a new zeroed [`TextureBuffer`] for the given dimensions and format.
#[allow(dead_code)]
pub fn new_texture_buffer(width: u32, height: u32, format: PixelFormat) -> TextureBuffer {
    let bytes_per_pixel = match &format {
        PixelFormat::Rgb8 => 3,
        PixelFormat::Rgba8 => 4,
        PixelFormat::GrayF32 => 4,
    };
    let size = (width * height) as usize * bytes_per_pixel;
    TextureBuffer {
        width,
        height,
        format,
        data: vec![0u8; size],
    }
}

// ── Pixel access (RGB8) ───────────────────────────────────────────────────────

/// Set an RGB pixel at `(x, y)`.  No-op if out of bounds or format is not RGB8.
#[allow(dead_code)]
pub fn set_pixel_rgb(buf: &mut TextureBuffer, x: u32, y: u32, r: u8, g: u8, b: u8) {
    if buf.format != PixelFormat::Rgb8 {
        return;
    }
    if x >= buf.width || y >= buf.height {
        return;
    }
    let idx = ((y * buf.width + x) * 3) as usize;
    if idx + 2 < buf.data.len() {
        buf.data[idx] = r;
        buf.data[idx + 1] = g;
        buf.data[idx + 2] = b;
    }
}

/// Get the RGB value at `(x, y)`.  Returns `(0, 0, 0)` on out-of-bounds or
/// if the format is not RGB8.
#[allow(dead_code)]
pub fn get_pixel_rgb(buf: &TextureBuffer, x: u32, y: u32) -> (u8, u8, u8) {
    if buf.format != PixelFormat::Rgb8 {
        return (0, 0, 0);
    }
    if x >= buf.width || y >= buf.height {
        return (0, 0, 0);
    }
    let idx = ((y * buf.width + x) * 3) as usize;
    if idx + 2 < buf.data.len() {
        (buf.data[idx], buf.data[idx + 1], buf.data[idx + 2])
    } else {
        (0, 0, 0)
    }
}

// ── Queries ───────────────────────────────────────────────────────────────────

/// Return the width of a texture buffer.
#[allow(dead_code)]
pub fn texture_width(buf: &TextureBuffer) -> u32 {
    buf.width
}

/// Return the height of a texture buffer.
#[allow(dead_code)]
pub fn texture_height(buf: &TextureBuffer) -> u32 {
    buf.height
}

/// Return the total number of pixels.
#[allow(dead_code)]
pub fn pixel_count(buf: &TextureBuffer) -> usize {
    (buf.width * buf.height) as usize
}

// ── Encoding ──────────────────────────────────────────────────────────────────

/// Encode a RGB8 buffer to PPM P6 bytes.
///
/// If the buffer is not RGB8, returns an empty `Vec`.
#[allow(dead_code)]
pub fn encode_ppm_rgb(buf: &TextureBuffer) -> EncodedTexture {
    if buf.format != PixelFormat::Rgb8 {
        return Vec::new();
    }
    let header = format!("P6\n{} {}\n255\n", buf.width, buf.height);
    let mut out = header.into_bytes();
    out.extend_from_slice(&buf.data);
    out
}

/// Encode a GrayF32 buffer to PPM P5 (greyscale) bytes.
///
/// Float values are clamped to `[0.0, 1.0]` and mapped to `0–255`.
/// If the buffer is not GrayF32, returns an empty `Vec`.
#[allow(dead_code)]
pub fn encode_ppm_gray(buf: &TextureBuffer) -> EncodedTexture {
    if buf.format != PixelFormat::GrayF32 {
        return Vec::new();
    }
    let header = format!("P5\n{} {}\n255\n", buf.width, buf.height);
    let mut out = header.into_bytes();
    let n = (buf.width * buf.height) as usize;
    for i in 0..n {
        let offset = i * 4;
        if offset + 3 < buf.data.len() {
            let bytes: [u8; 4] = [
                buf.data[offset],
                buf.data[offset + 1],
                buf.data[offset + 2],
                buf.data[offset + 3],
            ];
            let f = f32::from_le_bytes(bytes).clamp(0.0, 1.0);
            out.push((f * 255.0) as u8);
        } else {
            out.push(0);
        }
    }
    out
}

// ── Raw byte conversion ───────────────────────────────────────────────────────

/// Return a copy of the raw pixel bytes of a buffer.
#[allow(dead_code)]
pub fn texture_to_raw_bytes(buf: &TextureBuffer) -> Vec<u8> {
    buf.data.clone()
}

/// Reconstruct a [`TextureBuffer`] from raw bytes, width, height, and format.
///
/// Returns `None` if `bytes.len()` does not match the expected size.
#[allow(dead_code)]
pub fn raw_bytes_to_texture(
    bytes: &[u8],
    width: u32,
    height: u32,
    format: PixelFormat,
) -> Option<TextureBuffer> {
    let bpp = match &format {
        PixelFormat::Rgb8 => 3usize,
        PixelFormat::Rgba8 => 4,
        PixelFormat::GrayF32 => 4,
    };
    let expected = (width * height) as usize * bpp;
    if bytes.len() != expected {
        return None;
    }
    Some(TextureBuffer {
        width,
        height,
        format,
        data: bytes.to_vec(),
    })
}

// ── Transforms ────────────────────────────────────────────────────────────────

/// Resize a texture buffer using nearest-neighbour sampling.
///
/// Works for RGB8 and RGBA8. Returns the original buffer unchanged for
/// GrayF32 (which would need f32-aware sampling).
#[allow(dead_code)]
pub fn resize_texture(buf: &TextureBuffer, new_w: u32, new_h: u32) -> TextureBuffer {
    let bpp: usize = match &buf.format {
        PixelFormat::Rgb8 => 3,
        PixelFormat::Rgba8 => 4,
        PixelFormat::GrayF32 => {
            // Return unchanged for float format
            return buf.clone();
        }
    };
    let mut out = vec![0u8; (new_w * new_h) as usize * bpp];
    for ny in 0..new_h {
        for nx in 0..new_w {
            let src_x = (nx * buf.width / new_w.max(1)).min(buf.width.saturating_sub(1));
            let src_y = (ny * buf.height / new_h.max(1)).min(buf.height.saturating_sub(1));
            let src_idx = ((src_y * buf.width + src_x) as usize) * bpp;
            let dst_idx = ((ny * new_w + nx) as usize) * bpp;
            if src_idx + bpp <= buf.data.len() && dst_idx + bpp <= out.len() {
                out[dst_idx..dst_idx + bpp].copy_from_slice(&buf.data[src_idx..src_idx + bpp]);
            }
        }
    }
    TextureBuffer {
        width: new_w,
        height: new_h,
        format: buf.format.clone(),
        data: out,
    }
}

/// Flip a texture buffer vertically (top ↔ bottom) in place.
#[allow(dead_code)]
pub fn flip_vertical(buf: &mut TextureBuffer) {
    let bpp: usize = match &buf.format {
        PixelFormat::Rgb8 => 3,
        PixelFormat::Rgba8 => 4,
        PixelFormat::GrayF32 => 4,
    };
    let row_bytes = buf.width as usize * bpp;
    let h = buf.height as usize;
    for y in 0..(h / 2) {
        let top = y * row_bytes;
        let bot = (h - 1 - y) * row_bytes;
        for x in 0..row_bytes {
            buf.data.swap(top + x, bot + x);
        }
    }
}

/// Apply gamma correction to an RGB8 buffer in place.
///
/// Each channel is raised to the power `1.0 / gamma`. A gamma of 1.0 is a no-op.
#[allow(dead_code)]
pub fn apply_texture_gamma(buf: &mut TextureBuffer, gamma: f32) {
    if buf.format != PixelFormat::Rgb8 {
        return;
    }
    if (gamma - 1.0).abs() < 1e-6 {
        return;
    }
    let inv = 1.0 / gamma.max(0.01);
    for byte in buf.data.iter_mut() {
        let f = (*byte as f32 / 255.0).powf(inv);
        *byte = (f.clamp(0.0, 1.0) * 255.0) as u8;
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_gamma_one() {
        let cfg = default_texture_config();
        assert!((cfg.gamma - 1.0).abs() < 1e-6);
        assert_eq!(cfg.format, PixelFormat::Rgb8);
    }

    #[test]
    fn new_buffer_correct_size_rgb8() {
        let buf = new_texture_buffer(4, 4, PixelFormat::Rgb8);
        assert_eq!(buf.data.len(), 4 * 4 * 3);
    }

    #[test]
    fn new_buffer_correct_size_rgba8() {
        let buf = new_texture_buffer(2, 3, PixelFormat::Rgba8);
        assert_eq!(buf.data.len(), 2 * 3 * 4);
    }

    #[test]
    fn set_and_get_pixel_rgb() {
        let mut buf = new_texture_buffer(8, 8, PixelFormat::Rgb8);
        set_pixel_rgb(&mut buf, 3, 2, 100, 150, 200);
        let (r, g, b) = get_pixel_rgb(&buf, 3, 2);
        assert_eq!((r, g, b), (100, 150, 200));
    }

    #[test]
    fn get_pixel_out_of_bounds_returns_zero() {
        let buf = new_texture_buffer(4, 4, PixelFormat::Rgb8);
        assert_eq!(get_pixel_rgb(&buf, 100, 100), (0, 0, 0));
    }

    #[test]
    fn texture_width_and_height() {
        let buf = new_texture_buffer(16, 32, PixelFormat::Rgb8);
        assert_eq!(texture_width(&buf), 16);
        assert_eq!(texture_height(&buf), 32);
    }

    #[test]
    fn pixel_count_correct() {
        let buf = new_texture_buffer(10, 5, PixelFormat::Rgb8);
        assert_eq!(pixel_count(&buf), 50);
    }

    #[test]
    fn encode_ppm_rgb_starts_with_header() {
        let mut buf = new_texture_buffer(2, 2, PixelFormat::Rgb8);
        set_pixel_rgb(&mut buf, 0, 0, 255, 0, 0);
        let ppm = encode_ppm_rgb(&buf);
        let header = "P6\n2 2\n255\n";
        assert!(ppm.starts_with(header.as_bytes()));
    }

    #[test]
    fn encode_ppm_rgb_wrong_format_empty() {
        let buf = new_texture_buffer(2, 2, PixelFormat::Rgba8);
        assert!(encode_ppm_rgb(&buf).is_empty());
    }

    #[test]
    fn encode_ppm_gray_starts_with_header() {
        let buf = new_texture_buffer(2, 2, PixelFormat::GrayF32);
        let ppm = encode_ppm_gray(&buf);
        let header = "P5\n2 2\n255\n";
        assert!(ppm.starts_with(header.as_bytes()));
    }

    #[test]
    fn encode_ppm_gray_wrong_format_empty() {
        let buf = new_texture_buffer(2, 2, PixelFormat::Rgb8);
        assert!(encode_ppm_gray(&buf).is_empty());
    }

    #[test]
    fn texture_to_raw_bytes_round_trip() {
        let mut buf = new_texture_buffer(4, 4, PixelFormat::Rgb8);
        set_pixel_rgb(&mut buf, 1, 1, 50, 100, 150);
        let raw = texture_to_raw_bytes(&buf);
        let restored = raw_bytes_to_texture(&raw, 4, 4, PixelFormat::Rgb8).unwrap();
        let (r, g, b) = get_pixel_rgb(&restored, 1, 1);
        assert_eq!((r, g, b), (50, 100, 150));
    }

    #[test]
    fn raw_bytes_to_texture_wrong_size_returns_none() {
        let bad = vec![0u8; 10];
        assert!(raw_bytes_to_texture(&bad, 4, 4, PixelFormat::Rgb8).is_none());
    }

    #[test]
    fn resize_texture_changes_dimensions() {
        let buf = new_texture_buffer(8, 8, PixelFormat::Rgb8);
        let resized = resize_texture(&buf, 4, 4);
        assert_eq!(texture_width(&resized), 4);
        assert_eq!(texture_height(&resized), 4);
    }

    #[test]
    fn flip_vertical_changes_pixels() {
        let mut buf = new_texture_buffer(2, 2, PixelFormat::Rgb8);
        set_pixel_rgb(&mut buf, 0, 0, 255, 0, 0);
        set_pixel_rgb(&mut buf, 0, 1, 0, 0, 255);
        flip_vertical(&mut buf);
        let (r, _, _) = get_pixel_rgb(&buf, 0, 0);
        assert_eq!(r, 0, "top row should now be blue (r=0)");
        let (r2, _, _) = get_pixel_rgb(&buf, 0, 1);
        assert_eq!(r2, 255, "bottom row should now be red (r=255)");
    }

    #[test]
    fn apply_texture_gamma_noop_at_one() {
        let mut buf = new_texture_buffer(2, 2, PixelFormat::Rgb8);
        set_pixel_rgb(&mut buf, 0, 0, 128, 64, 32);
        apply_texture_gamma(&mut buf, 1.0);
        let (r, g, b) = get_pixel_rgb(&buf, 0, 0);
        assert_eq!((r, g, b), (128, 64, 32));
    }

    #[test]
    fn apply_texture_gamma_changes_values() {
        let mut buf = new_texture_buffer(1, 1, PixelFormat::Rgb8);
        set_pixel_rgb(&mut buf, 0, 0, 128, 128, 128);
        apply_texture_gamma(&mut buf, 2.2);
        let (r, _, _) = get_pixel_rgb(&buf, 0, 0);
        // After gamma correction the value should increase (brightening)
        assert!(r > 128, "gamma < 1 exponent should brighten");
    }
}
