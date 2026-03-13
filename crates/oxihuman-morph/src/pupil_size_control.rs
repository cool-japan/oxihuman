// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Pupil dilation morph control.

use std::f32::consts::PI;

/// Pupil size control parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct PupilSizeParams {
    /// Left pupil dilation 0..=1 (0 = fully constricted, 1 = fully dilated).
    pub dilation_l: f32,
    /// Right pupil dilation 0..=1.
    pub dilation_r: f32,
    /// Minimum pupil radius as fraction of iris radius.
    pub min_radius: f32,
    /// Maximum pupil radius as fraction of iris radius.
    pub max_radius: f32,
}

impl Default for PupilSizeParams {
    fn default() -> Self {
        Self {
            dilation_l: 0.4,
            dilation_r: 0.4,
            min_radius: 0.15,
            max_radius: 0.85,
        }
    }
}

/// Create default params.
#[allow(dead_code)]
pub fn default_pupil_size_params() -> PupilSizeParams {
    PupilSizeParams::default()
}

/// Compute actual radius fraction for a given dilation value.
#[allow(dead_code)]
pub fn pupil_radius_fraction(dilation: f32, params: &PupilSizeParams) -> f32 {
    let d = dilation.clamp(0.0, 1.0);
    params.min_radius + (params.max_radius - params.min_radius) * d
}

/// Set left pupil dilation.
#[allow(dead_code)]
pub fn set_pupil_dilation_left(params: &mut PupilSizeParams, value: f32) {
    params.dilation_l = value.clamp(0.0, 1.0);
}

/// Set right pupil dilation.
#[allow(dead_code)]
pub fn set_pupil_dilation_right(params: &mut PupilSizeParams, value: f32) {
    params.dilation_r = value.clamp(0.0, 1.0);
}

/// Set both pupils equally.
#[allow(dead_code)]
pub fn set_pupil_dilation_both(params: &mut PupilSizeParams, value: f32) {
    let v = value.clamp(0.0, 1.0);
    params.dilation_l = v;
    params.dilation_r = v;
}

/// Simulate light-level response (inverse relationship).
#[allow(dead_code)]
pub fn pupil_from_light(luminance: f32) -> f32 {
    let l = luminance.clamp(0.0, 1.0);
    // Use smooth curve: dark = dilated, bright = constricted
    let t = 1.0 - l;
    t * t * (3.0 - 2.0 * t)
}

/// Compute pupil area fraction (circular area = pi*r^2 / pi*1^2 = r^2).
#[allow(dead_code)]
pub fn pupil_area_fraction(dilation: f32, params: &PupilSizeParams) -> f32 {
    let r = pupil_radius_fraction(dilation, params);
    r * r
}

/// Asymmetry between left and right pupils.
#[allow(dead_code)]
pub fn pupil_asymmetry(params: &PupilSizeParams) -> f32 {
    (params.dilation_l - params.dilation_r).abs()
}

/// Blend two param sets.
#[allow(dead_code)]
pub fn blend_pupil_size(a: &PupilSizeParams, b: &PupilSizeParams, t: f32) -> PupilSizeParams {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    PupilSizeParams {
        dilation_l: a.dilation_l * inv + b.dilation_l * t,
        dilation_r: a.dilation_r * inv + b.dilation_r * t,
        min_radius: a.min_radius * inv + b.min_radius * t,
        max_radius: a.max_radius * inv + b.max_radius * t,
    }
}

/// Reset to default.
#[allow(dead_code)]
pub fn reset_pupil_size(params: &mut PupilSizeParams) {
    *params = PupilSizeParams::default();
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn pupil_size_to_json(params: &PupilSizeParams) -> String {
    format!(
        r#"{{"dilation_l":{:.4},"dilation_r":{:.4},"min_radius":{:.4},"max_radius":{:.4}}}"#,
        params.dilation_l, params.dilation_r, params.min_radius, params.max_radius
    )
}

/// Reference: use PI to compute circumference factor.
#[allow(dead_code)]
pub fn pupil_circumference_ratio(dilation: f32, params: &PupilSizeParams) -> f32 {
    let r = pupil_radius_fraction(dilation, params);
    2.0 * PI * r
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_default_valid() {
        let p = PupilSizeParams::default();
        assert!((0.0..=1.0).contains(&p.dilation_l));
        assert!((0.0..=1.0).contains(&p.dilation_r));
    }

    #[test]
    fn test_radius_fraction_min() {
        let p = PupilSizeParams::default();
        let r = pupil_radius_fraction(0.0, &p);
        assert!((r - p.min_radius).abs() < 1e-6);
    }

    #[test]
    fn test_radius_fraction_max() {
        let p = PupilSizeParams::default();
        let r = pupil_radius_fraction(1.0, &p);
        assert!((r - p.max_radius).abs() < 1e-6);
    }

    #[test]
    fn test_set_dilation_left_clamp() {
        let mut p = PupilSizeParams::default();
        set_pupil_dilation_left(&mut p, 5.0);
        assert!((p.dilation_l - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_dilation_both() {
        let mut p = PupilSizeParams::default();
        set_pupil_dilation_both(&mut p, 0.7);
        assert!((p.dilation_l - 0.7).abs() < 1e-6);
        assert!((p.dilation_r - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_from_light_dark() {
        let d = pupil_from_light(0.0);
        assert!((d - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_from_light_bright() {
        let d = pupil_from_light(1.0);
        assert!(d.abs() < 1e-6);
    }

    #[test]
    fn test_area_fraction_positive() {
        let p = PupilSizeParams::default();
        let a = pupil_area_fraction(0.5, &p);
        assert!(a > 0.0);
    }

    #[test]
    fn test_blend_midpoint() {
        let a = PupilSizeParams {
            dilation_l: 0.0,
            ..Default::default()
        };
        let b = PupilSizeParams {
            dilation_l: 1.0,
            ..Default::default()
        };
        let r = blend_pupil_size(&a, &b, 0.5);
        assert!((r.dilation_l - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_circumference_uses_pi() {
        let p = PupilSizeParams::default();
        let c = pupil_circumference_ratio(1.0, &p);
        assert!((c - 2.0 * PI * p.max_radius).abs() < 1e-5);
    }
}
