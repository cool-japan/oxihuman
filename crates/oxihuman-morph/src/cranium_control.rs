// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Cranium shape control for character head morphing.

use std::f32::consts::PI;

/// Cranium shape parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CraniumParams {
    pub length: f32,
    pub width: f32,
    pub height: f32,
    pub forehead_slope: f32,
    pub occiput_protrusion: f32,
}

/// Result of cranium evaluation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CraniumResult {
    pub length_weight: f32,
    pub width_weight: f32,
    pub height_weight: f32,
    pub slope_weight: f32,
    pub volume_estimate: f32,
}

/// Default cranium parameters.
#[allow(dead_code)]
pub fn default_cranium() -> CraniumParams {
    CraniumParams {
        length: 0.5,
        width: 0.5,
        height: 0.5,
        forehead_slope: 0.5,
        occiput_protrusion: 0.3,
    }
}

/// Evaluate cranium morph weights.
#[allow(dead_code)]
pub fn evaluate_cranium(params: &CraniumParams) -> CraniumResult {
    let l = params.length.clamp(0.0, 1.0);
    let w = params.width.clamp(0.0, 1.0);
    let h = params.height.clamp(0.0, 1.0);
    let s = params.forehead_slope.clamp(0.0, 1.0);
    // Approximate cranial volume as ellipsoid
    let volume = (4.0 / 3.0) * PI * (l * 0.1) * (w * 0.08) * (h * 0.09);
    CraniumResult {
        length_weight: l,
        width_weight: w,
        height_weight: h,
        slope_weight: s,
        volume_estimate: volume,
    }
}

/// Blend cranium params.
#[allow(dead_code)]
pub fn blend_cranium(a: &CraniumParams, b: &CraniumParams, t: f32) -> CraniumParams {
    let t = t.clamp(0.0, 1.0);
    CraniumParams {
        length: a.length + (b.length - a.length) * t,
        width: a.width + (b.width - a.width) * t,
        height: a.height + (b.height - a.height) * t,
        forehead_slope: a.forehead_slope + (b.forehead_slope - a.forehead_slope) * t,
        occiput_protrusion: a.occiput_protrusion + (b.occiput_protrusion - a.occiput_protrusion) * t,
    }
}

/// Set cranium width.
#[allow(dead_code)]
pub fn set_cranium_width(params: &mut CraniumParams, value: f32) {
    params.width = value.clamp(0.0, 1.0);
}

/// Set cranium length.
#[allow(dead_code)]
pub fn set_cranium_length(params: &mut CraniumParams, value: f32) {
    params.length = value.clamp(0.0, 1.0);
}

/// Compute cephalic index (width/length ratio).
#[allow(dead_code)]
pub fn cephalic_index(params: &CraniumParams) -> f32 {
    if params.length > 0.001 {
        params.width / params.length
    } else {
        1.0
    }
}

/// Validate cranium params.
#[allow(dead_code)]
pub fn is_valid_cranium(params: &CraniumParams) -> bool {
    (0.0..=1.0).contains(&params.length)
        && (0.0..=1.0).contains(&params.width)
        && (0.0..=1.0).contains(&params.height)
        && (0.0..=1.0).contains(&params.forehead_slope)
        && (0.0..=1.0).contains(&params.occiput_protrusion)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let p = default_cranium();
        assert!((p.length - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate() {
        let p = default_cranium();
        let r = evaluate_cranium(&p);
        assert!(r.volume_estimate > 0.0);
    }

    #[test]
    fn test_blend() {
        let a = default_cranium();
        let mut b = default_cranium();
        b.width = 1.0;
        let c = blend_cranium(&a, &b, 0.5);
        assert!((c.width - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_set_width() {
        let mut p = default_cranium();
        set_cranium_width(&mut p, 0.8);
        assert!((p.width - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_length() {
        let mut p = default_cranium();
        set_cranium_length(&mut p, 0.3);
        assert!((p.length - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_cephalic_index() {
        let p = default_cranium();
        let ci = cephalic_index(&p);
        assert!((ci - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_valid() {
        assert!(is_valid_cranium(&default_cranium()));
    }

    #[test]
    fn test_invalid() {
        let p = CraniumParams { length: 2.0, width: 0.5, height: 0.5, forehead_slope: 0.5, occiput_protrusion: 0.3 };
        assert!(!is_valid_cranium(&p));
    }

    #[test]
    fn test_volume_positive() {
        let p = CraniumParams { length: 1.0, width: 1.0, height: 1.0, forehead_slope: 0.5, occiput_protrusion: 0.5 };
        let r = evaluate_cranium(&p);
        assert!(r.volume_estimate > 0.0);
    }

    #[test]
    fn test_blend_identity() {
        let a = default_cranium();
        let c = blend_cranium(&a, &a, 0.5);
        assert!((c.length - a.length).abs() < 1e-6);
    }
}
