// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Colour utility helpers: conversion, blending, clamping.

use std::f32::consts::PI;

/// Clamp a float to [0.0, 1.0].
#[allow(dead_code)]
pub fn clamp01(x: f32) -> f32 {
    x.clamp(0.0, 1.0)
}

/// Linear interpolation between two colours in `[r,g,b,a]`.
#[allow(dead_code)]
pub fn lerp_rgba(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    let t = clamp01(t);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

/// Convert sRGB component to linear.
#[allow(dead_code)]
pub fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Convert linear component to sRGB.
#[allow(dead_code)]
pub fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// Luminance of an sRGB colour (Rec. 709).
#[allow(dead_code)]
pub fn luminance_srgb(r: f32, g: f32, b: f32) -> f32 {
    let rl = srgb_to_linear(r);
    let gl = srgb_to_linear(g);
    let bl = srgb_to_linear(b);
    0.2126 * rl + 0.7152 * gl + 0.0722 * bl
}

/// Hue rotation in degrees applied to an RGB colour.
#[allow(dead_code)]
pub fn hue_rotate(rgb: [f32; 3], degrees: f32) -> [f32; 3] {
    let rad = degrees * PI / 180.0;
    let cos_a = rad.cos();
    let sin_a = rad.sin();
    let sqrt3 = 3.0_f32.sqrt();
    let one_third = 1.0 / 3.0;
    [
        clamp01(
            rgb[0] * (cos_a + (1.0 - cos_a) * one_third)
                + rgb[1] * (one_third * (1.0 - cos_a) - sqrt3 / 3.0 * sin_a)
                + rgb[2] * (one_third * (1.0 - cos_a) + sqrt3 / 3.0 * sin_a),
        ),
        clamp01(
            rgb[0] * (one_third * (1.0 - cos_a) + sqrt3 / 3.0 * sin_a)
                + rgb[1] * (cos_a + one_third * (1.0 - cos_a))
                + rgb[2] * (one_third * (1.0 - cos_a) - sqrt3 / 3.0 * sin_a),
        ),
        clamp01(
            rgb[0] * (one_third * (1.0 - cos_a) - sqrt3 / 3.0 * sin_a)
                + rgb[1] * (one_third * (1.0 - cos_a) + sqrt3 / 3.0 * sin_a)
                + rgb[2] * (cos_a + one_third * (1.0 - cos_a)),
        ),
    ]
}

/// Pack RGB [0..1] to a u32 as 0x00RRGGBB.
#[allow(dead_code)]
pub fn rgb_to_u32(r: f32, g: f32, b: f32) -> u32 {
    let ri = (clamp01(r) * 255.0).round() as u32;
    let gi = (clamp01(g) * 255.0).round() as u32;
    let bi = (clamp01(b) * 255.0).round() as u32;
    (ri << 16) | (gi << 8) | bi
}

/// Unpack u32 0x00RRGGBB to [r, g, b] in [0..1].
#[allow(dead_code)]
pub fn u32_to_rgb(packed: u32) -> [f32; 3] {
    let r = ((packed >> 16) & 0xFF) as f32 / 255.0;
    let g = ((packed >> 8) & 0xFF) as f32 / 255.0;
    let b = (packed & 0xFF) as f32 / 255.0;
    [r, g, b]
}

/// Validate that a color component is in [0.0, 1.0].
#[allow(dead_code)]
pub fn is_valid_component(c: f32) -> bool {
    (0.0..=1.0).contains(&c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp01_bounds() {
        assert_eq!(clamp01(-0.5), 0.0);
        assert_eq!(clamp01(1.5), 1.0);
        assert_eq!(clamp01(0.5), 0.5);
    }

    #[test]
    fn lerp_rgba_midpoint() {
        let a = [0.0, 0.0, 0.0, 1.0];
        let b = [1.0, 1.0, 1.0, 1.0];
        let m = lerp_rgba(a, b, 0.5);
        assert!((m[0] - 0.5).abs() < 1e-6);
        assert!((m[1] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn srgb_linear_round_trip() {
        let v = 0.5_f32;
        let lin = srgb_to_linear(v);
        let back = linear_to_srgb(lin);
        assert!((back - v).abs() < 1e-5);
    }

    #[test]
    fn luminance_white_is_one() {
        let l = luminance_srgb(1.0, 1.0, 1.0);
        assert!((l - 1.0).abs() < 0.01);
    }

    #[test]
    fn luminance_black_is_zero() {
        let l = luminance_srgb(0.0, 0.0, 0.0);
        assert!(l.abs() < 1e-6);
    }

    #[test]
    fn rgb_pack_round_trip() {
        let packed = rgb_to_u32(0.5, 0.25, 0.75);
        let rgb = u32_to_rgb(packed);
        assert!((rgb[0] - 0.5).abs() < 0.01);
        assert!((rgb[1] - 0.25).abs() < 0.01);
        assert!((rgb[2] - 0.75).abs() < 0.01);
    }

    #[test]
    fn hue_rotate_360_identity() {
        let rgb = [0.8, 0.3, 0.5];
        let rotated = hue_rotate(rgb, 360.0);
        assert!((rotated[0] - rgb[0]).abs() < 0.02);
        assert!((rotated[1] - rgb[1]).abs() < 0.02);
        assert!((rotated[2] - rgb[2]).abs() < 0.02);
    }

    #[test]
    fn is_valid_component_checks_range() {
        assert!(is_valid_component(0.0));
        assert!(is_valid_component(1.0));
        assert!(is_valid_component(0.5));
        assert!(!is_valid_component(-0.1));
        assert!(!is_valid_component(1.1));
    }

    #[test]
    fn srgb_to_linear_low_values() {
        let v = 0.01_f32;
        let lin = srgb_to_linear(v);
        assert!(lin < v);
    }

    #[test]
    fn lerp_rgba_at_zero() {
        let a = [0.2, 0.4, 0.6, 0.8];
        let b = [0.9, 0.1, 0.3, 0.5];
        let r = lerp_rgba(a, b, 0.0);
        assert!((r[0] - a[0]).abs() < 1e-6);
    }
}
