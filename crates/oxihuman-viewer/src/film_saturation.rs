// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Film saturation control — vibrance, per-channel hue rotation, and HSL saturation.

/// Saturation config.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct FilmSaturationConfig {
    /// Global saturation scale (1 = unchanged, 0 = desaturate, 2 = double).
    pub saturation: f32,
    /// Vibrance — protects already-saturated colours, 0..=2.
    pub vibrance: f32,
    /// Per-channel hue shift in degrees [r, g, b].
    pub hue_shift: [f32; 3],
    pub enabled: bool,
}

impl Default for FilmSaturationConfig {
    fn default() -> Self {
        Self {
            saturation: 1.0,
            vibrance: 0.0,
            hue_shift: [0.0, 0.0, 0.0],
            enabled: true,
        }
    }
}

#[allow(dead_code)]
pub fn new_film_saturation() -> FilmSaturationConfig {
    FilmSaturationConfig::default()
}

#[allow(dead_code)]
pub fn fs_set_saturation(cfg: &mut FilmSaturationConfig, v: f32) {
    cfg.saturation = v.max(0.0);
}

#[allow(dead_code)]
pub fn fs_set_vibrance(cfg: &mut FilmSaturationConfig, v: f32) {
    cfg.vibrance = v.clamp(0.0, 2.0);
}

#[allow(dead_code)]
pub fn fs_set_hue_shift(cfg: &mut FilmSaturationConfig, channel: usize, deg: f32) {
    if channel < 3 {
        cfg.hue_shift[channel] = deg;
    }
}

#[allow(dead_code)]
pub fn fs_set_enabled(cfg: &mut FilmSaturationConfig, en: bool) {
    cfg.enabled = en;
}

#[allow(dead_code)]
pub fn fs_reset(cfg: &mut FilmSaturationConfig) {
    *cfg = FilmSaturationConfig::default();
}

/// Apply saturation to a linear RGB pixel.
#[allow(dead_code)]
pub fn fs_apply(cfg: &FilmSaturationConfig, rgb: [f32; 3]) -> [f32; 3] {
    if !cfg.enabled {
        return rgb;
    }
    let lum = 0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2];
    // Vibrance: less effect on already-saturated
    let max_c = rgb[0].max(rgb[1]).max(rgb[2]);
    let vib_amt = cfg.vibrance * (1.0 - max_c);
    let total_sat = (cfg.saturation + vib_amt).max(0.0);
    [
        lum + (rgb[0] - lum) * total_sat,
        lum + (rgb[1] - lum) * total_sat,
        lum + (rgb[2] - lum) * total_sat,
    ]
}

/// Saturation of an RGB pixel (max − min).
#[allow(dead_code)]
pub fn fs_pixel_saturation(rgb: [f32; 3]) -> f32 {
    let max = rgb[0].max(rgb[1]).max(rgb[2]);
    let min = rgb[0].min(rgb[1]).min(rgb[2]);
    max - min
}

/// Whether the config is identity.
#[allow(dead_code)]
pub fn fs_is_identity(cfg: &FilmSaturationConfig) -> bool {
    (cfg.saturation - 1.0).abs() < 1e-4
        && cfg.vibrance.abs() < 1e-4
        && cfg.hue_shift.iter().all(|h| h.abs() < 1e-4)
}

#[allow(dead_code)]
pub fn fs_blend(
    a: &FilmSaturationConfig,
    b: &FilmSaturationConfig,
    t: f32,
) -> FilmSaturationConfig {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    FilmSaturationConfig {
        saturation: a.saturation * inv + b.saturation * t,
        vibrance: a.vibrance * inv + b.vibrance * t,
        hue_shift: [
            a.hue_shift[0] * inv + b.hue_shift[0] * t,
            a.hue_shift[1] * inv + b.hue_shift[1] * t,
            a.hue_shift[2] * inv + b.hue_shift[2] * t,
        ],
        enabled: a.enabled || b.enabled,
    }
}

#[allow(dead_code)]
pub fn fs_to_json(cfg: &FilmSaturationConfig) -> String {
    format!(
        "{{\"saturation\":{:.4},\"vibrance\":{:.4},\"enabled\":{}}}",
        cfg.saturation, cfg.vibrance, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_identity() {
        assert!(fs_is_identity(&new_film_saturation()));
    }

    #[test]
    fn desaturate_zero_gives_gray() {
        let mut cfg = new_film_saturation();
        fs_set_saturation(&mut cfg, 0.0);
        let out = fs_apply(&cfg, [1.0, 0.0, 0.0]);
        assert!((out[0] - out[1]).abs() < 1e-5);
    }

    #[test]
    fn disabled_returns_input() {
        let mut cfg = new_film_saturation();
        fs_set_enabled(&mut cfg, false);
        let rgb = [0.2, 0.5, 0.8];
        let out = fs_apply(&cfg, rgb);
        assert!((out[0] - rgb[0]).abs() < 1e-6);
    }

    #[test]
    fn pixel_saturation_gray_zero() {
        assert!(fs_pixel_saturation([0.5, 0.5, 0.5]) < 1e-6);
    }

    #[test]
    fn pixel_saturation_red_positive() {
        assert!(fs_pixel_saturation([1.0, 0.0, 0.0]) > 0.0);
    }

    #[test]
    fn saturation_clamps_low() {
        let mut cfg = new_film_saturation();
        fs_set_saturation(&mut cfg, -5.0);
        assert!(cfg.saturation >= 0.0);
    }

    #[test]
    fn vibrance_clamps() {
        let mut cfg = new_film_saturation();
        fs_set_vibrance(&mut cfg, 10.0);
        assert!(cfg.vibrance <= 2.0);
    }

    #[test]
    fn reset_restores_identity() {
        let mut cfg = new_film_saturation();
        fs_set_saturation(&mut cfg, 2.0);
        fs_reset(&mut cfg);
        assert!(fs_is_identity(&cfg));
    }

    #[test]
    fn blend_midpoint() {
        let mut b = new_film_saturation();
        fs_set_saturation(&mut b, 2.0);
        let r = fs_blend(&new_film_saturation(), &b, 0.5);
        assert!((r.saturation - 1.5).abs() < 1e-5);
    }

    #[test]
    fn json_has_saturation() {
        let j = fs_to_json(&new_film_saturation());
        assert!(j.contains("saturation"));
    }
}
