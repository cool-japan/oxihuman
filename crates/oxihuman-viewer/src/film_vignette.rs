// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Film vignette post-process effect.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FilmVignetteConfig {
    pub strength: f32,
    pub radius: f32,
    pub softness: f32,
    pub enabled: bool,
}

impl Default for FilmVignetteConfig {
    fn default() -> Self {
        Self {
            strength: 0.5,
            radius: 0.75,
            softness: 0.45,
            enabled: true,
        }
    }
}

#[allow(dead_code)]
pub fn default_film_vignette_config() -> FilmVignetteConfig {
    FilmVignetteConfig::default()
}

#[allow(dead_code)]
pub fn fvig_set_strength(cfg: &mut FilmVignetteConfig, v: f32) {
    cfg.strength = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn fvig_set_radius(cfg: &mut FilmVignetteConfig, v: f32) {
    cfg.radius = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn fvig_set_softness(cfg: &mut FilmVignetteConfig, v: f32) {
    cfg.softness = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn fvig_set_enabled(cfg: &mut FilmVignetteConfig, v: bool) {
    cfg.enabled = v;
}

#[allow(dead_code)]
pub fn fvig_reset(cfg: &mut FilmVignetteConfig) {
    *cfg = FilmVignetteConfig::default();
}

/// Compute vignette alpha at normalized distance `d` from image centre.
#[allow(dead_code)]
pub fn fvig_alpha(cfg: &FilmVignetteConfig, d: f32) -> f32 {
    if !cfg.enabled {
        return 0.0;
    }
    let d = d.clamp(0.0, 1.0);
    let soft = cfg.softness.max(1e-6);
    let v = ((d - cfg.radius) / soft).clamp(0.0, 1.0);
    v * v * (3.0 - 2.0 * v) * cfg.strength
}

#[allow(dead_code)]
pub fn fvig_rim_alpha(cfg: &FilmVignetteConfig) -> f32 {
    fvig_alpha(cfg, 1.0)
}

#[allow(dead_code)]
pub fn fvig_fade_angle_rad(cfg: &FilmVignetteConfig) -> f32 {
    cfg.radius * PI * 0.5
}

#[allow(dead_code)]
pub fn fvig_to_json(cfg: &FilmVignetteConfig) -> String {
    format!(
        "{{\"strength\":{:.4},\"radius\":{:.4},\"enabled\":{}}}",
        cfg.strength, cfg.radius, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_enabled() {
        assert!(default_film_vignette_config().enabled);
    }
    #[test]
    fn alpha_at_centre_zero() {
        let c = default_film_vignette_config();
        assert!(fvig_alpha(&c, 0.0).abs() < 1e-6);
    }
    #[test]
    fn alpha_range_0_to_strength() {
        let c = default_film_vignette_config();
        let a = fvig_alpha(&c, 1.0);
        assert!((0.0..=c.strength).contains(&a) || a <= c.strength + 1e-4);
    }
    #[test]
    fn disabled_alpha_is_zero() {
        let mut c = default_film_vignette_config();
        fvig_set_enabled(&mut c, false);
        assert!(fvig_alpha(&c, 1.0).abs() < 1e-6);
    }
    #[test]
    fn set_strength_clamps() {
        let mut c = default_film_vignette_config();
        fvig_set_strength(&mut c, 5.0);
        assert!((0.0..=1.0).contains(&c.strength));
    }
    #[test]
    fn set_radius_clamps() {
        let mut c = default_film_vignette_config();
        fvig_set_radius(&mut c, -1.0);
        assert!(c.radius.abs() < 1e-6);
    }
    #[test]
    fn reset_restores_defaults() {
        let mut c = default_film_vignette_config();
        fvig_set_strength(&mut c, 0.0);
        fvig_reset(&mut c);
        assert!(c.enabled);
    }
    #[test]
    fn rim_alpha_matches() {
        let c = default_film_vignette_config();
        assert!((fvig_rim_alpha(&c) - fvig_alpha(&c, 1.0)).abs() < 1e-6);
    }
    #[test]
    fn fade_angle_nonneg() {
        assert!(fvig_fade_angle_rad(&default_film_vignette_config()) >= 0.0);
    }
    #[test]
    fn to_json_has_radius() {
        assert!(fvig_to_json(&default_film_vignette_config()).contains("\"radius\""));
    }
}
