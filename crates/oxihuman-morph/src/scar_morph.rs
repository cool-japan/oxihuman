// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Scar tissue morph (raised, depressed, linear).

/// Type of scar.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScarType {
    Raised,
    Depressed,
    Linear,
    Keloid,
    Atrophic,
}

impl ScarType {
    #[allow(dead_code)]
    pub fn name(self) -> &'static str {
        match self {
            ScarType::Raised => "raised",
            ScarType::Depressed => "depressed",
            ScarType::Linear => "linear",
            ScarType::Keloid => "keloid",
            ScarType::Atrophic => "atrophic",
        }
    }
}

/// Scar morph parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ScarMorphParams {
    pub scar_type: ScarType,
    pub prominence: f32,
    pub width: f32,
    pub length: f32,
    pub roughness: f32,
}

impl ScarMorphParams {
    #[allow(dead_code)]
    pub fn new(scar_type: ScarType) -> Self {
        ScarMorphParams {
            scar_type,
            prominence: 0.0,
            width: 0.0,
            length: 0.0,
            roughness: 0.0,
        }
    }
}

#[allow(dead_code)]
pub fn default_scar_morph_params() -> ScarMorphParams {
    ScarMorphParams::new(ScarType::Linear)
}

#[allow(dead_code)]
pub fn scar_set_prominence(p: &mut ScarMorphParams, v: f32) {
    p.prominence = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn scar_set_width(p: &mut ScarMorphParams, v: f32) {
    p.width = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn scar_set_length(p: &mut ScarMorphParams, v: f32) {
    p.length = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn scar_set_roughness(p: &mut ScarMorphParams, v: f32) {
    p.roughness = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn scar_reset(p: &mut ScarMorphParams) {
    let t = p.scar_type;
    *p = ScarMorphParams::new(t);
}

#[allow(dead_code)]
pub fn scar_is_neutral(p: &ScarMorphParams) -> bool {
    p.prominence.abs() < 1e-6 && p.width.abs() < 1e-6 && p.length.abs() < 1e-6
}

#[allow(dead_code)]
pub fn scar_visibility(p: &ScarMorphParams) -> f32 {
    p.prominence * 0.5 + p.roughness * 0.3 + (p.width * p.length).sqrt() * 0.2
}

#[allow(dead_code)]
pub fn scar_blend(a: &ScarMorphParams, b: &ScarMorphParams, t: f32) -> ScarMorphParams {
    let t = t.clamp(0.0, 1.0);
    ScarMorphParams {
        scar_type: if t < 0.5 { a.scar_type } else { b.scar_type },
        prominence: a.prominence + (b.prominence - a.prominence) * t,
        width: a.width + (b.width - a.width) * t,
        length: a.length + (b.length - a.length) * t,
        roughness: a.roughness + (b.roughness - a.roughness) * t,
    }
}

#[allow(dead_code)]
pub fn scar_to_json(p: &ScarMorphParams) -> String {
    format!(
        r#"{{"type":"{}","prominence":{:.4},"width":{:.4},"length":{:.4},"roughness":{:.4}}}"#,
        p.scar_type.name(),
        p.prominence,
        p.width,
        p.length,
        p.roughness
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_neutral() {
        assert!(scar_is_neutral(&default_scar_morph_params()));
    }

    #[test]
    fn set_prominence_clamps() {
        let mut p = default_scar_morph_params();
        scar_set_prominence(&mut p, 5.0);
        assert!((p.prominence - 1.0).abs() < 1e-6);
    }

    #[test]
    fn scar_type_names() {
        assert_eq!(ScarType::Raised.name(), "raised");
        assert_eq!(ScarType::Keloid.name(), "keloid");
    }

    #[test]
    fn reset_clears() {
        let mut p = ScarMorphParams::new(ScarType::Raised);
        scar_set_prominence(&mut p, 0.8);
        scar_reset(&mut p);
        assert!(scar_is_neutral(&p));
    }

    #[test]
    fn reset_preserves_type() {
        let mut p = ScarMorphParams::new(ScarType::Keloid);
        scar_reset(&mut p);
        assert_eq!(p.scar_type, ScarType::Keloid);
    }

    #[test]
    fn visibility_zero_when_neutral() {
        assert!(scar_visibility(&default_scar_morph_params()).abs() < 1e-6);
    }

    #[test]
    fn visibility_positive_with_prominence() {
        let mut p = default_scar_morph_params();
        scar_set_prominence(&mut p, 1.0);
        assert!(scar_visibility(&p) > 0.0);
    }

    #[test]
    fn blend_midpoint() {
        let a = default_scar_morph_params();
        let mut b = default_scar_morph_params();
        scar_set_prominence(&mut b, 1.0);
        let m = scar_blend(&a, &b, 0.5);
        assert!((m.prominence - 0.5).abs() < 1e-5);
    }

    #[test]
    fn to_json_contains_type() {
        assert!(scar_to_json(&default_scar_morph_params()).contains("type"));
    }
}
