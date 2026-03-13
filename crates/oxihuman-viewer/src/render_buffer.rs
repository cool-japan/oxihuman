// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// GPU render buffer descriptor.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderBuffer {
    pub width: u32,
    pub height: u32,
    /// Internal format code (0 = RGBA8, 1 = RGBA16F, 2 = Depth24).
    pub format: u8,
    pub samples: u32,
    pub is_depth: bool,
}

/// Create a standard RGBA8 color render buffer.
#[allow(dead_code)]
pub fn new_color_buffer(w: u32, h: u32) -> RenderBuffer {
    RenderBuffer { width: w, height: h, format: 0, samples: 1, is_depth: false }
}

/// Create a depth buffer (Depth24 format).
#[allow(dead_code)]
pub fn new_depth_buffer(w: u32, h: u32) -> RenderBuffer {
    RenderBuffer { width: w, height: h, format: 2, samples: 1, is_depth: true }
}

/// Create an MSAA color buffer.
#[allow(dead_code)]
pub fn new_msaa_buffer(w: u32, h: u32, samples: u32) -> RenderBuffer {
    RenderBuffer {
        width: w,
        height: h,
        format: 0,
        samples: samples.max(1),
        is_depth: false,
    }
}

/// Total pixel count (width × height).
#[allow(dead_code)]
pub fn buffer_pixel_count(b: &RenderBuffer) -> u64 {
    b.width as u64 * b.height as u64
}

/// Human-readable format name.
#[allow(dead_code)]
pub fn buffer_format_name(b: &RenderBuffer) -> &'static str {
    match b.format {
        0 => "RGBA8",
        1 => "RGBA16F",
        2 => "Depth24",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_buffer_not_depth() {
        let b = new_color_buffer(800, 600);
        assert!(!b.is_depth);
    }

    #[test]
    fn depth_buffer_is_depth() {
        let b = new_depth_buffer(800, 600);
        assert!(b.is_depth);
    }

    #[test]
    fn color_buffer_dimensions() {
        let b = new_color_buffer(1920, 1080);
        assert_eq!(b.width, 1920);
        assert_eq!(b.height, 1080);
    }

    #[test]
    fn buffer_pixel_count_correct() {
        let b = new_color_buffer(4, 4);
        assert_eq!(buffer_pixel_count(&b), 16);
    }

    #[test]
    fn buffer_format_name_rgba8() {
        let b = new_color_buffer(1, 1);
        assert_eq!(buffer_format_name(&b), "RGBA8");
    }

    #[test]
    fn buffer_format_name_depth() {
        let b = new_depth_buffer(1, 1);
        assert_eq!(buffer_format_name(&b), "Depth24");
    }

    #[test]
    fn msaa_buffer_samples() {
        let b = new_msaa_buffer(1280, 720, 4);
        assert_eq!(b.samples, 4);
    }

    #[test]
    fn msaa_buffer_samples_min_one() {
        let b = new_msaa_buffer(1280, 720, 0);
        assert!(b.samples >= 1);
    }

    #[test]
    fn depth_buffer_format_code() {
        let b = new_depth_buffer(256, 256);
        assert_eq!(b.format, 2);
    }

    #[test]
    fn color_buffer_samples_one() {
        let b = new_color_buffer(100, 100);
        assert_eq!(b.samples, 1);
    }
}
