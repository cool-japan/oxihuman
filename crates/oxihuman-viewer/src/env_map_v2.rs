// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Environment map v2 — extended cubemap with IBL prefilter support.

use std::f32::consts::PI;

pub const MAX_MIP_LEVELS_V2: usize = 8;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvMapV2Source {
    Cubemap,
    Spherical,
    Panoramic,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EnvMapV2Config {
    pub intensity: f32,
    pub rotation_deg: f32,
    pub source: EnvMapV2Source,
    pub enabled: bool,
    pub mip_count: usize,
}

impl Default for EnvMapV2Config {
    fn default() -> Self {
        Self {
            intensity: 1.0,
            rotation_deg: 0.0,
            source: EnvMapV2Source::Cubemap,
            enabled: true,
            mip_count: 6,
        }
    }
}

#[allow(dead_code)]
pub fn default_env_map_v2_config() -> EnvMapV2Config {
    EnvMapV2Config::default()
}

#[allow(dead_code)]
pub fn emv2_set_intensity(cfg: &mut EnvMapV2Config, v: f32) {
    cfg.intensity = v.max(0.0);
}

#[allow(dead_code)]
pub fn emv2_set_rotation_deg(cfg: &mut EnvMapV2Config, deg: f32) {
    cfg.rotation_deg = deg % 360.0;
}

#[allow(dead_code)]
pub fn emv2_set_enabled(cfg: &mut EnvMapV2Config, v: bool) {
    cfg.enabled = v;
}

#[allow(dead_code)]
pub fn emv2_set_mip_count(cfg: &mut EnvMapV2Config, n: usize) {
    cfg.mip_count = n.clamp(1, MAX_MIP_LEVELS_V2);
}

#[allow(dead_code)]
pub fn emv2_rotation_rad(cfg: &EnvMapV2Config) -> f32 {
    cfg.rotation_deg.to_radians()
}

#[allow(dead_code)]
pub fn emv2_roughness_for_mip(cfg: &EnvMapV2Config, mip: usize) -> f32 {
    if cfg.mip_count <= 1 {
        return 0.0;
    }
    (mip as f32 / (cfg.mip_count - 1) as f32).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn emv2_effective_intensity(cfg: &EnvMapV2Config) -> f32 {
    if cfg.enabled {
        cfg.intensity
    } else {
        0.0
    }
}

#[allow(dead_code)]
pub fn emv2_source_name(cfg: &EnvMapV2Config) -> &'static str {
    match cfg.source {
        EnvMapV2Source::Cubemap => "cubemap",
        EnvMapV2Source::Spherical => "spherical",
        EnvMapV2Source::Panoramic => "panoramic",
    }
}

#[allow(dead_code)]
pub fn emv2_solid_angle(cfg: &EnvMapV2Config) -> f32 {
    4.0 * PI * cfg.intensity
}

#[allow(dead_code)]
pub fn emv2_to_json(cfg: &EnvMapV2Config) -> String {
    format!(
        "{{\"intensity\":{:.4},\"enabled\":{},\"mips\":{}}}",
        cfg.intensity, cfg.enabled, cfg.mip_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_enabled() {
        assert!(default_env_map_v2_config().enabled);
    }
    #[test]
    fn set_intensity_negative_clamps_to_zero() {
        let mut c = default_env_map_v2_config();
        emv2_set_intensity(&mut c, -1.0);
        assert!(c.intensity.abs() < 1e-6);
    }
    #[test]
    fn set_enabled_false() {
        let mut c = default_env_map_v2_config();
        emv2_set_enabled(&mut c, false);
        assert!(!c.enabled);
    }
    #[test]
    fn effective_intensity_zero_when_disabled() {
        let mut c = default_env_map_v2_config();
        emv2_set_enabled(&mut c, false);
        assert!(emv2_effective_intensity(&c).abs() < 1e-6);
    }
    #[test]
    fn rotation_rad_matches() {
        let mut c = default_env_map_v2_config();
        emv2_set_rotation_deg(&mut c, 180.0);
        assert!((emv2_rotation_rad(&c) - PI).abs() < 1e-4);
    }
    #[test]
    fn roughness_first_mip_zero() {
        let c = default_env_map_v2_config();
        assert!(emv2_roughness_for_mip(&c, 0).abs() < 1e-6);
    }
    #[test]
    fn roughness_last_mip_one() {
        let c = default_env_map_v2_config();
        assert!((emv2_roughness_for_mip(&c, c.mip_count - 1) - 1.0).abs() < 1e-5);
    }
    #[test]
    fn mip_count_clamped() {
        let mut c = default_env_map_v2_config();
        emv2_set_mip_count(&mut c, 0);
        assert_eq!(c.mip_count, 1);
    }
    #[test]
    fn source_name_cubemap() {
        let c = default_env_map_v2_config();
        assert_eq!(emv2_source_name(&c), "cubemap");
    }
    #[test]
    fn to_json_has_intensity() {
        assert!(emv2_to_json(&default_env_map_v2_config()).contains("\"intensity\""));
    }
}
