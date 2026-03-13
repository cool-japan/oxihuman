// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Cheek hollow control for facial morphing.

/// Parameters for cheek hollow.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheekHollowParams {
    pub depth: f32,
    pub width: f32,
    pub height_offset: f32,
    pub asymmetry: f32,
}

/// Result of cheek hollow evaluation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheekHollowResult {
    pub left_weight: f32,
    pub right_weight: f32,
    pub depth_weight: f32,
}

/// Default cheek hollow parameters.
#[allow(dead_code)]
pub fn default_cheek_hollow() -> CheekHollowParams {
    CheekHollowParams {
        depth: 0.0,
        width: 0.5,
        height_offset: 0.0,
        asymmetry: 0.0,
    }
}

/// Evaluate cheek hollow morph.
#[allow(dead_code)]
pub fn evaluate_cheek_hollow(params: &CheekHollowParams) -> CheekHollowResult {
    let d = params.depth.clamp(0.0, 1.0);
    let asym = params.asymmetry.clamp(-1.0, 1.0);
    CheekHollowResult {
        left_weight: (d + asym * 0.5).clamp(0.0, 1.0),
        right_weight: (d - asym * 0.5).clamp(0.0, 1.0),
        depth_weight: d,
    }
}

/// Blend cheek hollow params.
#[allow(dead_code)]
pub fn blend_cheek_hollow(
    a: &CheekHollowParams,
    b: &CheekHollowParams,
    t: f32,
) -> CheekHollowParams {
    let t = t.clamp(0.0, 1.0);
    CheekHollowParams {
        depth: a.depth + (b.depth - a.depth) * t,
        width: a.width + (b.width - a.width) * t,
        height_offset: a.height_offset + (b.height_offset - a.height_offset) * t,
        asymmetry: a.asymmetry + (b.asymmetry - a.asymmetry) * t,
    }
}

/// Set hollow depth.
#[allow(dead_code)]
pub fn set_cheek_hollow_depth(params: &mut CheekHollowParams, value: f32) {
    params.depth = value.clamp(0.0, 1.0);
}

/// Compute hollow intensity.
#[allow(dead_code)]
pub fn cheek_hollow_intensity(params: &CheekHollowParams) -> f32 {
    (params.depth * params.width.max(0.1)).clamp(0.0, 1.0)
}

/// Check if within valid range.
#[allow(dead_code)]
pub fn is_valid_cheek_hollow(params: &CheekHollowParams) -> bool {
    (0.0..=1.0).contains(&params.depth)
        && (0.0..=1.0).contains(&params.width)
        && (-1.0..=1.0).contains(&params.height_offset)
        && (-1.0..=1.0).contains(&params.asymmetry)
}

/// Reset to defaults.
#[allow(dead_code)]
pub fn reset_cheek_hollow(params: &mut CheekHollowParams) {
    *params = default_cheek_hollow();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let p = default_cheek_hollow();
        assert!(p.depth.abs() < 1e-6);
    }

    #[test]
    fn test_evaluate() {
        let p = CheekHollowParams {
            depth: 0.5,
            width: 0.5,
            height_offset: 0.0,
            asymmetry: 0.0,
        };
        let r = evaluate_cheek_hollow(&p);
        assert!((r.depth_weight - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_blend() {
        let a = default_cheek_hollow();
        let mut b = default_cheek_hollow();
        b.depth = 1.0;
        let c = blend_cheek_hollow(&a, &b, 0.5);
        assert!((c.depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_depth() {
        let mut p = default_cheek_hollow();
        set_cheek_hollow_depth(&mut p, 0.7);
        assert!((p.depth - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_intensity() {
        let p = CheekHollowParams {
            depth: 1.0,
            width: 1.0,
            height_offset: 0.0,
            asymmetry: 0.0,
        };
        let v = cheek_hollow_intensity(&p);
        assert!((v - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_valid() {
        let p = default_cheek_hollow();
        assert!(is_valid_cheek_hollow(&p));
    }

    #[test]
    fn test_invalid() {
        let p = CheekHollowParams {
            depth: 2.0,
            width: 0.5,
            height_offset: 0.0,
            asymmetry: 0.0,
        };
        assert!(!is_valid_cheek_hollow(&p));
    }

    #[test]
    fn test_reset() {
        let mut p = CheekHollowParams {
            depth: 0.9,
            width: 0.1,
            height_offset: 0.5,
            asymmetry: 0.3,
        };
        reset_cheek_hollow(&mut p);
        assert!(p.depth.abs() < 1e-6);
    }

    #[test]
    fn test_asymmetry() {
        let p = CheekHollowParams {
            depth: 0.5,
            width: 0.5,
            height_offset: 0.0,
            asymmetry: 1.0,
        };
        let r = evaluate_cheek_hollow(&p);
        assert!(r.left_weight > r.right_weight);
    }

    #[test]
    fn test_blend_extremes() {
        let a = default_cheek_hollow();
        let b = default_cheek_hollow();
        let c = blend_cheek_hollow(&a, &b, 0.0);
        assert!((c.depth - a.depth).abs() < 1e-6);
    }
}
