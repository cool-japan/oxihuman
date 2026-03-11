// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub struct NerfConfig {
    pub resolution: u32,
    pub num_samples: u32,
    pub near: f32,
    pub far: f32,
    pub network_width: usize,
}

pub fn new_nerf_config(resolution: u32) -> NerfConfig {
    NerfConfig {
        resolution,
        num_samples: 128,
        near: 0.1,
        far: 100.0,
        network_width: 256,
    }
}

pub fn nerf_ray_count(cfg: &NerfConfig) -> u64 {
    (cfg.resolution as u64) * (cfg.resolution as u64)
}

pub fn nerf_sample_count_per_frame(cfg: &NerfConfig) -> u64 {
    nerf_ray_count(cfg) * (cfg.num_samples as u64)
}

pub fn nerf_memory_mb(cfg: &NerfConfig) -> f32 {
    let weights = cfg.network_width * cfg.network_width * 8;
    (weights * 4) as f32 / (1024.0 * 1024.0)
}

pub fn nerf_is_valid(cfg: &NerfConfig) -> bool {
    cfg.resolution > 0 && cfg.num_samples > 0 && cfg.near < cfg.far
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_config() {
        /* resolution is set */
        let cfg = new_nerf_config(256);
        assert_eq!(cfg.resolution, 256);
    }

    #[test]
    fn test_is_valid() {
        /* default config is valid */
        let cfg = new_nerf_config(128);
        assert!(nerf_is_valid(&cfg));
    }

    #[test]
    fn test_ray_count() {
        /* ray count = res^2 */
        let cfg = new_nerf_config(10);
        assert_eq!(nerf_ray_count(&cfg), 100);
    }

    #[test]
    fn test_sample_count() {
        /* sample count = ray_count * num_samples */
        let cfg = new_nerf_config(10);
        let expected = 100 * cfg.num_samples as u64;
        assert_eq!(nerf_sample_count_per_frame(&cfg), expected);
    }

    #[test]
    fn test_memory_mb_positive() {
        /* memory is positive */
        let cfg = new_nerf_config(64);
        assert!(nerf_memory_mb(&cfg) > 0.0);
    }
}
