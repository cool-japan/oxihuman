// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Eye recess (depth) control for facial morphing.

/// Eye recess parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeRecessParams {
    pub depth: f32,
    pub left_offset: f32,
    pub right_offset: f32,
    pub symmetrical: bool,
}

/// Eye recess evaluation result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeRecessResult {
    pub left_weight: f32,
    pub right_weight: f32,
    pub depth_weight: f32,
}

/// Default eye recess parameters.
#[allow(dead_code)]
pub fn default_eye_recess() -> EyeRecessParams {
    EyeRecessParams {
        depth: 0.5,
        left_offset: 0.0,
        right_offset: 0.0,
        symmetrical: true,
    }
}

/// Evaluate eye recess morph weights.
#[allow(dead_code)]
pub fn evaluate_eye_recess(params: &EyeRecessParams) -> EyeRecessResult {
    let d = params.depth.clamp(0.0, 1.0);
    let lo = params.left_offset.clamp(-0.5, 0.5);
    let ro = if params.symmetrical { lo } else { params.right_offset.clamp(-0.5, 0.5) };
    EyeRecessResult {
        left_weight: (d + lo).clamp(0.0, 1.0),
        right_weight: (d + ro).clamp(0.0, 1.0),
        depth_weight: d,
    }
}

/// Blend eye recess params.
#[allow(dead_code)]
pub fn blend_eye_recess(a: &EyeRecessParams, b: &EyeRecessParams, t: f32) -> EyeRecessParams {
    let t = t.clamp(0.0, 1.0);
    EyeRecessParams {
        depth: a.depth + (b.depth - a.depth) * t,
        left_offset: a.left_offset + (b.left_offset - a.left_offset) * t,
        right_offset: a.right_offset + (b.right_offset - a.right_offset) * t,
        symmetrical: a.symmetrical,
    }
}

/// Set recess depth.
#[allow(dead_code)]
pub fn set_eye_recess_depth(params: &mut EyeRecessParams, value: f32) {
    params.depth = value.clamp(0.0, 1.0);
}

/// Validate params.
#[allow(dead_code)]
pub fn is_valid_eye_recess(params: &EyeRecessParams) -> bool {
    (0.0..=1.0).contains(&params.depth)
        && (-0.5..=0.5).contains(&params.left_offset)
        && (-0.5..=0.5).contains(&params.right_offset)
}

/// Reset to defaults.
#[allow(dead_code)]
pub fn reset_eye_recess(params: &mut EyeRecessParams) {
    *params = default_eye_recess();
}

/// Compute recess asymmetry.
#[allow(dead_code)]
pub fn eye_recess_asymmetry(params: &EyeRecessParams) -> f32 {
    (params.left_offset - params.right_offset).abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let p = default_eye_recess();
        assert!((p.depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate() {
        let p = default_eye_recess();
        let r = evaluate_eye_recess(&p);
        assert!((r.depth_weight - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_symmetrical() {
        let p = default_eye_recess();
        let r = evaluate_eye_recess(&p);
        assert!((r.left_weight - r.right_weight).abs() < 1e-6);
    }

    #[test]
    fn test_blend() {
        let a = default_eye_recess();
        let mut b = default_eye_recess();
        b.depth = 1.0;
        let c = blend_eye_recess(&a, &b, 0.5);
        assert!((c.depth - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_set_depth() {
        let mut p = default_eye_recess();
        set_eye_recess_depth(&mut p, 0.2);
        assert!((p.depth - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_valid() {
        assert!(is_valid_eye_recess(&default_eye_recess()));
    }

    #[test]
    fn test_invalid() {
        let p = EyeRecessParams { depth: 2.0, left_offset: 0.0, right_offset: 0.0, symmetrical: true };
        assert!(!is_valid_eye_recess(&p));
    }

    #[test]
    fn test_reset() {
        let mut p = EyeRecessParams { depth: 0.9, left_offset: 0.3, right_offset: 0.3, symmetrical: false };
        reset_eye_recess(&mut p);
        assert!((p.depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_asymmetry() {
        let p = EyeRecessParams { depth: 0.5, left_offset: 0.2, right_offset: -0.1, symmetrical: false };
        assert!((eye_recess_asymmetry(&p) - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_symmetrical_zero_asymmetry() {
        let p = default_eye_recess();
        assert!(eye_recess_asymmetry(&p).abs() < 1e-6);
    }
}
