// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cubemap mip-chain generation helpers.

use std::f32::consts::PI;

/// A single mip level descriptor.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct CubemapMipLevel {
    pub level: u32,
    pub size: u32,
    pub roughness: f32,
}

/// Mip chain.
#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct CubemapMipChain {
    pub levels: Vec<CubemapMipLevel>,
    pub base_size: u32,
}

#[allow(dead_code)]
pub fn new_mip_chain(base_size: u32) -> CubemapMipChain {
    CubemapMipChain {
        levels: Vec::new(),
        base_size,
    }
}

/// Build a mip chain from a base size, mapping roughness linearly per level.
#[allow(dead_code)]
pub fn cm_build_mip_chain(base_size: u32) -> CubemapMipChain {
    let mut chain = new_mip_chain(base_size);
    let mut size = base_size;
    let mut level = 0u32;
    while size >= 1 {
        let roughness = if level == 0 {
            0.0
        } else {
            level as f32 / cm_max_levels(base_size) as f32
        };
        chain.levels.push(CubemapMipLevel {
            level,
            size,
            roughness,
        });
        size /= 2;
        level += 1;
    }
    chain
}

#[allow(dead_code)]
pub fn cm_max_levels(base_size: u32) -> u32 {
    if base_size == 0 {
        return 0;
    }
    (base_size as f32).log2().floor() as u32 + 1
}

#[allow(dead_code)]
pub fn cm_level_count(chain: &CubemapMipChain) -> usize {
    chain.levels.len()
}

#[allow(dead_code)]
pub fn cm_roughness_to_level(chain: &CubemapMipChain, roughness: f32) -> u32 {
    let r = roughness.clamp(0.0, 1.0);
    let max = chain.levels.len().saturating_sub(1) as f32;
    (r * max).round() as u32
}

#[allow(dead_code)]
pub fn cm_level_to_roughness(chain: &CubemapMipChain, level: u32) -> f32 {
    let max = chain.levels.len().saturating_sub(1);
    if max == 0 {
        return 0.0;
    }
    (level as f32 / max as f32).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn cm_texel_solid_angle(size: u32) -> f32 {
    if size == 0 {
        return 0.0;
    }
    4.0 * PI / (6.0 * (size * size) as f32)
}

#[allow(dead_code)]
pub fn cm_memory_bytes(base_size: u32, bytes_per_texel: u32) -> u64 {
    let mut total = 0u64;
    let mut s = base_size;
    while s >= 1 {
        total += 6 * s as u64 * s as u64 * bytes_per_texel as u64;
        s /= 2;
    }
    total
}

#[allow(dead_code)]
pub fn cm_chain_to_json(chain: &CubemapMipChain) -> String {
    let lvls: Vec<String> = chain
        .levels
        .iter()
        .map(|l| {
            format!(
                "{{\"level\":{},\"size\":{},\"roughness\":{:.4}}}",
                l.level, l.size, l.roughness
            )
        })
        .collect();
    format!(
        "{{\"base_size\":{},\"levels\":[{}]}}",
        chain.base_size,
        lvls.join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mip_chain_256() {
        let c = cm_build_mip_chain(256);
        assert_eq!(cm_level_count(&c), 9); // 256,128,64,32,16,8,4,2,1
    }

    #[test]
    fn base_is_zero_roughness() {
        let c = cm_build_mip_chain(128);
        assert!(!c.levels.is_empty());
        assert!((c.levels[0].roughness).abs() < 1e-5);
    }

    #[test]
    fn last_level_roughness_one() {
        let c = cm_build_mip_chain(4);
        let last = c.levels.last().expect("should succeed");
        assert!((last.roughness - 1.0).abs() < 0.5);
    }

    #[test]
    fn roughness_to_level_zero() {
        let c = cm_build_mip_chain(64);
        assert_eq!(cm_roughness_to_level(&c, 0.0), 0);
    }

    #[test]
    fn level_to_roughness_zero() {
        let c = cm_build_mip_chain(64);
        assert!((cm_level_to_roughness(&c, 0)).abs() < 1e-5);
    }

    #[test]
    fn solid_angle_positive() {
        assert!(cm_texel_solid_angle(128) > 0.0);
    }

    #[test]
    fn solid_angle_zero_for_size_zero() {
        assert!((cm_texel_solid_angle(0)).abs() < 1e-5);
    }

    #[test]
    fn memory_bytes_positive() {
        assert!(cm_memory_bytes(128, 8) > 0);
    }

    #[test]
    fn json_has_base_size() {
        let c = cm_build_mip_chain(32);
        assert!(cm_chain_to_json(&c).contains("base_size"));
    }

    #[test]
    fn max_levels_power_of_two() {
        assert_eq!(cm_max_levels(1), 1);
        assert_eq!(cm_max_levels(4), 3);
    }
}
