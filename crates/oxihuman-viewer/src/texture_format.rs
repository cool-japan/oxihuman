// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Information about a texture format.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFormatInfo {
    R8,
    Rg8,
    Rgba8,
    Rgba8Srgb,
    Rgba16F,
    Rgba32F,
    Bc1,
    Bc3,
    Bc7,
    Depth24,
}

/// Return bytes per pixel (or average for compressed).
#[allow(dead_code)]
pub fn format_bytes_per_pixel(fmt: TextureFormatInfo) -> f32 {
    match fmt {
        TextureFormatInfo::R8 => 1.0,
        TextureFormatInfo::Rg8 => 2.0,
        TextureFormatInfo::Rgba8 | TextureFormatInfo::Rgba8Srgb => 4.0,
        TextureFormatInfo::Rgba16F => 8.0,
        TextureFormatInfo::Rgba32F => 16.0,
        TextureFormatInfo::Bc1 => 0.5,
        TextureFormatInfo::Bc3 | TextureFormatInfo::Bc7 => 1.0,
        TextureFormatInfo::Depth24 => 3.0,
    }
}

/// Check if the format is block-compressed.
#[allow(dead_code)]
pub fn format_is_compressed(fmt: TextureFormatInfo) -> bool {
    matches!(
        fmt,
        TextureFormatInfo::Bc1 | TextureFormatInfo::Bc3 | TextureFormatInfo::Bc7
    )
}

/// Check if the format has an alpha channel.
#[allow(dead_code)]
pub fn format_has_alpha(fmt: TextureFormatInfo) -> bool {
    matches!(
        fmt,
        TextureFormatInfo::Rgba8
            | TextureFormatInfo::Rgba8Srgb
            | TextureFormatInfo::Rgba16F
            | TextureFormatInfo::Rgba32F
            | TextureFormatInfo::Bc3
            | TextureFormatInfo::Bc7
    )
}

/// Return the number of channels.
#[allow(dead_code)]
pub fn format_channel_count(fmt: TextureFormatInfo) -> usize {
    match fmt {
        TextureFormatInfo::R8 | TextureFormatInfo::Depth24 => 1,
        TextureFormatInfo::Rg8 => 2,
        TextureFormatInfo::Bc1 => 3,
        _ => 4,
    }
}

/// Return the format name.
#[allow(dead_code)]
pub fn format_name_tf(fmt: TextureFormatInfo) -> &'static str {
    match fmt {
        TextureFormatInfo::R8 => "R8",
        TextureFormatInfo::Rg8 => "RG8",
        TextureFormatInfo::Rgba8 => "RGBA8",
        TextureFormatInfo::Rgba8Srgb => "RGBA8_SRGB",
        TextureFormatInfo::Rgba16F => "RGBA16F",
        TextureFormatInfo::Rgba32F => "RGBA32F",
        TextureFormatInfo::Bc1 => "BC1",
        TextureFormatInfo::Bc3 => "BC3",
        TextureFormatInfo::Bc7 => "BC7",
        TextureFormatInfo::Depth24 => "DEPTH24",
    }
}

/// Serialize format info to JSON.
#[allow(dead_code)]
pub fn format_to_json(fmt: TextureFormatInfo) -> String {
    format!(
        "{{\"name\":\"{}\",\"bpp\":{:.1},\"compressed\":{},\"alpha\":{}}}",
        format_name_tf(fmt),
        format_bytes_per_pixel(fmt),
        format_is_compressed(fmt),
        format_has_alpha(fmt),
    )
}

/// Check if the format is sRGB.
#[allow(dead_code)]
pub fn format_is_srgb(fmt: TextureFormatInfo) -> bool {
    fmt == TextureFormatInfo::Rgba8Srgb
}

/// Return the maximum representable value per channel.
#[allow(dead_code)]
pub fn format_max_value(fmt: TextureFormatInfo) -> f32 {
    match fmt {
        TextureFormatInfo::R8
        | TextureFormatInfo::Rg8
        | TextureFormatInfo::Rgba8
        | TextureFormatInfo::Rgba8Srgb
        | TextureFormatInfo::Bc1
        | TextureFormatInfo::Bc3
        | TextureFormatInfo::Bc7
        | TextureFormatInfo::Depth24 => 1.0,
        TextureFormatInfo::Rgba16F => 65504.0,
        TextureFormatInfo::Rgba32F => f32::MAX,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgba8_bpp() {
        assert!((format_bytes_per_pixel(TextureFormatInfo::Rgba8) - 4.0).abs() < 1e-6);
    }

    #[test]
    fn bc1_compressed() {
        assert!(format_is_compressed(TextureFormatInfo::Bc1));
    }

    #[test]
    fn rgba8_not_compressed() {
        assert!(!format_is_compressed(TextureFormatInfo::Rgba8));
    }

    #[test]
    fn rgba8_has_alpha() {
        assert!(format_has_alpha(TextureFormatInfo::Rgba8));
    }

    #[test]
    fn r8_no_alpha() {
        assert!(!format_has_alpha(TextureFormatInfo::R8));
    }

    #[test]
    fn channel_count_r8() {
        assert_eq!(format_channel_count(TextureFormatInfo::R8), 1);
    }

    #[test]
    fn channel_count_rgba() {
        assert_eq!(format_channel_count(TextureFormatInfo::Rgba8), 4);
    }

    #[test]
    fn format_name() {
        assert_eq!(format_name_tf(TextureFormatInfo::Rgba16F), "RGBA16F");
    }

    #[test]
    fn srgb_check() {
        assert!(format_is_srgb(TextureFormatInfo::Rgba8Srgb));
        assert!(!format_is_srgb(TextureFormatInfo::Rgba8));
    }

    #[test]
    fn max_value_f16() {
        assert!((format_max_value(TextureFormatInfo::Rgba16F) - 65504.0).abs() < 1.0);
    }
}
