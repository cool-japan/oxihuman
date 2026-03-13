// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Film grain v2 — improved grain simulation with luma-dependent intensity.

/// Grain pattern.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GrainPattern {
    Gaussian,
    Poisson,
    Structured,
}

/// Config.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct FilmGrainV2Config {
    pub pattern: GrainPattern,
    pub intensity: f32,
    /// Luma sensitivity: 0 = uniform, 1 = more grain in midtones.
    pub luma_sensitivity: f32,
    pub size: f32,
    pub animated: bool,
}

impl Default for FilmGrainV2Config {
    fn default() -> Self {
        Self {
            pattern: GrainPattern::Gaussian,
            intensity: 0.05,
            luma_sensitivity: 0.5,
            size: 1.0,
            animated: true,
        }
    }
}

#[allow(dead_code)]
pub fn new_film_grain_v2() -> FilmGrainV2Config {
    FilmGrainV2Config::default()
}

#[allow(dead_code)]
pub fn fg2_set_intensity(cfg: &mut FilmGrainV2Config, v: f32) {
    cfg.intensity = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn fg2_set_size(cfg: &mut FilmGrainV2Config, v: f32) {
    cfg.size = v.clamp(0.5, 4.0);
}

#[allow(dead_code)]
pub fn fg2_set_luma_sensitivity(cfg: &mut FilmGrainV2Config, v: f32) {
    cfg.luma_sensitivity = v.clamp(0.0, 1.0);
}

/// Compute grain intensity at a given luma value (0..1).
#[allow(dead_code)]
pub fn fg2_grain_at_luma(cfg: &FilmGrainV2Config, luma: f32) -> f32 {
    let l = luma.clamp(0.0, 1.0);
    let shape = 1.0 - (2.0 * l - 1.0).powi(2); // peaks at midtone
    cfg.intensity * (1.0 - cfg.luma_sensitivity + cfg.luma_sensitivity * shape)
}

#[allow(dead_code)]
pub fn fg2_pattern_name(p: GrainPattern) -> &'static str {
    match p {
        GrainPattern::Gaussian => "gaussian",
        GrainPattern::Poisson => "poisson",
        GrainPattern::Structured => "structured",
    }
}

#[allow(dead_code)]
pub fn fg2_blend(a: &FilmGrainV2Config, b: &FilmGrainV2Config, t: f32) -> FilmGrainV2Config {
    let t = t.clamp(0.0, 1.0);
    FilmGrainV2Config {
        pattern: if t < 0.5 { a.pattern } else { b.pattern },
        intensity: a.intensity + (b.intensity - a.intensity) * t,
        luma_sensitivity: a.luma_sensitivity + (b.luma_sensitivity - a.luma_sensitivity) * t,
        size: a.size + (b.size - a.size) * t,
        animated: a.animated,
    }
}

#[allow(dead_code)]
pub fn fg2_to_json(cfg: &FilmGrainV2Config) -> String {
    format!(
        "{{\"pattern\":\"{}\",\"intensity\":{:.4},\"luma_sens\":{:.4},\"size\":{:.4}}}",
        fg2_pattern_name(cfg.pattern),
        cfg.intensity,
        cfg.luma_sensitivity,
        cfg.size
    )
}

#[allow(dead_code)]
pub fn fg2_is_disabled(cfg: &FilmGrainV2Config) -> bool {
    cfg.intensity < 1e-5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_gaussian() {
        assert_eq!(new_film_grain_v2().pattern, GrainPattern::Gaussian);
    }

    #[test]
    fn intensity_clamp() {
        let mut c = new_film_grain_v2();
        fg2_set_intensity(&mut c, 5.0);
        assert!(c.intensity <= 1.0);
    }

    #[test]
    fn intensity_not_negative() {
        let mut c = new_film_grain_v2();
        fg2_set_intensity(&mut c, -1.0);
        assert!(c.intensity >= 0.0);
    }

    #[test]
    fn size_clamp_min() {
        let mut c = new_film_grain_v2();
        fg2_set_size(&mut c, 0.0);
        assert!(c.size >= 0.5);
    }

    #[test]
    fn grain_at_midtone_positive() {
        let c = new_film_grain_v2();
        assert!(fg2_grain_at_luma(&c, 0.5) > 0.0);
    }

    #[test]
    fn grain_at_luma_bounded() {
        let c = new_film_grain_v2();
        let v = fg2_grain_at_luma(&c, 0.3);
        assert!((0.0..=1.0).contains(&v));
    }

    #[test]
    fn pattern_name_poisson() {
        assert_eq!(fg2_pattern_name(GrainPattern::Poisson), "poisson");
    }

    #[test]
    fn blend_midpoint() {
        let a = new_film_grain_v2();
        let mut b = new_film_grain_v2();
        fg2_set_intensity(&mut b, 0.0);
        let m = fg2_blend(&a, &b, 1.0);
        assert!((m.intensity).abs() < 1e-5);
    }

    #[test]
    fn disabled_when_zero() {
        let mut c = new_film_grain_v2();
        fg2_set_intensity(&mut c, 0.0);
        assert!(fg2_is_disabled(&c));
    }

    #[test]
    fn json_has_pattern() {
        assert!(fg2_to_json(&new_film_grain_v2()).contains("pattern"));
    }
}
