// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Hand dorsum thickness and knuckle prominence morph (v2).

/// Parameters for hand v2 morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HandV2Params {
    pub dorsum_thickness: f32,
    pub knuckle_prominence: f32,
    pub vein_raise: f32,
    pub tendon_visibility: f32,
}

impl Default for HandV2Params {
    fn default() -> Self {
        HandV2Params {
            dorsum_thickness: 0.0,
            knuckle_prominence: 0.0,
            vein_raise: 0.0,
            tendon_visibility: 0.0,
        }
    }
}

#[allow(dead_code)]
pub fn default_hand_v2_params() -> HandV2Params {
    HandV2Params::default()
}

#[allow(dead_code)]
pub fn hv2_set_dorsum(p: &mut HandV2Params, v: f32) {
    p.dorsum_thickness = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn hv2_set_knuckle(p: &mut HandV2Params, v: f32) {
    p.knuckle_prominence = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn hv2_set_vein(p: &mut HandV2Params, v: f32) {
    p.vein_raise = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn hv2_set_tendon(p: &mut HandV2Params, v: f32) {
    p.tendon_visibility = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn hv2_reset(p: &mut HandV2Params) {
    *p = HandV2Params::default();
}

#[allow(dead_code)]
pub fn hv2_is_neutral(p: &HandV2Params) -> bool {
    p.dorsum_thickness.abs() < 1e-6
        && p.knuckle_prominence.abs() < 1e-6
        && p.vein_raise.abs() < 1e-6
        && p.tendon_visibility.abs() < 1e-6
}

#[allow(dead_code)]
pub fn hv2_blend(a: &HandV2Params, b: &HandV2Params, t: f32) -> HandV2Params {
    let t = t.clamp(0.0, 1.0);
    HandV2Params {
        dorsum_thickness: a.dorsum_thickness + (b.dorsum_thickness - a.dorsum_thickness) * t,
        knuckle_prominence: a.knuckle_prominence
            + (b.knuckle_prominence - a.knuckle_prominence) * t,
        vein_raise: a.vein_raise + (b.vein_raise - a.vein_raise) * t,
        tendon_visibility: a.tendon_visibility + (b.tendon_visibility - a.tendon_visibility) * t,
    }
}

#[allow(dead_code)]
pub fn hv2_surface_detail_estimate(p: &HandV2Params) -> f32 {
    p.knuckle_prominence * 0.5 + p.vein_raise * 0.3 + p.tendon_visibility * 0.2
}

#[allow(dead_code)]
pub fn hv2_to_json(p: &HandV2Params) -> String {
    format!(
        r#"{{"dorsum_thickness":{:.4},"knuckle_prominence":{:.4},"vein_raise":{:.4},"tendon_visibility":{:.4}}}"#,
        p.dorsum_thickness, p.knuckle_prominence, p.vein_raise, p.tendon_visibility
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_neutral() {
        assert!(hv2_is_neutral(&default_hand_v2_params()));
    }

    #[test]
    fn set_dorsum_clamps() {
        let mut p = default_hand_v2_params();
        hv2_set_dorsum(&mut p, 5.0);
        assert!((p.dorsum_thickness - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_knuckle_clamps() {
        let mut p = default_hand_v2_params();
        hv2_set_knuckle(&mut p, -1.0);
        assert!(p.knuckle_prominence.abs() < 1e-6);
    }

    #[test]
    fn set_vein_positive() {
        let mut p = default_hand_v2_params();
        hv2_set_vein(&mut p, 0.7);
        assert!((p.vein_raise - 0.7).abs() < 1e-5);
    }

    #[test]
    fn reset_clears() {
        let mut p = default_hand_v2_params();
        hv2_set_knuckle(&mut p, 0.9);
        hv2_reset(&mut p);
        assert!(hv2_is_neutral(&p));
    }

    #[test]
    fn blend_midpoint() {
        let a = default_hand_v2_params();
        let mut b = default_hand_v2_params();
        hv2_set_knuckle(&mut b, 1.0);
        let m = hv2_blend(&a, &b, 0.5);
        assert!((m.knuckle_prominence - 0.5).abs() < 1e-5);
    }

    #[test]
    fn surface_detail_zero_when_neutral() {
        let p = default_hand_v2_params();
        assert!(hv2_surface_detail_estimate(&p).abs() < 1e-6);
    }

    #[test]
    fn surface_detail_positive() {
        let mut p = default_hand_v2_params();
        hv2_set_knuckle(&mut p, 1.0);
        assert!(hv2_surface_detail_estimate(&p) > 0.0);
    }

    #[test]
    fn to_json_has_fields() {
        let p = default_hand_v2_params();
        let j = hv2_to_json(&p);
        assert!(j.contains("dorsum_thickness"));
        assert!(j.contains("knuckle_prominence"));
    }
}
