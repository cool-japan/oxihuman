// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Depth buffer sampling and linearisation utilities.

use std::f32::consts::FRAC_PI_4;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DepthSampleConfig {
    pub near: f32,
    pub far: f32,
    pub reversed_z: bool,
}

impl Default for DepthSampleConfig {
    fn default() -> Self {
        Self {
            near: 0.1,
            far: 1000.0,
            reversed_z: false,
        }
    }
}

#[allow(dead_code)]
pub fn default_depth_sample_config() -> DepthSampleConfig {
    DepthSampleConfig::default()
}

#[allow(dead_code)]
pub fn ds_linearize(cfg: &DepthSampleConfig, ndc_depth: f32) -> f32 {
    let z = ndc_depth.clamp(0.0, 1.0);
    let z = if cfg.reversed_z { 1.0 - z } else { z };
    let n = cfg.near;
    let f = cfg.far;
    (2.0 * n * f) / (f + n - z * (f - n))
}

#[allow(dead_code)]
pub fn ds_ndc_from_linear(cfg: &DepthSampleConfig, linear_depth: f32) -> f32 {
    let n = cfg.near;
    let f = cfg.far;
    let z = (2.0 * n * f / linear_depth - f - n) / (n - f);
    let z = z.clamp(0.0, 1.0);
    if cfg.reversed_z {
        1.0 - z
    } else {
        z
    }
}

#[allow(dead_code)]
pub fn ds_depth_range(cfg: &DepthSampleConfig) -> f32 {
    cfg.far - cfg.near
}

#[allow(dead_code)]
pub fn ds_is_valid(cfg: &DepthSampleConfig, ndc_depth: f32) -> bool {
    (0.0..=1.0).contains(&ndc_depth) && cfg.near > 0.0 && cfg.far > cfg.near
}

#[allow(dead_code)]
pub fn ds_perspective_angle_rad(cfg: &DepthSampleConfig) -> f32 {
    (cfg.near / cfg.far).atan().min(FRAC_PI_4)
}

#[allow(dead_code)]
pub fn ds_sample_buffer(cfg: &DepthSampleConfig, buf: &[f32]) -> Vec<f32> {
    buf.iter().map(|&d| ds_linearize(cfg, d)).collect()
}

#[allow(dead_code)]
pub fn ds_average_linear(cfg: &DepthSampleConfig, buf: &[f32]) -> f32 {
    if buf.is_empty() {
        return 0.0;
    }
    let sum: f32 = buf.iter().map(|&d| ds_linearize(cfg, d)).sum();
    sum / buf.len() as f32
}

#[allow(dead_code)]
pub fn ds_to_json(cfg: &DepthSampleConfig) -> String {
    format!(
        "{{\"near\":{:.4},\"far\":{:.4},\"rev\":{}}}",
        cfg.near, cfg.far, cfg.reversed_z
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_near_far() {
        let c = default_depth_sample_config();
        assert!(c.near > 0.0 && c.far > c.near);
    }
    #[test]
    fn linearize_zero_near() {
        let c = default_depth_sample_config();
        let l = ds_linearize(&c, 0.0);
        assert!(l > 0.0);
    }
    #[test]
    fn linearize_one_is_far() {
        let c = default_depth_sample_config();
        let l = ds_linearize(&c, 1.0);
        assert!((l - c.far).abs() / c.far < 0.01);
    }
    #[test]
    fn depth_range_correct() {
        let c = default_depth_sample_config();
        assert!((ds_depth_range(&c) - (c.far - c.near)).abs() < 1e-4);
    }
    #[test]
    fn is_valid_mid() {
        let c = default_depth_sample_config();
        assert!(ds_is_valid(&c, 0.5));
    }
    #[test]
    fn is_invalid_neg() {
        let c = default_depth_sample_config();
        assert!(!ds_is_valid(&c, -0.1));
    }
    #[test]
    fn perspective_angle_nonneg() {
        assert!(ds_perspective_angle_rad(&default_depth_sample_config()) >= 0.0);
    }
    #[test]
    fn sample_buffer_same_len() {
        let c = default_depth_sample_config();
        let b = vec![0.2, 0.5, 0.8];
        assert_eq!(ds_sample_buffer(&c, &b).len(), b.len());
    }
    #[test]
    fn average_linear_empty_zero() {
        assert!(ds_average_linear(&default_depth_sample_config(), &[]).abs() < 1e-6);
    }
    #[test]
    fn to_json_has_near() {
        assert!(ds_to_json(&default_depth_sample_config()).contains("\"near\""));
    }
}
