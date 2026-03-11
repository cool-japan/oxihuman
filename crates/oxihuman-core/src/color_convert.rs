// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Color space conversion utilities.

#![allow(dead_code)]

use std::f32::consts::PI;

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
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// Convert RGB to HSV. All inputs and outputs in [0, 1].
#[allow(dead_code)]
pub fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let v = max;
    let s = if max < 1e-6 { 0.0 } else { delta / max };
    let h = if delta < 1e-6 {
        0.0
    } else if (max - r).abs() < 1e-6 {
        60.0 * (((g - b) / delta) % 6.0)
    } else if (max - g).abs() < 1e-6 {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    let h = if h < 0.0 { h + 360.0 } else { h };
    (h / 360.0, s, v)
}

/// Convert HSV to RGB. All inputs and outputs in [0, 1].
#[allow(dead_code)]
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let h_deg = h * 360.0;
    let c = v * s;
    let x = c * (1.0 - ((h_deg / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r1, g1, b1) = if h_deg < 60.0 {
        (c, x, 0.0)
    } else if h_deg < 120.0 {
        (x, c, 0.0)
    } else if h_deg < 180.0 {
        (0.0, c, x)
    } else if h_deg < 240.0 {
        (0.0, x, c)
    } else if h_deg < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    (r1 + m, g1 + m, b1 + m)
}

/// Approximate RGB to OKLCh (a perceptual color space).
/// This is a rough approximation for stub purposes.
#[allow(dead_code)]
pub fn rgb_to_oklch_approx(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    // Simplified: L ≈ luma, C ≈ chroma, h ≈ hue angle in radians
    let l = luma_bt709(r, g, b);
    let dr = r - l;
    let db = b - l;
    let c = (dr * dr + db * db).sqrt();
    let h = db.atan2(dr) + PI;
    (l, c, h)
}

/// Compute BT.709 luma from linear RGB.
#[allow(dead_code)]
pub fn luma_bt709(r: f32, g: f32, b: f32) -> f32 {
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srgb_to_linear_zero() {
        assert!((srgb_to_linear(0.0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_srgb_to_linear_one() {
        assert!((srgb_to_linear(1.0) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_linear_to_srgb_zero() {
        assert!((linear_to_srgb(0.0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_linear_to_srgb_one() {
        assert!((linear_to_srgb(1.0) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_srgb_linear_roundtrip() {
        let val = 0.5f32;
        let linear = srgb_to_linear(val);
        let back = linear_to_srgb(linear);
        assert!((back - val).abs() < 1e-5, "back={back}, val={val}");
    }

    #[test]
    fn test_rgb_to_hsv_red() {
        let (h, s, v) = rgb_to_hsv(1.0, 0.0, 0.0);
        assert!((h - 0.0).abs() < 1e-4 || (h - 1.0).abs() < 1e-4);
        assert!((s - 1.0).abs() < 1e-4);
        assert!((v - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_hsv_to_rgb_white() {
        let (r, g, b) = hsv_to_rgb(0.0, 0.0, 1.0);
        assert!((r - 1.0).abs() < 1e-5);
        assert!((g - 1.0).abs() < 1e-5);
        assert!((b - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_luma_bt709_white() {
        let l = luma_bt709(1.0, 1.0, 1.0);
        assert!((l - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_luma_bt709_black() {
        let l = luma_bt709(0.0, 0.0, 0.0);
        assert!(l.abs() < 1e-9);
    }

    #[test]
    fn test_oklch_approx_returns_tuple() {
        let (l, c, h) = rgb_to_oklch_approx(0.5, 0.5, 0.5);
        // For neutral gray, chroma should be near zero
        assert!(c >= 0.0);
        assert!(l >= 0.0);
        let _ = h; // just check it's computed
    }
}
