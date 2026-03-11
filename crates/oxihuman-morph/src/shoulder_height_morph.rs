// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Shoulder height asymmetry morph.

/// Shoulder height asymmetry configuration.
#[derive(Debug, Clone)]
pub struct ShoulderHeightMorphConfig {
    pub left_raise: f32,
    pub right_raise: f32,
    pub clavicle_tilt: f32,
}

impl Default for ShoulderHeightMorphConfig {
    fn default() -> Self {
        Self {
            left_raise: 0.0,
            right_raise: 0.0,
            clavicle_tilt: 0.0,
        }
    }
}

/// Shoulder height morph state.
#[derive(Debug, Clone)]
pub struct ShoulderHeightMorph {
    pub config: ShoulderHeightMorphConfig,
    pub intensity: f32,
    pub enabled: bool,
}

impl ShoulderHeightMorph {
    pub fn new() -> Self {
        Self {
            config: ShoulderHeightMorphConfig::default(),
            intensity: 0.0,
            enabled: true,
        }
    }
}

impl Default for ShoulderHeightMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new ShoulderHeightMorph.
pub fn new_shoulder_height_morph() -> ShoulderHeightMorph {
    ShoulderHeightMorph::new()
}

/// Set left shoulder raise amount (-1.0 to 1.0).
pub fn shoulder_height_set_left(morph: &mut ShoulderHeightMorph, v: f32) {
    morph.config.left_raise = v.clamp(-1.0, 1.0);
}

/// Set right shoulder raise amount.
pub fn shoulder_height_set_right(morph: &mut ShoulderHeightMorph, v: f32) {
    morph.config.right_raise = v.clamp(-1.0, 1.0);
}

/// Set overall clavicle tilt.
pub fn shoulder_height_set_tilt(morph: &mut ShoulderHeightMorph, v: f32) {
    morph.config.clavicle_tilt = v.clamp(-1.0, 1.0);
}

/// Compute shoulder asymmetry magnitude.
pub fn shoulder_height_asymmetry(morph: &ShoulderHeightMorph) -> f32 {
    (morph.config.left_raise - morph.config.right_raise).abs() * morph.intensity
}

/// Serialize to JSON.
pub fn shoulder_height_to_json(morph: &ShoulderHeightMorph) -> String {
    format!(
        r#"{{"intensity":{},"left_raise":{},"right_raise":{},"clavicle_tilt":{}}}"#,
        morph.intensity,
        morph.config.left_raise,
        morph.config.right_raise,
        morph.config.clavicle_tilt,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = new_shoulder_height_morph();
        assert!((m.config.left_raise - 0.0).abs() < 1e-6 /* default zero */);
    }

    #[test]
    fn test_left_clamp() {
        let mut m = new_shoulder_height_morph();
        shoulder_height_set_left(&mut m, 5.0);
        assert!((m.config.left_raise - 1.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_right_negative() {
        let mut m = new_shoulder_height_morph();
        shoulder_height_set_right(&mut m, -0.5);
        assert!((m.config.right_raise - (-0.5)).abs() < 1e-6 /* negative ok */);
    }

    #[test]
    fn test_tilt() {
        let mut m = new_shoulder_height_morph();
        shoulder_height_set_tilt(&mut m, 0.3);
        assert!((m.config.clavicle_tilt - 0.3).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_asymmetry_zero() {
        let m = new_shoulder_height_morph();
        assert!((shoulder_height_asymmetry(&m) - 0.0).abs() < 1e-6 /* zero */);
    }

    #[test]
    fn test_asymmetry_nonzero() {
        let mut m = new_shoulder_height_morph();
        shoulder_height_set_left(&mut m, 0.5);
        shoulder_height_set_right(&mut m, -0.5);
        m.intensity = 1.0;
        assert!((shoulder_height_asymmetry(&m) - 1.0).abs() < 1e-6 /* 1.0 diff */);
    }

    #[test]
    fn test_json_key() {
        let m = new_shoulder_height_morph();
        let j = shoulder_height_to_json(&m);
        assert!(j.contains("left_raise") /* key */);
    }

    #[test]
    fn test_default_enabled() {
        let m = ShoulderHeightMorph::default();
        assert!(m.enabled /* enabled */);
    }

    #[test]
    fn test_clone() {
        let m = new_shoulder_height_morph();
        let c = m.clone();
        assert!((c.intensity - m.intensity).abs() < 1e-6 /* equal */);
    }
}
