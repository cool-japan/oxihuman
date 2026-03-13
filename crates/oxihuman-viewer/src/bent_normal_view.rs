// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Bent normal visualization for ambient occlusion debugging.

use std::f32::consts::PI;

/// Configuration for bent normal view.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BentNormalConfig {
    pub intensity: f32,
    pub bias: f32,
    pub radius: f32,
    pub show_directions: f32,
    pub blend_factor: f32,
}

#[allow(dead_code)]
pub fn default_bent_normal_view() -> BentNormalConfig {
    BentNormalConfig { intensity: 0.5, bias: 0.01, radius: 0.3, show_directions: 0.0, blend_factor: 0.5 }
}

#[allow(dead_code)]
pub fn set_bent_normal_view_intensity(cfg: &mut BentNormalConfig, value: f32) {
    cfg.intensity = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_bent_normal_view_bias(cfg: &mut BentNormalConfig, value: f32) {
    cfg.bias = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_bent_normal_view_radius(cfg: &mut BentNormalConfig, value: f32) {
    cfg.radius = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_bent_normal_view_show_directions(cfg: &mut BentNormalConfig, value: f32) {
    cfg.show_directions = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_bent_normal_view_blend_factor(cfg: &mut BentNormalConfig, value: f32) {
    cfg.blend_factor = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn bent_normal_view_weight(cfg: &BentNormalConfig) -> f32 {
    (cfg.intensity * (PI * 0.25).sin()).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn blend_bent_normal_view(a: &BentNormalConfig, b: &BentNormalConfig, t: f32) -> BentNormalConfig {
    let t = t.clamp(0.0, 1.0);
    BentNormalConfig {
        intensity: a.intensity + (b.intensity - a.intensity) * t,
        bias: a.bias + (b.bias - a.bias) * t,
        radius: a.radius + (b.radius - a.radius) * t,
        show_directions: a.show_directions + (b.show_directions - a.show_directions) * t,
        blend_factor: a.blend_factor + (b.blend_factor - a.blend_factor) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let cfg = default_bent_normal_view();
        assert!((cfg.intensity - 0.5_f32).abs() < 1e-3);
    }

    #[test]
    fn test_set_intensity() {
        let mut cfg = default_bent_normal_view();
        set_bent_normal_view_intensity(&mut cfg, 0.7);
        assert!((cfg.intensity - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_bias() {
        let mut cfg = default_bent_normal_view();
        set_bent_normal_view_bias(&mut cfg, 0.8);
        assert!((cfg.bias - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_radius() {
        let mut cfg = default_bent_normal_view();
        set_bent_normal_view_radius(&mut cfg, 0.6);
        assert!((cfg.radius - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_show_directions() {
        let mut cfg = default_bent_normal_view();
        set_bent_normal_view_show_directions(&mut cfg, 0.5);
        assert!((cfg.show_directions - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_blend_factor() {
        let mut cfg = default_bent_normal_view();
        set_bent_normal_view_blend_factor(&mut cfg, 0.4);
        assert!((cfg.blend_factor - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_weight() {
        let cfg = default_bent_normal_view();
        let w = bent_normal_view_weight(&cfg);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_blend() {
        let a = default_bent_normal_view();
        let mut b = default_bent_normal_view();
        b.intensity = 1.0;
        let mid = blend_bent_normal_view(&a, &b, 0.5);
        assert!((mid.intensity - 0.75_f32).abs() < 1e-3);
    }

    #[test]
    fn test_blend_zero() {
        let a = default_bent_normal_view();
        let b = default_bent_normal_view();
        let r = blend_bent_normal_view(&a, &b, 0.0);
        assert!((r.intensity - a.intensity).abs() < 1e-6);
    }

    #[test]
    fn test_blend_one() {
        let a = default_bent_normal_view();
        let b = default_bent_normal_view();
        let r = blend_bent_normal_view(&a, &b, 1.0);
        assert!((r.intensity - b.intensity).abs() < 1e-6);
    }

    #[test]
    fn test_blend_clamp() {
        let a = default_bent_normal_view();
        let b = default_bent_normal_view();
        let r = blend_bent_normal_view(&a, &b, 2.0);
        assert!((r.intensity - b.intensity).abs() < 1e-6);
    }
}
