// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Halo/glow effect for selected objects or highlights.

use std::f32::consts::PI;

/// Halo effect configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HaloConfig {
    pub color: [f32; 3],
    pub intensity: f32,
    pub radius: f32,
    pub falloff: f32,
    pub enabled: bool,
}

#[allow(dead_code)]
pub fn default_halo_config() -> HaloConfig {
    HaloConfig {
        color: [1.0, 0.9, 0.3],
        intensity: 1.0,
        radius: 5.0,
        falloff: 2.0,
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn set_halo_color(cfg: &mut HaloConfig, r: f32, g: f32, b: f32) {
    cfg.color = [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)];
}

#[allow(dead_code)]
pub fn set_halo_intensity(cfg: &mut HaloConfig, value: f32) {
    cfg.intensity = value.clamp(0.0, 10.0);
}

#[allow(dead_code)]
pub fn set_halo_radius(cfg: &mut HaloConfig, value: f32) {
    cfg.radius = value.clamp(0.1, 100.0);
}

#[allow(dead_code)]
pub fn set_halo_falloff(cfg: &mut HaloConfig, value: f32) {
    cfg.falloff = value.clamp(0.1, 10.0);
}

#[allow(dead_code)]
pub fn enable_halo(cfg: &mut HaloConfig) {
    cfg.enabled = true;
}

#[allow(dead_code)]
pub fn disable_halo(cfg: &mut HaloConfig) {
    cfg.enabled = false;
}

#[allow(dead_code)]
pub fn halo_strength_at_distance(cfg: &HaloConfig, distance: f32) -> f32 {
    if distance <= 0.0 { return cfg.intensity; }
    let ratio = (distance / cfg.radius).clamp(0.0, 1.0);
    let angle = ratio * PI * 0.5;
    cfg.intensity * (1.0 - angle.sin()).powf(cfg.falloff)
}

#[allow(dead_code)]
pub fn halo_area(cfg: &HaloConfig) -> f32 {
    PI * cfg.radius * cfg.radius
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_halo_config();
        assert!(cfg.enabled);
        assert!((cfg.intensity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_color() {
        let mut cfg = default_halo_config();
        set_halo_color(&mut cfg, 0.5, 0.5, 0.5);
        assert!((cfg.color[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_intensity_clamp() {
        let mut cfg = default_halo_config();
        set_halo_intensity(&mut cfg, 15.0);
        assert!((cfg.intensity - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_radius() {
        let mut cfg = default_halo_config();
        set_halo_radius(&mut cfg, 20.0);
        assert!((cfg.radius - 20.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_falloff() {
        let mut cfg = default_halo_config();
        set_halo_falloff(&mut cfg, 3.0);
        assert!((cfg.falloff - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_enable_disable() {
        let mut cfg = default_halo_config();
        disable_halo(&mut cfg);
        assert!(!cfg.enabled);
        enable_halo(&mut cfg);
        assert!(cfg.enabled);
    }

    #[test]
    fn test_strength_at_zero() {
        let cfg = default_halo_config();
        let s = halo_strength_at_distance(&cfg, 0.0);
        assert!((s - cfg.intensity).abs() < 1e-6);
    }

    #[test]
    fn test_strength_at_radius() {
        let cfg = default_halo_config();
        let s = halo_strength_at_distance(&cfg, cfg.radius);
        assert!(s < 0.01);
    }

    #[test]
    fn test_halo_area() {
        let cfg = default_halo_config();
        let a = halo_area(&cfg);
        assert!((a - PI * 25.0).abs() < 1e-4);
    }

    #[test]
    fn test_strength_decreases() {
        let cfg = default_halo_config();
        let s1 = halo_strength_at_distance(&cfg, 1.0);
        let s2 = halo_strength_at_distance(&cfg, 3.0);
        assert!(s1 > s2);
    }
}
