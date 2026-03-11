// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Fog rendering utilities for the 3D viewer.

use std::f32::consts::E;

/// Fog type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FogType {
    Linear,
    Exponential,
    ExponentialSquared,
}

/// Fog configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FogConfig {
    pub fog_type: FogType,
    pub color: [f32; 3],
    pub near: f32,
    pub far: f32,
    pub density: f32,
    pub enabled: bool,
}

/// Default fog config.
#[allow(dead_code)]
pub fn default_fog_config() -> FogConfig {
    FogConfig {
        fog_type: FogType::Linear,
        color: [0.7, 0.7, 0.8],
        near: 10.0,
        far: 100.0,
        density: 0.02,
        enabled: false,
    }
}

/// Compute fog factor for a given depth.
#[allow(dead_code)]
pub fn compute_fog_factor(depth: f32, config: &FogConfig) -> f32 {
    if !config.enabled {
        return 0.0;
    }
    let factor = match config.fog_type {
        FogType::Linear => {
            if (config.far - config.near).abs() < 1e-6 {
                1.0
            } else {
                ((config.far - depth) / (config.far - config.near)).clamp(0.0, 1.0)
            }
        }
        FogType::Exponential => {
            E.powf(-config.density * depth).clamp(0.0, 1.0)
        }
        FogType::ExponentialSquared => {
            let dd = config.density * depth;
            E.powf(-dd * dd).clamp(0.0, 1.0)
        }
    };
    1.0 - factor
}

/// Apply fog to a color.
#[allow(dead_code)]
pub fn apply_fog(color: [f32; 3], depth: f32, config: &FogConfig) -> [f32; 3] {
    let f = compute_fog_factor(depth, config);
    [
        color[0] * (1.0 - f) + config.color[0] * f,
        color[1] * (1.0 - f) + config.color[1] * f,
        color[2] * (1.0 - f) + config.color[2] * f,
    ]
}

/// Enable fog.
#[allow(dead_code)]
pub fn enable_fog(config: &mut FogConfig) {
    config.enabled = true;
}

/// Disable fog.
#[allow(dead_code)]
pub fn disable_fog(config: &mut FogConfig) {
    config.enabled = false;
}

/// Set fog density.
#[allow(dead_code)]
pub fn set_fog_density(config: &mut FogConfig, density: f32) {
    config.density = density.clamp(0.001, 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_fog_config();
        assert!(!c.enabled);
    }

    #[test]
    fn test_disabled_fog() {
        let c = default_fog_config();
        assert!(compute_fog_factor(50.0, &c).abs() < 1e-6);
    }

    #[test]
    fn test_linear_fog_near() {
        let mut c = default_fog_config();
        enable_fog(&mut c);
        let f = compute_fog_factor(10.0, &c);
        assert!(f.abs() < 1e-4);
    }

    #[test]
    fn test_linear_fog_far() {
        let mut c = default_fog_config();
        enable_fog(&mut c);
        let f = compute_fog_factor(100.0, &c);
        assert!((f - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_exponential_fog() {
        let mut c = default_fog_config();
        c.fog_type = FogType::Exponential;
        enable_fog(&mut c);
        let f = compute_fog_factor(50.0, &c);
        assert!(f > 0.0 && f < 1.0);
    }

    #[test]
    fn test_exp_squared_fog() {
        let mut c = default_fog_config();
        c.fog_type = FogType::ExponentialSquared;
        enable_fog(&mut c);
        let f = compute_fog_factor(50.0, &c);
        assert!(f > 0.0 && f < 1.0);
    }

    #[test]
    fn test_apply_fog() {
        let mut c = default_fog_config();
        enable_fog(&mut c);
        let result = apply_fog([1.0, 0.0, 0.0], 100.0, &c);
        assert!((result[0] - c.color[0]).abs() < 1e-4);
    }

    #[test]
    fn test_enable_disable() {
        let mut c = default_fog_config();
        enable_fog(&mut c);
        assert!(c.enabled);
        disable_fog(&mut c);
        assert!(!c.enabled);
    }

    #[test]
    fn test_set_density() {
        let mut c = default_fog_config();
        set_fog_density(&mut c, 0.5);
        assert!((c.density - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_no_fog_preserves_color() {
        let c = default_fog_config();
        let result = apply_fog([1.0, 0.0, 0.0], 50.0, &c);
        assert!((result[0] - 1.0).abs() < 1e-6);
    }
}
