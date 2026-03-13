// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Distance fog — linear, exponential, and exponential-squared fog models
//! for depth-based atmospheric scattering.

use std::f32::consts::E;

/// Fog mode.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FogMode {
    Linear,
    Exponential,
    ExponentialSquared,
}

/// Fog configuration.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct FogConfig {
    pub mode: FogMode,
    /// Fog colour [r, g, b] in linear space.
    pub color: [f32; 3],
    /// Start distance (linear fog).
    pub start: f32,
    /// End distance (linear fog).
    pub end: f32,
    /// Density (exponential modes).
    pub density: f32,
    /// Height-based fog: fog is denser below this Y value.
    pub height_falloff: f32,
    /// Base height for height fog.
    pub base_height: f32,
}

impl Default for FogConfig {
    fn default() -> Self {
        Self {
            mode: FogMode::Linear,
            color: [0.7, 0.75, 0.8],
            start: 10.0,
            end: 100.0,
            density: 0.02,
            height_falloff: 0.0,
            base_height: 0.0,
        }
    }
}

/// Compute fog factor (0 = no fog, 1 = fully fogged).
#[allow(dead_code)]
pub fn fog_factor(distance: f32, config: &FogConfig) -> f32 {
    let f = match config.mode {
        FogMode::Linear => {
            if (config.end - config.start).abs() < 1e-6 {
                if distance >= config.end { 1.0 } else { 0.0 }
            } else {
                ((distance - config.start) / (config.end - config.start)).clamp(0.0, 1.0)
            }
        }
        FogMode::Exponential => {
            1.0 - (-config.density * distance).exp()
        }
        FogMode::ExponentialSquared => {
            let d = config.density * distance;
            1.0 - (-d * d).exp()
        }
    };
    f.clamp(0.0, 1.0)
}

/// Height-based fog modifier: denser below base_height.
#[allow(dead_code)]
pub fn height_fog_modifier(y: f32, base_height: f32, falloff: f32) -> f32 {
    if falloff.abs() < 1e-6 {
        return 1.0;
    }
    let delta = y - base_height;
    if delta <= 0.0 {
        1.0
    } else {
        (-delta * falloff).exp()
    }
}

/// Apply fog to a fragment colour.
///
/// `frag_color` and fog colour are in linear space `[r, g, b]`.
#[allow(dead_code)]
pub fn apply_fog(frag_color: [f32; 3], distance: f32, y: f32, config: &FogConfig) -> [f32; 3] {
    let base_factor = fog_factor(distance, config);
    let height_mod = height_fog_modifier(y, config.base_height, config.height_falloff);
    let f = (base_factor * height_mod).clamp(0.0, 1.0);
    [
        frag_color[0] * (1.0 - f) + config.color[0] * f,
        frag_color[1] * (1.0 - f) + config.color[1] * f,
        frag_color[2] * (1.0 - f) + config.color[2] * f,
    ]
}

/// Compute the distance at which fog reaches a given threshold.
#[allow(dead_code)]
pub fn distance_for_fog_threshold(threshold: f32, config: &FogConfig) -> f32 {
    let threshold = threshold.clamp(0.01, 0.99);
    match config.mode {
        FogMode::Linear => {
            config.start + threshold * (config.end - config.start)
        }
        FogMode::Exponential => {
            if config.density.abs() < 1e-6 { f32::MAX } else {
                -(1.0 - threshold).ln() / config.density
            }
        }
        FogMode::ExponentialSquared => {
            if config.density.abs() < 1e-6 { f32::MAX } else {
                (-(1.0 - threshold).ln()).sqrt() / config.density
            }
        }
    }
}

/// Blend two fog configs.
#[allow(dead_code)]
pub fn blend_fog_configs(a: &FogConfig, b: &FogConfig, t: f32) -> FogConfig {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    FogConfig {
        mode: if t < 0.5 { a.mode } else { b.mode },
        color: [
            a.color[0] * inv + b.color[0] * t,
            a.color[1] * inv + b.color[1] * t,
            a.color[2] * inv + b.color[2] * t,
        ],
        start: a.start * inv + b.start * t,
        end: a.end * inv + b.end * t,
        density: a.density * inv + b.density * t,
        height_falloff: a.height_falloff * inv + b.height_falloff * t,
        base_height: a.base_height * inv + b.base_height * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = FogConfig::default();
        assert_eq!(c.mode, FogMode::Linear);
        assert!(c.end > c.start);
    }

    #[test]
    fn test_linear_fog_zero() {
        let c = FogConfig::default();
        let f = fog_factor(0.0, &c);
        assert!(f.abs() < 1e-5);
    }

    #[test]
    fn test_linear_fog_full() {
        let c = FogConfig { start: 10.0, end: 100.0, ..Default::default() };
        let f = fog_factor(100.0, &c);
        assert!((f - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_exponential_fog_zero_distance() {
        let c = FogConfig { mode: FogMode::Exponential, density: 0.1, ..Default::default() };
        let f = fog_factor(0.0, &c);
        assert!(f.abs() < 1e-5);
    }

    #[test]
    fn test_exp_squared_increases() {
        let c = FogConfig { mode: FogMode::ExponentialSquared, density: 0.1, ..Default::default() };
        let f1 = fog_factor(5.0, &c);
        let f2 = fog_factor(20.0, &c);
        assert!(f2 > f1);
    }

    #[test]
    fn test_height_fog_below() {
        let m = height_fog_modifier(-5.0, 0.0, 1.0);
        assert!((m - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_height_fog_above() {
        let m = height_fog_modifier(10.0, 0.0, 0.5);
        assert!(m < 1.0);
    }

    #[test]
    fn test_apply_fog_no_fog() {
        let c = FogConfig { start: 100.0, end: 200.0, ..Default::default() };
        let result = apply_fog([1.0, 0.0, 0.0], 0.0, 0.0, &c);
        assert!((result[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_distance_for_threshold_linear() {
        let c = FogConfig { start: 0.0, end: 100.0, ..Default::default() };
        let d = distance_for_fog_threshold(0.5, &c);
        assert!((d - 50.0).abs() < 1e-3);
    }

    #[test]
    fn test_blend_fog_configs() {
        let a = FogConfig { density: 0.1, ..Default::default() };
        let b = FogConfig { density: 0.5, ..Default::default() };
        let r = blend_fog_configs(&a, &b, 0.5);
        assert!((r.density - 0.3).abs() < 1e-5);
    }
}
