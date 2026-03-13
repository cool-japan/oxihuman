// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Hip tilt / pelvic obliquity morph.

/// Hip tilt configuration.
#[derive(Debug, Clone)]
pub struct HipTiltMorphConfig {
    pub lateral_tilt: f32,
    pub anterior_tilt: f32,
    pub pelvic_rotation: f32,
}

impl Default for HipTiltMorphConfig {
    fn default() -> Self {
        Self {
            lateral_tilt: 0.0,
            anterior_tilt: 0.0,
            pelvic_rotation: 0.0,
        }
    }
}

/// Hip tilt morph state.
#[derive(Debug, Clone)]
pub struct HipTiltMorph {
    pub config: HipTiltMorphConfig,
    pub intensity: f32,
    pub enabled: bool,
}

impl HipTiltMorph {
    pub fn new() -> Self {
        Self {
            config: HipTiltMorphConfig::default(),
            intensity: 0.0,
            enabled: true,
        }
    }
}

impl Default for HipTiltMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new HipTiltMorph.
pub fn new_hip_tilt_morph() -> HipTiltMorph {
    HipTiltMorph::new()
}

/// Set lateral tilt factor (-1.0 left, 1.0 right).
pub fn hip_tilt_set_lateral(morph: &mut HipTiltMorph, v: f32) {
    morph.config.lateral_tilt = v.clamp(-1.0, 1.0);
}

/// Set anterior tilt factor.
pub fn hip_tilt_set_anterior(morph: &mut HipTiltMorph, v: f32) {
    morph.config.anterior_tilt = v.clamp(-1.0, 1.0);
}

/// Set pelvic rotation factor.
pub fn hip_tilt_set_rotation(morph: &mut HipTiltMorph, v: f32) {
    morph.config.pelvic_rotation = v.clamp(-1.0, 1.0);
}

/// Compute total tilt magnitude.
pub fn hip_tilt_magnitude(morph: &HipTiltMorph) -> f32 {
    let l = morph.config.lateral_tilt;
    let a = morph.config.anterior_tilt;
    ((l * l + a * a).sqrt() * morph.intensity).min(1.0)
}

/// Serialize to JSON.
pub fn hip_tilt_to_json(morph: &HipTiltMorph) -> String {
    format!(
        r#"{{"intensity":{},"lateral_tilt":{},"anterior_tilt":{},"pelvic_rotation":{}}}"#,
        morph.intensity,
        morph.config.lateral_tilt,
        morph.config.anterior_tilt,
        morph.config.pelvic_rotation,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = new_hip_tilt_morph();
        assert!((m.config.lateral_tilt - 0.0).abs() < 1e-6 /* default zero */);
    }

    #[test]
    fn test_lateral_clamp() {
        let mut m = new_hip_tilt_morph();
        hip_tilt_set_lateral(&mut m, 5.0);
        assert!((m.config.lateral_tilt - 1.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_anterior_negative() {
        let mut m = new_hip_tilt_morph();
        hip_tilt_set_anterior(&mut m, -0.4);
        assert!((m.config.anterior_tilt - (-0.4)).abs() < 1e-6 /* negative stored */);
    }

    #[test]
    fn test_rotation() {
        let mut m = new_hip_tilt_morph();
        hip_tilt_set_rotation(&mut m, 0.5);
        assert!((m.config.pelvic_rotation - 0.5).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_magnitude_zero() {
        let m = new_hip_tilt_morph();
        assert!((hip_tilt_magnitude(&m) - 0.0).abs() < 1e-6 /* zero */);
    }

    #[test]
    fn test_magnitude_nonzero() {
        let mut m = new_hip_tilt_morph();
        hip_tilt_set_lateral(&mut m, 1.0);
        m.intensity = 1.0;
        assert!(hip_tilt_magnitude(&m) > 0.0 /* positive */);
    }

    #[test]
    fn test_json_key() {
        let m = new_hip_tilt_morph();
        let j = hip_tilt_to_json(&m);
        assert!(j.contains("lateral_tilt") /* key */);
    }

    #[test]
    fn test_default_enabled() {
        let m = HipTiltMorph::default();
        assert!(m.enabled /* enabled */);
    }

    #[test]
    fn test_clone() {
        let m = new_hip_tilt_morph();
        let c = m.clone();
        assert!((c.intensity - m.intensity).abs() < 1e-6 /* equal */);
    }
}
