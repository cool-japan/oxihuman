// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Neck length control for character morphing.

/// Neck length parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NeckLengthParams {
    pub length: f32,
    pub girth: f32,
    pub forward_tilt: f32,
}

/// Neck length result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NeckLengthResult {
    pub length_weight: f32,
    pub girth_weight: f32,
    pub tilt_weight: f32,
    pub combined_weight: f32,
}

/// Default neck length parameters.
#[allow(dead_code)]
pub fn default_neck_length() -> NeckLengthParams {
    NeckLengthParams {
        length: 0.5,
        girth: 0.5,
        forward_tilt: 0.0,
    }
}

/// Evaluate neck length morph.
#[allow(dead_code)]
pub fn evaluate_neck_length(params: &NeckLengthParams) -> NeckLengthResult {
    let l = params.length.clamp(0.0, 1.0);
    let g = params.girth.clamp(0.0, 1.0);
    let t = params.forward_tilt.clamp(-1.0, 1.0);
    NeckLengthResult {
        length_weight: l,
        girth_weight: g,
        tilt_weight: (t + 1.0) * 0.5,
        combined_weight: l * 0.5 + g * 0.3 + t.abs() * 0.2,
    }
}

/// Blend neck length params.
#[allow(dead_code)]
pub fn blend_neck_length(a: &NeckLengthParams, b: &NeckLengthParams, t: f32) -> NeckLengthParams {
    let t = t.clamp(0.0, 1.0);
    NeckLengthParams {
        length: a.length + (b.length - a.length) * t,
        girth: a.girth + (b.girth - a.girth) * t,
        forward_tilt: a.forward_tilt + (b.forward_tilt - a.forward_tilt) * t,
    }
}

/// Set neck length.
#[allow(dead_code)]
pub fn set_neck_length(params: &mut NeckLengthParams, value: f32) {
    params.length = value.clamp(0.0, 1.0);
}

/// Validate params.
#[allow(dead_code)]
pub fn is_valid_neck_length(params: &NeckLengthParams) -> bool {
    (0.0..=1.0).contains(&params.length)
        && (0.0..=1.0).contains(&params.girth)
        && (-1.0..=1.0).contains(&params.forward_tilt)
}

/// Reset to defaults.
#[allow(dead_code)]
pub fn reset_neck_length(params: &mut NeckLengthParams) {
    *params = default_neck_length();
}

/// Compute length-to-girth ratio.
#[allow(dead_code)]
pub fn neck_proportion(params: &NeckLengthParams) -> f32 {
    if params.girth > 0.001 {
        params.length / params.girth
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let p = default_neck_length();
        assert!((p.length - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate() {
        let p = default_neck_length();
        let r = evaluate_neck_length(&p);
        assert!((0.0..=1.0).contains(&r.combined_weight));
    }

    #[test]
    fn test_blend() {
        let a = default_neck_length();
        let mut b = default_neck_length();
        b.length = 1.0;
        let c = blend_neck_length(&a, &b, 0.5);
        assert!((c.length - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_set() {
        let mut p = default_neck_length();
        set_neck_length(&mut p, 0.8);
        assert!((p.length - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_valid() {
        assert!(is_valid_neck_length(&default_neck_length()));
    }

    #[test]
    fn test_invalid() {
        let p = NeckLengthParams { length: 2.0, girth: 0.5, forward_tilt: 0.0 };
        assert!(!is_valid_neck_length(&p));
    }

    #[test]
    fn test_reset() {
        let mut p = NeckLengthParams { length: 0.9, girth: 0.1, forward_tilt: 0.5 };
        reset_neck_length(&mut p);
        assert!((p.length - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_proportion() {
        let p = default_neck_length();
        assert!((neck_proportion(&p) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_tilt_weight() {
        let p = NeckLengthParams { length: 0.5, girth: 0.5, forward_tilt: 1.0 };
        let r = evaluate_neck_length(&p);
        assert!((r.tilt_weight - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = default_neck_length();
        let c = blend_neck_length(&a, &a, 0.5);
        assert!((c.length - a.length).abs() < 1e-6);
    }
}
