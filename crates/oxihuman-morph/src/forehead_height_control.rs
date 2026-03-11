// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Forehead height control for facial morphing.

/// Forehead height parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ForeheadHeightParams {
    pub height: f32,
    pub slope: f32,
    pub width: f32,
}

/// Result of forehead height evaluation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ForeheadHeightResult {
    pub height_weight: f32,
    pub slope_weight: f32,
    pub combined_weight: f32,
}

/// Default forehead height parameters.
#[allow(dead_code)]
pub fn default_forehead_height() -> ForeheadHeightParams {
    ForeheadHeightParams {
        height: 0.5,
        slope: 0.5,
        width: 0.5,
    }
}

/// Evaluate forehead height morph.
#[allow(dead_code)]
pub fn evaluate_forehead_height(params: &ForeheadHeightParams) -> ForeheadHeightResult {
    let h = params.height.clamp(0.0, 1.0);
    let s = params.slope.clamp(0.0, 1.0);
    ForeheadHeightResult {
        height_weight: h,
        slope_weight: s,
        combined_weight: h * 0.7 + s * 0.3,
    }
}

/// Blend forehead height params.
#[allow(dead_code)]
pub fn blend_forehead_height(a: &ForeheadHeightParams, b: &ForeheadHeightParams, t: f32) -> ForeheadHeightParams {
    let t = t.clamp(0.0, 1.0);
    ForeheadHeightParams {
        height: a.height + (b.height - a.height) * t,
        slope: a.slope + (b.slope - a.slope) * t,
        width: a.width + (b.width - a.width) * t,
    }
}

/// Set forehead height.
#[allow(dead_code)]
pub fn set_forehead_height(params: &mut ForeheadHeightParams, value: f32) {
    params.height = value.clamp(0.0, 1.0);
}

/// Validate params.
#[allow(dead_code)]
pub fn is_valid_forehead_height(params: &ForeheadHeightParams) -> bool {
    (0.0..=1.0).contains(&params.height)
        && (0.0..=1.0).contains(&params.slope)
        && (0.0..=1.0).contains(&params.width)
}

/// Reset to defaults.
#[allow(dead_code)]
pub fn reset_forehead_height(params: &mut ForeheadHeightParams) {
    *params = default_forehead_height();
}

/// Compute height-to-width ratio.
#[allow(dead_code)]
pub fn forehead_ratio(params: &ForeheadHeightParams) -> f32 {
    if params.width > 0.001 {
        params.height / params.width
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let p = default_forehead_height();
        assert!((p.height - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate() {
        let p = default_forehead_height();
        let r = evaluate_forehead_height(&p);
        assert!((0.0..=1.0).contains(&r.combined_weight));
    }

    #[test]
    fn test_blend() {
        let a = default_forehead_height();
        let mut b = default_forehead_height();
        b.height = 1.0;
        let c = blend_forehead_height(&a, &b, 0.5);
        assert!((c.height - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_set() {
        let mut p = default_forehead_height();
        set_forehead_height(&mut p, 0.8);
        assert!((p.height - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_valid() {
        assert!(is_valid_forehead_height(&default_forehead_height()));
    }

    #[test]
    fn test_invalid() {
        let p = ForeheadHeightParams { height: 2.0, slope: 0.5, width: 0.5 };
        assert!(!is_valid_forehead_height(&p));
    }

    #[test]
    fn test_reset() {
        let mut p = ForeheadHeightParams { height: 0.9, slope: 0.1, width: 0.2 };
        reset_forehead_height(&mut p);
        assert!((p.height - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_ratio() {
        let p = default_forehead_height();
        assert!((forehead_ratio(&p) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_full_height() {
        let p = ForeheadHeightParams { height: 1.0, slope: 1.0, width: 0.5 };
        let r = evaluate_forehead_height(&p);
        assert!((r.combined_weight - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_zero_height() {
        let p = ForeheadHeightParams { height: 0.0, slope: 0.0, width: 0.5 };
        let r = evaluate_forehead_height(&p);
        assert!(r.combined_weight.abs() < 1e-6);
    }
}
