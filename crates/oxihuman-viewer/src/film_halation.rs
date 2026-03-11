// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Film halation — emulsion back-scatter glow around bright areas.

/// Configuration for film halation effect.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FilmHalationConfig {
    pub threshold: f32,
    pub radius: f32,
    pub intensity: f32,
    pub tint: [f32; 3],
    pub enabled: bool,
}

#[allow(dead_code)]
pub fn default_film_halation_config() -> FilmHalationConfig {
    FilmHalationConfig {
        threshold: 0.8,
        radius: 4.0,
        intensity: 0.15,
        tint: [1.0, 0.4, 0.1],
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn fhal_set_threshold(cfg: &mut FilmHalationConfig, v: f32) {
    cfg.threshold = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn fhal_set_radius(cfg: &mut FilmHalationConfig, r: f32) {
    cfg.radius = r.max(0.5);
}

#[allow(dead_code)]
pub fn fhal_set_intensity(cfg: &mut FilmHalationConfig, v: f32) {
    cfg.intensity = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn fhal_set_tint(cfg: &mut FilmHalationConfig, tint: [f32; 3]) {
    cfg.tint = [
        tint[0].clamp(0.0, 1.0),
        tint[1].clamp(0.0, 1.0),
        tint[2].clamp(0.0, 1.0),
    ];
}

#[allow(dead_code)]
pub fn fhal_set_enabled(cfg: &mut FilmHalationConfig, enabled: bool) {
    cfg.enabled = enabled;
}

#[allow(dead_code)]
pub fn fhal_apply(cfg: &FilmHalationConfig, pixel_luma: f32) -> [f32; 3] {
    if !cfg.enabled || pixel_luma < cfg.threshold {
        return [0.0, 0.0, 0.0];
    }
    let strength = ((pixel_luma - cfg.threshold) / (1.0 - cfg.threshold + 1e-6)).clamp(0.0, 1.0);
    let s = strength * cfg.intensity;
    [cfg.tint[0] * s, cfg.tint[1] * s, cfg.tint[2] * s]
}

#[allow(dead_code)]
pub fn fhal_luminance(rgb: [f32; 3]) -> f32 {
    0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2]
}

#[allow(dead_code)]
pub fn fhal_blend(a: &FilmHalationConfig, b: &FilmHalationConfig, t: f32) -> FilmHalationConfig {
    let t = t.clamp(0.0, 1.0);
    FilmHalationConfig {
        threshold: a.threshold + (b.threshold - a.threshold) * t,
        radius: a.radius + (b.radius - a.radius) * t,
        intensity: a.intensity + (b.intensity - a.intensity) * t,
        tint: [
            a.tint[0] + (b.tint[0] - a.tint[0]) * t,
            a.tint[1] + (b.tint[1] - a.tint[1]) * t,
            a.tint[2] + (b.tint[2] - a.tint[2]) * t,
        ],
        enabled: a.enabled,
    }
}

#[allow(dead_code)]
pub fn fhal_to_json(cfg: &FilmHalationConfig) -> String {
    format!(
        r#"{{"threshold":{:.4},"radius":{:.4},"intensity":{:.4},"enabled":{}}}"#,
        cfg.threshold, cfg.radius, cfg.intensity, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_film_halation_config();
        assert!((cfg.threshold - 0.8).abs() < 1e-6);
    }

    #[test]
    fn no_halation_below_threshold() {
        let cfg = default_film_halation_config();
        let result = fhal_apply(&cfg, 0.5);
        assert!(result.iter().all(|v| v.abs() < 1e-6));
    }

    #[test]
    fn halation_above_threshold() {
        let cfg = default_film_halation_config();
        let result = fhal_apply(&cfg, 1.0);
        assert!(result.iter().any(|v| *v > 0.0));
    }

    #[test]
    fn disabled_no_effect() {
        let mut cfg = default_film_halation_config();
        fhal_set_enabled(&mut cfg, false);
        let result = fhal_apply(&cfg, 1.0);
        assert!(result.iter().all(|v| v.abs() < 1e-6));
    }

    #[test]
    fn set_threshold_clamps() {
        let mut cfg = default_film_halation_config();
        fhal_set_threshold(&mut cfg, 2.0);
        assert!((cfg.threshold - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_radius_min() {
        let mut cfg = default_film_halation_config();
        fhal_set_radius(&mut cfg, 0.0);
        assert!(cfg.radius >= 0.5);
    }

    #[test]
    fn luminance_white() {
        assert!((fhal_luminance([1.0, 1.0, 1.0]) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn blend_midpoint() {
        let a = default_film_halation_config();
        let mut b = default_film_halation_config();
        fhal_set_intensity(&mut b, 1.0);
        let m = fhal_blend(&a, &b, 0.5);
        assert!(m.intensity > a.intensity && m.intensity < b.intensity);
    }

    #[test]
    fn to_json_fields() {
        let cfg = default_film_halation_config();
        let j = fhal_to_json(&cfg);
        assert!(j.contains("threshold"));
        assert!(j.contains("enabled"));
    }
}
