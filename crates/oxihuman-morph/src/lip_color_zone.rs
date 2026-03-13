// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Lip color zone morph targets — vermillion, cupid's bow, etc.

/// Lip color zones.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LipZone {
    Vermillion,
    CupidsBow,
    Commissure,
    UpperBody,
    LowerBody,
    PhiltrumColumn,
}

/// Lip color zone parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct LipColorZoneParams {
    /// Vermillion border saturation boost 0..=1.
    pub vermillion_saturation: f32,
    /// Cupid's bow definition 0..=1.
    pub cupids_bow: f32,
    /// Commissure (corner) darkening 0..=1.
    pub commissure_dark: f32,
    /// Upper lip body redness 0..=1.
    pub upper_redness: f32,
    /// Lower lip body redness 0..=1.
    pub lower_redness: f32,
    /// Overall gloss/shine 0..=1.
    pub gloss: f32,
    /// Desaturation (chapping) 0..=1.
    pub desaturation: f32,
}

impl Default for LipColorZoneParams {
    fn default() -> Self {
        Self {
            vermillion_saturation: 0.5,
            cupids_bow: 0.4,
            commissure_dark: 0.2,
            upper_redness: 0.5,
            lower_redness: 0.5,
            gloss: 0.3,
            desaturation: 0.0,
        }
    }
}

/// Create default params.
#[allow(dead_code)]
pub fn default_lip_color_zone_params() -> LipColorZoneParams {
    LipColorZoneParams::default()
}

/// Set zone value.
#[allow(dead_code)]
pub fn set_lip_zone(params: &mut LipColorZoneParams, zone: LipZone, value: f32) {
    let v = value.clamp(0.0, 1.0);
    match zone {
        LipZone::Vermillion => params.vermillion_saturation = v,
        LipZone::CupidsBow => params.cupids_bow = v,
        LipZone::Commissure => params.commissure_dark = v,
        LipZone::UpperBody => params.upper_redness = v,
        LipZone::LowerBody => params.lower_redness = v,
        LipZone::PhiltrumColumn => {
            params.vermillion_saturation = (params.vermillion_saturation + v) * 0.5
        }
    }
}

/// Set gloss.
#[allow(dead_code)]
pub fn set_lip_gloss(params: &mut LipColorZoneParams, value: f32) {
    params.gloss = value.clamp(0.0, 1.0);
}

/// Set desaturation (chapping).
#[allow(dead_code)]
pub fn set_lip_desaturation(params: &mut LipColorZoneParams, value: f32) {
    params.desaturation = value.clamp(0.0, 1.0);
}

/// Apply lip color zone to an RGB color.
#[allow(dead_code)]
pub fn apply_lip_zone_color(base: [f32; 3], params: &LipColorZoneParams) -> [f32; 3] {
    let r_boost =
        (params.upper_redness + params.lower_redness) * 0.5 * params.vermillion_saturation * 0.3;
    let desat_factor = 1.0 - params.desaturation * 0.5;
    let lum = 0.2126 * base[0] + 0.7152 * base[1] + 0.0722 * base[2];
    let r = (base[0] * desat_factor + lum * (1.0 - desat_factor) + r_boost).clamp(0.0, 1.0);
    let g = (base[1] * desat_factor + lum * (1.0 - desat_factor)).clamp(0.0, 1.0);
    let b = (base[2] * desat_factor + lum * (1.0 - desat_factor)).clamp(0.0, 1.0);
    [r, g, b]
}

/// Blend two lip color zone params.
#[allow(dead_code)]
pub fn blend_lip_color_zone(
    a: &LipColorZoneParams,
    b: &LipColorZoneParams,
    t: f32,
) -> LipColorZoneParams {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    LipColorZoneParams {
        vermillion_saturation: a.vermillion_saturation * inv + b.vermillion_saturation * t,
        cupids_bow: a.cupids_bow * inv + b.cupids_bow * t,
        commissure_dark: a.commissure_dark * inv + b.commissure_dark * t,
        upper_redness: a.upper_redness * inv + b.upper_redness * t,
        lower_redness: a.lower_redness * inv + b.lower_redness * t,
        gloss: a.gloss * inv + b.gloss * t,
        desaturation: a.desaturation * inv + b.desaturation * t,
    }
}

/// Reset to default.
#[allow(dead_code)]
pub fn reset_lip_color_zone(params: &mut LipColorZoneParams) {
    *params = LipColorZoneParams::default();
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn lip_color_zone_to_json(params: &LipColorZoneParams) -> String {
    format!(
        r#"{{"vermillion":{:.4},"cupids_bow":{:.4},"gloss":{:.4},"desaturation":{:.4}}}"#,
        params.vermillion_saturation, params.cupids_bow, params.gloss, params.desaturation
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let p = LipColorZoneParams::default();
        assert!((0.0..=1.0).contains(&p.gloss));
    }

    #[test]
    fn test_set_zone_vermillion() {
        let mut p = LipColorZoneParams::default();
        set_lip_zone(&mut p, LipZone::Vermillion, 0.9);
        assert!((p.vermillion_saturation - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_set_zone_clamp() {
        let mut p = LipColorZoneParams::default();
        set_lip_zone(&mut p, LipZone::CupidsBow, 5.0);
        assert!((p.cupids_bow - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_gloss() {
        let mut p = LipColorZoneParams::default();
        set_lip_gloss(&mut p, 0.8);
        assert!((p.gloss - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_desaturation_clamp() {
        let mut p = LipColorZoneParams::default();
        set_lip_desaturation(&mut p, -1.0);
        assert!(p.desaturation.abs() < 1e-6);
    }

    #[test]
    fn test_apply_neutral() {
        let p = LipColorZoneParams {
            desaturation: 0.0,
            upper_redness: 0.0,
            lower_redness: 0.0,
            vermillion_saturation: 0.0,
            ..Default::default()
        };
        let base = [0.8f32, 0.4, 0.4];
        let out = apply_lip_zone_color(base, &p);
        assert!((out[0] - 0.8).abs() < 1e-5);
    }

    #[test]
    fn test_blend_midpoint() {
        let a = LipColorZoneParams {
            gloss: 0.0,
            ..Default::default()
        };
        let b = LipColorZoneParams {
            gloss: 1.0,
            ..Default::default()
        };
        let r = blend_lip_color_zone(&a, &b, 0.5);
        assert!((r.gloss - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let mut p = LipColorZoneParams {
            gloss: 0.99,
            ..Default::default()
        };
        reset_lip_color_zone(&mut p);
        assert!((p.gloss - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let j = lip_color_zone_to_json(&LipColorZoneParams::default());
        assert!(j.contains("vermillion"));
        assert!(j.contains("gloss"));
    }

    #[test]
    fn test_commissure_zone() {
        let mut p = LipColorZoneParams::default();
        set_lip_zone(&mut p, LipZone::Commissure, 0.7);
        assert!((p.commissure_dark - 0.7).abs() < 1e-6);
    }
}
