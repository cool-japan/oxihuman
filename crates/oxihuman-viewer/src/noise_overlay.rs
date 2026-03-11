// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Procedural noise overlay (grain, static patterns).

use std::f32::consts::PI;

/// Noise pattern type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoisePattern {
    Grain,
    Static,
    Perlin,
    Voronoi,
}

impl NoisePattern {
    #[allow(dead_code)]
    pub fn name(self) -> &'static str {
        match self {
            NoisePattern::Grain => "grain",
            NoisePattern::Static => "static",
            NoisePattern::Perlin => "perlin",
            NoisePattern::Voronoi => "voronoi",
        }
    }
}

/// Configuration for noise overlay.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NoiseOverlayConfig {
    pub pattern: NoisePattern,
    pub intensity: f32,
    pub scale: f32,
    pub speed: f32,
    pub enabled: bool,
}

impl Default for NoiseOverlayConfig {
    fn default() -> Self {
        NoiseOverlayConfig {
            pattern: NoisePattern::Grain,
            intensity: 0.0,
            scale: 1.0,
            speed: 0.0,
            enabled: false,
        }
    }
}

#[allow(dead_code)]
pub fn default_noise_overlay_config() -> NoiseOverlayConfig {
    NoiseOverlayConfig::default()
}

#[allow(dead_code)]
pub fn noise_set_intensity(cfg: &mut NoiseOverlayConfig, v: f32) {
    cfg.intensity = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn noise_set_scale(cfg: &mut NoiseOverlayConfig, v: f32) {
    cfg.scale = v.clamp(0.01, 100.0);
}

#[allow(dead_code)]
pub fn noise_set_pattern(cfg: &mut NoiseOverlayConfig, p: NoisePattern) {
    cfg.pattern = p;
}

#[allow(dead_code)]
pub fn noise_enable(cfg: &mut NoiseOverlayConfig) {
    cfg.enabled = true;
}

#[allow(dead_code)]
pub fn noise_disable(cfg: &mut NoiseOverlayConfig) {
    cfg.enabled = false;
}

/// Deterministic noise value at pixel (x, y) with seed.
#[allow(dead_code)]
pub fn noise_value_at(x: u32, y: u32, seed: u32, time: f32) -> f32 {
    let h = x.wrapping_mul(2654435761)
        ^ y.wrapping_mul(2246822519)
        ^ seed.wrapping_mul(3266489917)
        ^ (time * 1000.0) as u32;
    let h = h.wrapping_mul(2654435761);
    (h >> 8) as f32 / 16777215.0
}

/// Sample grain noise (fast hash-based).
#[allow(dead_code)]
pub fn grain_sample(x: u32, y: u32, frame: u32) -> f32 {
    noise_value_at(x, y, frame, 0.0)
}

/// Sample perlin-like noise using sin/cos approximation.
#[allow(dead_code)]
pub fn perlin_approx(x: f32, y: f32, scale: f32) -> f32 {
    let sx = x * scale;
    let sy = y * scale;
    let v = (sx * PI * 2.0).sin() * (sy * PI * 2.0).cos();
    (v + 1.0) * 0.5
}

#[allow(dead_code)]
pub fn noise_overlay_to_json(cfg: &NoiseOverlayConfig) -> String {
    format!(
        r#"{{"pattern":"{}","intensity":{:.4},"scale":{:.4},"enabled":{}}}"#,
        cfg.pattern.name(),
        cfg.intensity,
        cfg.scale,
        cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_disabled() {
        assert!(!default_noise_overlay_config().enabled);
    }

    #[test]
    fn set_intensity_clamps() {
        let mut cfg = default_noise_overlay_config();
        noise_set_intensity(&mut cfg, 2.0);
        assert!((cfg.intensity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn enable_disable() {
        let mut cfg = default_noise_overlay_config();
        noise_enable(&mut cfg);
        assert!(cfg.enabled);
        noise_disable(&mut cfg);
        assert!(!cfg.enabled);
    }

    #[test]
    fn grain_sample_in_range() {
        let v = grain_sample(10, 20, 5);
        assert!((0.0..=1.0).contains(&v));
    }

    #[test]
    fn grain_sample_varies() {
        let a = grain_sample(0, 0, 0);
        let b = grain_sample(1, 0, 0);
        assert!((a - b).abs() > 1e-6);
    }

    #[test]
    fn perlin_approx_in_range() {
        let v = perlin_approx(0.3, 0.7, 1.0);
        assert!((0.0..=1.0).contains(&v));
    }

    #[test]
    fn set_pattern() {
        let mut cfg = default_noise_overlay_config();
        noise_set_pattern(&mut cfg, NoisePattern::Perlin);
        assert_eq!(cfg.pattern, NoisePattern::Perlin);
    }

    #[test]
    fn set_scale_clamps_low() {
        let mut cfg = default_noise_overlay_config();
        noise_set_scale(&mut cfg, 0.0);
        assert!((cfg.scale - 0.01).abs() < 1e-6);
    }

    #[test]
    fn to_json_has_pattern() {
        assert!(noise_overlay_to_json(&default_noise_overlay_config()).contains("pattern"));
    }
}
