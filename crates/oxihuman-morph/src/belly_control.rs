// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Belly/abdomen shape morph control.

#![allow(dead_code)]

use std::f32::consts::PI;

/// Belly/abdomen shape parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BellyControl {
    pub protrusion: f32,
    pub width: f32,
    pub height: f32,
}

/// Returns a default `BellyControl`.
#[allow(dead_code)]
pub fn default_belly_control() -> BellyControl {
    BellyControl {
        protrusion: 0.0,
        width: 0.5,
        height: 0.5,
    }
}

/// Applies belly control values to a weight slice.
/// Indices: [0] = protrusion, [1] = width, [2] = height.
#[allow(dead_code)]
pub fn apply_belly_control(weights: &mut [f32], bc: &BellyControl) {
    if !weights.is_empty() {
        weights[0] = bc.protrusion;
    }
    if weights.len() > 1 {
        weights[1] = bc.width;
    }
    if weights.len() > 2 {
        weights[2] = bc.height;
    }
}

/// Linearly blends two `BellyControl` values by parameter `t` in [0, 1].
#[allow(dead_code)]
pub fn belly_control_blend(a: &BellyControl, b: &BellyControl, t: f32) -> BellyControl {
    let t = t.clamp(0.0, 1.0);
    BellyControl {
        protrusion: a.protrusion + (b.protrusion - a.protrusion) * t,
        width: a.width + (b.width - a.width) * t,
        height: a.height + (b.height - a.height) * t,
    }
}

/// Approximate belly volume as a half-ellipsoid.
#[allow(dead_code)]
pub fn belly_volume_approx(bc: &BellyControl) -> f32 {
    // half-ellipsoid: (2/3) * pi * a * b * c
    (2.0 / 3.0) * PI * bc.protrusion * bc.width * bc.height
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_belly_control() {
        let bc = default_belly_control();
        assert!((bc.protrusion - 0.0).abs() < 1e-6);
        assert!((bc.width - 0.5).abs() < 1e-6);
        assert!((bc.height - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_apply_belly_full() {
        let bc = BellyControl { protrusion: 0.3, width: 0.6, height: 0.4 };
        let mut w = [0.0f32; 3];
        apply_belly_control(&mut w, &bc);
        assert!((w[0] - 0.3).abs() < 1e-6);
        assert!((w[1] - 0.6).abs() < 1e-6);
        assert!((w[2] - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_apply_belly_partial() {
        let bc = default_belly_control();
        let mut w = [0.0f32; 1];
        apply_belly_control(&mut w, &bc);
        assert!((w[0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_apply_belly_empty() {
        let bc = default_belly_control();
        let mut w: [f32; 0] = [];
        apply_belly_control(&mut w, &bc);
    }

    #[test]
    fn test_blend_at_zero() {
        let a = default_belly_control();
        let b = BellyControl { protrusion: 1.0, width: 1.0, height: 1.0 };
        let r = belly_control_blend(&a, &b, 0.0);
        assert!((r.protrusion - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_blend_at_one() {
        let a = default_belly_control();
        let b = BellyControl { protrusion: 1.0, width: 1.0, height: 1.0 };
        let r = belly_control_blend(&a, &b, 1.0);
        assert!((r.protrusion - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_blend_mid() {
        let a = BellyControl { protrusion: 0.0, width: 0.0, height: 0.0 };
        let b = BellyControl { protrusion: 1.0, width: 1.0, height: 1.0 };
        let r = belly_control_blend(&a, &b, 0.5);
        assert!((r.protrusion - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_blend_clamps_over() {
        let a = default_belly_control();
        let b = BellyControl { protrusion: 1.0, width: 1.0, height: 1.0 };
        let r = belly_control_blend(&a, &b, 2.0);
        assert!((r.protrusion - b.protrusion).abs() < 1e-6);
    }

    #[test]
    fn test_belly_volume_zero_protrusion() {
        let bc = BellyControl { protrusion: 0.0, width: 0.5, height: 0.5 };
        assert!((belly_volume_approx(&bc)).abs() < 1e-9);
    }

    #[test]
    fn test_belly_volume_positive() {
        let bc = BellyControl { protrusion: 0.3, width: 0.5, height: 0.5 };
        assert!(belly_volume_approx(&bc) > 0.0);
    }
}
