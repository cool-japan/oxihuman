// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Film tone mapping — filmic S-curve and color grading operators.

use std::f32::consts::LOG2_E;

/// Tone mapping operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum FilmToneOp {
    Linear,
    Reinhard,
    AcesFilmic,
    Hable,
    Logarithmic,
}

/// Film tone config.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct FilmToneConfig {
    pub operator: FilmToneOp,
    pub exposure: f32,
    pub gamma: f32,
    pub white_point: f32,
    pub enabled: bool,
}

impl Default for FilmToneConfig {
    fn default() -> Self {
        Self {
            operator: FilmToneOp::AcesFilmic,
            exposure: 1.0,
            gamma: 2.2,
            white_point: 11.2,
            enabled: true,
        }
    }
}

/// Create default config.
#[allow(dead_code)]
pub fn default_film_tone_config() -> FilmToneConfig {
    FilmToneConfig::default()
}

/// Apply linear tone mapping.
#[allow(dead_code)]
pub fn tone_linear(x: f32, exposure: f32) -> f32 {
    (x * exposure).clamp(0.0, 1.0)
}

/// Apply Reinhard tone mapping.
#[allow(dead_code)]
pub fn tone_reinhard(x: f32, exposure: f32, white: f32) -> f32 {
    let v = x * exposure;
    v * (1.0 + v / (white * white)) / (1.0 + v)
}

/// ACES filmic approximation.
#[allow(dead_code)]
pub fn tone_aces_filmic(x: f32) -> f32 {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    ((x * (a * x + b)) / (x * (c * x + d) + e)).clamp(0.0, 1.0)
}

/// Hable (Uncharted 2) filmic curve partial.
#[allow(dead_code)]
fn hable_partial(x: f32) -> f32 {
    let a = 0.15;
    let b = 0.50;
    let c = 0.10;
    let d = 0.20;
    let e = 0.02;
    let f = 0.30;
    ((x * (a * x + c * b) + d * e) / (x * (a * x + b) + d * f)) - e / f
}

/// Hable (Uncharted 2) tone map.
#[allow(dead_code)]
pub fn tone_hable(x: f32, exposure: f32, white: f32) -> f32 {
    let curr = hable_partial(x * exposure * 2.0);
    let white_scale = 1.0 / hable_partial(white);
    (curr * white_scale).clamp(0.0, 1.0)
}

/// Log tone mapping using LOG2_E for conversion.
#[allow(dead_code)]
pub fn tone_logarithmic(x: f32, exposure: f32) -> f32 {
    let v = (x * exposure).max(0.0);
    ((v + 1.0).ln() * LOG2_E * 0.1).clamp(0.0, 1.0)
}

/// Apply gamma correction.
#[allow(dead_code)]
pub fn apply_gamma(x: f32, gamma: f32) -> f32 {
    x.max(0.0).powf(1.0 / gamma)
}

/// Apply full pipeline to a color value.
#[allow(dead_code)]
pub fn apply_film_tone(x: f32, cfg: &FilmToneConfig) -> f32 {
    if !cfg.enabled {
        return x;
    }
    let mapped = match cfg.operator {
        FilmToneOp::Linear => tone_linear(x, cfg.exposure),
        FilmToneOp::Reinhard => tone_reinhard(x, cfg.exposure, cfg.white_point),
        FilmToneOp::AcesFilmic => tone_aces_filmic(x * cfg.exposure),
        FilmToneOp::Hable => tone_hable(x, cfg.exposure, cfg.white_point),
        FilmToneOp::Logarithmic => tone_logarithmic(x, cfg.exposure),
    };
    apply_gamma(mapped, cfg.gamma)
}

/// Export config to JSON-like string.
#[allow(dead_code)]
pub fn film_tone_to_json(cfg: &FilmToneConfig) -> String {
    format!(
        r#"{{"exposure":{:.4},"gamma":{:.4},"enabled":{}}}"#,
        cfg.exposure, cfg.gamma, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_zero_in_zero_out() {
        assert!((tone_linear(0.0, 1.0)).abs() < 1e-6);
    }

    #[test]
    fn linear_clamps_at_one() {
        assert!((tone_linear(100.0, 1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn reinhard_positive() {
        let v = tone_reinhard(1.0, 1.0, 11.2);
        assert!(v > 0.0 && v <= 1.0);
    }

    #[test]
    fn aces_clamps() {
        assert!((0.0..=1.0).contains(&tone_aces_filmic(5.0)));
    }

    #[test]
    fn log_uses_log2e() {
        let v = tone_logarithmic(1.0, 1.0);
        assert!((0.0..=1.0).contains(&v));
    }

    #[test]
    fn gamma_identity_at_one() {
        assert!((apply_gamma(1.0, 1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn disabled_returns_input() {
        let cfg = FilmToneConfig {
            enabled: false,
            ..Default::default()
        };
        assert!((apply_film_tone(0.5, &cfg) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn apply_aces_nonzero_input() {
        let cfg = FilmToneConfig::default();
        let v = apply_film_tone(1.0, &cfg);
        assert!(v > 0.0);
    }

    #[test]
    fn json_contains_exposure() {
        assert!(film_tone_to_json(&FilmToneConfig::default()).contains("exposure"));
    }

    #[test]
    fn hable_clamps() {
        let v = tone_hable(10.0, 1.0, 11.2);
        assert!((0.0..=1.0).contains(&v));
    }
}
