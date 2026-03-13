// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Convert sRGB `[0,1]` to HSL `[h in 0..360, s in 0..1, l in 0..1]`.
pub fn rgb_to_hsl(r: f32, g: f32, b: f32) -> [f32; 3] {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    if (max - min).abs() < 1e-12 {
        return [0.0, 0.0, l];
    }
    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };
    let h = if (max - r).abs() < 1e-9 {
        ((g - b) / d).rem_euclid(6.0) * 60.0
    } else if (max - g).abs() < 1e-9 {
        ((b - r) / d + 2.0) * 60.0
    } else {
        ((r - g) / d + 4.0) * 60.0
    };
    [h, s, l]
}

/// Convert HSL `[h in 0..360, s, l]` to sRGB `[0,1]`.
pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> [f32; 3] {
    if s < 1e-9 {
        return [l, l, l];
    }
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let hk = h / 360.0;
    let channel = |t: f32| -> f32 {
        let t = t.rem_euclid(1.0);
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    };
    [
        channel(hk + 1.0 / 3.0),
        channel(hk),
        channel(hk - 1.0 / 3.0),
    ]
}

/// Convert sRGB `[0,1]` to HSV `[h in 0..360, s in 0..1, v in 0..1]`.
pub fn rgb_to_hsv(r: f32, g: f32, b: f32) -> [f32; 3] {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let v = max;
    let s = if max < 1e-12 { 0.0 } else { d / max };
    if d < 1e-12 {
        return [0.0, s, v];
    }
    let h = if (max - r).abs() < 1e-9 {
        ((g - b) / d).rem_euclid(6.0) * 60.0
    } else if (max - g).abs() < 1e-9 {
        ((b - r) / d + 2.0) * 60.0
    } else {
        ((r - g) / d + 4.0) * 60.0
    };
    [h, s, v]
}

/// Convert HSV `[h in 0..360, s, v]` to sRGB `[0,1]`.
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    if s < 1e-9 {
        return [v, v, v];
    }
    let i = (h / 60.0).floor() as u32;
    let f = h / 60.0 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    match i % 6 {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        _ => [v, p, q],
    }
}

/// Linearly interpolate between two RGB colours.
pub fn color_lerp(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Gamma-decode sRGB component to linear light.
pub fn srgb_to_linear_cm(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Gamma-encode linear light component to sRGB.
pub fn linear_to_srgb_cm(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// Compute luminance from linear RGB using BT.709 coefficients.
pub fn luminance_cm(r: f32, g: f32, b: f32) -> f32 {
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Clamp all channels of a colour to [0, 1].
pub fn color_clamp(c: [f32; 3]) -> [f32; 3] {
    [
        c[0].clamp(0.0, 1.0),
        c[1].clamp(0.0, 1.0),
        c[2].clamp(0.0, 1.0),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_hsl_white() {
        /* white should have saturation 0 */
        let hsl = rgb_to_hsl(1.0, 1.0, 1.0);
        assert!(hsl[1].abs() < 1e-5);
        assert!((hsl[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_hsl_to_rgb_grey() {
        /* grey round-trip */
        let rgb = hsl_to_rgb(0.0, 0.0, 0.5);
        assert!((rgb[0] - 0.5).abs() < 1e-5);
        assert!((rgb[1] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_rgb_hsl_roundtrip() {
        /* red round-trip */
        let hsl = rgb_to_hsl(1.0, 0.0, 0.0);
        let rgb = hsl_to_rgb(hsl[0], hsl[1], hsl[2]);
        assert!((rgb[0] - 1.0).abs() < 1e-4);
        assert!(rgb[1].abs() < 1e-4);
        assert!(rgb[2].abs() < 1e-4);
    }

    #[test]
    fn test_rgb_to_hsv_black() {
        /* black */
        let hsv = rgb_to_hsv(0.0, 0.0, 0.0);
        assert!(hsv[2].abs() < 1e-6);
    }

    #[test]
    fn test_hsv_to_rgb_roundtrip() {
        /* green round-trip */
        let hsv = rgb_to_hsv(0.0, 1.0, 0.0);
        let rgb = hsv_to_rgb(hsv[0], hsv[1], hsv[2]);
        assert!(rgb[0].abs() < 1e-4);
        assert!((rgb[1] - 1.0).abs() < 1e-4);
        assert!(rgb[2].abs() < 1e-4);
    }

    #[test]
    fn test_color_lerp_mid() {
        /* midpoint lerp */
        let c = color_lerp([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 0.5);
        assert!((c[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_srgb_to_linear() {
        /* sRGB near-zero is linear */
        let lin = srgb_to_linear_cm(0.0);
        assert!(lin.abs() < 1e-9);
        let lin2 = srgb_to_linear_cm(1.0);
        assert!((lin2 - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_linear_to_srgb() {
        /* round-trip */
        let x = 0.5f32;
        let lin = srgb_to_linear_cm(x);
        let back = linear_to_srgb_cm(lin);
        assert!((back - x).abs() < 1e-4);
    }

    #[test]
    fn test_luminance() {
        /* white luminance = 1 */
        let lum = luminance_cm(1.0, 1.0, 1.0);
        assert!((lum - 1.0).abs() < 1e-5);
        /* black = 0 */
        assert!(luminance_cm(0.0, 0.0, 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_color_clamp() {
        /* clamping out-of-range values */
        let c = color_clamp([-0.1, 0.5, 1.5]);
        assert!(c[0] >= 0.0);
        assert!(c[2] <= 1.0);
    }
}
