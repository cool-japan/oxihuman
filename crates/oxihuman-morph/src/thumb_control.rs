// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Thumb opposition and curvature morph control.

/// Thumb morph parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ThumbParams {
    pub opposition: f32,
    pub curvature: f32,
    pub girth: f32,
    pub length_scale: f32,
}

#[allow(dead_code)]
pub fn default_thumb_params() -> ThumbParams {
    ThumbParams::default()
}

#[allow(dead_code)]
pub fn thumb_set_opposition(p: &mut ThumbParams, v: f32) {
    p.opposition = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn thumb_set_curvature(p: &mut ThumbParams, v: f32) {
    p.curvature = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn thumb_set_girth(p: &mut ThumbParams, v: f32) {
    p.girth = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn thumb_set_length_scale(p: &mut ThumbParams, v: f32) {
    p.length_scale = v.clamp(-0.5, 0.5);
}

#[allow(dead_code)]
pub fn thumb_reset(p: &mut ThumbParams) {
    *p = ThumbParams::default();
}

#[allow(dead_code)]
pub fn thumb_is_neutral(p: &ThumbParams) -> bool {
    p.opposition.abs() < 1e-6
        && p.curvature.abs() < 1e-6
        && p.girth.abs() < 1e-6
        && p.length_scale.abs() < 1e-6
}

#[allow(dead_code)]
pub fn thumb_blend(a: &ThumbParams, b: &ThumbParams, t: f32) -> ThumbParams {
    let t = t.clamp(0.0, 1.0);
    ThumbParams {
        opposition: a.opposition + (b.opposition - a.opposition) * t,
        curvature: a.curvature + (b.curvature - a.curvature) * t,
        girth: a.girth + (b.girth - a.girth) * t,
        length_scale: a.length_scale + (b.length_scale - a.length_scale) * t,
    }
}

#[allow(dead_code)]
pub fn thumb_opposition_angle_deg(p: &ThumbParams) -> f32 {
    p.opposition * 70.0
}

#[allow(dead_code)]
pub fn thumb_to_json(p: &ThumbParams) -> String {
    format!(
        r#"{{"opposition":{:.4},"curvature":{:.4},"girth":{:.4},"length_scale":{:.4}}}"#,
        p.opposition, p.curvature, p.girth, p.length_scale
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_neutral() {
        assert!(thumb_is_neutral(&default_thumb_params()));
    }

    #[test]
    fn set_opposition_clamps_above() {
        let mut p = default_thumb_params();
        thumb_set_opposition(&mut p, 2.0);
        assert!((p.opposition - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_opposition_clamps_below() {
        let mut p = default_thumb_params();
        thumb_set_opposition(&mut p, -1.0);
        assert!(p.opposition.abs() < 1e-6);
    }

    #[test]
    fn set_curvature_negative() {
        let mut p = default_thumb_params();
        thumb_set_curvature(&mut p, -0.5);
        assert!((p.curvature - (-0.5)).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let mut p = default_thumb_params();
        thumb_set_girth(&mut p, 0.8);
        thumb_reset(&mut p);
        assert!(thumb_is_neutral(&p));
    }

    #[test]
    fn blend_at_zero() {
        let a = default_thumb_params();
        let b = default_thumb_params();
        let m = thumb_blend(&a, &b, 0.5);
        assert!(thumb_is_neutral(&m));
    }

    #[test]
    fn blend_midpoint() {
        let a = default_thumb_params();
        let mut b = default_thumb_params();
        thumb_set_opposition(&mut b, 1.0);
        let m = thumb_blend(&a, &b, 0.5);
        assert!((m.opposition - 0.5).abs() < 1e-5);
    }

    #[test]
    fn opposition_angle_full() {
        let mut p = default_thumb_params();
        thumb_set_opposition(&mut p, 1.0);
        assert!((thumb_opposition_angle_deg(&p) - 70.0).abs() < 1e-4);
    }

    #[test]
    fn to_json_contains_girth() {
        assert!(thumb_to_json(&default_thumb_params()).contains("girth"));
    }
}
