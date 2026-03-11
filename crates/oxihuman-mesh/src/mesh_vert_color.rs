// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Per-vertex color operations: painting, blending, and conversion.

/// RGBA color.
#[allow(dead_code)]
pub type Rgba = [f32; 4];

/// Fill all vertices with a uniform color.
#[allow(dead_code)]
pub fn fill_color(n: usize, color: Rgba) -> Vec<Rgba> {
    vec![color; n]
}

/// Blend two color buffers with a given factor.
#[allow(dead_code)]
pub fn blend_colors(a: &[Rgba], b: &[Rgba], factor: f32) -> Vec<Rgba> {
    let n = a.len().min(b.len());
    let f = factor.clamp(0.0, 1.0);
    (0..n).map(|i| {
        [
            a[i][0] * (1.0 - f) + b[i][0] * f,
            a[i][1] * (1.0 - f) + b[i][1] * f,
            a[i][2] * (1.0 - f) + b[i][2] * f,
            a[i][3] * (1.0 - f) + b[i][3] * f,
        ]
    }).collect()
}

/// Convert linear color to sRGB.
#[allow(dead_code)]
pub fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// Convert sRGB to linear.
#[allow(dead_code)]
pub fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Apply gamma to vertex color buffer.
#[allow(dead_code)]
pub fn apply_gamma(colors: &mut [Rgba]) {
    for c in colors.iter_mut() {
        c[0] = linear_to_srgb(c[0].clamp(0.0, 1.0));
        c[1] = linear_to_srgb(c[1].clamp(0.0, 1.0));
        c[2] = linear_to_srgb(c[2].clamp(0.0, 1.0));
    }
}

/// Paint a sphere region: set color for vertices within radius of center.
#[allow(dead_code)]
pub fn paint_sphere(
    positions: &[[f32; 3]],
    colors: &mut [Rgba],
    center: [f32; 3],
    radius: f32,
    color: Rgba,
) -> usize {
    let r2 = radius * radius;
    let mut count = 0usize;
    for (i, p) in positions.iter().enumerate() {
        let dx = p[0] - center[0];
        let dy = p[1] - center[1];
        let dz = p[2] - center[2];
        if dx * dx + dy * dy + dz * dz <= r2 {
            colors[i] = color;
            count += 1;
        }
    }
    count
}

/// Color to u8 bytes.
#[allow(dead_code)]
pub fn rgba_to_u8(c: Rgba) -> [u8; 4] {
    [
        (c[0].clamp(0.0, 1.0) * 255.0) as u8,
        (c[1].clamp(0.0, 1.0) * 255.0) as u8,
        (c[2].clamp(0.0, 1.0) * 255.0) as u8,
        (c[3].clamp(0.0, 1.0) * 255.0) as u8,
    ]
}

/// U8 bytes to float color.
#[allow(dead_code)]
pub fn u8_to_rgba(c: [u8; 4]) -> Rgba {
    [
        c[0] as f32 / 255.0,
        c[1] as f32 / 255.0,
        c[2] as f32 / 255.0,
        c[3] as f32 / 255.0,
    ]
}

/// Average color of the entire buffer.
#[allow(dead_code)]
pub fn average_color(colors: &[Rgba]) -> Rgba {
    if colors.is_empty() { return [0.0; 4]; }
    let n = colors.len() as f32;
    let mut sum = [0.0f32; 4];
    for c in colors {
        sum[0] += c[0]; sum[1] += c[1]; sum[2] += c[2]; sum[3] += c[3];
    }
    [sum[0] / n, sum[1] / n, sum[2] / n, sum[3] / n]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill_color() {
        let colors = fill_color(5, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(colors.len(), 5);
        assert!((colors[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_blend_half() {
        let a = vec![[1.0, 0.0, 0.0, 1.0]];
        let b = vec![[0.0, 1.0, 0.0, 1.0]];
        let c = blend_colors(&a, &b, 0.5);
        assert!((c[0][0] - 0.5).abs() < 1e-5);
        assert!((c[0][1] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_linear_srgb_roundtrip() {
        let val = 0.5f32;
        let srgb = linear_to_srgb(val);
        let back = srgb_to_linear(srgb);
        assert!((back - val).abs() < 1e-4);
    }

    #[test]
    fn test_paint_sphere() {
        let pos = vec![[0.0; 3], [0.1, 0.0, 0.0], [10.0, 0.0, 0.0]];
        let mut colors = fill_color(3, [0.0; 4]);
        let count = paint_sphere(&pos, &mut colors, [0.0; 3], 1.0, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_rgba_to_u8() {
        let bytes = rgba_to_u8([1.0, 0.5, 0.0, 1.0]);
        assert_eq!(bytes[0], 255);
        assert_eq!(bytes[2], 0);
    }

    #[test]
    fn test_u8_to_rgba() {
        let c = u8_to_rgba([255, 0, 128, 255]);
        assert!((c[0] - 1.0).abs() < 1e-2);
        assert!((c[1] - 0.0).abs() < 1e-2);
    }

    #[test]
    fn test_average_color() {
        let colors = vec![[1.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, 1.0]];
        let avg = average_color(&colors);
        assert!((avg[0] - 0.5).abs() < 1e-5);
        assert!((avg[1] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_apply_gamma_no_crash() {
        let mut colors = vec![[0.5, 0.5, 0.5, 1.0]];
        apply_gamma(&mut colors);
        assert!(colors[0][0] > 0.0);
    }

    #[test]
    fn test_blend_zero_factor() {
        let a = vec![[1.0, 0.0, 0.0, 1.0]];
        let b = vec![[0.0, 1.0, 0.0, 1.0]];
        let c = blend_colors(&a, &b, 0.0);
        assert!((c[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_average_empty() {
        let avg = average_color(&[]);
        assert!((avg[0] - 0.0).abs() < 1e-5);
    }

}
