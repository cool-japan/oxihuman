// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Alpha pre-multiplication helpers — convert between straight and premultiplied alpha.

/// RGBA pixel with f32 components in 0..=1.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RgbaF32 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl RgbaF32 {
    #[allow(dead_code)]
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    #[allow(dead_code)]
    pub fn from_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        const INV: f32 = 1.0 / 255.0;
        Self {
            r: r as f32 * INV,
            g: g as f32 * INV,
            b: b as f32 * INV,
            a: a as f32 * INV,
        }
    }
}

/// Convert straight-alpha to premultiplied-alpha.
#[allow(dead_code)]
pub fn premultiply(p: RgbaF32) -> RgbaF32 {
    RgbaF32 {
        r: p.r * p.a,
        g: p.g * p.a,
        b: p.b * p.a,
        a: p.a,
    }
}

/// Convert premultiplied-alpha back to straight-alpha.
/// Returns transparent black if alpha ≤ epsilon.
#[allow(dead_code)]
pub fn unpremultiply(p: RgbaF32) -> RgbaF32 {
    if p.a < 1e-7 {
        return RgbaF32 {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        };
    }
    let inv = 1.0 / p.a;
    RgbaF32 {
        r: (p.r * inv).min(1.0),
        g: (p.g * inv).min(1.0),
        b: (p.b * inv).min(1.0),
        a: p.a,
    }
}

/// Alpha-blend two premultiplied pixels (src over dst).
#[allow(dead_code)]
pub fn alpha_composite(src: RgbaF32, dst: RgbaF32) -> RgbaF32 {
    let inv_src_a = 1.0 - src.a;
    RgbaF32 {
        r: src.r + dst.r * inv_src_a,
        g: src.g + dst.g * inv_src_a,
        b: src.b + dst.b * inv_src_a,
        a: src.a + dst.a * inv_src_a,
    }
}

/// Premultiply a whole buffer in-place.
#[allow(dead_code)]
pub fn premultiply_buffer(buf: &mut [RgbaF32]) {
    for p in buf.iter_mut() {
        *p = premultiply(*p);
    }
}

/// Unmultiply a whole buffer in-place.
#[allow(dead_code)]
pub fn unpremultiply_buffer(buf: &mut [RgbaF32]) {
    for p in buf.iter_mut() {
        *p = unpremultiply(*p);
    }
}

/// Linearise a sRGB u8 component to linear f32.
#[allow(dead_code)]
pub fn srgb_u8_to_linear(v: u8) -> f32 {
    let s = v as f32 / 255.0;
    if s <= 0.04045 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

/// Encode linear f32 to sRGB u8.
#[allow(dead_code)]
pub fn linear_to_srgb_u8(v: f32) -> u8 {
    let v = v.clamp(0.0, 1.0);
    let s = if v <= 0.0031308 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    (s * 255.0).round() as u8
}

/// Luminance of a linear RGB pixel (BT.709 coefficients).
#[allow(dead_code)]
pub fn luminance(p: RgbaF32) -> f32 {
    0.2126 * p.r + 0.7152 * p.g + 0.0722 * p.b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn premultiply_half_alpha() {
        let p = premultiply(RgbaF32::new(1.0, 0.0, 0.0, 0.5));
        assert!((p.r - 0.5).abs() < 1e-6);
        assert!((p.a - 0.5).abs() < 1e-6);
    }

    #[test]
    fn unpremultiply_recovers() {
        let orig = RgbaF32::new(0.8, 0.4, 0.2, 0.8);
        let pm = premultiply(orig);
        let back = unpremultiply(pm);
        assert!((back.r - orig.r).abs() < 1e-5);
    }

    #[test]
    fn unpremultiply_zero_alpha() {
        let p = unpremultiply(RgbaF32::new(0.0, 0.0, 0.0, 0.0));
        assert!((p.r).abs() < 1e-6);
    }

    #[test]
    fn composite_opaque_src_over_anything() {
        let src = RgbaF32::new(1.0, 0.0, 0.0, 1.0);
        let dst = RgbaF32::new(0.0, 1.0, 0.0, 1.0);
        let out = alpha_composite(src, dst);
        assert!((out.r - 1.0).abs() < 1e-6);
        assert!((out.g).abs() < 1e-6);
    }

    #[test]
    fn composite_transparent_src_passthrough() {
        let src = RgbaF32::new(1.0, 0.0, 0.0, 0.0);
        let dst = RgbaF32::new(0.0, 1.0, 0.0, 1.0);
        let out = alpha_composite(src, dst);
        assert!((out.g - 1.0).abs() < 1e-6);
    }

    #[test]
    fn premultiply_buffer_modifies_in_place() {
        let mut buf = vec![RgbaF32::new(1.0, 1.0, 1.0, 0.5)];
        premultiply_buffer(&mut buf);
        assert!((buf[0].r - 0.5).abs() < 1e-6);
    }

    #[test]
    fn srgb_roundtrip_midgray() {
        let lin = srgb_u8_to_linear(128);
        let back = linear_to_srgb_u8(lin);
        assert!((back as i32 - 128).abs() <= 1);
    }

    #[test]
    fn luminance_white() {
        let white = RgbaF32::new(1.0, 1.0, 1.0, 1.0);
        assert!((luminance(white) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn luminance_black() {
        let black = RgbaF32::new(0.0, 0.0, 0.0, 1.0);
        assert!(luminance(black) < 1e-6);
    }

    #[test]
    fn from_u8_full_white() {
        let p = RgbaF32::from_u8(255, 255, 255, 255);
        assert!((p.r - 1.0).abs() < 0.005);
    }
}
