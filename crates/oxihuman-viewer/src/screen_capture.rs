// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Screen capture utility for OxiHuman viewer.
//!
//! Provides stubs for capturing the current viewport as a pixel buffer and
//! converting it to various image formats.  Actual GPU readback is a Phase 2
//! task; here the buffer is filled with zeroes and size estimates are computed.

// ── Types ─────────────────────────────────────────────────────────────────────

/// Image format used when saving a screen capture.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum CaptureFormat {
    /// Lossless PNG.
    Png,
    /// Lossy JPEG.
    Jpeg,
    /// Uncompressed BMP.
    Bmp,
}

/// Configuration for a screen capture operation.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ScreenCaptureConfig {
    /// Capture width in pixels.
    pub width: u32,
    /// Capture height in pixels.
    pub height: u32,
    /// Output image format.
    pub format: CaptureFormat,
    /// Whether to include an alpha channel.
    pub include_alpha: bool,
}

/// Raw pixel data from a captured frame.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CaptureBuffer {
    /// Pixel bytes in row-major order.  Each pixel is `channels` bytes wide.
    pub pixels: Vec<u8>,
    /// Buffer width in pixels.
    pub width: u32,
    /// Buffer height in pixels.
    pub height: u32,
    /// Number of channels per pixel (e.g., 3 = RGB, 4 = RGBA).
    pub channels: u32,
}

/// Result produced by [`capture_stub`].
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CaptureResult {
    /// Captured pixel buffer.
    pub buffer: CaptureBuffer,
    /// Format of the encoded image.
    pub format: CaptureFormat,
    /// Byte size of the pixel buffer (not the compressed file).
    pub byte_size: usize,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Return a default [`ScreenCaptureConfig`] for the given dimensions.
#[allow(dead_code)]
pub fn default_screen_capture_config(w: u32, h: u32) -> ScreenCaptureConfig {
    ScreenCaptureConfig {
        width: w,
        height: h,
        format: CaptureFormat::Png,
        include_alpha: false,
    }
}

/// Allocate a zeroed [`CaptureBuffer`] for the given dimensions and channel count.
#[allow(dead_code)]
pub fn new_capture_buffer(w: u32, h: u32, channels: u32) -> CaptureBuffer {
    let size = w as usize * h as usize * channels as usize;
    CaptureBuffer {
        pixels: vec![0u8; size],
        width: w,
        height: h,
        channels,
    }
}

/// Stub capture — returns a zero-filled buffer with size metadata.
///
/// No GPU readback is performed.
#[allow(dead_code)]
pub fn capture_stub(cfg: &ScreenCaptureConfig) -> CaptureResult {
    let channels = if cfg.include_alpha { 4 } else { 3 };
    let buffer = new_capture_buffer(cfg.width, cfg.height, channels);
    let byte_size = buffer.pixels.len();
    CaptureResult {
        buffer,
        format: cfg.format.clone(),
        byte_size,
    }
}

/// Return the slice of bytes representing the pixel at `(x, y)`.
///
/// Panics in debug if the coordinates are out of bounds.
#[allow(dead_code)]
pub fn capture_pixel(buf: &CaptureBuffer, x: u32, y: u32) -> &[u8] {
    let ch = buf.channels as usize;
    let idx = (y as usize * buf.width as usize + x as usize) * ch;
    &buf.pixels[idx..idx + ch]
}

/// Return the canonical name string for a [`CaptureFormat`].
#[allow(dead_code)]
pub fn capture_format_name(fmt: &CaptureFormat) -> &'static str {
    match fmt {
        CaptureFormat::Png => "png",
        CaptureFormat::Jpeg => "jpeg",
        CaptureFormat::Bmp => "bmp",
    }
}

/// Serialise a [`CaptureResult`] to a compact JSON string.
#[allow(dead_code)]
pub fn capture_result_to_json(r: &CaptureResult) -> String {
    format!(
        r#"{{"format":"{}","width":{},"height":{},"channels":{},"byte_size":{}}}"#,
        capture_format_name(&r.format),
        r.buffer.width,
        r.buffer.height,
        r.buffer.channels,
        r.byte_size,
    )
}

