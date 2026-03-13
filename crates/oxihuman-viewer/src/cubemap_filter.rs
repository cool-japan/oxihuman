// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cubemap filter — GGX specular and diffuse irradiance prefiltering utilities.

use std::f32::consts::{FRAC_1_PI, PI};

/// Cubemap face index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum CubeFaceFilter {
    PosX = 0,
    NegX = 1,
    PosY = 2,
    NegY = 3,
    PosZ = 4,
    NegZ = 5,
}

/// Prefilter configuration.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct CubemapFilterConfig {
    pub resolution: u32,
    pub mip_levels: u32,
    pub sample_count: u32,
    pub enabled: bool,
}

impl Default for CubemapFilterConfig {
    fn default() -> Self {
        Self {
            resolution: 256,
            mip_levels: 8,
            sample_count: 1024,
            enabled: true,
        }
    }
}

/// Mip prefilter entry.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct MipEntry {
    pub mip: u32,
    pub roughness: f32,
    pub sample_count: u32,
}

/// Cubemap filter state.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct CubemapFilter {
    pub config: CubemapFilterConfig,
    pub mips: Vec<MipEntry>,
}

/// Create new cubemap filter.
#[allow(dead_code)]
pub fn new_cubemap_filter(cfg: CubemapFilterConfig) -> CubemapFilter {
    CubemapFilter {
        config: cfg,
        mips: Vec::new(),
    }
}

/// Build mip chain with roughness mapping.
#[allow(dead_code)]
pub fn build_mip_chain(f: &mut CubemapFilter) {
    f.mips.clear();
    let levels = f.config.mip_levels;
    #[allow(clippy::needless_range_loop)]
    for i in 0..levels as usize {
        let roughness = if levels <= 1 {
            0.0
        } else {
            i as f32 / (levels - 1) as f32
        };
        f.mips.push(MipEntry {
            mip: i as u32,
            roughness,
            sample_count: f.config.sample_count,
        });
    }
}

/// Get mip count.
#[allow(dead_code)]
pub fn mip_count(f: &CubemapFilter) -> usize {
    f.mips.len()
}

/// GGX distribution for prefiltering.
#[allow(dead_code)]
pub fn ggx_d(n_dot_h: f32, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let denom = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
    a2 * FRAC_1_PI / (denom * denom + 1e-7)
}

/// Roughness to mip level mapping.
#[allow(dead_code)]
pub fn roughness_to_mip(roughness: f32, max_mip: u32) -> f32 {
    roughness.sqrt() * max_mip as f32
}

/// Mip level to roughness.
#[allow(dead_code)]
pub fn mip_to_roughness(mip: f32, max_mip: u32) -> f32 {
    let t = mip / max_mip as f32;
    t * t
}

/// Integrate diffuse irradiance approximation (uses PI).
#[allow(dead_code)]
pub fn diffuse_irradiance_approx(normal: [f32; 3], env_color: [f32; 3]) -> [f32; 3] {
    let scale = FRAC_1_PI * (normal[1] * 0.5 + 0.5);
    [
        env_color[0] * scale,
        env_color[1] * scale,
        env_color[2] * scale,
    ]
}

/// Memory estimate for prefiltered cubemap.
#[allow(dead_code)]
pub fn estimated_filter_memory(cfg: &CubemapFilterConfig) -> u64 {
    let mut total = 0u64;
    let mut res = cfg.resolution;
    #[allow(clippy::needless_range_loop)]
    for _ in 0..cfg.mip_levels as usize {
        total += (res as u64) * (res as u64) * 6 * 8; // 6 faces, 8 bytes/pixel (RGBA16F)
        res = (res / 2).max(1);
    }
    total
}

/// Export config to JSON-like string.
#[allow(dead_code)]
pub fn cubemap_filter_to_json(f: &CubemapFilter) -> String {
    format!(
        r#"{{"resolution":{},"mip_levels":{},"mips_built":{}}}"#,
        f.config.resolution,
        f.config.mip_levels,
        f.mips.len()
    )
}

/// Circular area element using PI.
#[allow(dead_code)]
pub fn solid_angle_estimate(roughness: f32) -> f32 {
    4.0 * PI * roughness * roughness
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_filter_empty_mips() {
        let f = new_cubemap_filter(CubemapFilterConfig::default());
        assert_eq!(mip_count(&f), 0);
    }

    #[test]
    fn build_mip_chain_correct_count() {
        let mut f = new_cubemap_filter(CubemapFilterConfig {
            mip_levels: 5,
            ..Default::default()
        });
        build_mip_chain(&mut f);
        assert_eq!(mip_count(&f), 5);
    }

    #[test]
    fn roughness_mip0_is_zero() {
        let mut f = new_cubemap_filter(CubemapFilterConfig::default());
        build_mip_chain(&mut f);
        if !f.mips.is_empty() {
            assert!(f.mips[0].roughness.abs() < 1e-6);
        }
    }

    #[test]
    fn roughness_last_mip_is_one() {
        let mut f = new_cubemap_filter(CubemapFilterConfig {
            mip_levels: 4,
            ..Default::default()
        });
        build_mip_chain(&mut f);
        let last = f.mips.last().expect("should succeed");
        assert!((last.roughness - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ggx_d_positive() {
        let d = ggx_d(0.9, 0.3);
        assert!(d > 0.0);
    }

    #[test]
    fn roughness_to_mip_range() {
        let m = roughness_to_mip(0.25, 8);
        assert!((0.0..=8.0).contains(&m));
    }

    #[test]
    fn mip_to_roughness_roundtrip() {
        let r0 = 0.5f32;
        let mip = roughness_to_mip(r0, 8);
        let r1 = mip_to_roughness(mip, 8);
        // roughness_to_mip uses sqrt, mip_to_roughness squares back → r0
        assert!((r0 - r1).abs() < 0.01);
    }

    #[test]
    fn memory_estimate_nonzero() {
        assert!(estimated_filter_memory(&CubemapFilterConfig::default()) > 0);
    }

    #[test]
    fn solid_angle_uses_pi() {
        let sa = solid_angle_estimate(1.0);
        assert!((sa - 4.0 * PI).abs() < 1e-4);
    }

    #[test]
    fn json_contains_resolution() {
        let f = new_cubemap_filter(CubemapFilterConfig::default());
        assert!(cubemap_filter_to_json(&f).contains("resolution"));
    }
}
