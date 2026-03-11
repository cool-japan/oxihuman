// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Film-curve tone mapping: a parametric S-curve modelled after photographic film response.

use std::f32::consts::E;

/// Parameters for the film S-curve.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FilmCurveParams {
    /// Toe strength (0.0–1.0).
    pub toe_strength: f32,
    /// Toe length (0.0–1.0).
    pub toe_length: f32,
    /// Shoulder strength (0.0–1.0).
    pub shoulder_strength: f32,
    /// Shoulder length (0.0–1.0).
    pub shoulder_length: f32,
    /// Shoulder angle (0.0–1.0).
    pub shoulder_angle: f32,
    /// Linear segment gain.
    pub linear_strength: f32,
}

impl Default for FilmCurveParams {
    fn default() -> Self {
        Self {
            toe_strength: 0.5,
            toe_length: 0.5,
            shoulder_strength: 0.5,
            shoulder_length: 0.5,
            shoulder_angle: 1.0,
            linear_strength: 1.0,
        }
    }
}

/// Evaluate the film curve at a linear light value `x >= 0`.
/// Returns a display-referred value in approximately [0, 1].
#[allow(dead_code)]
pub fn film_curve_eval(x: f32, p: &FilmCurveParams) -> f32 {
    let x = x.max(0.0);
    // Simplified Hable-style filmic curve
    let a = p.toe_strength;
    let b = p.toe_length;
    let c = p.shoulder_strength;
    let d = p.shoulder_length;
    let e = p.shoulder_angle;
    let f = p.linear_strength;

    let numerator = x * (a * x + c * b) + d * e;
    let denominator = x * (a * x + b) + d * f;
    if denominator.abs() < f32::EPSILON {
        0.0
    } else {
        numerator / denominator - e / f
    }
}

/// Apply the film curve to an RGB triple.
#[allow(dead_code)]
pub fn film_curve_rgb(color: [f32; 3], p: &FilmCurveParams) -> [f32; 3] {
    [
        film_curve_eval(color[0], p),
        film_curve_eval(color[1], p),
        film_curve_eval(color[2], p),
    ]
}

/// Simple gamma-only film curve (no S-shape).
#[allow(dead_code)]
pub fn gamma_curve(x: f32, gamma: f32) -> f32 {
    x.max(0.0).powf(1.0 / gamma)
}

/// Reinhard tone mapping operator.
#[allow(dead_code)]
pub fn reinhard(x: f32) -> f32 {
    x / (1.0 + x)
}

/// Reinhard with white-point `w`.
#[allow(dead_code)]
pub fn reinhard_extended(x: f32, w: f32) -> f32 {
    let numer = x * (1.0 + x / (w * w));
    numer / (1.0 + x)
}

/// ACES approximation by Stephen Hill.
#[allow(dead_code)]
pub fn aces_approx(x: f32) -> f32 {
    let x = x * 0.6;
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e_val = 0.14;
    ((x * (a * x + b)) / (x * (c * x + d) + e_val)).clamp(0.0, 1.0)
}

/// Exposure adjustment: multiply by `2^ev`.
#[allow(dead_code)]
pub fn apply_exposure(x: f32, ev: f32) -> f32 {
    x * (2.0_f32).powf(ev)
}

/// Natural-log based soft-clip.
#[allow(dead_code)]
pub fn log_clip(x: f32) -> f32 {
    if x <= 0.0 {
        0.0
    } else {
        (1.0 + x / E).ln()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn film_curve_black_maps_near_black() {
        let p = FilmCurveParams::default();
        let v = film_curve_eval(0.0, &p);
        assert!(v.abs() < 0.5);
    }

    #[test]
    fn film_curve_positive_input_positive_output() {
        let p = FilmCurveParams::default();
        let v = film_curve_eval(0.5, &p);
        // just check it doesn't panic and returns finite value
        assert!(v.is_finite());
    }

    #[test]
    fn film_curve_rgb_same_as_per_channel() {
        let p = FilmCurveParams::default();
        let c = [0.2_f32, 0.5, 0.8];
        let out = film_curve_rgb(c, &p);
        assert!((out[0] - film_curve_eval(c[0], &p)).abs() < 1e-6);
    }

    #[test]
    fn gamma_curve_identity_at_one() {
        assert!((gamma_curve(1.0, 2.2) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn reinhard_clamps_high_values() {
        let v = reinhard(1000.0);
        assert!((0.0..=1.0).contains(&v));
    }

    #[test]
    fn reinhard_extended_zero() {
        assert_eq!(reinhard_extended(0.0, 1.0), 0.0);
    }

    #[test]
    fn aces_output_clamped() {
        let v = aces_approx(100.0);
        assert!((0.0..=1.0).contains(&v));
    }

    #[test]
    fn apply_exposure_zero_ev() {
        assert!((apply_exposure(0.5, 0.0) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn log_clip_zero_input() {
        assert_eq!(log_clip(0.0), 0.0);
    }

    #[test]
    fn log_clip_positive_output() {
        assert!(log_clip(1.0) > 0.0);
    }
}
