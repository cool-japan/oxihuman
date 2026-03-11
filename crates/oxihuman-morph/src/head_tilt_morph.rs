// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Head tilt / torticollis morph.

/// Head tilt configuration.
#[derive(Debug, Clone)]
pub struct HeadTiltMorphConfig {
    pub lateral_tilt: f32,
    pub forward_tilt: f32,
    pub axial_rotation: f32,
}

impl Default for HeadTiltMorphConfig {
    fn default() -> Self {
        Self {
            lateral_tilt: 0.0,
            forward_tilt: 0.0,
            axial_rotation: 0.0,
        }
    }
}

/// Head tilt morph state.
#[derive(Debug, Clone)]
pub struct HeadTiltMorph {
    pub config: HeadTiltMorphConfig,
    pub intensity: f32,
    pub enabled: bool,
}

impl HeadTiltMorph {
    pub fn new() -> Self {
        Self {
            config: HeadTiltMorphConfig::default(),
            intensity: 0.0,
            enabled: true,
        }
    }
}

impl Default for HeadTiltMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new HeadTiltMorph.
pub fn new_head_tilt_morph() -> HeadTiltMorph {
    HeadTiltMorph::new()
}

/// Set lateral head tilt (-1.0 left, 1.0 right).
pub fn head_tilt_set_lateral(morph: &mut HeadTiltMorph, v: f32) {
    morph.config.lateral_tilt = v.clamp(-1.0, 1.0);
}

/// Set forward tilt factor (positive = forward).
pub fn head_tilt_set_forward(morph: &mut HeadTiltMorph, v: f32) {
    morph.config.forward_tilt = v.clamp(-1.0, 1.0);
}

/// Set axial rotation factor.
pub fn head_tilt_set_axial(morph: &mut HeadTiltMorph, v: f32) {
    morph.config.axial_rotation = v.clamp(-1.0, 1.0);
}

/// Compute total head displacement magnitude.
pub fn head_tilt_magnitude(morph: &HeadTiltMorph) -> f32 {
    let l = morph.config.lateral_tilt;
    let f = morph.config.forward_tilt;
    let r = morph.config.axial_rotation;
    ((l * l + f * f + r * r).sqrt() * morph.intensity).min(1.0)
}

/// Serialize to JSON.
pub fn head_tilt_to_json(morph: &HeadTiltMorph) -> String {
    format!(
        r#"{{"intensity":{},"lateral_tilt":{},"forward_tilt":{},"axial_rotation":{}}}"#,
        morph.intensity,
        morph.config.lateral_tilt,
        morph.config.forward_tilt,
        morph.config.axial_rotation,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = new_head_tilt_morph();
        assert!((m.config.lateral_tilt - 0.0).abs() < 1e-6 /* default */);
    }

    #[test]
    fn test_lateral_clamp_high() {
        let mut m = new_head_tilt_morph();
        head_tilt_set_lateral(&mut m, 5.0);
        assert!((m.config.lateral_tilt - 1.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_lateral_clamp_low() {
        let mut m = new_head_tilt_morph();
        head_tilt_set_lateral(&mut m, -5.0);
        assert!((m.config.lateral_tilt - (-1.0)).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_forward() {
        let mut m = new_head_tilt_morph();
        head_tilt_set_forward(&mut m, 0.6);
        assert!((m.config.forward_tilt - 0.6).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_axial() {
        let mut m = new_head_tilt_morph();
        head_tilt_set_axial(&mut m, 0.3);
        assert!((m.config.axial_rotation - 0.3).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_magnitude_zero() {
        let m = new_head_tilt_morph();
        assert!((head_tilt_magnitude(&m) - 0.0).abs() < 1e-6 /* zero */);
    }

    #[test]
    fn test_magnitude_nonzero() {
        let mut m = new_head_tilt_morph();
        head_tilt_set_lateral(&mut m, 1.0);
        m.intensity = 1.0;
        assert!(head_tilt_magnitude(&m) > 0.0 /* nonzero */);
    }

    #[test]
    fn test_json_key() {
        let m = new_head_tilt_morph();
        let j = head_tilt_to_json(&m);
        assert!(j.contains("lateral_tilt") /* key */);
    }

    #[test]
    fn test_default() {
        let m = HeadTiltMorph::default();
        assert!(m.enabled /* enabled */);
    }
}
