// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Brow ridge morphology control for character faces.

/// Brow ridge parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrowRidgeParams {
    pub prominence: f32,
    pub width: f32,
    pub height: f32,
    pub asymmetry: f32,
}

/// Result of brow ridge evaluation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrowRidgeResult {
    pub left_weight: f32,
    pub right_weight: f32,
    pub prominence_weight: f32,
    pub width_weight: f32,
}

/// Default brow ridge parameters.
#[allow(dead_code)]
pub fn default_brow_ridge() -> BrowRidgeParams {
    BrowRidgeParams {
        prominence: 0.5,
        width: 0.5,
        height: 0.5,
        asymmetry: 0.0,
    }
}

/// Clamp a value to morph range.
#[allow(dead_code)]
pub fn clamp_morph(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

/// Evaluate brow ridge morph weights.
#[allow(dead_code)]
pub fn evaluate_brow_ridge(params: &BrowRidgeParams) -> BrowRidgeResult {
    let p = clamp_morph(params.prominence);
    let w = clamp_morph(params.width);
    let asym = params.asymmetry.clamp(-1.0, 1.0);
    let base = p * 0.7 + w * 0.3;
    BrowRidgeResult {
        left_weight: (base + asym * 0.5).clamp(0.0, 1.0),
        right_weight: (base - asym * 0.5).clamp(0.0, 1.0),
        prominence_weight: p,
        width_weight: w,
    }
}

/// Blend two brow ridge params.
#[allow(dead_code)]
pub fn blend_brow_ridge(a: &BrowRidgeParams, b: &BrowRidgeParams, t: f32) -> BrowRidgeParams {
    let t = t.clamp(0.0, 1.0);
    BrowRidgeParams {
        prominence: a.prominence + (b.prominence - a.prominence) * t,
        width: a.width + (b.width - a.width) * t,
        height: a.height + (b.height - a.height) * t,
        asymmetry: a.asymmetry + (b.asymmetry - a.asymmetry) * t,
    }
}

/// Set prominence and clamp.
#[allow(dead_code)]
pub fn set_prominence(params: &mut BrowRidgeParams, value: f32) {
    params.prominence = clamp_morph(value);
}

/// Set width and clamp.
#[allow(dead_code)]
pub fn set_ridge_width(params: &mut BrowRidgeParams, value: f32) {
    params.width = clamp_morph(value);
}

/// Compute combined intensity.
#[allow(dead_code)]
pub fn ridge_intensity(params: &BrowRidgeParams) -> f32 {
    (params.prominence * 0.6 + params.width * 0.4).clamp(0.0, 1.0)
}

/// Check if ridge is in valid range.
#[allow(dead_code)]
pub fn is_valid_ridge(params: &BrowRidgeParams) -> bool {
    (0.0..=1.0).contains(&params.prominence)
        && (0.0..=1.0).contains(&params.width)
        && (0.0..=1.0).contains(&params.height)
        && (-1.0..=1.0).contains(&params.asymmetry)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_brow_ridge() {
        let p = default_brow_ridge();
        assert!((p.prominence - 0.5).abs() < 1e-6);
        assert!((p.asymmetry).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_morph() {
        assert!((clamp_morph(-0.5)).abs() < 1e-6);
        assert!((clamp_morph(1.5) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_brow_ridge() {
        let p = default_brow_ridge();
        let r = evaluate_brow_ridge(&p);
        assert!((0.0..=1.0).contains(&r.left_weight));
        assert!((0.0..=1.0).contains(&r.right_weight));
    }

    #[test]
    fn test_blend_brow_ridge() {
        let a = default_brow_ridge();
        let mut b = default_brow_ridge();
        b.prominence = 1.0;
        let c = blend_brow_ridge(&a, &b, 0.5);
        assert!((c.prominence - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_set_prominence() {
        let mut p = default_brow_ridge();
        set_prominence(&mut p, 0.8);
        assert!((p.prominence - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_ridge_width() {
        let mut p = default_brow_ridge();
        set_ridge_width(&mut p, 0.3);
        assert!((p.width - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_ridge_intensity() {
        let p = default_brow_ridge();
        let v = ridge_intensity(&p);
        assert!((0.0..=1.0).contains(&v));
    }

    #[test]
    fn test_is_valid_ridge() {
        let p = default_brow_ridge();
        assert!(is_valid_ridge(&p));
    }

    #[test]
    fn test_asymmetry_effect() {
        let mut p = default_brow_ridge();
        p.asymmetry = 1.0;
        let r = evaluate_brow_ridge(&p);
        assert!(r.left_weight > r.right_weight);
    }

    #[test]
    fn test_invalid_ridge() {
        let p = BrowRidgeParams {
            prominence: 2.0,
            width: 0.5,
            height: 0.5,
            asymmetry: 0.0,
        };
        assert!(!is_valid_ridge(&p));
    }
}
