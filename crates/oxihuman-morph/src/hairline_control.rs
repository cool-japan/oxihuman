// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Hairline shape morph control — recession patterns, widow's peak,
//! temple recession, and forehead height.

use std::f32::consts::PI;

/// Hairline shape type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HairlineType {
    Straight,
    Rounded,
    MShaped,
    WidowsPeak,
    Receding,
}

/// Hairline morph parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct HairlineParams {
    /// Forehead height factor, 0 = low hairline, 1 = high.
    pub height: f32,
    /// Temple recession depth, 0..=1.
    pub temple_recession: f32,
    /// Widow's peak prominence, 0..=1.
    pub widows_peak: f32,
    /// Overall recession (Norwood-like scale), 0..=1.
    pub recession: f32,
    /// Lateral width of the hairline, 0..=1.
    pub width: f32,
    /// Asymmetry (-1 = left receded more, 1 = right).
    pub asymmetry: f32,
}

impl Default for HairlineParams {
    fn default() -> Self {
        Self {
            height: 0.5,
            temple_recession: 0.2,
            widows_peak: 0.0,
            recession: 0.0,
            width: 0.5,
            asymmetry: 0.0,
        }
    }
}

/// Hairline evaluation result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HairlineResult {
    /// Per-vertex scalp boundary displacement (index, height_offset).
    pub displacements: Vec<(usize, f32)>,
    /// Classified hairline type.
    pub hairline_type: HairlineType,
    /// Effective forehead area (normalised).
    pub forehead_area: f32,
}

/// Base hairline curve: maps lateral position to height.
///
/// `x` from -1 (left temple) to 1 (right temple).
#[allow(dead_code)]
pub fn base_curve(x: f32, height: f32) -> f32 {
    let x = x.clamp(-1.0, 1.0);
    // Gentle arc
    height * 0.03 * (1.0 - 0.3 * x * x)
}

/// Temple recession: dip at the temples.
#[allow(dead_code)]
pub fn temple_dip(x: f32, depth: f32) -> f32 {
    let x = x.clamp(-1.0, 1.0);
    let ax = x.abs();
    // Temples are at ~0.7 laterally
    let dist = (ax - 0.7).abs();
    if dist < 0.25 {
        -depth * 0.02 * (1.0 - dist / 0.25)
    } else {
        0.0
    }
}

/// Widow's peak: central downward point.
#[allow(dead_code)]
pub fn widows_peak(x: f32, prominence: f32) -> f32 {
    let x = x.clamp(-1.0, 1.0);
    if x.abs() < 0.15 {
        -prominence * 0.015 * (1.0 - x.abs() / 0.15)
    } else {
        0.0
    }
}

/// Overall recession offset.
#[allow(dead_code)]
pub fn recession_offset(recession: f32) -> f32 {
    recession.clamp(0.0, 1.0) * 0.04
}

/// Asymmetry modifier.
#[allow(dead_code)]
pub fn asymmetry_modifier(x: f32, asymmetry: f32) -> f32 {
    let x = x.clamp(-1.0, 1.0);
    1.0 + asymmetry * x * 0.2
}

/// Classify hairline type from parameters.
#[allow(dead_code)]
pub fn classify_hairline(params: &HairlineParams) -> HairlineType {
    if params.recession > 0.6 {
        HairlineType::Receding
    } else if params.widows_peak > 0.5 {
        HairlineType::WidowsPeak
    } else if params.temple_recession > 0.5 {
        HairlineType::MShaped
    } else if params.temple_recession < 0.1 && params.widows_peak < 0.1 {
        HairlineType::Straight
    } else {
        HairlineType::Rounded
    }
}

/// Evaluate hairline morph.
///
/// `scalp_coords`: per-vertex `(lateral_x, distance_from_hairline)`.
#[allow(dead_code)]
pub fn evaluate_hairline(
    scalp_coords: &[(f32, f32)],
    params: &HairlineParams,
) -> HairlineResult {
    let mut displacements = Vec::with_capacity(scalp_coords.len());

    for (i, &(x, dist)) in scalp_coords.iter().enumerate() {
        let base = base_curve(x, params.height);
        let temple = temple_dip(x, params.temple_recession);
        let wp = widows_peak(x, params.widows_peak);
        let recess = recession_offset(params.recession);
        let asym = asymmetry_modifier(x, params.asymmetry);

        let total = (base + temple + wp + recess) * asym;
        let falloff = (-dist * dist * 200.0).exp();
        let disp = total * falloff;

        if disp.abs() > 1e-7 {
            displacements.push((i, disp));
        }
    }

    let hairline_type = classify_hairline(params);
    let forehead_area = params.height * (1.0 + params.recession);

    HairlineResult {
        displacements,
        hairline_type,
        forehead_area,
    }
}

/// Blend hairline params.
#[allow(dead_code)]
pub fn blend_hairline_params(a: &HairlineParams, b: &HairlineParams, t: f32) -> HairlineParams {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    HairlineParams {
        height: a.height * inv + b.height * t,
        temple_recession: a.temple_recession * inv + b.temple_recession * t,
        widows_peak: a.widows_peak * inv + b.widows_peak * t,
        recession: a.recession * inv + b.recession * t,
        width: a.width * inv + b.width * t,
        asymmetry: a.asymmetry * inv + b.asymmetry * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_params() {
        let p = HairlineParams::default();
        assert!((0.0..=1.0).contains(&p.height));
        assert!((p.recession).abs() < 1e-6);
    }

    #[test]
    fn test_base_curve_centre() {
        let v = base_curve(0.0, 1.0);
        assert!(v > 0.0);
    }

    #[test]
    fn test_temple_dip_at_temple() {
        let d = temple_dip(0.7, 1.0);
        assert!(d < 0.0);
    }

    #[test]
    fn test_temple_dip_at_centre() {
        let d = temple_dip(0.0, 1.0);
        assert!(d.abs() < 1e-6);
    }

    #[test]
    fn test_widows_peak_centre() {
        let w = widows_peak(0.0, 1.0);
        assert!(w < 0.0);
    }

    #[test]
    fn test_widows_peak_lateral() {
        let w = widows_peak(0.5, 1.0);
        assert!(w.abs() < 1e-6);
    }

    #[test]
    fn test_classify_receding() {
        let p = HairlineParams { recession: 0.8, ..Default::default() };
        assert_eq!(classify_hairline(&p), HairlineType::Receding);
    }

    #[test]
    fn test_classify_widows_peak() {
        let p = HairlineParams { widows_peak: 0.7, ..Default::default() };
        assert_eq!(classify_hairline(&p), HairlineType::WidowsPeak);
    }

    #[test]
    fn test_evaluate_empty() {
        let r = evaluate_hairline(&[], &HairlineParams::default());
        assert!(r.displacements.is_empty());
    }

    #[test]
    fn test_blend_hairline() {
        let a = HairlineParams::default();
        let b = HairlineParams { height: 1.0, ..Default::default() };
        let r = blend_hairline_params(&a, &b, 0.5);
        assert!((r.height - 0.75).abs() < 1e-5);
    }
}
