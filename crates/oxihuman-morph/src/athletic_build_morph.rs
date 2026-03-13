// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Athletic/muscular build morph.

/// Configuration for the athletic build morph.
#[derive(Debug, Clone)]
pub struct AthleticBuildConfig {
    pub muscle_definition: f32,
    pub shoulder_width: f32,
    pub waist_taper: f32,
}

impl Default for AthleticBuildConfig {
    fn default() -> Self {
        AthleticBuildConfig {
            muscle_definition: 0.8,
            shoulder_width: 0.7,
            waist_taper: 0.6,
        }
    }
}

/// State for the athletic build morph.
#[derive(Debug, Clone)]
pub struct AthleticBuildMorph {
    /// Overall intensity [0, 1].
    pub intensity: f32,
    pub config: AthleticBuildConfig,
    pub enabled: bool,
}

/// Create a new athletic build morph at zero intensity.
pub fn new_athletic_build_morph() -> AthleticBuildMorph {
    AthleticBuildMorph {
        intensity: 0.0,
        config: AthleticBuildConfig::default(),
        enabled: true,
    }
}

/// Set overall intensity (clamped 0–1).
pub fn ab_set_intensity(m: &mut AthleticBuildMorph, v: f32) {
    m.intensity = v.clamp(0.0, 1.0);
}

/// Muscle definition weight.
pub fn ab_muscle_weight(m: &AthleticBuildMorph) -> f32 {
    m.intensity * m.config.muscle_definition
}

/// Shoulder width delta.
pub fn ab_shoulder_delta(m: &AthleticBuildMorph) -> f32 {
    m.intensity * m.config.shoulder_width
}

/// Waist taper weight.
pub fn ab_waist_taper(m: &AthleticBuildMorph) -> f32 {
    m.intensity * m.config.waist_taper
}

/// Serialise to JSON.
pub fn ab_to_json(m: &AthleticBuildMorph) -> String {
    format!(
        r#"{{"intensity":{:.3},"muscle":{:.3},"enabled":{}}}"#,
        m.intensity,
        ab_muscle_weight(m),
        m.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_intensity_zero() {
        let m = new_athletic_build_morph();
        assert!((m.intensity - 0.0).abs() < 1e-6 /* zero intensity */);
    }

    #[test]
    fn clamp_intensity() {
        let mut m = new_athletic_build_morph();
        ab_set_intensity(&mut m, 5.0);
        assert!((m.intensity - 1.0).abs() < 1e-6 /* clamped to 1 */);
        ab_set_intensity(&mut m, -1.0);
        assert!((m.intensity - 0.0).abs() < 1e-6 /* clamped to 0 */);
    }

    #[test]
    fn muscle_weight_scales_with_intensity() {
        let mut m = new_athletic_build_morph();
        ab_set_intensity(&mut m, 0.5);
        let w = ab_muscle_weight(&m);
        assert!(w > 0.0 && w < 1.0 /* partial weight */);
    }

    #[test]
    fn shoulder_delta_nonzero_at_full() {
        let mut m = new_athletic_build_morph();
        ab_set_intensity(&mut m, 1.0);
        assert!(ab_shoulder_delta(&m) > 0.0 /* nonzero delta */);
    }

    #[test]
    fn waist_taper_zero_at_zero_intensity() {
        let m = new_athletic_build_morph();
        assert!((ab_waist_taper(&m) - 0.0).abs() < 1e-6 /* no taper at zero */);
    }

    #[test]
    fn json_contains_intensity() {
        let mut m = new_athletic_build_morph();
        ab_set_intensity(&mut m, 0.75);
        assert!(ab_to_json(&m).contains("0.750") /* json has intensity */);
    }

    #[test]
    fn enabled_default() {
        let m = new_athletic_build_morph();
        assert!(m.enabled /* enabled by default */);
    }

    #[test]
    fn config_values_valid() {
        let m = new_athletic_build_morph();
        assert!(m.config.muscle_definition > 0.0 /* positive muscle def */);
    }
}
