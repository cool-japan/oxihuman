// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Indirect lighting and global illumination approximation.

use std::f32::consts::PI;

/// Configuration for indirect light.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IndirectLightConfig {
    pub bounce_intensity: f32,
    pub falloff: f32,
    pub max_distance: f32,
    pub color_bleed: f32,
    pub quality: f32,
}

#[allow(dead_code)]
pub fn default_indirect_light() -> IndirectLightConfig {
    IndirectLightConfig { bounce_intensity: 0.5, falloff: 2.0, max_distance: 10.0, color_bleed: 0.3, quality: 0.5 }
}

#[allow(dead_code)]
pub fn set_indirect_light_bounce_intensity(cfg: &mut IndirectLightConfig, value: f32) {
    cfg.bounce_intensity = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_indirect_light_falloff(cfg: &mut IndirectLightConfig, value: f32) {
    cfg.falloff = value.clamp(0.1_f32, 10.0_f32);
}

#[allow(dead_code)]
pub fn set_indirect_light_max_distance(cfg: &mut IndirectLightConfig, value: f32) {
    cfg.max_distance = value.clamp(0.1_f32, 10000.0_f32);
}

#[allow(dead_code)]
pub fn set_indirect_light_color_bleed(cfg: &mut IndirectLightConfig, value: f32) {
    cfg.color_bleed = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_indirect_light_quality(cfg: &mut IndirectLightConfig, value: f32) {
    cfg.quality = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn indirect_light_weight(cfg: &IndirectLightConfig) -> f32 {
    (cfg.bounce_intensity * (PI * 0.25).sin()).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn blend_indirect_light(a: &IndirectLightConfig, b: &IndirectLightConfig, t: f32) -> IndirectLightConfig {
    let t = t.clamp(0.0, 1.0);
    IndirectLightConfig {
        bounce_intensity: a.bounce_intensity + (b.bounce_intensity - a.bounce_intensity) * t,
        falloff: a.falloff + (b.falloff - a.falloff) * t,
        max_distance: a.max_distance + (b.max_distance - a.max_distance) * t,
        color_bleed: a.color_bleed + (b.color_bleed - a.color_bleed) * t,
        quality: a.quality + (b.quality - a.quality) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let cfg = default_indirect_light();
        assert!((cfg.bounce_intensity - 0.5_f32).abs() < 1e-3);
    }

    #[test]
    fn test_set_bounce_intensity() {
        let mut cfg = default_indirect_light();
        set_indirect_light_bounce_intensity(&mut cfg, 0.7);
        assert!((cfg.bounce_intensity - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_falloff() {
        let mut cfg = default_indirect_light();
        set_indirect_light_falloff(&mut cfg, 0.8);
        assert!((cfg.falloff - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_max_distance() {
        let mut cfg = default_indirect_light();
        set_indirect_light_max_distance(&mut cfg, 0.6);
        assert!((cfg.max_distance - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_color_bleed() {
        let mut cfg = default_indirect_light();
        set_indirect_light_color_bleed(&mut cfg, 0.5);
        assert!((cfg.color_bleed - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_quality() {
        let mut cfg = default_indirect_light();
        set_indirect_light_quality(&mut cfg, 0.4);
        assert!((cfg.quality - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_weight() {
        let cfg = default_indirect_light();
        let w = indirect_light_weight(&cfg);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_blend() {
        let a = default_indirect_light();
        let mut b = default_indirect_light();
        b.bounce_intensity = 1.0;
        let mid = blend_indirect_light(&a, &b, 0.5);
        assert!((mid.bounce_intensity - 0.75_f32).abs() < 1e-3);
    }

    #[test]
    fn test_blend_zero() {
        let a = default_indirect_light();
        let b = default_indirect_light();
        let r = blend_indirect_light(&a, &b, 0.0);
        assert!((r.bounce_intensity - a.bounce_intensity).abs() < 1e-6);
    }

    #[test]
    fn test_blend_one() {
        let a = default_indirect_light();
        let b = default_indirect_light();
        let r = blend_indirect_light(&a, &b, 1.0);
        assert!((r.bounce_intensity - b.bounce_intensity).abs() < 1e-6);
    }

    #[test]
    fn test_blend_clamp() {
        let a = default_indirect_light();
        let b = default_indirect_light();
        let r = blend_indirect_light(&a, &b, 2.0);
        assert!((r.bounce_intensity - b.bounce_intensity).abs() < 1e-6);
    }
}
