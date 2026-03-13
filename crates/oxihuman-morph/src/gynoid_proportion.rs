// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Gynoid body proportion morph — weight centred lower body/hips.

/// Configuration for the gynoid proportion morph.
#[derive(Debug, Clone)]
pub struct GynoidProportionConfig {
    pub hip_fullness: f32,
    pub thigh_girth: f32,
    pub upper_body_slim: f32,
}

impl Default for GynoidProportionConfig {
    fn default() -> Self {
        GynoidProportionConfig {
            hip_fullness: 0.8,
            thigh_girth: 0.7,
            upper_body_slim: 0.5,
        }
    }
}

/// State for the gynoid proportion morph.
#[derive(Debug, Clone)]
pub struct GynoidProportion {
    pub intensity: f32,
    pub config: GynoidProportionConfig,
    pub enabled: bool,
}

/// Create a new gynoid proportion morph.
pub fn new_gynoid_proportion() -> GynoidProportion {
    GynoidProportion {
        intensity: 0.0,
        config: GynoidProportionConfig::default(),
        enabled: true,
    }
}

/// Set intensity [0, 1].
pub fn gyn_set_intensity(m: &mut GynoidProportion, v: f32) {
    m.intensity = v.clamp(0.0, 1.0);
}

/// Hip fullness weight.
pub fn gyn_hip_fullness(m: &GynoidProportion) -> f32 {
    m.intensity * m.config.hip_fullness
}

/// Thigh girth weight.
pub fn gyn_thigh_girth(m: &GynoidProportion) -> f32 {
    m.intensity * m.config.thigh_girth
}

/// Upper body slimming weight.
pub fn gyn_upper_slim(m: &GynoidProportion) -> f32 {
    m.intensity * m.config.upper_body_slim
}

/// Serialise to JSON.
pub fn gyn_to_json(m: &GynoidProportion) -> String {
    format!(
        r#"{{"intensity":{:.3},"hip_fullness":{:.3},"enabled":{}}}"#,
        m.intensity,
        gyn_hip_fullness(m),
        m.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_zero() {
        let m = new_gynoid_proportion();
        assert!((m.intensity - 0.0).abs() < 1e-6 /* zero */);
    }

    #[test]
    fn clamp_intensity() {
        let mut m = new_gynoid_proportion();
        gyn_set_intensity(&mut m, 2.0);
        assert!((m.intensity - 1.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn hip_fullness_at_max() {
        let mut m = new_gynoid_proportion();
        gyn_set_intensity(&mut m, 1.0);
        assert!((gyn_hip_fullness(&m) - m.config.hip_fullness).abs() < 1e-6 /* correct */);
    }

    #[test]
    fn thigh_girth_zero_at_zero() {
        let m = new_gynoid_proportion();
        assert!((gyn_thigh_girth(&m) - 0.0).abs() < 1e-6 /* zero */);
    }

    #[test]
    fn upper_slim_increases() {
        let mut m = new_gynoid_proportion();
        gyn_set_intensity(&mut m, 0.5);
        let s5 = gyn_upper_slim(&m);
        gyn_set_intensity(&mut m, 1.0);
        let s10 = gyn_upper_slim(&m);
        assert!(s10 > s5 /* more slim at higher intensity */);
    }

    #[test]
    fn json_has_intensity() {
        let mut m = new_gynoid_proportion();
        gyn_set_intensity(&mut m, 0.3);
        assert!(gyn_to_json(&m).contains("0.300") /* intensity in json */);
    }

    #[test]
    fn enabled_default() {
        let m = new_gynoid_proportion();
        assert!(m.enabled /* enabled */);
    }

    #[test]
    fn thigh_girth_config_positive() {
        let m = new_gynoid_proportion();
        assert!(m.config.thigh_girth > 0.0 /* valid */);
    }
}
