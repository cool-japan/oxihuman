// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Tooth shape and alignment morph control.

/// Tooth shape parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ToothShapeParams {
    /// Overall tooth width scale 0..=1.
    pub width: f32,
    /// Tooth height scale 0..=1.
    pub height: f32,
    /// Incisor rounding 0..=1 (0 = sharp, 1 = very round).
    pub rounding: f32,
    /// Overbite magnitude 0..=1.
    pub overbite: f32,
    /// Crowding 0..=1 (overlap/misalignment).
    pub crowding: f32,
    /// Whiteness 0..=1.
    pub whiteness: f32,
    /// Translucency of incisal edge 0..=1.
    pub translucency: f32,
}

impl Default for ToothShapeParams {
    fn default() -> Self {
        Self {
            width: 0.5,
            height: 0.5,
            rounding: 0.4,
            overbite: 0.2,
            crowding: 0.0,
            whiteness: 0.8,
            translucency: 0.2,
        }
    }
}

/// Create default params.
#[allow(dead_code)]
pub fn default_tooth_shape_params() -> ToothShapeParams {
    ToothShapeParams::default()
}

/// Set tooth width.
#[allow(dead_code)]
pub fn set_tooth_width(params: &mut ToothShapeParams, value: f32) {
    params.width = value.clamp(0.0, 1.0);
}

/// Set tooth height.
#[allow(dead_code)]
pub fn set_tooth_height(params: &mut ToothShapeParams, value: f32) {
    params.height = value.clamp(0.0, 1.0);
}

/// Set rounding.
#[allow(dead_code)]
pub fn set_tooth_rounding(params: &mut ToothShapeParams, value: f32) {
    params.rounding = value.clamp(0.0, 1.0);
}

/// Set overbite.
#[allow(dead_code)]
pub fn set_tooth_overbite(params: &mut ToothShapeParams, value: f32) {
    params.overbite = value.clamp(0.0, 1.0);
}

/// Set crowding.
#[allow(dead_code)]
pub fn set_tooth_crowding(params: &mut ToothShapeParams, value: f32) {
    params.crowding = value.clamp(0.0, 1.0);
}

/// Set whiteness.
#[allow(dead_code)]
pub fn set_tooth_whiteness(params: &mut ToothShapeParams, value: f32) {
    params.whiteness = value.clamp(0.0, 1.0);
}

/// Compute tooth color RGB from whiteness and translucency.
#[allow(dead_code)]
pub fn tooth_color_rgb(params: &ToothShapeParams) -> [f32; 3] {
    let w = params.whiteness.clamp(0.0, 1.0);
    let t = params.translucency.clamp(0.0, 1.0);
    let r = 0.85 + w * 0.12;
    let g = 0.80 + w * 0.10 - t * 0.03;
    let b = 0.70 + w * 0.08 - t * 0.05;
    [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)]
}

/// Blend two tooth shape params.
#[allow(dead_code)]
pub fn blend_tooth_shape(a: &ToothShapeParams, b: &ToothShapeParams, t: f32) -> ToothShapeParams {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    ToothShapeParams {
        width: a.width * inv + b.width * t,
        height: a.height * inv + b.height * t,
        rounding: a.rounding * inv + b.rounding * t,
        overbite: a.overbite * inv + b.overbite * t,
        crowding: a.crowding * inv + b.crowding * t,
        whiteness: a.whiteness * inv + b.whiteness * t,
        translucency: a.translucency * inv + b.translucency * t,
    }
}

/// Reset to default.
#[allow(dead_code)]
pub fn reset_tooth_shape(params: &mut ToothShapeParams) {
    *params = ToothShapeParams::default();
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn tooth_shape_to_json(params: &ToothShapeParams) -> String {
    format!(
        r#"{{"width":{:.4},"height":{:.4},"rounding":{:.4},"overbite":{:.4},"crowding":{:.4},"whiteness":{:.4}}}"#,
        params.width,
        params.height,
        params.rounding,
        params.overbite,
        params.crowding,
        params.whiteness
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let p = ToothShapeParams::default();
        assert!((0.0..=1.0).contains(&p.width));
    }

    #[test]
    fn test_set_width_clamp() {
        let mut p = ToothShapeParams::default();
        set_tooth_width(&mut p, 5.0);
        assert!((p.width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_height_clamp() {
        let mut p = ToothShapeParams::default();
        set_tooth_height(&mut p, -1.0);
        assert!(p.height.abs() < 1e-6);
    }

    #[test]
    fn test_set_rounding() {
        let mut p = ToothShapeParams::default();
        set_tooth_rounding(&mut p, 0.9);
        assert!((p.rounding - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_set_whiteness() {
        let mut p = ToothShapeParams::default();
        set_tooth_whiteness(&mut p, 0.3);
        assert!((p.whiteness - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_tooth_color_bright() {
        let p = ToothShapeParams {
            whiteness: 1.0,
            translucency: 0.0,
            ..Default::default()
        };
        let c = tooth_color_rgb(&p);
        assert!(c[0] > 0.9);
    }

    #[test]
    fn test_tooth_color_range() {
        let p = ToothShapeParams::default();
        let c = tooth_color_rgb(&p);
        for ch in c {
            assert!((0.0..=1.0).contains(&ch));
        }
    }

    #[test]
    fn test_blend_midpoint() {
        let a = ToothShapeParams {
            width: 0.0,
            ..Default::default()
        };
        let b = ToothShapeParams {
            width: 1.0,
            ..Default::default()
        };
        let r = blend_tooth_shape(&a, &b, 0.5);
        assert!((r.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let mut p = ToothShapeParams {
            whiteness: 0.1,
            ..Default::default()
        };
        reset_tooth_shape(&mut p);
        assert!((p.whiteness - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let j = tooth_shape_to_json(&ToothShapeParams::default());
        assert!(j.contains("whiteness"));
        assert!(j.contains("crowding"));
    }
}
