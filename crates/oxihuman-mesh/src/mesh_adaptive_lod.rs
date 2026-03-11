// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Adaptive LOD selection based on screen-space projected size.

/// A single LOD level descriptor.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LodDescriptor {
    /// Number of triangles at this level.
    pub triangle_count: u32,
    /// Minimum screen-fraction to use this LOD (0.0–1.0).
    pub screen_fraction_min: f32,
}

/// Configuration for adaptive LOD selection.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AdaptiveLodConfig {
    /// Bounding-sphere radius of the object.
    pub bounding_radius: f32,
    /// Available LOD levels (from highest to lowest quality).
    pub levels: Vec<LodDescriptor>,
}

/// Select the appropriate LOD level for a given distance and FOV.
///
/// Returns the level index (0 = highest quality).
#[allow(dead_code)]
pub fn select_lod_level(config: &AdaptiveLodConfig, distance: f32, fov_radians: f32) -> usize {
    if distance <= 0.0 || config.levels.is_empty() {
        return 0;
    }
    let screen_fraction = config.bounding_radius / (distance * (fov_radians * 0.5).tan());
    let screen_fraction = screen_fraction.clamp(0.0, 1.0);
    for (i, level) in config.levels.iter().enumerate() {
        if screen_fraction >= level.screen_fraction_min {
            return i;
        }
    }
    config.levels.len() - 1
}

/// Compute the number of levels in the config.
#[allow(dead_code)]
pub fn lod_level_count(config: &AdaptiveLodConfig) -> usize {
    config.levels.len()
}

/// Return the triangle count at a given level index.
#[allow(dead_code)]
pub fn triangle_count_at(config: &AdaptiveLodConfig, level: usize) -> u32 {
    config
        .levels
        .get(level)
        .map(|l| l.triangle_count)
        .unwrap_or(0)
}

/// Compute the reduction ratio from level 0 to `level`.
#[allow(dead_code)]
pub fn lod_reduction_ratio(config: &AdaptiveLodConfig, level: usize) -> f32 {
    if config.levels.is_empty() {
        return 0.0;
    }
    let base = config.levels[0].triangle_count;
    if base == 0 {
        return 0.0;
    }
    let current = triangle_count_at(config, level);
    1.0 - current as f32 / base as f32
}

/// Build a default LOD config with 4 levels.
#[allow(dead_code)]
pub fn default_adaptive_lod_config(bounding_radius: f32) -> AdaptiveLodConfig {
    AdaptiveLodConfig {
        bounding_radius,
        levels: vec![
            LodDescriptor {
                triangle_count: 10_000,
                screen_fraction_min: 0.3,
            },
            LodDescriptor {
                triangle_count: 4_000,
                screen_fraction_min: 0.1,
            },
            LodDescriptor {
                triangle_count: 1_000,
                screen_fraction_min: 0.03,
            },
            LodDescriptor {
                triangle_count: 200,
                screen_fraction_min: 0.0,
            },
        ],
    }
}

/// Serialize config to JSON.
#[allow(dead_code)]
pub fn adaptive_lod_to_json(config: &AdaptiveLodConfig) -> String {
    format!(
        "{{\"levels\":{},\"bounding_radius\":{:.4}}}",
        config.levels.len(),
        config.bounding_radius
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_select_lod_close() {
        let cfg = default_adaptive_lod_config(1.0);
        let level = select_lod_level(&cfg, 1.0, PI * 0.5);
        assert_eq!(level, 0);
    }

    #[test]
    fn test_select_lod_far() {
        let cfg = default_adaptive_lod_config(1.0);
        let level = select_lod_level(&cfg, 1000.0, PI * 0.5);
        assert!(level > 0);
    }

    #[test]
    fn test_lod_level_count() {
        let cfg = default_adaptive_lod_config(1.0);
        assert_eq!(lod_level_count(&cfg), 4);
    }

    #[test]
    fn test_triangle_count_at_zero() {
        let cfg = default_adaptive_lod_config(1.0);
        assert_eq!(triangle_count_at(&cfg, 0), 10_000);
    }

    #[test]
    fn test_triangle_count_at_oob() {
        let cfg = default_adaptive_lod_config(1.0);
        assert_eq!(triangle_count_at(&cfg, 100), 0);
    }

    #[test]
    fn test_lod_reduction_ratio_level0() {
        let cfg = default_adaptive_lod_config(1.0);
        assert!((lod_reduction_ratio(&cfg, 0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_lod_reduction_ratio_last() {
        let cfg = default_adaptive_lod_config(1.0);
        let last = cfg.levels.len() - 1;
        assert!(lod_reduction_ratio(&cfg, last) > 0.9);
    }

    #[test]
    fn test_select_lod_zero_distance() {
        let cfg = default_adaptive_lod_config(1.0);
        let level = select_lod_level(&cfg, 0.0, PI * 0.5);
        assert_eq!(level, 0);
    }

    #[test]
    fn test_adaptive_lod_to_json() {
        let cfg = default_adaptive_lod_config(1.0);
        let j = adaptive_lod_to_json(&cfg);
        assert!(j.contains("levels"));
    }

    #[test]
    fn test_empty_levels() {
        let cfg = AdaptiveLodConfig {
            bounding_radius: 1.0,
            levels: vec![],
        };
        assert_eq!(select_lod_level(&cfg, 10.0, PI * 0.5), 0);
    }
}
