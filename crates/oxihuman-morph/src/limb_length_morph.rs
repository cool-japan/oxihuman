// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Limb length asymmetry morph.

/// Limb length asymmetry configuration.
#[derive(Debug, Clone)]
pub struct LimbLengthMorphConfig {
    pub left_leg_scale: f32,
    pub right_leg_scale: f32,
    pub left_arm_scale: f32,
    pub right_arm_scale: f32,
}

impl Default for LimbLengthMorphConfig {
    fn default() -> Self {
        Self {
            left_leg_scale: 1.0,
            right_leg_scale: 1.0,
            left_arm_scale: 1.0,
            right_arm_scale: 1.0,
        }
    }
}

/// Limb length morph state.
#[derive(Debug, Clone)]
pub struct LimbLengthMorph {
    pub config: LimbLengthMorphConfig,
    pub intensity: f32,
    pub enabled: bool,
}

impl LimbLengthMorph {
    pub fn new() -> Self {
        Self {
            config: LimbLengthMorphConfig::default(),
            intensity: 1.0,
            enabled: true,
        }
    }
}

impl Default for LimbLengthMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new LimbLengthMorph.
pub fn new_limb_length_morph() -> LimbLengthMorph {
    LimbLengthMorph::new()
}

/// Set left leg length scale (0.5–1.5).
pub fn limb_set_left_leg(morph: &mut LimbLengthMorph, scale: f32) {
    morph.config.left_leg_scale = scale.clamp(0.5, 1.5);
}

/// Set right leg length scale.
pub fn limb_set_right_leg(morph: &mut LimbLengthMorph, scale: f32) {
    morph.config.right_leg_scale = scale.clamp(0.5, 1.5);
}

/// Set left arm length scale.
pub fn limb_set_left_arm(morph: &mut LimbLengthMorph, scale: f32) {
    morph.config.left_arm_scale = scale.clamp(0.5, 1.5);
}

/// Set right arm length scale.
pub fn limb_set_right_arm(morph: &mut LimbLengthMorph, scale: f32) {
    morph.config.right_arm_scale = scale.clamp(0.5, 1.5);
}

/// Compute leg length discrepancy (absolute difference).
pub fn limb_leg_discrepancy(morph: &LimbLengthMorph) -> f32 {
    (morph.config.left_leg_scale - morph.config.right_leg_scale).abs()
}

/// Serialize to JSON.
pub fn limb_length_to_json(morph: &LimbLengthMorph) -> String {
    format!(
        r#"{{"intensity":{},"left_leg":{},"right_leg":{},"left_arm":{},"right_arm":{}}}"#,
        morph.intensity,
        morph.config.left_leg_scale,
        morph.config.right_leg_scale,
        morph.config.left_arm_scale,
        morph.config.right_arm_scale,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = new_limb_length_morph();
        assert!((m.config.left_leg_scale - 1.0).abs() < 1e-6 /* default 1 */);
    }

    #[test]
    fn test_left_leg_clamp_high() {
        let mut m = new_limb_length_morph();
        limb_set_left_leg(&mut m, 3.0);
        assert!((m.config.left_leg_scale - 1.5).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_left_leg_clamp_low() {
        let mut m = new_limb_length_morph();
        limb_set_left_leg(&mut m, 0.1);
        assert!((m.config.left_leg_scale - 0.5).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_right_leg() {
        let mut m = new_limb_length_morph();
        limb_set_right_leg(&mut m, 0.9);
        assert!((m.config.right_leg_scale - 0.9).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_left_arm() {
        let mut m = new_limb_length_morph();
        limb_set_left_arm(&mut m, 1.1);
        assert!((m.config.left_arm_scale - 1.1).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_discrepancy_zero() {
        let m = new_limb_length_morph();
        assert!((limb_leg_discrepancy(&m) - 0.0).abs() < 1e-6 /* symmetric */);
    }

    #[test]
    fn test_discrepancy_nonzero() {
        let mut m = new_limb_length_morph();
        limb_set_left_leg(&mut m, 1.1);
        limb_set_right_leg(&mut m, 0.9);
        assert!((limb_leg_discrepancy(&m) - 0.2).abs() < 1e-4 /* 0.2 diff */);
    }

    #[test]
    fn test_json_key() {
        let m = new_limb_length_morph();
        let j = limb_length_to_json(&m);
        assert!(j.contains("left_leg") /* key */);
    }

    #[test]
    fn test_default() {
        let m = LimbLengthMorph::default();
        assert!(m.enabled /* enabled */);
    }
}
