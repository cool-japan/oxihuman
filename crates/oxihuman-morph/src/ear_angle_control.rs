// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Ear angle control for character ear morphing.

use std::f32::consts::PI;

/// Ear angle parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EarAngleParams {
    pub left_angle_deg: f32,
    pub right_angle_deg: f32,
    pub symmetrical: bool,
}

/// Ear angle result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EarAngleResult {
    pub left_weight: f32,
    pub right_weight: f32,
    pub left_angle_rad: f32,
    pub right_angle_rad: f32,
}

/// Default ear angle parameters.
#[allow(dead_code)]
pub fn default_ear_angle() -> EarAngleParams {
    EarAngleParams {
        left_angle_deg: 15.0,
        right_angle_deg: 15.0,
        symmetrical: true,
    }
}

/// Evaluate ear angle morph weights.
#[allow(dead_code)]
pub fn evaluate_ear_angle(params: &EarAngleParams) -> EarAngleResult {
    let max_angle = 45.0_f32;
    let left = params.left_angle_deg.clamp(0.0, max_angle);
    let right = if params.symmetrical { left } else { params.right_angle_deg.clamp(0.0, max_angle) };
    EarAngleResult {
        left_weight: left / max_angle,
        right_weight: right / max_angle,
        left_angle_rad: left * PI / 180.0,
        right_angle_rad: right * PI / 180.0,
    }
}

/// Set ear angle for both sides.
#[allow(dead_code)]
pub fn set_ear_angle_both(params: &mut EarAngleParams, angle_deg: f32) {
    let v = angle_deg.clamp(0.0, 45.0);
    params.left_angle_deg = v;
    params.right_angle_deg = v;
}

/// Blend ear angle params.
#[allow(dead_code)]
pub fn blend_ear_angle(a: &EarAngleParams, b: &EarAngleParams, t: f32) -> EarAngleParams {
    let t = t.clamp(0.0, 1.0);
    EarAngleParams {
        left_angle_deg: a.left_angle_deg + (b.left_angle_deg - a.left_angle_deg) * t,
        right_angle_deg: a.right_angle_deg + (b.right_angle_deg - a.right_angle_deg) * t,
        symmetrical: a.symmetrical,
    }
}

/// Compute the protrusion factor (normalized 0..1).
#[allow(dead_code)]
pub fn ear_protrusion_factor(angle_deg: f32) -> f32 {
    (angle_deg / 45.0).clamp(0.0, 1.0)
}

/// Validate params.
#[allow(dead_code)]
pub fn is_valid_ear_angle(params: &EarAngleParams) -> bool {
    (0.0..=45.0).contains(&params.left_angle_deg)
        && (0.0..=45.0).contains(&params.right_angle_deg)
}

/// Reset to defaults.
#[allow(dead_code)]
pub fn reset_ear_angle(params: &mut EarAngleParams) {
    *params = default_ear_angle();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let p = default_ear_angle();
        assert!((p.left_angle_deg - 15.0).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate() {
        let p = default_ear_angle();
        let r = evaluate_ear_angle(&p);
        assert!((0.0..=1.0).contains(&r.left_weight));
    }

    #[test]
    fn test_symmetrical() {
        let mut p = default_ear_angle();
        p.right_angle_deg = 30.0;
        p.symmetrical = true;
        let r = evaluate_ear_angle(&p);
        assert!((r.left_weight - r.right_weight).abs() < 1e-6);
    }

    #[test]
    fn test_set_both() {
        let mut p = default_ear_angle();
        set_ear_angle_both(&mut p, 25.0);
        assert!((p.left_angle_deg - 25.0).abs() < 1e-6);
        assert!((p.right_angle_deg - 25.0).abs() < 1e-6);
    }

    #[test]
    fn test_blend() {
        let a = default_ear_angle();
        let mut b = default_ear_angle();
        b.left_angle_deg = 45.0;
        b.right_angle_deg = 45.0;
        let c = blend_ear_angle(&a, &b, 0.5);
        assert!((c.left_angle_deg - 30.0).abs() < 1e-6);
    }

    #[test]
    fn test_protrusion_factor() {
        assert!((ear_protrusion_factor(0.0)).abs() < 1e-6);
        assert!((ear_protrusion_factor(45.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_valid() {
        assert!(is_valid_ear_angle(&default_ear_angle()));
    }

    #[test]
    fn test_invalid() {
        let p = EarAngleParams { left_angle_deg: 60.0, right_angle_deg: 15.0, symmetrical: false };
        assert!(!is_valid_ear_angle(&p));
    }

    #[test]
    fn test_reset() {
        let mut p = EarAngleParams { left_angle_deg: 40.0, right_angle_deg: 40.0, symmetrical: false };
        reset_ear_angle(&mut p);
        assert!((p.left_angle_deg - 15.0).abs() < 1e-6);
    }

    #[test]
    fn test_radians() {
        let p = EarAngleParams { left_angle_deg: 45.0, right_angle_deg: 45.0, symmetrical: true };
        let r = evaluate_ear_angle(&p);
        assert!((r.left_angle_rad - PI / 4.0).abs() < 1e-4);
    }
}
