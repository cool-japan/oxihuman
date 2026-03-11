// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Eyebrow thickness and fullness morph.

/// Side of the face.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EyebrowSide {
    Left,
    Right,
    Both,
}

/// Eyebrow thickness morph parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyebrowThicknessParams {
    pub thickness_left: f32,
    pub thickness_right: f32,
    pub fullness: f32,
    pub density: f32,
}

impl Default for EyebrowThicknessParams {
    fn default() -> Self {
        EyebrowThicknessParams {
            thickness_left: 0.0,
            thickness_right: 0.0,
            fullness: 0.0,
            density: 0.0,
        }
    }
}

#[allow(dead_code)]
pub fn default_eyebrow_thickness_params() -> EyebrowThicknessParams {
    EyebrowThicknessParams::default()
}

#[allow(dead_code)]
pub fn ebt_set_thickness(p: &mut EyebrowThicknessParams, side: EyebrowSide, v: f32) {
    let v = v.clamp(-1.0, 1.0);
    match side {
        EyebrowSide::Left => p.thickness_left = v,
        EyebrowSide::Right => p.thickness_right = v,
        EyebrowSide::Both => {
            p.thickness_left = v;
            p.thickness_right = v;
        }
    }
}

#[allow(dead_code)]
pub fn ebt_set_fullness(p: &mut EyebrowThicknessParams, v: f32) {
    p.fullness = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn ebt_set_density(p: &mut EyebrowThicknessParams, v: f32) {
    p.density = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn ebt_reset(p: &mut EyebrowThicknessParams) {
    *p = EyebrowThicknessParams::default();
}

#[allow(dead_code)]
pub fn ebt_is_neutral(p: &EyebrowThicknessParams) -> bool {
    p.thickness_left.abs() < 1e-6
        && p.thickness_right.abs() < 1e-6
        && p.fullness.abs() < 1e-6
        && p.density.abs() < 1e-6
}

#[allow(dead_code)]
pub fn ebt_average_thickness(p: &EyebrowThicknessParams) -> f32 {
    (p.thickness_left + p.thickness_right) * 0.5
}

#[allow(dead_code)]
pub fn ebt_asymmetry(p: &EyebrowThicknessParams) -> f32 {
    (p.thickness_left - p.thickness_right).abs()
}

#[allow(dead_code)]
pub fn ebt_blend(
    a: &EyebrowThicknessParams,
    b: &EyebrowThicknessParams,
    t: f32,
) -> EyebrowThicknessParams {
    let t = t.clamp(0.0, 1.0);
    EyebrowThicknessParams {
        thickness_left: a.thickness_left + (b.thickness_left - a.thickness_left) * t,
        thickness_right: a.thickness_right + (b.thickness_right - a.thickness_right) * t,
        fullness: a.fullness + (b.fullness - a.fullness) * t,
        density: a.density + (b.density - a.density) * t,
    }
}

#[allow(dead_code)]
pub fn ebt_to_json(p: &EyebrowThicknessParams) -> String {
    format!(
        r#"{{"thickness_left":{:.4},"thickness_right":{:.4},"fullness":{:.4},"density":{:.4}}}"#,
        p.thickness_left, p.thickness_right, p.fullness, p.density
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_neutral() {
        assert!(ebt_is_neutral(&default_eyebrow_thickness_params()));
    }

    #[test]
    fn set_both_sides() {
        let mut p = default_eyebrow_thickness_params();
        ebt_set_thickness(&mut p, EyebrowSide::Both, 0.6);
        assert!((p.thickness_left - 0.6).abs() < 1e-6);
        assert!((p.thickness_right - 0.6).abs() < 1e-6);
    }

    #[test]
    fn set_left_only() {
        let mut p = default_eyebrow_thickness_params();
        ebt_set_thickness(&mut p, EyebrowSide::Left, 0.5);
        assert!(p.thickness_right.abs() < 1e-6);
    }

    #[test]
    fn thickness_clamps() {
        let mut p = default_eyebrow_thickness_params();
        ebt_set_thickness(&mut p, EyebrowSide::Right, 3.0);
        assert!((p.thickness_right - 1.0).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let mut p = default_eyebrow_thickness_params();
        ebt_set_fullness(&mut p, 0.9);
        ebt_reset(&mut p);
        assert!(ebt_is_neutral(&p));
    }

    #[test]
    fn average_thickness_symmetric() {
        let mut p = default_eyebrow_thickness_params();
        ebt_set_thickness(&mut p, EyebrowSide::Both, 0.8);
        assert!((ebt_average_thickness(&p) - 0.8).abs() < 1e-5);
    }

    #[test]
    fn asymmetry_zero_when_symmetric() {
        let mut p = default_eyebrow_thickness_params();
        ebt_set_thickness(&mut p, EyebrowSide::Both, 0.5);
        assert!(ebt_asymmetry(&p).abs() < 1e-6);
    }

    #[test]
    fn blend_midpoint() {
        let a = default_eyebrow_thickness_params();
        let mut b = default_eyebrow_thickness_params();
        ebt_set_fullness(&mut b, 1.0);
        let m = ebt_blend(&a, &b, 0.5);
        assert!((m.fullness - 0.5).abs() < 1e-5);
    }

    #[test]
    fn to_json_has_fullness() {
        assert!(ebt_to_json(&default_eyebrow_thickness_params()).contains("fullness"));
    }
}
