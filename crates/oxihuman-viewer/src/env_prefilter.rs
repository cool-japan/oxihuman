// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Environment prefilter — prefiltered environment maps for PBR specular IBL.

use std::f32::consts::PI;

/// Number of mip levels to generate.
pub const MAX_MIP_LEVELS: usize = 8;

/// Prefilter configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EnvPrefilterConfig {
    pub sample_count: u32,
    pub base_resolution: u32,
    pub mip_levels: usize,
}

/// A prefiltered specular environment map descriptor.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PrefilterMap {
    pub name: String,
    pub resolution: u32,
    pub mip_levels: usize,
    pub is_ready: bool,
}

#[allow(dead_code)]
pub fn default_env_prefilter_config() -> EnvPrefilterConfig {
    EnvPrefilterConfig {
        sample_count: 1024,
        base_resolution: 256,
        mip_levels: 5,
    }
}

#[allow(dead_code)]
pub fn new_prefilter_map(name: &str, resolution: u32, mip_levels: usize) -> PrefilterMap {
    PrefilterMap {
        name: name.to_string(),
        resolution,
        mip_levels: mip_levels.min(MAX_MIP_LEVELS),
        is_ready: false,
    }
}

#[allow(dead_code)]
pub fn ep_mark_ready(map: &mut PrefilterMap) {
    map.is_ready = true;
}

#[allow(dead_code)]
pub fn ep_resolution_at_mip(map: &PrefilterMap, mip: usize) -> u32 {
    (map.resolution >> mip).max(1)
}

#[allow(dead_code)]
pub fn ep_roughness_for_mip(map: &PrefilterMap, mip: usize) -> f32 {
    if map.mip_levels <= 1 {
        return 0.0;
    }
    mip as f32 / (map.mip_levels - 1) as f32
}

/// GGX importance-sample direction (for reference/testing, deterministic).
#[allow(dead_code)]
pub fn ep_ggx_ndf(n_dot_h: f32, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let d = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
    a2 / (PI * d * d).max(1e-7)
}

#[allow(dead_code)]
pub fn ep_memory_bytes(map: &PrefilterMap) -> u64 {
    let mut total = 0u64;
    #[allow(clippy::needless_range_loop)]
    for mip in 0..map.mip_levels {
        let res = ep_resolution_at_mip(map, mip) as u64;
        total += res * res * 6 * 8; // 6 faces, 8 bytes (RGBA16F)
    }
    total
}

#[allow(dead_code)]
pub fn ep_set_sample_count(cfg: &mut EnvPrefilterConfig, count: u32) {
    cfg.sample_count = count.clamp(16, 4096);
}

#[allow(dead_code)]
pub fn ep_to_json(map: &PrefilterMap) -> String {
    format!(
        r#"{{"name":"{}","resolution":{},"mip_levels":{},"ready":{}}}"#,
        map.name, map.resolution, map.mip_levels, map.is_ready
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_sane() {
        let cfg = default_env_prefilter_config();
        assert!(cfg.sample_count > 0);
        assert!(cfg.mip_levels > 0);
    }

    #[test]
    fn new_map_not_ready() {
        let m = new_prefilter_map("sky", 256, 5);
        assert!(!m.is_ready);
    }

    #[test]
    fn mark_ready() {
        let mut m = new_prefilter_map("sky", 256, 5);
        ep_mark_ready(&mut m);
        assert!(m.is_ready);
    }

    #[test]
    fn resolution_halves_per_mip() {
        let m = new_prefilter_map("sky", 256, 5);
        assert_eq!(ep_resolution_at_mip(&m, 0), 256);
        assert_eq!(ep_resolution_at_mip(&m, 1), 128);
        assert_eq!(ep_resolution_at_mip(&m, 2), 64);
    }

    #[test]
    fn resolution_minimum_one() {
        let m = new_prefilter_map("sky", 1, 5);
        assert_eq!(ep_resolution_at_mip(&m, 10), 1);
    }

    #[test]
    fn roughness_at_mip_zero_is_zero() {
        let m = new_prefilter_map("sky", 256, 5);
        assert!((ep_roughness_for_mip(&m, 0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn roughness_at_last_mip_is_one() {
        let m = new_prefilter_map("sky", 256, 5);
        assert!((ep_roughness_for_mip(&m, 4) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ggx_ndf_positive() {
        let v = ep_ggx_ndf(1.0, 0.5);
        assert!(v > 0.0);
    }

    #[test]
    fn memory_bytes_positive() {
        let m = new_prefilter_map("sky", 64, 3);
        assert!(ep_memory_bytes(&m) > 0);
    }

    #[test]
    fn to_json_fields() {
        let m = new_prefilter_map("test", 128, 4);
        let j = ep_to_json(&m);
        assert!(j.contains("resolution"));
        assert!(j.contains("mip_levels"));
    }
}
