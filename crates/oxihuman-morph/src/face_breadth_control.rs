// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Face breadth control for facial morphing.

/// Face breadth parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceBreadthParams {
    pub overall_breadth: f32,
    pub upper_breadth: f32,
    pub lower_breadth: f32,
    pub mid_breadth: f32,
}

/// Face breadth result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceBreadthResult {
    pub overall_weight: f32,
    pub upper_weight: f32,
    pub lower_weight: f32,
    pub mid_weight: f32,
}

/// Default face breadth params.
#[allow(dead_code)]
pub fn default_face_breadth() -> FaceBreadthParams {
    FaceBreadthParams {
        overall_breadth: 0.5,
        upper_breadth: 0.5,
        lower_breadth: 0.5,
        mid_breadth: 0.5,
    }
}

/// Evaluate face breadth morph weights.
#[allow(dead_code)]
pub fn evaluate_face_breadth(params: &FaceBreadthParams) -> FaceBreadthResult {
    FaceBreadthResult {
        overall_weight: params.overall_breadth.clamp(0.0, 1.0),
        upper_weight: params.upper_breadth.clamp(0.0, 1.0),
        lower_weight: params.lower_breadth.clamp(0.0, 1.0),
        mid_weight: params.mid_breadth.clamp(0.0, 1.0),
    }
}

/// Blend face breadth params.
#[allow(dead_code)]
pub fn blend_face_breadth(a: &FaceBreadthParams, b: &FaceBreadthParams, t: f32) -> FaceBreadthParams {
    let t = t.clamp(0.0, 1.0);
    FaceBreadthParams {
        overall_breadth: a.overall_breadth + (b.overall_breadth - a.overall_breadth) * t,
        upper_breadth: a.upper_breadth + (b.upper_breadth - a.upper_breadth) * t,
        lower_breadth: a.lower_breadth + (b.lower_breadth - a.lower_breadth) * t,
        mid_breadth: a.mid_breadth + (b.mid_breadth - a.mid_breadth) * t,
    }
}

/// Set overall breadth.
#[allow(dead_code)]
pub fn set_face_breadth(params: &mut FaceBreadthParams, value: f32) {
    params.overall_breadth = value.clamp(0.0, 1.0);
}

/// Compute taper ratio (upper / lower).
#[allow(dead_code)]
pub fn face_taper_ratio(params: &FaceBreadthParams) -> f32 {
    if params.lower_breadth > 0.001 {
        params.upper_breadth / params.lower_breadth
    } else {
        1.0
    }
}

/// Validate params.
#[allow(dead_code)]
pub fn is_valid_face_breadth(params: &FaceBreadthParams) -> bool {
    (0.0..=1.0).contains(&params.overall_breadth)
        && (0.0..=1.0).contains(&params.upper_breadth)
        && (0.0..=1.0).contains(&params.lower_breadth)
        && (0.0..=1.0).contains(&params.mid_breadth)
}

/// Reset to defaults.
#[allow(dead_code)]
pub fn reset_face_breadth(params: &mut FaceBreadthParams) {
    *params = default_face_breadth();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let p = default_face_breadth();
        assert!((p.overall_breadth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate() {
        let p = default_face_breadth();
        let r = evaluate_face_breadth(&p);
        assert!((r.overall_weight - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_blend() {
        let a = default_face_breadth();
        let mut b = default_face_breadth();
        b.overall_breadth = 1.0;
        let c = blend_face_breadth(&a, &b, 0.5);
        assert!((c.overall_breadth - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_set() {
        let mut p = default_face_breadth();
        set_face_breadth(&mut p, 0.8);
        assert!((p.overall_breadth - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_taper_ratio() {
        let p = default_face_breadth();
        let r = face_taper_ratio(&p);
        assert!((r - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_valid() {
        assert!(is_valid_face_breadth(&default_face_breadth()));
    }

    #[test]
    fn test_invalid() {
        let p = FaceBreadthParams { overall_breadth: 2.0, upper_breadth: 0.5, lower_breadth: 0.5, mid_breadth: 0.5 };
        assert!(!is_valid_face_breadth(&p));
    }

    #[test]
    fn test_reset() {
        let mut p = FaceBreadthParams { overall_breadth: 0.9, upper_breadth: 0.1, lower_breadth: 0.2, mid_breadth: 0.3 };
        reset_face_breadth(&mut p);
        assert!((p.overall_breadth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_taper_zero_lower() {
        let p = FaceBreadthParams { overall_breadth: 0.5, upper_breadth: 0.5, lower_breadth: 0.0, mid_breadth: 0.5 };
        let r = face_taper_ratio(&p);
        assert!((r - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = default_face_breadth();
        let c = blend_face_breadth(&a, &a, 0.5);
        assert!((c.overall_breadth - a.overall_breadth).abs() < 1e-6);
    }
}
