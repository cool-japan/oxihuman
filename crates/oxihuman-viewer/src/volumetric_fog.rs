// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Volumetric fog rendering with distance-based density and height falloff.

use std::f32::consts::E;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum FogFalloff {
    Linear,
    Exponential,
    ExponentialSquared,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VolumetricFogConfig {
    pub color: [f32; 3],
    pub density: f32,
    pub start_distance: f32,
    pub end_distance: f32,
    pub height_falloff: f32,
    pub falloff: FogFalloff,
    pub enabled: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FogResult {
    pub fog_factor: f32,
    pub fog_color: [f32; 3],
}

#[allow(dead_code)]
pub fn default_volumetric_fog_config() -> VolumetricFogConfig {
    VolumetricFogConfig {
        color: [0.7, 0.75, 0.8],
        density: 0.02,
        start_distance: 10.0,
        end_distance: 200.0,
        height_falloff: 0.1,
        falloff: FogFalloff::Exponential,
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn linear_fog_factor(distance: f32, start: f32, end: f32) -> f32 {
    if end <= start {
        return 0.0;
    }
    ((distance - start) / (end - start)).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn exponential_fog_factor(distance: f32, density: f32) -> f32 {
    let exponent = -density * distance;
    1.0 - E.powf(exponent)
}

#[allow(dead_code)]
pub fn exponential_squared_fog_factor(distance: f32, density: f32) -> f32 {
    let d = density * distance;
    1.0 - E.powf(-d * d)
}

#[allow(dead_code)]
pub fn height_fog_modifier(height: f32, falloff: f32) -> f32 {
    E.powf(-falloff * height.max(0.0))
}

#[allow(dead_code)]
pub fn compute_fog(cfg: &VolumetricFogConfig, distance: f32, height: f32) -> FogResult {
    if !cfg.enabled {
        return FogResult {
            fog_factor: 0.0,
            fog_color: cfg.color,
        };
    }
    let base = match cfg.falloff {
        FogFalloff::Linear => linear_fog_factor(distance, cfg.start_distance, cfg.end_distance),
        FogFalloff::Exponential => exponential_fog_factor(distance, cfg.density),
        FogFalloff::ExponentialSquared => exponential_squared_fog_factor(distance, cfg.density),
    };
    let h_mod = height_fog_modifier(height, cfg.height_falloff);
    let fog_factor = (base * h_mod).clamp(0.0, 1.0);
    FogResult {
        fog_factor,
        fog_color: cfg.color,
    }
}

#[allow(dead_code)]
pub fn apply_fog_to_color(scene_color: [f32; 3], fog: &FogResult) -> [f32; 3] {
    let f = fog.fog_factor;
    let inv = 1.0 - f;
    [
        scene_color[0] * inv + fog.fog_color[0] * f,
        scene_color[1] * inv + fog.fog_color[1] * f,
        scene_color[2] * inv + fog.fog_color[2] * f,
    ]
}

#[allow(dead_code)]
pub fn fog_to_json(cfg: &VolumetricFogConfig) -> String {
    let f = match &cfg.falloff {
        FogFalloff::Linear => "linear",
        FogFalloff::Exponential => "exponential",
        FogFalloff::ExponentialSquared => "exp_squared",
    };
    format!(
        r#"{{"density":{},"start":{},"end":{},"falloff":"{}","enabled":{}}}"#,
        cfg.density, cfg.start_distance, cfg.end_distance, f, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_volumetric_fog_config();
        assert!(c.enabled);
        assert_eq!(c.falloff, FogFalloff::Exponential);
    }

    #[test]
    fn test_linear_fog_start() {
        let f = linear_fog_factor(10.0, 10.0, 100.0);
        assert!(f.abs() < 1e-6);
    }

    #[test]
    fn test_linear_fog_end() {
        let f = linear_fog_factor(100.0, 10.0, 100.0);
        assert!((f - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_exp_fog_zero() {
        let f = exponential_fog_factor(0.0, 0.1);
        assert!(f.abs() < 1e-4);
    }

    #[test]
    fn test_exp_fog_far() {
        let f = exponential_fog_factor(1000.0, 0.1);
        assert!((f - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_exp_sq_fog() {
        let f = exponential_squared_fog_factor(50.0, 0.02);
        assert!((0.0..=1.0).contains(&f));
    }

    #[test]
    fn test_height_modifier() {
        let m = height_fog_modifier(0.0, 0.1);
        assert!((m - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_compute_fog_disabled() {
        let mut cfg = default_volumetric_fog_config();
        cfg.enabled = false;
        let r = compute_fog(&cfg, 50.0, 0.0);
        assert!(r.fog_factor.abs() < 1e-6);
    }

    #[test]
    fn test_apply_fog() {
        let fog = FogResult {
            fog_factor: 0.5,
            fog_color: [1.0, 1.0, 1.0],
        };
        let result = apply_fog_to_color([0.0, 0.0, 0.0], &fog);
        assert!((result[0] - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_to_json() {
        let c = default_volumetric_fog_config();
        let j = fog_to_json(&c);
        assert!(j.contains("exponential"));
    }
}
