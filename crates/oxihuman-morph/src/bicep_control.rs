// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Bicep morph control — muscle size, peak shape, vein definition,
//! and elbow-flexion correctives.

use std::f32::consts::PI;

/// Bicep morph parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct BicepParams {
    /// Overall muscle size, 0..=1.
    pub size: f32,
    /// Peak height when flexed, 0..=1.
    pub peak: f32,
    /// Vein definition / vascularity, 0..=1.
    pub vascularity: f32,
    /// Bicep head separation visibility, 0..=1.
    pub head_separation: f32,
    /// Elbow flexion angle in radians.
    pub flexion_angle: f32,
    /// Left (false) or right (true) arm.
    pub is_right: bool,
}

impl Default for BicepParams {
    fn default() -> Self {
        Self {
            size: 0.5,
            peak: 0.3,
            vascularity: 0.1,
            head_separation: 0.2,
            flexion_angle: 0.0,
            is_right: false,
        }
    }
}

/// Result of bicep evaluation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BicepResult {
    /// Radial displacements per vertex: (index, displacement).
    pub displacements: Vec<(usize, f32)>,
    /// Estimated cross-section area increase.
    pub area_increase: f32,
    /// Flexion corrective weight.
    pub flexion_weight: f32,
}

/// Muscle belly profile along the upper arm length.
///
/// `t` from 0 (shoulder insertion) to 1 (elbow insertion).
#[allow(dead_code)]
pub fn belly_profile(t: f32, size: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    // Bell curve centred at 55% of upper arm
    let centre = 0.55;
    let sigma = 0.2;
    let g = (-(t - centre) * (t - centre) / (2.0 * sigma * sigma)).exp();
    size * g
}

/// Peak profile for flexed bicep — sharper bump on top.
#[allow(dead_code)]
pub fn peak_profile(t: f32, angle_around: f32, peak: f32, flexion: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let flexion_factor = (flexion / (PI / 2.0)).clamp(0.0, 1.0);

    // Peak at ~55% length, top of arm (angle ~= PI/2)
    let length_factor = (-((t - 0.55) / 0.1).powi(2)).exp();
    let angular_factor = 0.5 * (1.0 + (angle_around - PI / 2.0).cos());

    peak * flexion_factor * length_factor * angular_factor
}

/// Flexion corrective weight (smoothstep).
#[allow(dead_code)]
pub fn flexion_corrective(angle: f32) -> f32 {
    let t = (angle / (PI / 2.0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Vein displacement — sinusoidal pattern along the arm surface.
#[allow(dead_code)]
pub fn vein_displacement(t: f32, angle: f32, vascularity: f32) -> f32 {
    let v = vascularity.clamp(0.0, 1.0);
    if v < 0.01 {
        return 0.0;
    }
    let wave = (t * 8.0 * PI + angle * 2.0).sin();
    v * 0.001 * wave.max(0.0)
}

/// Head separation groove between long and short heads.
#[allow(dead_code)]
pub fn head_separation_groove(angle: f32, separation: f32) -> f32 {
    let s = separation.clamp(0.0, 1.0);
    // Groove at medial/lateral boundaries
    let groove_angle = PI / 2.0;
    let diff = (angle - groove_angle).abs();
    let diff = if diff > PI { 2.0 * PI - diff } else { diff };
    if diff < 0.3 {
        -s * 0.002 * (1.0 - diff / 0.3)
    } else {
        0.0
    }
}

/// Evaluate bicep morph.
///
/// `arm_coords`: per-vertex `(length_t, angle_around_axis, radial_distance)`.
#[allow(dead_code)]
pub fn evaluate_bicep(arm_coords: &[(f32, f32, f32)], params: &BicepParams) -> BicepResult {
    let flex_w = flexion_corrective(params.flexion_angle);
    let mut displacements = Vec::with_capacity(arm_coords.len());
    let mut total_area = 0.0_f32;

    for (i, &(t, angle, _radius)) in arm_coords.iter().enumerate() {
        let belly = belly_profile(t, params.size);
        let pk = peak_profile(t, angle, params.peak, params.flexion_angle);
        let vein = vein_displacement(t, angle, params.vascularity);
        let groove = head_separation_groove(angle, params.head_separation);

        let disp = belly * 0.02 + pk * 0.01 + vein + groove;
        if disp.abs() > 1e-7 {
            displacements.push((i, disp));
            total_area += disp.abs();
        }
    }

    BicepResult {
        displacements,
        area_increase: total_area * 0.01,
        flexion_weight: flex_w,
    }
}

/// Mirror bicep params (swap left/right).
#[allow(dead_code)]
pub fn mirror_bicep(params: &BicepParams) -> BicepParams {
    BicepParams {
        is_right: !params.is_right,
        ..params.clone()
    }
}

/// Blend two bicep param sets.
#[allow(dead_code)]
pub fn blend_bicep_params(a: &BicepParams, b: &BicepParams, t: f32) -> BicepParams {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    BicepParams {
        size: a.size * inv + b.size * t,
        peak: a.peak * inv + b.peak * t,
        vascularity: a.vascularity * inv + b.vascularity * t,
        head_separation: a.head_separation * inv + b.head_separation * t,
        flexion_angle: a.flexion_angle * inv + b.flexion_angle * t,
        is_right: if t < 0.5 { a.is_right } else { b.is_right },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_default_params() {
        let p = BicepParams::default();
        assert!((0.0..=1.0).contains(&p.size));
    }

    #[test]
    fn test_belly_profile_centre() {
        let v = belly_profile(0.55, 1.0);
        assert!(v > 0.9, "Should be near peak at centre, got {v}");
    }

    #[test]
    fn test_belly_profile_edges() {
        let h = belly_profile(0.0, 1.0);
        let t = belly_profile(1.0, 1.0);
        let c = belly_profile(0.55, 1.0);
        assert!(c > h);
        assert!(c > t);
    }

    #[test]
    fn test_flexion_corrective_bounds() {
        let w0 = flexion_corrective(0.0);
        let w1 = flexion_corrective(PI / 2.0);
        assert!(w0.abs() < 1e-5);
        assert!((w1 - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_vein_zero_vascularity() {
        assert_eq!(vein_displacement(0.5, 0.5, 0.0), 0.0);
    }

    #[test]
    fn test_head_separation_outside() {
        let g = head_separation_groove(PI, 1.0);
        assert!(g.abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_empty() {
        let r = evaluate_bicep(&[], &BicepParams::default());
        assert!(r.displacements.is_empty());
    }

    #[test]
    fn test_mirror_bicep() {
        let p = BicepParams {
            is_right: false,
            ..Default::default()
        };
        let m = mirror_bicep(&p);
        assert!(m.is_right);
    }

    #[test]
    fn test_blend_bicep_midpoint() {
        let a = BicepParams {
            size: 0.0,
            ..Default::default()
        };
        let b = BicepParams {
            size: 1.0,
            ..Default::default()
        };
        let r = blend_bicep_params(&a, &b, 0.5);
        assert!((r.size - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_peak_zero_flexion() {
        let pk = peak_profile(0.55, PI / 2.0, 1.0, 0.0);
        assert!(pk.abs() < 1e-6);
    }
}
