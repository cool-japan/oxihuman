// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Trapezius muscle morph control — upper/middle/lower trapezius,
//! neck-to-shoulder slope, and shrug correctives.

use std::f32::consts::PI;

/// Trapezius morph parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct TrapeziusParams {
    /// Overall trapezius size, 0..=1.
    pub size: f32,
    /// Upper trapezius emphasis (neck slope), 0..=1.
    pub upper: f32,
    /// Middle trapezius emphasis, 0..=1.
    pub middle: f32,
    /// Lower trapezius emphasis, 0..=1.
    pub lower: f32,
    /// Neck-to-shoulder slope angle (steeper = more developed), 0..=1.
    pub neck_slope: f32,
    /// Shoulder shrug amount in radians.
    pub shrug_angle: f32,
}

impl Default for TrapeziusParams {
    fn default() -> Self {
        Self {
            size: 0.5,
            upper: 0.4,
            middle: 0.3,
            lower: 0.3,
            neck_slope: 0.3,
            shrug_angle: 0.0,
        }
    }
}

/// Evaluation result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TrapeziusResult {
    pub displacements: Vec<(usize, f32)>,
    pub shrug_corrective: f32,
    pub total_volume: f32,
}

/// Upper trap profile — from base of skull to shoulder.
///
/// `t` from 0 (neck/skull) to 1 (shoulder tip).
#[allow(dead_code)]
pub fn upper_trap_profile(t: f32, emphasis: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    // Tapers from neck to shoulder
    emphasis * (1.0 - t * 0.7)
}

/// Middle trap profile — across upper back.
///
/// `t` from 0 (spine) to 1 (lateral edge).
#[allow(dead_code)]
pub fn middle_trap_profile(t: f32, emphasis: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let peak = 0.4;
    let sigma = 0.25;
    emphasis * (-(t - peak).powi(2) / (2.0 * sigma * sigma)).exp()
}

/// Lower trap profile — descending from mid-back to thoracic spine.
///
/// `v` from 0 (upper) to 1 (lower).
#[allow(dead_code)]
pub fn lower_trap_profile(v: f32, emphasis: f32) -> f32 {
    let v = v.clamp(0.0, 1.0);
    emphasis * v.powf(0.5) * (1.0 - v)
}

/// Neck slope: determines the curvature from neck to shoulder.
#[allow(dead_code)]
pub fn neck_slope_offset(lateral_t: f32, slope: f32) -> f32 {
    let t = lateral_t.clamp(0.0, 1.0);
    // Higher slope = more convex (muscle bulk)
    slope * 0.015 * (1.0 - t) * (1.0 - t)
}

/// Shrug corrective (smoothstep).
#[allow(dead_code)]
pub fn shrug_corrective(angle: f32) -> f32 {
    let t = (angle / (PI / 6.0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Evaluate trapezius morph.
///
/// `trap_coords`: `(lateral_t, vertical_v, depth)` per vertex.
#[allow(dead_code)]
pub fn evaluate_trapezius(
    trap_coords: &[(f32, f32, f32)],
    params: &TrapeziusParams,
) -> TrapeziusResult {
    let shrug_w = shrug_corrective(params.shrug_angle);
    let mut disps = Vec::with_capacity(trap_coords.len());
    let mut total_vol = 0.0_f32;

    for (i, &(lat, vert, _depth)) in trap_coords.iter().enumerate() {
        let up = upper_trap_profile(lat, params.upper) * (1.0 - vert).max(0.0);
        let mid = middle_trap_profile(lat, params.middle) * (-((vert - 0.4) / 0.2).powi(2)).exp();
        let low = lower_trap_profile(vert, params.lower) * (1.0 - lat).max(0.0);
        let slope = neck_slope_offset(lat, params.neck_slope) * (1.0 - vert).max(0.0);

        let disp = params.size * (up + mid + low) * 0.015 + slope;
        if disp.abs() > 1e-7 {
            disps.push((i, disp));
            total_vol += disp.abs();
        }
    }

    TrapeziusResult {
        displacements: disps,
        shrug_corrective: shrug_w,
        total_volume: total_vol * 0.001,
    }
}

/// Blend trapezius params.
#[allow(dead_code)]
pub fn blend_trap_params(a: &TrapeziusParams, b: &TrapeziusParams, t: f32) -> TrapeziusParams {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    TrapeziusParams {
        size: a.size * inv + b.size * t,
        upper: a.upper * inv + b.upper * t,
        middle: a.middle * inv + b.middle * t,
        lower: a.lower * inv + b.lower * t,
        neck_slope: a.neck_slope * inv + b.neck_slope * t,
        shrug_angle: a.shrug_angle * inv + b.shrug_angle * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_default_params() {
        let p = TrapeziusParams::default();
        assert!((0.0..=1.0).contains(&p.size));
        let sum = p.upper + p.middle + p.lower;
        assert!((sum - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_upper_trap_at_neck() {
        let v = upper_trap_profile(0.0, 1.0);
        assert!((v - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_upper_trap_tapers() {
        let neck = upper_trap_profile(0.0, 1.0);
        let shoulder = upper_trap_profile(1.0, 1.0);
        assert!(neck > shoulder);
    }

    #[test]
    fn test_middle_trap_peak() {
        let v = middle_trap_profile(0.4, 1.0);
        assert!(v > 0.9);
    }

    #[test]
    fn test_lower_trap_endpoints() {
        let top = lower_trap_profile(0.0, 1.0);
        let bottom = lower_trap_profile(1.0, 1.0);
        assert!(top.abs() < 1e-6);
        assert!(bottom.abs() < 1e-6);
    }

    #[test]
    fn test_neck_slope_at_neck() {
        let v = neck_slope_offset(0.0, 1.0);
        assert!(v > 0.0);
    }

    #[test]
    fn test_shrug_corrective_bounds() {
        let w0 = shrug_corrective(0.0);
        let w1 = shrug_corrective(PI / 6.0);
        assert!(w0.abs() < 1e-5);
        assert!((w1 - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_evaluate_empty() {
        let r = evaluate_trapezius(&[], &TrapeziusParams::default());
        assert!(r.displacements.is_empty());
    }

    #[test]
    fn test_evaluate_produces_output() {
        let coords = vec![(0.2, 0.1, 0.05), (0.5, 0.4, 0.05)];
        let params = TrapeziusParams {
            size: 1.0,
            ..Default::default()
        };
        let r = evaluate_trapezius(&coords, &params);
        assert!(!r.displacements.is_empty());
    }

    #[test]
    fn test_blend_trap_midpoint() {
        let a = TrapeziusParams {
            size: 0.0,
            ..Default::default()
        };
        let b = TrapeziusParams {
            size: 1.0,
            ..Default::default()
        };
        let r = blend_trap_params(&a, &b, 0.5);
        assert!((r.size - 0.5).abs() < 1e-6);
    }
}
