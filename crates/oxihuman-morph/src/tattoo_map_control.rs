// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Tattoo placement and opacity parameters.

/// Body region for tattoo placement.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TattooRegion {
    LeftArm,
    RightArm,
    Chest,
    Back,
    Neck,
    Leg,
    Hand,
}

impl TattooRegion {
    #[allow(dead_code)]
    pub fn name(self) -> &'static str {
        match self {
            TattooRegion::LeftArm => "left_arm",
            TattooRegion::RightArm => "right_arm",
            TattooRegion::Chest => "chest",
            TattooRegion::Back => "back",
            TattooRegion::Neck => "neck",
            TattooRegion::Leg => "leg",
            TattooRegion::Hand => "hand",
        }
    }
}

/// Parameters for a tattoo placement.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TattooParams {
    pub region: TattooRegion,
    pub opacity: f32,
    pub u_offset: f32,
    pub v_offset: f32,
    pub scale: f32,
    pub rotation_deg: f32,
    pub enabled: bool,
}

impl TattooParams {
    #[allow(dead_code)]
    pub fn new(region: TattooRegion) -> Self {
        TattooParams {
            region,
            opacity: 1.0,
            u_offset: 0.0,
            v_offset: 0.0,
            scale: 1.0,
            rotation_deg: 0.0,
            enabled: false,
        }
    }
}

#[allow(dead_code)]
pub fn default_tattoo_params() -> TattooParams {
    TattooParams::new(TattooRegion::LeftArm)
}

#[allow(dead_code)]
pub fn tattoo_set_opacity(p: &mut TattooParams, v: f32) {
    p.opacity = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn tattoo_set_offset(p: &mut TattooParams, u: f32, v: f32) {
    p.u_offset = u.clamp(-1.0, 1.0);
    p.v_offset = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn tattoo_set_scale(p: &mut TattooParams, s: f32) {
    p.scale = s.clamp(0.1, 5.0);
}

#[allow(dead_code)]
pub fn tattoo_set_rotation(p: &mut TattooParams, deg: f32) {
    p.rotation_deg = deg % 360.0;
}

#[allow(dead_code)]
pub fn tattoo_enable(p: &mut TattooParams) {
    p.enabled = true;
}

#[allow(dead_code)]
pub fn tattoo_disable(p: &mut TattooParams) {
    p.enabled = false;
}

#[allow(dead_code)]
pub fn tattoo_reset(p: &mut TattooParams) {
    let region = p.region;
    *p = TattooParams::new(region);
}

#[allow(dead_code)]
pub fn tattoo_effective_opacity(p: &TattooParams) -> f32 {
    if p.enabled {
        p.opacity
    } else {
        0.0
    }
}

#[allow(dead_code)]
pub fn tattoo_blend(a: &TattooParams, b: &TattooParams, t: f32) -> TattooParams {
    let t = t.clamp(0.0, 1.0);
    TattooParams {
        region: a.region,
        opacity: a.opacity + (b.opacity - a.opacity) * t,
        u_offset: a.u_offset + (b.u_offset - a.u_offset) * t,
        v_offset: a.v_offset + (b.v_offset - a.v_offset) * t,
        scale: a.scale + (b.scale - a.scale) * t,
        rotation_deg: a.rotation_deg + (b.rotation_deg - a.rotation_deg) * t,
        enabled: a.enabled || b.enabled,
    }
}

#[allow(dead_code)]
pub fn tattoo_to_json(p: &TattooParams) -> String {
    format!(
        r#"{{"region":"{}","opacity":{:.4},"enabled":{},"scale":{:.4}}}"#,
        p.region.name(),
        p.opacity,
        p.enabled,
        p.scale
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_not_enabled() {
        assert!(!default_tattoo_params().enabled);
    }

    #[test]
    fn effective_opacity_zero_when_disabled() {
        let p = default_tattoo_params();
        assert!(tattoo_effective_opacity(&p).abs() < 1e-6);
    }

    #[test]
    fn effective_opacity_when_enabled() {
        let mut p = default_tattoo_params();
        tattoo_enable(&mut p);
        assert!((tattoo_effective_opacity(&p) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_opacity_clamps() {
        let mut p = default_tattoo_params();
        tattoo_set_opacity(&mut p, 2.0);
        assert!((p.opacity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_scale_clamps_min() {
        let mut p = default_tattoo_params();
        tattoo_set_scale(&mut p, 0.0);
        assert!((p.scale - 0.1).abs() < 1e-6);
    }

    #[test]
    fn set_offset_clamps() {
        let mut p = default_tattoo_params();
        tattoo_set_offset(&mut p, 5.0, -5.0);
        assert!((p.u_offset - 1.0).abs() < 1e-6);
        assert!((p.v_offset - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn reset_preserves_region() {
        let mut p = TattooParams::new(TattooRegion::Back);
        tattoo_enable(&mut p);
        tattoo_reset(&mut p);
        assert_eq!(p.region, TattooRegion::Back);
        assert!(!p.enabled);
    }

    #[test]
    fn blend_midpoint_opacity() {
        let mut a = default_tattoo_params();
        let mut b = default_tattoo_params();
        tattoo_set_opacity(&mut a, 0.0);
        tattoo_set_opacity(&mut b, 1.0);
        let m = tattoo_blend(&a, &b, 0.5);
        assert!((m.opacity - 0.5).abs() < 1e-5);
    }

    #[test]
    fn to_json_contains_region() {
        assert!(tattoo_to_json(&default_tattoo_params()).contains("region"));
    }
}
