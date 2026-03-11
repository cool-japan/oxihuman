// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Environment specular IBL — split-sum approximation config and helpers.

use std::f32::consts::PI;

/// Config for environment specular lookup.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct EnvSpecularConfig {
    pub intensity: f32,
    pub max_mip_level: u32,
    pub brdf_lut_size: u32,
    pub enabled: bool,
}

impl Default for EnvSpecularConfig {
    fn default() -> Self {
        Self {
            intensity: 1.0,
            max_mip_level: 8,
            brdf_lut_size: 512,
            enabled: true,
        }
    }
}

#[allow(dead_code)]
pub fn new_env_specular() -> EnvSpecularConfig {
    EnvSpecularConfig::default()
}

#[allow(dead_code)]
pub fn es_set_intensity(cfg: &mut EnvSpecularConfig, v: f32) {
    cfg.intensity = v.max(0.0);
}

#[allow(dead_code)]
pub fn es_set_enabled(cfg: &mut EnvSpecularConfig, en: bool) {
    cfg.enabled = en;
}

#[allow(dead_code)]
pub fn es_reset(cfg: &mut EnvSpecularConfig) {
    *cfg = EnvSpecularConfig::default();
}

/// Map roughness to mip level.
#[allow(dead_code)]
pub fn es_roughness_to_mip(cfg: &EnvSpecularConfig, roughness: f32) -> f32 {
    let r = roughness.clamp(0.0, 1.0);
    r * cfg.max_mip_level as f32
}

/// GGX distribution (D term, isotropic).
#[allow(dead_code)]
pub fn es_ggx_d(n_dot_h: f32, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let d = (n_dot_h * n_dot_h) * (a2 - 1.0) + 1.0;
    a2 / (PI * d * d).max(1e-7)
}

/// Fresnel-Schlick approximation.
#[allow(dead_code)]
pub fn es_fresnel(f0: [f32; 3], v_dot_h: f32) -> [f32; 3] {
    let v = (1.0 - v_dot_h).powi(5);
    [
        f0[0] + (1.0 - f0[0]) * v,
        f0[1] + (1.0 - f0[1]) * v,
        f0[2] + (1.0 - f0[2]) * v,
    ]
}

/// Geometry Smith GGX (combined for indirect).
#[allow(dead_code)]
pub fn es_geometry_smith(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32 {
    let k = (roughness + 1.0).powi(2) / 8.0;
    let g_v = n_dot_v / (n_dot_v * (1.0 - k) + k).max(1e-7);
    let g_l = n_dot_l / (n_dot_l * (1.0 - k) + k).max(1e-7);
    g_v * g_l
}

/// Estimated memory for prefiltered env map in bytes.
#[allow(dead_code)]
pub fn es_memory_bytes(cfg: &EnvSpecularConfig, face_size: u32) -> u64 {
    let mut total = 0u64;
    let mut size = face_size;
    #[allow(clippy::needless_range_loop)]
    for _ in 0..=cfg.max_mip_level {
        total += 6 * size as u64 * size as u64 * 8; // RGBA16F
        if size > 1 {
            size /= 2;
        }
    }
    total
}

#[allow(dead_code)]
pub fn es_to_json(cfg: &EnvSpecularConfig) -> String {
    format!(
        "{{\"intensity\":{:.4},\"max_mip\":{},\"enabled\":{}}}",
        cfg.intensity, cfg.max_mip_level, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_enabled() {
        assert!(new_env_specular().enabled);
    }

    #[test]
    fn roughness_to_mip_zero() {
        let cfg = new_env_specular();
        assert!(es_roughness_to_mip(&cfg, 0.0) < 1e-5);
    }

    #[test]
    fn roughness_to_mip_one() {
        let cfg = new_env_specular();
        assert!((es_roughness_to_mip(&cfg, 1.0) - cfg.max_mip_level as f32).abs() < 1e-5);
    }

    #[test]
    fn ggx_d_positive() {
        assert!(es_ggx_d(0.9, 0.5) > 0.0);
    }

    #[test]
    fn fresnel_at_zero_angle_equals_f0() {
        let f0 = [0.04, 0.04, 0.04];
        let f = es_fresnel(f0, 1.0);
        assert!((f[0] - f0[0]).abs() < 1e-5);
    }

    #[test]
    fn fresnel_increases_at_grazing() {
        let f0 = [0.04, 0.04, 0.04];
        let f_normal = es_fresnel(f0, 1.0);
        let f_grazing = es_fresnel(f0, 0.0);
        assert!(f_grazing[0] > f_normal[0]);
    }

    #[test]
    fn geometry_smith_positive() {
        assert!(es_geometry_smith(0.9, 0.9, 0.5) > 0.0);
    }

    #[test]
    fn memory_bytes_positive() {
        let cfg = new_env_specular();
        assert!(es_memory_bytes(&cfg, 512) > 0);
    }

    #[test]
    fn reset_restores() {
        let mut cfg = new_env_specular();
        es_set_intensity(&mut cfg, 5.0);
        es_reset(&mut cfg);
        assert!((cfg.intensity - 1.0).abs() < 1e-5);
    }

    #[test]
    fn json_has_intensity() {
        let j = es_to_json(&new_env_specular());
        assert!(j.contains("intensity"));
    }
}