/// Convert an RGB or RGBA [`CaptureBuffer`] to an 8-bit grayscale buffer.
///
/// Uses the luminance formula `Y = 0.299·R + 0.587·G + 0.114·B`.
/// If `buf.channels < 3` each pixel is copied directly.
#[allow(dead_code)]
pub fn capture_buffer_to_grayscale(buf: &CaptureBuffer) -> CaptureBuffer {
    let pixel_count = buf.width as usize * buf.height as usize;
    let ch = buf.channels as usize;
    let mut gray = vec![0u8; pixel_count];

    for (i, g) in gray.iter_mut().enumerate() {
        let base = i * ch;
        *g = if ch >= 3 {
            let r = buf.pixels[base] as f32;
            let gv = buf.pixels[base + 1] as f32;
            let b = buf.pixels[base + 2] as f32;
            (0.299 * r + 0.587 * gv + 0.114 * b) as u8
        } else if ch == 1 {
            buf.pixels.get(base).copied().unwrap_or(0)
        } else {
            0
        };
    }

    CaptureBuffer {
        pixels: gray,
        width: buf.width,
        height: buf.height,
        channels: 1,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_fields() {
        let cfg = default_screen_capture_config(1920, 1080);
        assert_eq!(cfg.width, 1920);
        assert_eq!(cfg.height, 1080);
        assert_eq!(cfg.format, CaptureFormat::Png);
        assert!(!cfg.include_alpha);
    }

    #[test]
    fn new_capture_buffer_size() {
        let buf = new_capture_buffer(10, 10, 3);
        assert_eq!(buf.pixels.len(), 300);
        assert!(buf.pixels.iter().all(|&b| b == 0));
    }

    #[test]
    fn capture_stub_rgb() {
        let cfg = default_screen_capture_config(4, 4);
        let r = capture_stub(&cfg);
        assert_eq!(r.buffer.channels, 3);
        assert_eq!(r.byte_size, 4 * 4 * 3);
    }

    #[test]
    fn capture_stub_rgba() {
        let cfg = ScreenCaptureConfig {
            width: 2,
            height: 2,
            format: CaptureFormat::Png,
            include_alpha: true,
        };
        let r = capture_stub(&cfg);
        assert_eq!(r.buffer.channels, 4);
        assert_eq!(r.byte_size, 16);
    }

    #[test]
    fn capture_pixel_returns_correct_slice() {
        let mut buf = new_capture_buffer(4, 4, 3);
        // Write a known pixel at (1, 0)
        buf.pixels[3] = 10;
        buf.pixels[4] = 20;
        buf.pixels[5] = 30;
        let px = capture_pixel(&buf, 1, 0);
        assert_eq!(px, &[10, 20, 30]);
    }

    #[test]
    fn capture_format_names() {
        assert_eq!(capture_format_name(&CaptureFormat::Png), "png");
        assert_eq!(capture_format_name(&CaptureFormat::Jpeg), "jpeg");
        assert_eq!(capture_format_name(&CaptureFormat::Bmp), "bmp");
    }

    #[test]
    fn result_to_json_contains_fields() {
        let cfg = default_screen_capture_config(64, 64);
        let r = capture_stub(&cfg);
        let json = capture_result_to_json(&r);
        assert!(json.contains("png"));
        assert!(json.contains("64"));
        assert!(json.contains("byte_size"));
    }

    #[test]
    fn grayscale_conversion_dimensions() {
        let buf = new_capture_buffer(8, 8, 3);
        let gray = capture_buffer_to_grayscale(&buf);
        assert_eq!(gray.channels, 1);
        assert_eq!(gray.width, 8);
        assert_eq!(gray.height, 8);
        assert_eq!(gray.pixels.len(), 64);
    }

    #[test]
    fn grayscale_luminance_correct() {
        let mut buf = new_capture_buffer(1, 1, 3);
        // Pure red = (255, 0, 0) → Y ≈ 76
        buf.pixels[0] = 255;
        buf.pixels[1] = 0;
        buf.pixels[2] = 0;
        let gray = capture_buffer_to_grayscale(&buf);
        // 0.299 * 255 ≈ 76.2
        assert!((gray.pixels[0] as i32 - 76).abs() <= 1);
    }

    #[test]
    fn grayscale_single_channel_passthrough() {
        let mut buf = new_capture_buffer(2, 2, 1);
        buf.pixels[0] = 200;
        let gray = capture_buffer_to_grayscale(&buf);
        assert_eq!(gray.pixels[0], 200);
    }
}
