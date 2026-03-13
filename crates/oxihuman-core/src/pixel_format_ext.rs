#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Pixel format utilities (extended).

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormatExt {
    Rgb8,
    Rgba8,
    Rgb16,
    Rgba16,
    R32F,
    Rgba32F,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PixelInfo {
    pub format: PixelFormatExt,
    pub bytes_per_pixel: u8,
    pub channels: u8,
}

#[allow(dead_code)]
pub fn pixel_info(fmt: PixelFormatExt) -> PixelInfo {
    match fmt {
        PixelFormatExt::Rgb8 => PixelInfo { format: fmt, bytes_per_pixel: 3, channels: 3 },
        PixelFormatExt::Rgba8 => PixelInfo { format: fmt, bytes_per_pixel: 4, channels: 4 },
        PixelFormatExt::Rgb16 => PixelInfo { format: fmt, bytes_per_pixel: 6, channels: 3 },
        PixelFormatExt::Rgba16 => PixelInfo { format: fmt, bytes_per_pixel: 8, channels: 4 },
        PixelFormatExt::R32F => PixelInfo { format: fmt, bytes_per_pixel: 4, channels: 1 },
        PixelFormatExt::Rgba32F => PixelInfo { format: fmt, bytes_per_pixel: 16, channels: 4 },
    }
}

#[allow(dead_code)]
pub fn pixel_stride(fmt: PixelFormatExt) -> usize {
    pixel_info(fmt).bytes_per_pixel as usize
}

#[allow(dead_code)]
pub fn pixel_buffer_size(w: u32, h: u32, fmt: PixelFormatExt) -> usize {
    w as usize * h as usize * pixel_stride(fmt)
}

#[allow(dead_code)]
pub fn format_name(fmt: PixelFormatExt) -> &'static str {
    match fmt {
        PixelFormatExt::Rgb8 => "RGB8",
        PixelFormatExt::Rgba8 => "RGBA8",
        PixelFormatExt::Rgb16 => "RGB16",
        PixelFormatExt::Rgba16 => "RGBA16",
        PixelFormatExt::R32F => "R32F",
        PixelFormatExt::Rgba32F => "RGBA32F",
    }
}

#[allow(dead_code)]
pub fn is_float_format(fmt: PixelFormatExt) -> bool {
    matches!(fmt, PixelFormatExt::R32F | PixelFormatExt::Rgba32F)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb8_stride() {
        assert_eq!(pixel_stride(PixelFormatExt::Rgb8), 3);
    }

    #[test]
    fn test_rgba8_stride() {
        assert_eq!(pixel_stride(PixelFormatExt::Rgba8), 4);
    }

    #[test]
    fn test_rgba32f_stride() {
        assert_eq!(pixel_stride(PixelFormatExt::Rgba32F), 16);
    }

    #[test]
    fn test_buffer_size_rgb8() {
        assert_eq!(pixel_buffer_size(10, 10, PixelFormatExt::Rgb8), 300);
    }

    #[test]
    fn test_buffer_size_rgba32f() {
        assert_eq!(pixel_buffer_size(4, 4, PixelFormatExt::Rgba32F), 256);
    }

    #[test]
    fn test_format_name_rgb8() {
        assert_eq!(format_name(PixelFormatExt::Rgb8), "RGB8");
    }

    #[test]
    fn test_is_float_r32f() {
        assert!(is_float_format(PixelFormatExt::R32F));
    }

    #[test]
    fn test_is_float_rgba32f() {
        assert!(is_float_format(PixelFormatExt::Rgba32F));
    }

    #[test]
    fn test_not_float_rgb8() {
        assert!(!is_float_format(PixelFormatExt::Rgb8));
    }

    #[test]
    fn test_channels_rgba16() {
        let info = pixel_info(PixelFormatExt::Rgba16);
        assert_eq!(info.channels, 4);
        assert_eq!(info.bytes_per_pixel, 8);
    }
}
