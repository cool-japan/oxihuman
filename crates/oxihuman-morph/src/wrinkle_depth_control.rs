// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Wrinkle depth and density morph control.

/// Wrinkle zone identifier.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WrinkleZone {
    Forehead,
    Glabella,
    EyeCorner,
    NasolabialFold,
    Mouth,
    Neck,
    Custom,
}

/// Parameters for a single wrinkle zone.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WrinkleDepthParams {
    pub zone: WrinkleZone,
    pub depth: f32,
    pub density: f32,
    pub softness: f32,
}

impl WrinkleDepthParams {
    #[allow(dead_code)]
    pub fn new(zone: WrinkleZone) -> Self {
        WrinkleDepthParams {
            zone,
            depth: 0.0,
            density: 0.0,
            softness: 0.5,
        }
    }
}

#[allow(dead_code)]
pub fn wd_set_depth(p: &mut WrinkleDepthParams, v: f32) {
    p.depth = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn wd_set_density(p: &mut WrinkleDepthParams, v: f32) {
    p.density = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn wd_set_softness(p: &mut WrinkleDepthParams, v: f32) {
    p.softness = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn wd_reset(p: &mut WrinkleDepthParams) {
    let zone = p.zone;
    *p = WrinkleDepthParams::new(zone);
}

#[allow(dead_code)]
pub fn wd_is_neutral(p: &WrinkleDepthParams) -> bool {
    p.depth.abs() < 1e-6 && p.density.abs() < 1e-6
}

#[allow(dead_code)]
pub fn wd_visibility(p: &WrinkleDepthParams) -> f32 {
    p.depth * 0.6 + p.density * 0.4
}

#[allow(dead_code)]
pub fn wd_blend(a: &WrinkleDepthParams, b: &WrinkleDepthParams, t: f32) -> WrinkleDepthParams {
    let t = t.clamp(0.0, 1.0);
    WrinkleDepthParams {
        zone: a.zone,
        depth: a.depth + (b.depth - a.depth) * t,
        density: a.density + (b.density - a.density) * t,
        softness: a.softness + (b.softness - a.softness) * t,
    }
}

#[allow(dead_code)]
pub fn wd_to_json(p: &WrinkleDepthParams) -> String {
    format!(
        r#"{{"depth":{:.4},"density":{:.4},"softness":{:.4}}}"#,
        p.depth, p.density, p.softness
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_neutral() {
        assert!(wd_is_neutral(&WrinkleDepthParams::new(
            WrinkleZone::Forehead
        )));
    }

    #[test]
    fn set_depth_clamps() {
        let mut p = WrinkleDepthParams::new(WrinkleZone::Glabella);
        wd_set_depth(&mut p, 3.0);
        assert!((p.depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_density_clamps_negative() {
        let mut p = WrinkleDepthParams::new(WrinkleZone::Neck);
        wd_set_density(&mut p, -1.0);
        assert!(p.density.abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let mut p = WrinkleDepthParams::new(WrinkleZone::Mouth);
        wd_set_depth(&mut p, 0.7);
        wd_reset(&mut p);
        assert!(wd_is_neutral(&p));
    }

    #[test]
    fn visibility_zero_when_neutral() {
        let p = WrinkleDepthParams::new(WrinkleZone::EyeCorner);
        assert!(wd_visibility(&p).abs() < 1e-6);
    }

    #[test]
    fn visibility_increases() {
        let mut p = WrinkleDepthParams::new(WrinkleZone::Forehead);
        wd_set_depth(&mut p, 1.0);
        assert!(wd_visibility(&p) > 0.0);
    }

    #[test]
    fn blend_midpoint() {
        let a = WrinkleDepthParams::new(WrinkleZone::Forehead);
        let mut b = WrinkleDepthParams::new(WrinkleZone::Forehead);
        wd_set_depth(&mut b, 1.0);
        let m = wd_blend(&a, &b, 0.5);
        assert!((m.depth - 0.5).abs() < 1e-5);
    }

    #[test]
    fn zone_preserved_after_reset() {
        let mut p = WrinkleDepthParams::new(WrinkleZone::NasolabialFold);
        wd_reset(&mut p);
        assert_eq!(p.zone, WrinkleZone::NasolabialFold);
    }

    #[test]
    fn to_json_contains_depth() {
        let p = WrinkleDepthParams::new(WrinkleZone::Forehead);
        assert!(wd_to_json(&p).contains("depth"));
    }
}
