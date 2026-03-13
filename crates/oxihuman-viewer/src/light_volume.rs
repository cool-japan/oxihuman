// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Volumetric light and god-ray rendering.

use std::f32::consts::PI;

/// Configuration for light volume.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LightVolumeConfig {
    pub density: f32,
    pub scatter: f32,
    pub absorption: f32,
    pub step_count: f32,
    pub intensity: f32,
}

#[allow(dead_code)]
pub fn default_light_volume() -> LightVolumeConfig {
    LightVolumeConfig { density: 0.5, scatter: 0.3, absorption: 0.1, step_count: 64.0, intensity: 0.5 }
}

#[allow(dead_code)]
pub fn set_light_volume_density(cfg: &mut LightVolumeConfig, value: f32) {
    cfg.density = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_light_volume_scatter(cfg: &mut LightVolumeConfig, value: f32) {
    cfg.scatter = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_light_volume_absorption(cfg: &mut LightVolumeConfig, value: f32) {
    cfg.absorption = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_light_volume_step_count(cfg: &mut LightVolumeConfig, value: f32) {
    cfg.step_count = value.clamp(1.0_f32, 128.0_f32);
}

#[allow(dead_code)]
pub fn set_light_volume_intensity(cfg: &mut LightVolumeConfig, value: f32) {
    cfg.intensity = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn light_volume_weight(cfg: &LightVolumeConfig) -> f32 {
    (cfg.density * (PI * 0.25).sin()).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn blend_light_volume(a: &LightVolumeConfig, b: &LightVolumeConfig, t: f32) -> LightVolumeConfig {
    let t = t.clamp(0.0, 1.0);
    LightVolumeConfig {
        density: a.density + (b.density - a.density) * t,
        scatter: a.scatter + (b.scatter - a.scatter) * t,
        absorption: a.absorption + (b.absorption - a.absorption) * t,
        step_count: a.step_count + (b.step_count - a.step_count) * t,
        intensity: a.intensity + (b.intensity - a.intensity) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let cfg = default_light_volume();
        assert!((cfg.density - 0.5_f32).abs() < 1e-3);
    }

    #[test]
    fn test_set_density() {
        let mut cfg = default_light_volume();
        set_light_volume_density(&mut cfg, 0.7);
        assert!((cfg.density - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_scatter() {
        let mut cfg = default_light_volume();
        set_light_volume_scatter(&mut cfg, 0.8);
        assert!((cfg.scatter - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_absorption() {
        let mut cfg = default_light_volume();
        set_light_volume_absorption(&mut cfg, 0.6);
        assert!((cfg.absorption - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_step_count() {
        let mut cfg = default_light_volume();
        set_light_volume_step_count(&mut cfg, 0.5);
        assert!((cfg.step_count - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_intensity() {
        let mut cfg = default_light_volume();
        set_light_volume_intensity(&mut cfg, 0.4);
        assert!((cfg.intensity - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_weight() {
        let cfg = default_light_volume();
        let w = light_volume_weight(&cfg);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_blend() {
        let a = default_light_volume();
        let mut b = default_light_volume();
        b.density = 1.0;
        let mid = blend_light_volume(&a, &b, 0.5);
        assert!((mid.density - 0.75_f32).abs() < 1e-3);
    }

    #[test]
    fn test_blend_zero() {
        let a = default_light_volume();
        let b = default_light_volume();
        let r = blend_light_volume(&a, &b, 0.0);
        assert!((r.density - a.density).abs() < 1e-6);
    }

    #[test]
    fn test_blend_one() {
        let a = default_light_volume();
        let b = default_light_volume();
        let r = blend_light_volume(&a, &b, 1.0);
        assert!((r.density - b.density).abs() < 1e-6);
    }

    #[test]
    fn test_blend_clamp() {
        let a = default_light_volume();
        let b = default_light_volume();
        let r = blend_light_volume(&a, &b, 2.0);
        assert!((r.density - b.density).abs() < 1e-6);
    }
}
