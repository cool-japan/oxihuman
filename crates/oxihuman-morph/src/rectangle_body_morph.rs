// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Rectangle/straight body figure morph — minimal waist definition.

/// Configuration for the rectangle body morph.
#[derive(Debug, Clone)]
pub struct RectangleBodyConfig {
    pub waist_fullness: f32,
    pub shoulder_hip_balance: f32,
}

impl Default for RectangleBodyConfig {
    fn default() -> Self {
        RectangleBodyConfig {
            waist_fullness: 0.7,
            shoulder_hip_balance: 0.95,
        }
    }
}

/// State for the rectangle body morph.
#[derive(Debug, Clone)]
pub struct RectangleBodyMorph {
    pub intensity: f32,
    pub config: RectangleBodyConfig,
    pub enabled: bool,
}

/// Create a new rectangle body morph.
pub fn new_rectangle_body_morph() -> RectangleBodyMorph {
    RectangleBodyMorph {
        intensity: 0.0,
        config: RectangleBodyConfig::default(),
        enabled: true,
    }
}

/// Set intensity [0, 1].
pub fn rect_set_intensity(m: &mut RectangleBodyMorph, v: f32) {
    m.intensity = v.clamp(0.0, 1.0);
}

/// Waist fullness (reduces waist cinch).
pub fn rect_waist_fullness(m: &RectangleBodyMorph) -> f32 {
    m.intensity * m.config.waist_fullness
}

/// Shoulder-to-hip balance — closer to 1 = more rectangular.
pub fn rect_shoulder_hip_balance(m: &RectangleBodyMorph) -> f32 {
    let base = 0.8_f32;
    base + 0.15 * m.intensity * m.config.shoulder_hip_balance
}

/// Overall straightness score [0, 1].
pub fn rect_straightness(m: &RectangleBodyMorph) -> f32 {
    m.intensity
}

/// Serialise to JSON.
pub fn rect_to_json(m: &RectangleBodyMorph) -> String {
    format!(
        r#"{{"intensity":{:.3},"waist_fullness":{:.3},"enabled":{}}}"#,
        m.intensity,
        rect_waist_fullness(m),
        m.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_zero() {
        let m = new_rectangle_body_morph();
        assert!((m.intensity - 0.0).abs() < 1e-6 /* zero */);
    }

    #[test]
    fn clamp() {
        let mut m = new_rectangle_body_morph();
        rect_set_intensity(&mut m, 3.0);
        assert!((m.intensity - 1.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn waist_fullness_zero_at_zero() {
        let m = new_rectangle_body_morph();
        assert!((rect_waist_fullness(&m) - 0.0).abs() < 1e-6 /* zero */);
    }

    #[test]
    fn balance_increases() {
        let mut m = new_rectangle_body_morph();
        rect_set_intensity(&mut m, 0.0);
        let b0 = rect_shoulder_hip_balance(&m);
        rect_set_intensity(&mut m, 1.0);
        let b1 = rect_shoulder_hip_balance(&m);
        assert!(b1 > b0 /* more balance at higher intensity */);
    }

    #[test]
    fn straightness_equals_intensity() {
        let mut m = new_rectangle_body_morph();
        rect_set_intensity(&mut m, 0.7);
        assert!((rect_straightness(&m) - 0.7).abs() < 1e-6 /* correct */);
    }

    #[test]
    fn json_has_waist() {
        let m = new_rectangle_body_morph();
        assert!(rect_to_json(&m).contains("waist") /* json has field */);
    }

    #[test]
    fn enabled_default() {
        let m = new_rectangle_body_morph();
        assert!(m.enabled /* enabled */);
    }

    #[test]
    fn config_shoulder_hip_balance_positive() {
        let m = new_rectangle_body_morph();
        assert!(m.config.shoulder_hip_balance > 0.0 /* valid */);
    }
}
