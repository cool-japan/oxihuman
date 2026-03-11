// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Skin surface roughness texture morph parameters.

/// Parameters controlling skin roughness morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SkinRoughnessParams {
    pub roughness: f32,
    pub micro_roughness: f32,
    pub anisotropy: f32,
    pub scale: f32,
}

impl Default for SkinRoughnessParams {
    fn default() -> Self {
        SkinRoughnessParams {
            roughness: 0.4,
            micro_roughness: 0.0,
            anisotropy: 0.0,
            scale: 1.0,
        }
    }
}

#[allow(dead_code)]
pub fn default_skin_roughness_params() -> SkinRoughnessParams {
    SkinRoughnessParams::default()
}

#[allow(dead_code)]
pub fn sr_set_roughness(p: &mut SkinRoughnessParams, v: f32) {
    p.roughness = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn sr_set_micro_roughness(p: &mut SkinRoughnessParams, v: f32) {
    p.micro_roughness = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn sr_set_anisotropy(p: &mut SkinRoughnessParams, v: f32) {
    p.anisotropy = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn sr_set_scale(p: &mut SkinRoughnessParams, v: f32) {
    p.scale = v.clamp(0.1, 10.0);
}

#[allow(dead_code)]
pub fn sr_reset(p: &mut SkinRoughnessParams) {
    *p = SkinRoughnessParams::default();
}

#[allow(dead_code)]
pub fn sr_effective_roughness(p: &SkinRoughnessParams) -> f32 {
    (p.roughness + p.micro_roughness * 0.2).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn sr_blend(a: &SkinRoughnessParams, b: &SkinRoughnessParams, t: f32) -> SkinRoughnessParams {
    let t = t.clamp(0.0, 1.0);
    SkinRoughnessParams {
        roughness: a.roughness + (b.roughness - a.roughness) * t,
        micro_roughness: a.micro_roughness + (b.micro_roughness - a.micro_roughness) * t,
        anisotropy: a.anisotropy + (b.anisotropy - a.anisotropy) * t,
        scale: a.scale + (b.scale - a.scale) * t,
    }
}

#[allow(dead_code)]
pub fn sr_is_specular_dominant(p: &SkinRoughnessParams) -> bool {
    p.roughness < 0.3
}

#[allow(dead_code)]
pub fn sr_to_json(p: &SkinRoughnessParams) -> String {
    format!(
        r#"{{"roughness":{:.4},"micro_roughness":{:.4},"anisotropy":{:.4},"scale":{:.4}}}"#,
        p.roughness, p.micro_roughness, p.anisotropy, p.scale
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_roughness_is_mid() {
        let p = default_skin_roughness_params();
        assert!((p.roughness - 0.4).abs() < 1e-6);
    }

    #[test]
    fn set_roughness_clamps() {
        let mut p = default_skin_roughness_params();
        sr_set_roughness(&mut p, 2.0);
        assert!((p.roughness - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_anisotropy_negative() {
        let mut p = default_skin_roughness_params();
        sr_set_anisotropy(&mut p, -0.5);
        assert!((p.anisotropy - (-0.5)).abs() < 1e-6);
    }

    #[test]
    fn reset_restores_default() {
        let mut p = default_skin_roughness_params();
        sr_set_roughness(&mut p, 0.9);
        sr_reset(&mut p);
        assert!((p.roughness - 0.4).abs() < 1e-6);
    }

    #[test]
    fn effective_roughness_increases_with_micro() {
        let mut p = default_skin_roughness_params();
        let base = sr_effective_roughness(&p);
        sr_set_micro_roughness(&mut p, 1.0);
        assert!(sr_effective_roughness(&p) >= base);
    }

    #[test]
    fn specular_dominant_low_roughness() {
        let mut p = default_skin_roughness_params();
        sr_set_roughness(&mut p, 0.1);
        assert!(sr_is_specular_dominant(&p));
    }

    #[test]
    fn specular_not_dominant_high_roughness() {
        let mut p = default_skin_roughness_params();
        sr_set_roughness(&mut p, 0.9);
        assert!(!sr_is_specular_dominant(&p));
    }

    #[test]
    fn blend_midpoint() {
        let a = default_skin_roughness_params();
        let mut b = default_skin_roughness_params();
        sr_set_roughness(&mut b, 1.0);
        let m = sr_blend(&a, &b, 0.5);
        assert!((m.roughness - (a.roughness + 1.0) * 0.5).abs() < 1e-5);
    }

    #[test]
    fn to_json_contains_roughness() {
        assert!(sr_to_json(&default_skin_roughness_params()).contains("roughness"));
    }
}
