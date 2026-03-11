// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Film grain simulation overlay stub.

/// Film grain view config.
#[derive(Debug, Clone)]
pub struct FilmGrainViewConfig {
    pub intensity: f32,
    pub grain_size: f32,
    pub luma_sensitivity: f32,
    pub enabled: bool,
    pub animated: bool,
}

impl Default for FilmGrainViewConfig {
    fn default() -> Self {
        FilmGrainViewConfig {
            intensity: 0.1,
            grain_size: 1.5,
            luma_sensitivity: 0.5,
            enabled: true,
            animated: true,
        }
    }
}

/// Create a new film grain view config.
pub fn new_film_grain_view() -> FilmGrainViewConfig {
    FilmGrainViewConfig::default()
}

/// Set intensity.
pub fn fgv_set_intensity(cfg: &mut FilmGrainViewConfig, intensity: f32) {
    cfg.intensity = intensity.clamp(0.0, 1.0);
}

/// Set grain size.
pub fn fgv_set_grain_size(cfg: &mut FilmGrainViewConfig, size: f32) {
    cfg.grain_size = size.max(0.1);
}

/// Set luma sensitivity.
pub fn fgv_set_luma_sensitivity(cfg: &mut FilmGrainViewConfig, s: f32) {
    cfg.luma_sensitivity = s.clamp(0.0, 1.0);
}

/// Enable or disable.
pub fn fgv_set_enabled(cfg: &mut FilmGrainViewConfig, enabled: bool) {
    cfg.enabled = enabled;
}

/// Toggle animation.
pub fn fgv_toggle_animated(cfg: &mut FilmGrainViewConfig) {
    cfg.animated = !cfg.animated;
}

/// Simple hash-based grain value for a pixel.
pub fn fgv_grain_value(x: u32, y: u32, frame: u32, intensity: f32) -> f32 {
    let h = x.wrapping_mul(2654435761) ^ y.wrapping_mul(805459861) ^ frame.wrapping_mul(1234567);
    let f = (h & 0xFFFF) as f32 / 65535.0;
    (f - 0.5) * 2.0 * intensity
}

/// Apply grain to a pixel value.
pub fn fgv_apply(cfg: &FilmGrainViewConfig, pixel: f32, x: u32, y: u32, frame: u32) -> f32 {
    let grain = fgv_grain_value(x, y, frame, cfg.intensity);
    (pixel + grain * (1.0 - pixel * cfg.luma_sensitivity)).clamp(0.0, 1.0)
}

/// Return a JSON-like string.
pub fn fgv_to_json(cfg: &FilmGrainViewConfig) -> String {
    format!(
        r#"{{"intensity":{:.4},"grain_size":{:.4},"enabled":{}}}"#,
        cfg.intensity, cfg.grain_size, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_intensity() {
        let c = new_film_grain_view();
        assert!((c.intensity - 0.1).abs() < 1e-5, /* default intensity is 0.1 */);
    }

    #[test]
    fn test_set_intensity() {
        let mut c = new_film_grain_view();
        fgv_set_intensity(&mut c, 0.5);
        assert!((c.intensity - 0.5).abs() < 1e-5, /* intensity must match */);
    }

    #[test]
    fn test_set_intensity_clamps() {
        let mut c = new_film_grain_view();
        fgv_set_intensity(&mut c, 2.0);
        assert!((c.intensity - 1.0).abs() < 1e-5, /* intensity clamped to 1 */);
    }

    #[test]
    fn test_set_grain_size() {
        let mut c = new_film_grain_view();
        fgv_set_grain_size(&mut c, 3.0);
        assert!((c.grain_size - 3.0).abs() < 1e-5, /* grain size must match */);
    }

    #[test]
    fn test_set_enabled_false() {
        let mut c = new_film_grain_view();
        fgv_set_enabled(&mut c, false);
        assert!(!c.enabled /* should be disabled */,);
    }

    #[test]
    fn test_toggle_animated() {
        let mut c = new_film_grain_view();
        fgv_toggle_animated(&mut c);
        assert!(!c.animated /* animation toggled off */,);
    }

    #[test]
    fn test_grain_value_nonzero_intensity() {
        let g = fgv_grain_value(100, 200, 5, 0.5);
        /* grain can be positive or negative, just check it's in range */
        assert!(g.abs() <= 0.5, /* grain value should be within intensity range */);
    }

    #[test]
    fn test_apply_clamps_output() {
        let c = new_film_grain_view();
        let v = fgv_apply(&c, 0.0, 0, 0, 0);
        assert!((0.0..=1.0).contains(&v) /* output must be in [0,1] */,);
    }

    #[test]
    fn test_to_json_contains_intensity() {
        let c = new_film_grain_view();
        let j = fgv_to_json(&c);
        assert!(j.contains("intensity"), /* JSON must contain intensity */);
    }

    #[test]
    fn test_luma_sensitivity_default() {
        let c = new_film_grain_view();
        assert!((c.luma_sensitivity - 0.5).abs() < 1e-5, /* default luma sensitivity is 0.5 */);
    }
}
