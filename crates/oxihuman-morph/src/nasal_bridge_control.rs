// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Nasal bridge control for facial morphing.

/// Nasal bridge parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NasalBridgeParams {
    pub height: f32,
    pub width: f32,
    pub curvature: f32,
    pub asymmetry: f32,
}

/// Nasal bridge result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NasalBridgeResult {
    pub height_weight: f32,
    pub width_weight: f32,
    pub curvature_weight: f32,
    pub combined_weight: f32,
}

/// Default nasal bridge parameters.
#[allow(dead_code)]
pub fn default_nasal_bridge() -> NasalBridgeParams {
    NasalBridgeParams {
        height: 0.5,
        width: 0.5,
        curvature: 0.0,
        asymmetry: 0.0,
    }
}

/// Evaluate nasal bridge morph.
#[allow(dead_code)]
pub fn evaluate_nasal_bridge(params: &NasalBridgeParams) -> NasalBridgeResult {
    let h = params.height.clamp(0.0, 1.0);
    let w = params.width.clamp(0.0, 1.0);
    let c = params.curvature.clamp(-1.0, 1.0);
    NasalBridgeResult {
        height_weight: h,
        width_weight: w,
        curvature_weight: (c + 1.0) * 0.5,
        combined_weight: h * 0.5 + w * 0.3 + (c.abs()) * 0.2,
    }
}

/// Blend nasal bridge params.
#[allow(dead_code)]
pub fn blend_nasal_bridge(a: &NasalBridgeParams, b: &NasalBridgeParams, t: f32) -> NasalBridgeParams {
    let t = t.clamp(0.0, 1.0);
    NasalBridgeParams {
        height: a.height + (b.height - a.height) * t,
        width: a.width + (b.width - a.width) * t,
        curvature: a.curvature + (b.curvature - a.curvature) * t,
        asymmetry: a.asymmetry + (b.asymmetry - a.asymmetry) * t,
    }
}

/// Set bridge height.
#[allow(dead_code)]
pub fn set_nasal_bridge_height(params: &mut NasalBridgeParams, value: f32) {
    params.height = value.clamp(0.0, 1.0);
}

/// Validate params.
#[allow(dead_code)]
pub fn is_valid_nasal_bridge(params: &NasalBridgeParams) -> bool {
    (0.0..=1.0).contains(&params.height)
        && (0.0..=1.0).contains(&params.width)
        && (-1.0..=1.0).contains(&params.curvature)
        && (-1.0..=1.0).contains(&params.asymmetry)
}

/// Reset to defaults.
#[allow(dead_code)]
pub fn reset_nasal_bridge(params: &mut NasalBridgeParams) {
    *params = default_nasal_bridge();
}

/// Compute bridge prominence.
#[allow(dead_code)]
pub fn bridge_prominence(params: &NasalBridgeParams) -> f32 {
    (params.height * 0.7 + params.curvature.abs() * 0.3).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let p = default_nasal_bridge();
        assert!((p.height - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate() {
        let p = default_nasal_bridge();
        let r = evaluate_nasal_bridge(&p);
        assert!((0.0..=1.0).contains(&r.combined_weight));
    }

    #[test]
    fn test_blend() {
        let a = default_nasal_bridge();
        let mut b = default_nasal_bridge();
        b.height = 1.0;
        let c = blend_nasal_bridge(&a, &b, 0.5);
        assert!((c.height - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_set_height() {
        let mut p = default_nasal_bridge();
        set_nasal_bridge_height(&mut p, 0.8);
        assert!((p.height - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_valid() {
        assert!(is_valid_nasal_bridge(&default_nasal_bridge()));
    }

    #[test]
    fn test_invalid() {
        let p = NasalBridgeParams { height: 2.0, width: 0.5, curvature: 0.0, asymmetry: 0.0 };
        assert!(!is_valid_nasal_bridge(&p));
    }

    #[test]
    fn test_reset() {
        let mut p = NasalBridgeParams { height: 0.9, width: 0.1, curvature: 0.5, asymmetry: 0.3 };
        reset_nasal_bridge(&mut p);
        assert!((p.height - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_prominence() {
        let p = NasalBridgeParams { height: 1.0, width: 0.5, curvature: 1.0, asymmetry: 0.0 };
        assert!((bridge_prominence(&p) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_curvature_weight() {
        let p = NasalBridgeParams { height: 0.5, width: 0.5, curvature: -1.0, asymmetry: 0.0 };
        let r = evaluate_nasal_bridge(&p);
        assert!(r.curvature_weight.abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = default_nasal_bridge();
        let c = blend_nasal_bridge(&a, &a, 0.5);
        assert!((c.height - a.height).abs() < 1e-6);
    }
}
