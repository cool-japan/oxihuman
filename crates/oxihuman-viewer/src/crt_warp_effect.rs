// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! CRT warp effect — CRT screen barrel distortion and curvature parameters.

/// Config.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct CrtWarpEffectConfig {
    /// Barrel distortion coefficient k1 (-0.5..=0.5, negative = pincushion).
    pub k1: f32,
    /// Secondary distortion coefficient k2.
    pub k2: f32,
    /// Corner rounding radius 0..=1 (1 = fully round).
    pub corner_radius: f32,
    /// Vignette strength 0..=1 added at corners.
    pub vignette: f32,
    /// Screen curvature depth 0..=1.
    pub curvature: f32,
    pub enabled: bool,
}

impl Default for CrtWarpEffectConfig {
    fn default() -> Self {
        Self {
            k1: 0.1,
            k2: 0.02,
            corner_radius: 0.05,
            vignette: 0.4,
            curvature: 0.3,
            enabled: true,
        }
    }
}

#[allow(dead_code)]
pub fn new_crt_warp_effect_config() -> CrtWarpEffectConfig {
    CrtWarpEffectConfig::default()
}

#[allow(dead_code)]
pub fn cw_set_k1(cfg: &mut CrtWarpEffectConfig, v: f32) {
    cfg.k1 = v.clamp(-0.5, 0.5);
}

#[allow(dead_code)]
pub fn cw_set_k2(cfg: &mut CrtWarpEffectConfig, v: f32) {
    cfg.k2 = v.clamp(-0.5, 0.5);
}

#[allow(dead_code)]
pub fn cw_set_vignette(cfg: &mut CrtWarpEffectConfig, v: f32) {
    cfg.vignette = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn cw_set_curvature(cfg: &mut CrtWarpEffectConfig, v: f32) {
    cfg.curvature = v.clamp(0.0, 1.0);
}

/// Apply barrel distortion to a normalised screen UV coordinate (centred at 0,0).
/// Returns the remapped UV.
#[allow(dead_code)]
pub fn cw_distort_uv(uv: [f32; 2], cfg: &CrtWarpEffectConfig) -> [f32; 2] {
    if !cfg.enabled {
        return uv;
    }
    let r2 = uv[0] * uv[0] + uv[1] * uv[1];
    let factor = 1.0 + cfg.k1 * r2 + cfg.k2 * r2 * r2;
    [uv[0] * factor, uv[1] * factor]
}

/// Compute vignette darkening (0 = full dark, 1 = no darkening) for a centred UV.
#[allow(dead_code)]
pub fn cw_vignette_at(uv: [f32; 2], cfg: &CrtWarpEffectConfig) -> f32 {
    let r = (uv[0] * uv[0] + uv[1] * uv[1]).sqrt();
    let dark = r * r * cfg.vignette;
    (1.0 - dark).clamp(0.0, 1.0)
}

/// Returns true if a distorted UV is within the visible screen area (|u|,|v| <= 0.5).
#[allow(dead_code)]
pub fn cw_is_visible(distorted_uv: [f32; 2]) -> bool {
    distorted_uv[0].abs() <= 0.5 && distorted_uv[1].abs() <= 0.5
}

/// Total distortion magnitude at the corner (0.5, 0.5).
#[allow(dead_code)]
pub fn cw_corner_distortion(cfg: &CrtWarpEffectConfig) -> f32 {
    let uv = [0.5_f32, 0.5_f32];
    let d = cw_distort_uv(uv, cfg);
    let dx = d[0] - uv[0];
    let dy = d[1] - uv[1];
    (dx * dx + dy * dy).sqrt()
}

#[allow(dead_code)]
pub fn cw_to_json(cfg: &CrtWarpEffectConfig) -> String {
    format!(
        "{{\"k1\":{:.4},\"k2\":{:.4},\"vignette\":{:.3},\"curvature\":{:.3},\"enabled\":{}}}",
        cfg.k1, cfg.k2, cfg.vignette, cfg.curvature, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_returns_identity() {
        let mut cfg = new_crt_warp_effect_config();
        cfg.enabled = false;
        let uv = [0.3_f32, 0.2_f32];
        let d = cw_distort_uv(uv, &cfg);
        assert!((d[0] - uv[0]).abs() < 1e-6 && (d[1] - uv[1]).abs() < 1e-6);
    }

    #[test]
    fn positive_k1_expands() {
        let mut cfg = new_crt_warp_effect_config();
        cfg.k1 = 0.2;
        let d = cw_distort_uv([0.4, 0.0], &cfg);
        assert!(d[0] > 0.4);
    }

    #[test]
    fn negative_k1_contracts() {
        let mut cfg = new_crt_warp_effect_config();
        cfg.k1 = -0.2;
        let d = cw_distort_uv([0.4, 0.0], &cfg);
        assert!(d[0] < 0.4);
    }

    #[test]
    fn vignette_at_center_one() {
        let cfg = new_crt_warp_effect_config();
        let v = cw_vignette_at([0.0, 0.0], &cfg);
        assert!((v - 1.0).abs() < 1e-5);
    }

    #[test]
    fn vignette_at_edge_less_than_center() {
        let cfg = new_crt_warp_effect_config();
        let center = cw_vignette_at([0.0, 0.0], &cfg);
        let edge = cw_vignette_at([0.7, 0.7], &cfg);
        assert!(edge <= center);
    }

    #[test]
    fn k1_clamps() {
        let mut cfg = new_crt_warp_effect_config();
        cw_set_k1(&mut cfg, 5.0);
        assert!((cfg.k1 - 0.5).abs() < 1e-6);
    }

    #[test]
    fn vignette_clamps() {
        let mut cfg = new_crt_warp_effect_config();
        cw_set_vignette(&mut cfg, 5.0);
        assert!((cfg.vignette - 1.0).abs() < 1e-6);
    }

    #[test]
    fn corner_distortion_nonnegative() {
        let cfg = new_crt_warp_effect_config();
        assert!(cw_corner_distortion(&cfg) >= 0.0);
    }

    #[test]
    fn center_uv_visible() {
        assert!(cw_is_visible([0.0, 0.0]));
    }

    #[test]
    fn json_has_keys() {
        let j = cw_to_json(&new_crt_warp_effect_config());
        assert!(j.contains("k1") && j.contains("enabled"));
    }
}
