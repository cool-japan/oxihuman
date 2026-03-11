// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Procedural color palette generation utilities.

use std::f32::consts::TAU;

/// Generate a monochromatic palette: `n` colours of varying lightness around `hue`.
pub fn palette_monochromatic(hue: f32, n: usize) -> Vec<[f32; 3]> {
    if n == 0 {
        return vec![];
    }
    (0..n)
        .map(|i| {
            let l = (i + 1) as f32 / (n + 1) as f32;
            hsl_to_rgb_gen(hue, 0.7, l)
        })
        .collect()
}

/// Generate a complementary palette (two hues 180° apart), `n` colours each side.
pub fn palette_complementary(hue: f32, n: usize) -> Vec<[f32; 3]> {
    let mut out = palette_monochromatic(hue, n);
    let comp = (hue + 180.0) % 360.0;
    out.extend(palette_monochromatic(comp, n));
    out
}

/// Generate an analogous palette: hues within ±30° of base.
pub fn palette_analogous(hue: f32, n: usize) -> Vec<[f32; 3]> {
    if n == 0 {
        return vec![];
    }
    let step = 60.0 / (n as f32).max(1.0);
    (0..n)
        .map(|i| {
            let h = (hue - 30.0 + i as f32 * step).rem_euclid(360.0);
            hsl_to_rgb_gen(h, 0.7, 0.5)
        })
        .collect()
}

/// Generate a triadic palette: three hues 120° apart, with `n` shades each.
pub fn palette_triadic(hue: f32, n: usize) -> Vec<[f32; 3]> {
    let mut out = Vec::new();
    for k in 0..3 {
        let h = (hue + k as f32 * 120.0) % 360.0;
        out.extend(palette_monochromatic(h, n));
    }
    out
}

/// Generate a gradient palette between two RGB colours.
pub fn palette_gradient(start: [f32; 3], end: [f32; 3], n: usize) -> Vec<[f32; 3]> {
    if n == 0 {
        return vec![];
    }
    (0..n)
        .map(|i| {
            let t = if n == 1 {
                0.0
            } else {
                i as f32 / (n - 1) as f32
            };
            [
                start[0] + (end[0] - start[0]) * t,
                start[1] + (end[1] - start[1]) * t,
                start[2] + (end[2] - start[2]) * t,
            ]
        })
        .collect()
}

/// Generate a rainbow palette with `n` colours evenly spaced in hue.
pub fn palette_rainbow(n: usize) -> Vec<[f32; 3]> {
    if n == 0 {
        return vec![];
    }
    let step = 360.0 / n as f32;
    (0..n)
        .map(|i| hsl_to_rgb_gen(i as f32 * step, 1.0, 0.5))
        .collect()
}

/// Generate a warm-to-cool spectrum palette using cosine-based approach.
pub fn palette_cosine(n: usize) -> Vec<[f32; 3]> {
    if n == 0 {
        return vec![];
    }
    (0..n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let r = 0.5 + 0.5 * (TAU * (t * 0.5 + 0.0)).cos();
            let g = 0.5 + 0.5 * (TAU * (t * 0.5 + 0.333)).cos();
            let b = 0.5 + 0.5 * (TAU * (t * 0.5 + 0.667)).cos();
            [r, g, b]
        })
        .collect()
}

/// Internal: HSL to RGB (self-contained for palette gen module).
fn hsl_to_rgb_gen(h: f32, s: f32, l: f32) -> [f32; 3] {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monochromatic_count() {
        /* returns n colours */
        let p = palette_monochromatic(200.0, 5);
        assert_eq!(p.len(), 5);
    }

    #[test]
    fn test_monochromatic_empty() {
        /* n=0 returns empty */
        assert!(palette_monochromatic(0.0, 0).is_empty());
    }

    #[test]
    fn test_complementary_double() {
        /* complementary returns 2*n colours */
        let p = palette_complementary(60.0, 3);
        assert_eq!(p.len(), 6);
    }

    #[test]
    fn test_analogous_count() {
        /* analogous returns n colours */
        let p = palette_analogous(120.0, 4);
        assert_eq!(p.len(), 4);
    }

    #[test]
    fn test_triadic_count() {
        /* triadic returns 3*n colours */
        let p = palette_triadic(0.0, 2);
        assert_eq!(p.len(), 6);
    }

    #[test]
    fn test_gradient_endpoints() {
        /* first = start, last = end */
        let p = palette_gradient([1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 5);
        assert!((p[0][0] - 1.0).abs() < 1e-5);
        assert!((p[4][2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_rainbow_count() {
        /* rainbow returns n colours */
        let p = palette_rainbow(7);
        assert_eq!(p.len(), 7);
    }

    #[test]
    fn test_cosine_range() {
        /* all values in [0, 1] */
        let p = palette_cosine(10);
        for c in p {
            assert!(c[0] >= 0.0 && c[0] <= 1.0);
            assert!(c[1] >= 0.0 && c[1] <= 1.0);
            assert!(c[2] >= 0.0 && c[2] <= 1.0);
        }
    }
}
