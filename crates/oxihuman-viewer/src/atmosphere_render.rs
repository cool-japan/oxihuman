// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Atmospheric scattering and sky rendering controls.

use std::f32::consts::PI;

/// Configuration for atmosphere render.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AtmosphereConfig {
    pub density: f32,
    pub scatter_strength: f32,
    pub absorption: f32,
    pub altitude_scale: f32,
    pub sun_intensity: f32,
}

#[allow(dead_code)]
pub fn default_atmosphere_render() -> AtmosphereConfig {
    AtmosphereConfig { density: 0.5, scatter_strength: 0.3, absorption: 0.1, altitude_scale: 0.8, sun_intensity: 1.0 }
}

#[allow(dead_code)]
pub fn set_atmosphere_render_density(cfg: &mut AtmosphereConfig, value: f32) {
    cfg.density = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_atmosphere_render_scatter_strength(cfg: &mut AtmosphereConfig, value: f32) {
    cfg.scatter_strength = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_atmosphere_render_absorption(cfg: &mut AtmosphereConfig, value: f32) {
    cfg.absorption = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_atmosphere_render_altitude_scale(cfg: &mut AtmosphereConfig, value: f32) {
    cfg.altitude_scale = value.clamp(0.0_f32, 10.0_f32);
}

#[allow(dead_code)]
pub fn set_atmosphere_render_sun_intensity(cfg: &mut AtmosphereConfig, value: f32) {
    cfg.sun_intensity = value.clamp(0.0_f32, 10.0_f32);
}

#[allow(dead_code)]
pub fn atmosphere_render_weight(cfg: &AtmosphereConfig) -> f32 {
    (cfg.density * (PI * 0.25).sin()).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn blend_atmosphere_render(a: &AtmosphereConfig, b: &AtmosphereConfig, t: f32) -> AtmosphereConfig {
    let t = t.clamp(0.0, 1.0);
    AtmosphereConfig {
        density: a.density + (b.density - a.density) * t,
        scatter_strength: a.scatter_strength + (b.scatter_strength - a.scatter_strength) * t,
        absorption: a.absorption + (b.absorption - a.absorption) * t,
        altitude_scale: a.altitude_scale + (b.altitude_scale - a.altitude_scale) * t,
        sun_intensity: a.sun_intensity + (b.sun_intensity - a.sun_intensity) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let cfg = default_atmosphere_render();
        assert!((cfg.density - 0.5_f32).abs() < 1e-3);
    }

    #[test]
    fn test_set_density() {
        let mut cfg = default_atmosphere_render();
        set_atmosphere_render_density(&mut cfg, 0.7);
        assert!((cfg.density - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_scatter_strength() {
        let mut cfg = default_atmosphere_render();
        set_atmosphere_render_scatter_strength(&mut cfg, 0.8);
        assert!((cfg.scatter_strength - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_absorption() {
        let mut cfg = default_atmosphere_render();
        set_atmosphere_render_absorption(&mut cfg, 0.6);
        assert!((cfg.absorption - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_altitude_scale() {
        let mut cfg = default_atmosphere_render();
        set_atmosphere_render_altitude_scale(&mut cfg, 0.5);
        assert!((cfg.altitude_scale - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_sun_intensity() {
        let mut cfg = default_atmosphere_render();
        set_atmosphere_render_sun_intensity(&mut cfg, 0.4);
        assert!((cfg.sun_intensity - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_weight() {
        let cfg = default_atmosphere_render();
        let w = atmosphere_render_weight(&cfg);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_blend() {
        let a = default_atmosphere_render();
        let mut b = default_atmosphere_render();
        b.density = 1.0;
        let mid = blend_atmosphere_render(&a, &b, 0.5);
        assert!((mid.density - 0.75_f32).abs() < 1e-3);
    }

    #[test]
    fn test_blend_zero() {
        let a = default_atmosphere_render();
        let b = default_atmosphere_render();
        let r = blend_atmosphere_render(&a, &b, 0.0);
        assert!((r.density - a.density).abs() < 1e-6);
    }

    #[test]
    fn test_blend_one() {
        let a = default_atmosphere_render();
        let b = default_atmosphere_render();
        let r = blend_atmosphere_render(&a, &b, 1.0);
        assert!((r.density - b.density).abs() < 1e-6);
    }

    #[test]
    fn test_blend_clamp() {
        let a = default_atmosphere_render();
        let b = default_atmosphere_render();
        let r = blend_atmosphere_render(&a, &b, 2.0);
        assert!((r.density - b.density).abs() < 1e-6);
    }
}
