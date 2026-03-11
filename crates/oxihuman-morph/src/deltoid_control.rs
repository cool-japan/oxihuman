// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Deltoid muscle morph control — anterior, lateral, and posterior heads,
//! with arm-raise correctives.

use std::f32::consts::PI;

/// Deltoid morph parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct DeltoidParams {
    /// Overall deltoid size, 0..=1.
    pub size: f32,
    /// Anterior (front) head emphasis, 0..=1.
    pub anterior: f32,
    /// Lateral (side) head emphasis, 0..=1.
    pub lateral: f32,
    /// Posterior (rear) head emphasis, 0..=1.
    pub posterior: f32,
    /// Arm abduction angle in radians (0 = arm down).
    pub abduction_angle: f32,
    /// Separation between heads, 0..=1.
    pub head_definition: f32,
}

impl Default for DeltoidParams {
    fn default() -> Self {
        Self {
            size: 0.5,
            anterior: 0.33,
            lateral: 0.34,
            posterior: 0.33,
            abduction_angle: 0.0,
            head_definition: 0.2,
        }
    }
}

/// Deltoid evaluation result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DeltoidResult {
    pub displacements: Vec<(usize, f32)>,
    pub abduction_corrective: f32,
    pub total_volume: f32,
}

/// Profile of the deltoid along its height (shoulder cap to arm insertion).
///
/// `t` from 0 (acromion) to 1 (deltoid tuberosity).
#[allow(dead_code)]
pub fn height_profile(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    // Deltoid is widest at ~30% from top
    let peak = 0.3;
    let sigma = 0.25;
    (-(t - peak).powi(2) / (2.0 * sigma * sigma)).exp()
}

/// Angular profile for a specific head.
///
/// `theta` is the angle around the shoulder axis, `head_centre` is the
/// angular centre of that deltoid head, `spread` is the angular width.
#[allow(dead_code)]
pub fn head_angular_profile(theta: f32, head_centre: f32, spread: f32) -> f32 {
    if spread <= 0.0 {
        return 0.0;
    }
    let diff = (theta - head_centre).abs();
    let diff = if diff > PI { 2.0 * PI - diff } else { diff };
    let t = (diff / spread).clamp(0.0, 1.0);
    0.5 * (1.0 + (PI * t).cos())
}

/// Abduction corrective weight (smoothstep).
#[allow(dead_code)]
pub fn abduction_corrective(angle: f32) -> f32 {
    let t = (angle / (PI / 2.0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Separation groove between heads.
#[allow(dead_code)]
pub fn separation_groove(theta: f32, definition: f32) -> f32 {
    let d = definition.clamp(0.0, 1.0);
    // Grooves at the boundaries between heads
    let boundaries = [0.0, 2.0 * PI / 3.0, 4.0 * PI / 3.0];
    let mut min_dist = f32::MAX;
    for &b in &boundaries {
        let diff = (theta - b).abs();
        let diff = if diff > PI { 2.0 * PI - diff } else { diff };
        min_dist = min_dist.min(diff);
    }
    if min_dist < 0.15 {
        -d * 0.003 * (1.0 - min_dist / 0.15)
    } else {
        0.0
    }
}

/// Evaluate the three-headed deltoid morph.
///
/// `shoulder_coords`: per-vertex `(height_t, angle_theta, radial_dist)`.
#[allow(dead_code)]
pub fn evaluate_deltoid(
    shoulder_coords: &[(f32, f32, f32)],
    params: &DeltoidParams,
) -> DeltoidResult {
    let abd_w = abduction_corrective(params.abduction_angle);
    let head_centres = [0.0_f32, 2.0 * PI / 3.0, 4.0 * PI / 3.0];
    let head_weights = [params.anterior, params.lateral, params.posterior];
    let spread = PI / 3.0;

    let mut displacements = Vec::with_capacity(shoulder_coords.len());
    let mut total_vol = 0.0_f32;

    for (i, &(ht, theta, _radius)) in shoulder_coords.iter().enumerate() {
        let hp = height_profile(ht);
        let mut angular = 0.0_f32;
        for (j, &centre) in head_centres.iter().enumerate() {
            angular += head_weights[j] * head_angular_profile(theta, centre, spread);
        }
        let groove = separation_groove(theta, params.head_definition);
        let disp = params.size * hp * angular * 0.02 + groove;

        if disp.abs() > 1e-7 {
            displacements.push((i, disp));
            total_vol += disp.abs();
        }
    }

    DeltoidResult {
        displacements,
        abduction_corrective: abd_w,
        total_volume: total_vol * 0.001,
    }
}

/// Blend deltoid params.
#[allow(dead_code)]
pub fn blend_deltoid_params(a: &DeltoidParams, b: &DeltoidParams, t: f32) -> DeltoidParams {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    DeltoidParams {
        size: a.size * inv + b.size * t,
        anterior: a.anterior * inv + b.anterior * t,
        lateral: a.lateral * inv + b.lateral * t,
        posterior: a.posterior * inv + b.posterior * t,
        abduction_angle: a.abduction_angle * inv + b.abduction_angle * t,
        head_definition: a.head_definition * inv + b.head_definition * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_default_params() {
        let p = DeltoidParams::default();
        let sum = p.anterior + p.lateral + p.posterior;
        assert!((sum - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_height_profile_peak() {
        let v = height_profile(0.3);
        assert!(v > 0.95);
    }

    #[test]
    fn test_height_profile_edges() {
        let h0 = height_profile(0.0);
        let h1 = height_profile(1.0);
        let hp = height_profile(0.3);
        assert!(hp > h0);
        assert!(hp > h1);
    }

    #[test]
    fn test_head_angular_centre() {
        let v = head_angular_profile(0.5, 0.5, 1.0);
        assert!((v - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_head_angular_zero_spread() {
        assert_eq!(head_angular_profile(0.0, 0.0, 0.0), 0.0);
    }

    #[test]
    fn test_abduction_corrective_bounds() {
        let w0 = abduction_corrective(0.0);
        let w1 = abduction_corrective(PI / 2.0);
        assert!(w0.abs() < 1e-5);
        assert!((w1 - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_separation_groove_far() {
        let g = separation_groove(PI / 3.0, 1.0);
        // Should be far from boundaries
        assert!(g.abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_empty() {
        let r = evaluate_deltoid(&[], &DeltoidParams::default());
        assert!(r.displacements.is_empty());
    }

    #[test]
    fn test_blend_deltoid_midpoint() {
        let a = DeltoidParams { size: 0.0, ..Default::default() };
        let b = DeltoidParams { size: 1.0, ..Default::default() };
        let r = blend_deltoid_params(&a, &b, 0.5);
        assert!((r.size - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_produces_output() {
        let coords = vec![(0.3, 0.0, 0.05), (0.3, 2.0, 0.05), (0.3, 4.0, 0.05)];
        let params = DeltoidParams { size: 1.0, ..Default::default() };
        let r = evaluate_deltoid(&coords, &params);
        assert!(!r.displacements.is_empty());
    }
}
