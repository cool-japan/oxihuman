// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Temporal anti-aliasing (TAA) — accumulate subpixel samples across frames
//! with motion vector-based reprojection.

use std::f32::consts::PI;

/// TAA configuration.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct TaaConfig {
    /// Blend factor for history (0 = current only, 1 = history only).
    pub blend_factor: f32,
    /// Neighbourhood clamp mode.
    pub clamp_mode: ClampMode,
    /// Subpixel jitter sequence length.
    pub jitter_count: u32,
    /// Sharpening amount after TAA.
    pub sharpening: f32,
}

impl Default for TaaConfig {
    fn default() -> Self {
        Self {
            blend_factor: 0.9,
            clamp_mode: ClampMode::Aabb,
            jitter_count: 16,
            sharpening: 0.1,
        }
    }
}

/// Neighbourhood clamping mode.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClampMode {
    /// Simple min/max clamp.
    Aabb,
    /// Variance-based clamp.
    Variance,
    /// No clamping.
    None,
}

/// Generate Halton(2,3) jitter offset for frame `index`.
///
/// Returns `(dx, dy)` in -0.5..0.5 pixel range.
#[allow(dead_code)]
pub fn halton_jitter(index: u32) -> (f32, f32) {
    let x = halton(index, 2) - 0.5;
    let y = halton(index, 3) - 0.5;
    (x, y)
}

/// Halton sequence for base `b`.
#[allow(dead_code)]
pub fn halton(mut index: u32, base: u32) -> f32 {
    let mut f = 1.0_f32;
    let mut r = 0.0_f32;
    let b = base as f32;
    while index > 0 {
        f /= b;
        r += f * (index % base) as f32;
        index /= base;
    }
    r
}

/// Colour-space AABB clamp for neighbourhood rejection.
///
/// Clamps `history` to the min/max of `neighbourhood` (array of RGB values).
#[allow(dead_code)]
pub fn aabb_clamp(history: [f32; 3], neighbourhood: &[[f32; 3]]) -> [f32; 3] {
    if neighbourhood.is_empty() {
        return history;
    }
    let mut min_c = [f32::MAX; 3];
    let mut max_c = [f32::MIN; 3];
    for c in neighbourhood {
        for i in 0..3 {
            min_c[i] = min_c[i].min(c[i]);
            max_c[i] = max_c[i].max(c[i]);
        }
    }
    [
        history[0].clamp(min_c[0], max_c[0]),
        history[1].clamp(min_c[1], max_c[1]),
        history[2].clamp(min_c[2], max_c[2]),
    ]
}

/// Variance-based clamp.
#[allow(dead_code)]
pub fn variance_clamp(history: [f32; 3], neighbourhood: &[[f32; 3]], gamma: f32) -> [f32; 3] {
    if neighbourhood.is_empty() {
        return history;
    }
    let n = neighbourhood.len() as f32;
    let mut mean = [0.0_f32; 3];
    let mut mean_sq = [0.0_f32; 3];
    for c in neighbourhood {
        for i in 0..3 {
            mean[i] += c[i];
            mean_sq[i] += c[i] * c[i];
        }
    }
    let mut result = [0.0_f32; 3];
    for i in 0..3 {
        mean[i] /= n;
        mean_sq[i] /= n;
        let variance = (mean_sq[i] - mean[i] * mean[i]).max(0.0);
        let std_dev = variance.sqrt();
        let lo = mean[i] - gamma * std_dev;
        let hi = mean[i] + gamma * std_dev;
        result[i] = history[i].clamp(lo, hi);
    }
    result
}

/// Blend current and history samples.
#[allow(dead_code)]
pub fn blend_taa(current: [f32; 3], history: [f32; 3], factor: f32) -> [f32; 3] {
    let f = factor.clamp(0.0, 1.0);
    [
        current[0] * (1.0 - f) + history[0] * f,
        current[1] * (1.0 - f) + history[1] * f,
        current[2] * (1.0 - f) + history[2] * f,
    ]
}

/// Simple sharpening filter (unsharp mask).
///
/// `centre` is the TAA result, `neighbours_avg` is the average of surrounding pixels.
#[allow(dead_code)]
pub fn sharpen(centre: [f32; 3], neighbours_avg: [f32; 3], amount: f32) -> [f32; 3] {
    [
        (centre[0] + (centre[0] - neighbours_avg[0]) * amount).clamp(0.0, 1.0),
        (centre[1] + (centre[1] - neighbours_avg[1]) * amount).clamp(0.0, 1.0),
        (centre[2] + (centre[2] - neighbours_avg[2]) * amount).clamp(0.0, 1.0),
    ]
}

/// Reproject UV using motion vector.
#[allow(dead_code)]
pub fn reproject_uv(uv: (f32, f32), motion: (f32, f32)) -> (f32, f32) {
    (uv.0 - motion.0, uv.1 - motion.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = TaaConfig::default();
        assert!((0.0..=1.0).contains(&c.blend_factor));
    }

    #[test]
    fn test_halton_jitter_range() {
        for i in 0..16 {
            let (x, y) = halton_jitter(i);
            assert!((-0.5..=0.5).contains(&x));
            assert!((-0.5..=0.5).contains(&y));
        }
    }

    #[test]
    fn test_halton_zero() {
        assert!(halton(0, 2).abs() < 1e-5);
    }

    #[test]
    fn test_aabb_clamp_inside() {
        let h = [0.5, 0.5, 0.5];
        let n = [[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]];
        let r = aabb_clamp(h, &n);
        assert!((r[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_aabb_clamp_outside() {
        let h = [2.0, -1.0, 0.5];
        let n = [[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]];
        let r = aabb_clamp(h, &n);
        assert!((r[0] - 1.0).abs() < 1e-5);
        assert!(r[1].abs() < 1e-5);
    }

    #[test]
    fn test_aabb_clamp_empty() {
        let r = aabb_clamp([0.5, 0.5, 0.5], &[]);
        assert!((r[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_blend_taa_zero() {
        let r = blend_taa([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.0);
        assert!((r[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_blend_taa_one() {
        let r = blend_taa([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 1.0);
        assert!((r[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_sharpen_no_amount() {
        let r = sharpen([0.5, 0.5, 0.5], [0.4, 0.4, 0.4], 0.0);
        assert!((r[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_reproject_uv() {
        let (u, v) = reproject_uv((0.5, 0.5), (0.1, -0.1));
        assert!((u - 0.4).abs() < 1e-5);
        assert!((v - 0.6).abs() < 1e-5);
    }
}
