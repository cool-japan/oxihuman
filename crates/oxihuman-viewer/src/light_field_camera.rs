// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub struct LightFieldConfig {
    pub main_lens_f: f32,
    pub microlens_array_size: u32,
    pub views_per_lens: u32,
    pub resolution: u32,
}

pub fn new_light_field_config(resolution: u32) -> LightFieldConfig {
    LightFieldConfig {
        main_lens_f: 50.0,
        microlens_array_size: 100,
        views_per_lens: 9,
        resolution,
    }
}

pub fn lf_total_views(cfg: &LightFieldConfig) -> u64 {
    (cfg.microlens_array_size as u64)
        * (cfg.microlens_array_size as u64)
        * (cfg.views_per_lens as u64)
}

pub fn lf_angular_resolution(cfg: &LightFieldConfig) -> f32 {
    cfg.views_per_lens as f32
}

pub fn lf_spatial_resolution(cfg: &LightFieldConfig) -> f32 {
    cfg.resolution as f32 / cfg.microlens_array_size as f32
}

pub fn lf_memory_mb(cfg: &LightFieldConfig) -> f32 {
    let pixels = lf_total_views(cfg) * (cfg.resolution as u64) * (cfg.resolution as u64);
    (pixels * 3) as f32 / (1024.0 * 1024.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_config() {
        /* resolution is set */
        let cfg = new_light_field_config(1024);
        assert_eq!(cfg.resolution, 1024);
    }

    #[test]
    fn test_total_views() {
        /* total views = array^2 * views_per_lens */
        let cfg = new_light_field_config(128);
        let expected = 100u64 * 100 * 9;
        assert_eq!(lf_total_views(&cfg), expected);
    }

    #[test]
    fn test_angular_resolution() {
        /* angular resolution = views_per_lens */
        let cfg = new_light_field_config(128);
        assert!((lf_angular_resolution(&cfg) - 9.0).abs() < 1e-6);
    }

    #[test]
    fn test_spatial_resolution() {
        /* spatial_res = resolution / microlens_size */
        let cfg = new_light_field_config(1000);
        assert!((lf_spatial_resolution(&cfg) - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_memory_mb_positive() {
        /* memory > 0 */
        let cfg = new_light_field_config(64);
        assert!(lf_memory_mb(&cfg) > 0.0);
    }
}
