// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Hair shaft thickness morph parameters.

/// Parameters for hair thickness morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HairThicknessParams {
    pub shaft_thickness: f32,
    pub root_taper: f32,
    pub tip_taper: f32,
    pub medulla_fraction: f32,
}

impl Default for HairThicknessParams {
    fn default() -> Self {
        HairThicknessParams {
            shaft_thickness: 0.0,
            root_taper: 0.0,
            tip_taper: 0.0,
            medulla_fraction: 0.0,
        }
    }
}

#[allow(dead_code)]
pub fn default_hair_thickness_params() -> HairThicknessParams {
    HairThicknessParams::default()
}

#[allow(dead_code)]
pub fn ht_set_shaft(p: &mut HairThicknessParams, v: f32) {
    p.shaft_thickness = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn ht_set_root_taper(p: &mut HairThicknessParams, v: f32) {
    p.root_taper = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn ht_set_tip_taper(p: &mut HairThicknessParams, v: f32) {
    p.tip_taper = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn ht_set_medulla(p: &mut HairThicknessParams, v: f32) {
    p.medulla_fraction = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn ht_reset(p: &mut HairThicknessParams) {
    *p = HairThicknessParams::default();
}

#[allow(dead_code)]
pub fn ht_is_neutral(p: &HairThicknessParams) -> bool {
    p.shaft_thickness.abs() < 1e-6 && p.root_taper.abs() < 1e-6 && p.tip_taper.abs() < 1e-6
}

#[allow(dead_code)]
pub fn ht_effective_diameter_um(p: &HairThicknessParams) -> f32 {
    70.0 + p.shaft_thickness * 30.0
}

#[allow(dead_code)]
pub fn ht_blend(a: &HairThicknessParams, b: &HairThicknessParams, t: f32) -> HairThicknessParams {
    let t = t.clamp(0.0, 1.0);
    HairThicknessParams {
        shaft_thickness: a.shaft_thickness + (b.shaft_thickness - a.shaft_thickness) * t,
        root_taper: a.root_taper + (b.root_taper - a.root_taper) * t,
        tip_taper: a.tip_taper + (b.tip_taper - a.tip_taper) * t,
        medulla_fraction: a.medulla_fraction + (b.medulla_fraction - a.medulla_fraction) * t,
    }
}

#[allow(dead_code)]
pub fn ht_to_json(p: &HairThicknessParams) -> String {
    format!(
        r#"{{"shaft_thickness":{:.4},"root_taper":{:.4},"tip_taper":{:.4},"medulla_fraction":{:.4}}}"#,
        p.shaft_thickness, p.root_taper, p.tip_taper, p.medulla_fraction
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_neutral() {
        assert!(ht_is_neutral(&default_hair_thickness_params()));
    }

    #[test]
    fn set_shaft_clamps() {
        let mut p = default_hair_thickness_params();
        ht_set_shaft(&mut p, 5.0);
        assert!((p.shaft_thickness - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_root_taper_clamps_negative() {
        let mut p = default_hair_thickness_params();
        ht_set_root_taper(&mut p, -1.0);
        assert!(p.root_taper.abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let mut p = default_hair_thickness_params();
        ht_set_shaft(&mut p, 0.7);
        ht_reset(&mut p);
        assert!(ht_is_neutral(&p));
    }

    #[test]
    fn effective_diameter_base() {
        let p = default_hair_thickness_params();
        assert!((ht_effective_diameter_um(&p) - 70.0).abs() < 1e-4);
    }

    #[test]
    fn effective_diameter_increases() {
        let mut p = default_hair_thickness_params();
        ht_set_shaft(&mut p, 1.0);
        assert!(ht_effective_diameter_um(&p) > 70.0);
    }

    #[test]
    fn blend_midpoint() {
        let a = default_hair_thickness_params();
        let mut b = default_hair_thickness_params();
        ht_set_shaft(&mut b, 1.0);
        let m = ht_blend(&a, &b, 0.5);
        assert!((m.shaft_thickness - 0.5).abs() < 1e-5);
    }

    #[test]
    fn blend_t_clamped() {
        let a = default_hair_thickness_params();
        let b = default_hair_thickness_params();
        let m = ht_blend(&a, &b, 10.0);
        assert!((m.shaft_thickness - b.shaft_thickness).abs() < 1e-5);
    }

    #[test]
    fn to_json_has_shaft_thickness() {
        assert!(ht_to_json(&default_hair_thickness_params()).contains("shaft_thickness"));
    }
}
