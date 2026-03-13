// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Film look-up table (LUT) color grading.

use std::f32::consts::PI;

/// Configuration for film lookup.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FilmLookupConfig {
    pub intensity: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub temperature: f32,
    pub tint: f32,
}

#[allow(dead_code)]
pub fn default_film_lookup() -> FilmLookupConfig {
    FilmLookupConfig { intensity: 0.5, contrast: 1.0, saturation: 1.0, temperature: 6500.0, tint: 0.0 }
}

#[allow(dead_code)]
pub fn set_film_lookup_intensity(cfg: &mut FilmLookupConfig, value: f32) {
    cfg.intensity = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_film_lookup_contrast(cfg: &mut FilmLookupConfig, value: f32) {
    cfg.contrast = value.clamp(0.1_f32, 10.0_f32);
}

#[allow(dead_code)]
pub fn set_film_lookup_saturation(cfg: &mut FilmLookupConfig, value: f32) {
    cfg.saturation = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn set_film_lookup_temperature(cfg: &mut FilmLookupConfig, value: f32) {
    cfg.temperature = value.clamp(1000.0_f32, 20000.0_f32);
}

#[allow(dead_code)]
pub fn set_film_lookup_tint(cfg: &mut FilmLookupConfig, value: f32) {
    cfg.tint = value.clamp(0.0_f32, 1.0_f32);
}

#[allow(dead_code)]
pub fn film_lookup_weight(cfg: &FilmLookupConfig) -> f32 {
    (cfg.intensity * (PI * 0.25).sin()).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn blend_film_lookup(a: &FilmLookupConfig, b: &FilmLookupConfig, t: f32) -> FilmLookupConfig {
    let t = t.clamp(0.0, 1.0);
    FilmLookupConfig {
        intensity: a.intensity + (b.intensity - a.intensity) * t,
        contrast: a.contrast + (b.contrast - a.contrast) * t,
        saturation: a.saturation + (b.saturation - a.saturation) * t,
        temperature: a.temperature + (b.temperature - a.temperature) * t,
        tint: a.tint + (b.tint - a.tint) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let cfg = default_film_lookup();
        assert!((cfg.intensity - 0.5_f32).abs() < 1e-3);
    }

    #[test]
    fn test_set_intensity() {
        let mut cfg = default_film_lookup();
        set_film_lookup_intensity(&mut cfg, 0.7);
        assert!((cfg.intensity - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_contrast() {
        let mut cfg = default_film_lookup();
        set_film_lookup_contrast(&mut cfg, 0.8);
        assert!((cfg.contrast - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_saturation() {
        let mut cfg = default_film_lookup();
        set_film_lookup_saturation(&mut cfg, 0.6);
        assert!((cfg.saturation - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_temperature() {
        let mut cfg = default_film_lookup();
        set_film_lookup_temperature(&mut cfg, 0.5);
        assert!((cfg.temperature - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_tint() {
        let mut cfg = default_film_lookup();
        set_film_lookup_tint(&mut cfg, 0.4);
        assert!((cfg.tint - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_weight() {
        let cfg = default_film_lookup();
        let w = film_lookup_weight(&cfg);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_blend() {
        let a = default_film_lookup();
        let mut b = default_film_lookup();
        b.intensity = 1.0;
        let mid = blend_film_lookup(&a, &b, 0.5);
        assert!((mid.intensity - 0.75_f32).abs() < 1e-3);
    }

    #[test]
    fn test_blend_zero() {
        let a = default_film_lookup();
        let b = default_film_lookup();
        let r = blend_film_lookup(&a, &b, 0.0);
        assert!((r.intensity - a.intensity).abs() < 1e-6);
    }

    #[test]
    fn test_blend_one() {
        let a = default_film_lookup();
        let b = default_film_lookup();
        let r = blend_film_lookup(&a, &b, 1.0);
        assert!((r.intensity - b.intensity).abs() < 1e-6);
    }

    #[test]
    fn test_blend_clamp() {
        let a = default_film_lookup();
        let b = default_film_lookup();
        let r = blend_film_lookup(&a, &b, 2.0);
        assert!((r.intensity - b.intensity).abs() < 1e-6);
    }
}
