// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Body proportion scaling utilities for character morphing.

use std::f32::consts::PI;

/// Scale region enumeration.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScaleRegion {
    Head,
    Torso,
    Arms,
    Legs,
    Hands,
    Feet,
}

/// Parameters for body proportion scaling.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ProportionScaleParams {
    pub head_scale: f32,
    pub torso_scale: f32,
    pub arm_scale: f32,
    pub leg_scale: f32,
    pub hand_scale: f32,
    pub foot_scale: f32,
    pub overall_height: f32,
}

/// Result of proportion scaling.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ProportionScaleResult {
    pub weights: [f32; 6],
    pub total_height: f32,
    pub ratio_head_to_body: f32,
}

/// Create default proportion scale parameters.
#[allow(dead_code)]
pub fn default_proportion_scale() -> ProportionScaleParams {
    ProportionScaleParams {
        head_scale: 1.0,
        torso_scale: 1.0,
        arm_scale: 1.0,
        leg_scale: 1.0,
        hand_scale: 1.0,
        foot_scale: 1.0,
        overall_height: 1.75,
    }
}

/// Clamp a scale value to valid range.
#[allow(dead_code)]
pub fn clamp_scale(value: f32) -> f32 {
    value.clamp(0.5, 2.0)
}

/// Compute the proportion weights from parameters.
#[allow(dead_code)]
pub fn compute_proportion_weights(params: &ProportionScaleParams) -> [f32; 6] {
    [
        clamp_scale(params.head_scale),
        clamp_scale(params.torso_scale),
        clamp_scale(params.arm_scale),
        clamp_scale(params.leg_scale),
        clamp_scale(params.hand_scale),
        clamp_scale(params.foot_scale),
    ]
}

/// Evaluate proportion scale result.
#[allow(dead_code)]
pub fn evaluate_proportion_scale(params: &ProportionScaleParams) -> ProportionScaleResult {
    let weights = compute_proportion_weights(params);
    let total_height = params.overall_height * weights[1] * 0.5 + params.overall_height * weights[3] * 0.5;
    let ratio = weights[0] / (weights[1] + weights[3]).max(0.001);
    ProportionScaleResult {
        weights,
        total_height,
        ratio_head_to_body: ratio,
    }
}

/// Blend two proportion params by factor t in [0,1].
#[allow(dead_code)]
pub fn blend_proportion_scales(a: &ProportionScaleParams, b: &ProportionScaleParams, t: f32) -> ProportionScaleParams {
    let t = t.clamp(0.0, 1.0);
    ProportionScaleParams {
        head_scale: a.head_scale + (b.head_scale - a.head_scale) * t,
        torso_scale: a.torso_scale + (b.torso_scale - a.torso_scale) * t,
        arm_scale: a.arm_scale + (b.arm_scale - a.arm_scale) * t,
        leg_scale: a.leg_scale + (b.leg_scale - a.leg_scale) * t,
        hand_scale: a.hand_scale + (b.hand_scale - a.hand_scale) * t,
        foot_scale: a.foot_scale + (b.foot_scale - a.foot_scale) * t,
        overall_height: a.overall_height + (b.overall_height - a.overall_height) * t,
    }
}

/// Get the scale for a specific region.
#[allow(dead_code)]
pub fn region_scale(params: &ProportionScaleParams, region: ScaleRegion) -> f32 {
    match region {
        ScaleRegion::Head => params.head_scale,
        ScaleRegion::Torso => params.torso_scale,
        ScaleRegion::Arms => params.arm_scale,
        ScaleRegion::Legs => params.leg_scale,
        ScaleRegion::Hands => params.hand_scale,
        ScaleRegion::Feet => params.foot_scale,
    }
}

/// Normalize proportions so they average to 1.0.
#[allow(dead_code)]
pub fn normalize_proportions(params: &mut ProportionScaleParams) {
    let sum = params.head_scale + params.torso_scale + params.arm_scale
        + params.leg_scale + params.hand_scale + params.foot_scale;
    if sum > 0.001 {
        let inv = 6.0 / sum;
        params.head_scale *= inv;
        params.torso_scale *= inv;
        params.arm_scale *= inv;
        params.leg_scale *= inv;
        params.hand_scale *= inv;
        params.foot_scale *= inv;
    }
}

/// Compute a sinusoidal variation for proportion animation.
#[allow(dead_code)]
pub fn sinusoidal_proportion_variation(base: f32, amplitude: f32, phase: f32) -> f32 {
    base + amplitude * (phase * PI * 2.0).sin()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_proportion_scale() {
        let p = default_proportion_scale();
        assert!((p.head_scale - 1.0).abs() < 1e-6);
        assert!((p.overall_height - 1.75).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_scale() {
        assert!((clamp_scale(0.3) - 0.5).abs() < 1e-6);
        assert!((clamp_scale(3.0) - 2.0).abs() < 1e-6);
        assert!((clamp_scale(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_compute_proportion_weights() {
        let p = default_proportion_scale();
        let w = compute_proportion_weights(&p);
        for v in &w {
            assert!((*v - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn test_evaluate_proportion_scale() {
        let p = default_proportion_scale();
        let r = evaluate_proportion_scale(&p);
        assert!(r.total_height > 0.0);
        assert!(r.ratio_head_to_body > 0.0);
    }

    #[test]
    fn test_blend_proportion_scales() {
        let a = default_proportion_scale();
        let mut b = default_proportion_scale();
        b.head_scale = 2.0;
        let c = blend_proportion_scales(&a, &b, 0.5);
        assert!((c.head_scale - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_region_scale() {
        let p = default_proportion_scale();
        assert!((region_scale(&p, ScaleRegion::Head) - 1.0).abs() < 1e-6);
        assert!((region_scale(&p, ScaleRegion::Feet) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_proportions() {
        let mut p = default_proportion_scale();
        p.head_scale = 2.0;
        normalize_proportions(&mut p);
        let sum = p.head_scale + p.torso_scale + p.arm_scale
            + p.leg_scale + p.hand_scale + p.foot_scale;
        assert!((sum - 6.0).abs() < 1e-4);
    }

    #[test]
    fn test_sinusoidal_variation() {
        let v = sinusoidal_proportion_variation(1.0, 0.0, 0.0);
        assert!((v - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_blend_at_extremes() {
        let a = default_proportion_scale();
        let b = default_proportion_scale();
        let c0 = blend_proportion_scales(&a, &b, 0.0);
        let c1 = blend_proportion_scales(&a, &b, 1.0);
        assert!((c0.head_scale - a.head_scale).abs() < 1e-6);
        assert!((c1.head_scale - b.head_scale).abs() < 1e-6);
    }

    #[test]
    fn test_weights_len() {
        let p = default_proportion_scale();
        let w = compute_proportion_weights(&p);
        assert_eq!(w.len(), 6);
    }
}
