// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! u32 RGBA packed color with encode/decode utilities.

#![allow(dead_code)]

/// A packed RGBA color stored as a single u32: 0xRRGGBBAA.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PackedColor(pub u32);

/// Encode RGBA bytes (0-255) into a PackedColor.
#[allow(dead_code)]
pub fn rgba_u8(r: u8, g: u8, b: u8, a: u8) -> PackedColor {
    PackedColor(((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32))
}

/// Encode RGBA floats (0.0-1.0) into a PackedColor.
#[allow(dead_code)]
pub fn rgba_f32(r: f32, g: f32, b: f32, a: f32) -> PackedColor {
    rgba_u8(
        (r.clamp(0.0, 1.0) * 255.0).round() as u8,
        (g.clamp(0.0, 1.0) * 255.0).round() as u8,
        (b.clamp(0.0, 1.0) * 255.0).round() as u8,
        (a.clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}

/// Decode a PackedColor into RGBA bytes.
#[allow(dead_code)]
pub fn decode_rgba_u8(c: PackedColor) -> [u8; 4] {
    [
        ((c.0 >> 24) & 0xFF) as u8,
        ((c.0 >> 16) & 0xFF) as u8,
        ((c.0 >> 8) & 0xFF) as u8,
        (c.0 & 0xFF) as u8,
    ]
}

/// Decode a PackedColor into RGBA floats [0.0, 1.0].
#[allow(dead_code)]
pub fn decode_rgba_f32(c: PackedColor) -> [f32; 4] {
    let bytes = decode_rgba_u8(c);
    [
        bytes[0] as f32 / 255.0,
        bytes[1] as f32 / 255.0,
        bytes[2] as f32 / 255.0,
        bytes[3] as f32 / 255.0,
    ]
}

/// Extract the red channel as a byte.
#[allow(dead_code)]
pub fn color_r(c: PackedColor) -> u8 {
    ((c.0 >> 24) & 0xFF) as u8
}

/// Extract the green channel as a byte.
#[allow(dead_code)]
pub fn color_g(c: PackedColor) -> u8 {
    ((c.0 >> 16) & 0xFF) as u8
}

/// Extract the blue channel as a byte.
#[allow(dead_code)]
pub fn color_b(c: PackedColor) -> u8 {
    ((c.0 >> 8) & 0xFF) as u8
}

/// Extract the alpha channel as a byte.
#[allow(dead_code)]
pub fn color_a(c: PackedColor) -> u8 {
    (c.0 & 0xFF) as u8
}

/// Linearly interpolate between two packed colors.
#[allow(dead_code)]
pub fn color_lerp(a: PackedColor, b: PackedColor, t: f32) -> PackedColor {
    let t = t.clamp(0.0, 1.0);
    let fa = decode_rgba_f32(a);
    let fb = decode_rgba_f32(b);
    rgba_f32(
        fa[0] + t * (fb[0] - fa[0]),
        fa[1] + t * (fb[1] - fa[1]),
        fa[2] + t * (fb[2] - fa[2]),
        fa[3] + t * (fb[3] - fa[3]),
    )
}

/// Blend src over dst using alpha compositing (src-over).
#[allow(dead_code)]
pub fn color_blend(dst: PackedColor, src: PackedColor) -> PackedColor {
    let d = decode_rgba_f32(dst);
    let s = decode_rgba_f32(src);
    let sa = s[3];
    let isa = 1.0 - sa;
    rgba_f32(
        s[0] * sa + d[0] * isa,
        s[1] * sa + d[1] * isa,
        s[2] * sa + d[2] * isa,
        sa + d[3] * isa,
    )
}

/// Premultiply alpha: multiply RGB by alpha.
#[allow(dead_code)]
pub fn color_premul_alpha(c: PackedColor) -> PackedColor {
    let f = decode_rgba_f32(c);
    rgba_f32(f[0] * f[3], f[1] * f[3], f[2] * f[3], f[3])
}

/// Convert to grayscale (luminance) using BT.601 coefficients.
#[allow(dead_code)]
pub fn color_to_grayscale(c: PackedColor) -> PackedColor {
    let f = decode_rgba_f32(c);
    let lum = 0.299 * f[0] + 0.587 * f[1] + 0.114 * f[2];
    rgba_f32(lum, lum, lum, f[3])
}

/// Opaque black.
#[allow(dead_code)]
pub fn color_black() -> PackedColor {
    rgba_u8(0, 0, 0, 255)
}

/// Opaque white.
#[allow(dead_code)]
pub fn color_white() -> PackedColor {
    rgba_u8(255, 255, 255, 255)
}

/// Transparent (fully transparent black).
#[allow(dead_code)]
pub fn color_transparent() -> PackedColor {
    rgba_u8(0, 0, 0, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_u8() {
        let c = rgba_u8(255, 128, 64, 32);
        let d = decode_rgba_u8(c);
        assert_eq!(d, [255, 128, 64, 32]);
    }

    #[test]
    fn test_encode_decode_f32() {
        let c = rgba_f32(1.0, 0.5, 0.25, 1.0);
        let d = decode_rgba_f32(c);
        assert!((d[0] - 1.0).abs() < 0.005);
        assert!((d[1] - 0.5).abs() < 0.005);
    }

    #[test]
    fn test_channel_extraction() {
        let c = rgba_u8(10, 20, 30, 40);
        assert_eq!(color_r(c), 10);
        assert_eq!(color_g(c), 20);
        assert_eq!(color_b(c), 30);
        assert_eq!(color_a(c), 40);
    }

    #[test]
    fn test_lerp_t0() {
        let a = color_black();
        let b = color_white();
        let r = color_lerp(a, b, 0.0);
        assert_eq!(color_r(r), 0);
    }

    #[test]
    fn test_lerp_t1() {
        let a = color_black();
        let b = color_white();
        let r = color_lerp(a, b, 1.0);
        assert_eq!(color_r(r), 255);
    }

    #[test]
    fn test_white_black_constants() {
        assert_eq!(color_r(color_white()), 255);
        assert_eq!(color_r(color_black()), 0);
        assert_eq!(color_a(color_transparent()), 0);
    }

    #[test]
    fn test_premul_alpha() {
        let c = rgba_f32(1.0, 0.0, 0.0, 0.5);
        let p = color_premul_alpha(c);
        let f = decode_rgba_f32(p);
        assert!((f[0] - 0.5).abs() < 0.005);
    }

    #[test]
    fn test_grayscale() {
        let c = rgba_u8(128, 128, 128, 255);
        let g = color_to_grayscale(c);
        let d = decode_rgba_u8(g);
        assert_eq!(d[0], d[1]);
        assert_eq!(d[1], d[2]);
    }

    #[test]
    fn test_blend_opaque_src() {
        let dst = color_black();
        let src = color_white();
        let result = color_blend(dst, src);
        let d = decode_rgba_f32(result);
        assert!((d[0] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_packed_color_eq() {
        let a = rgba_u8(100, 100, 100, 100);
        let b = rgba_u8(100, 100, 100, 100);
        assert_eq!(a, b);
    }
}
