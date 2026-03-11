// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Upper lip body shape morph — controls vermilion height, fullness, and projection.

/// Upper lip body morph configuration.
#[derive(Debug, Clone)]
pub struct UpperLipBodyMorph {
    pub fullness: f32,
    pub projection: f32,
    pub vermilion_height: f32,
    pub roll: f32,
    pub flatness: f32,
}

impl UpperLipBodyMorph {
    pub fn new() -> Self {
        Self {
            fullness: 0.5,
            projection: 0.5,
            vermilion_height: 0.5,
            roll: 0.0,
            flatness: 0.0,
        }
    }
}

impl Default for UpperLipBodyMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new upper lip body morph.
pub fn new_upper_lip_body_morph() -> UpperLipBodyMorph {
    UpperLipBodyMorph::new()
}

/// Set lip fullness (0 = thin, 1 = full).
pub fn ulb_set_fullness(m: &mut UpperLipBodyMorph, v: f32) {
    m.fullness = v.clamp(0.0, 1.0);
}

/// Set anterior projection of upper lip.
pub fn ulb_set_projection(m: &mut UpperLipBodyMorph, v: f32) {
    m.projection = v.clamp(0.0, 1.0);
}

/// Set vermilion (red lip) height.
pub fn ulb_set_vermilion_height(m: &mut UpperLipBodyMorph, v: f32) {
    m.vermilion_height = v.clamp(0.0, 1.0);
}

/// Set lip roll / eversion amount.
pub fn ulb_set_roll(m: &mut UpperLipBodyMorph, v: f32) {
    m.roll = v.clamp(-1.0, 1.0);
}

/// Compute lip volume estimate.
pub fn ulb_volume_estimate(m: &UpperLipBodyMorph) -> f32 {
    m.fullness * m.projection * m.vermilion_height
}

/// Serialize to JSON-like string.
pub fn upper_lip_body_morph_to_json(m: &UpperLipBodyMorph) -> String {
    format!(
        r#"{{"fullness":{:.4},"projection":{:.4},"vermilion_height":{:.4},"roll":{:.4},"flatness":{:.4}}}"#,
        m.fullness, m.projection, m.vermilion_height, m.roll, m.flatness
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_upper_lip_body_morph();
        assert!((m.fullness - 0.5).abs() < 1e-6);
        assert_eq!(m.roll, 0.0);
    }

    #[test]
    fn test_fullness_clamp_high() {
        let mut m = new_upper_lip_body_morph();
        ulb_set_fullness(&mut m, 5.0);
        assert_eq!(m.fullness, 1.0);
    }

    #[test]
    fn test_fullness_clamp_low() {
        let mut m = new_upper_lip_body_morph();
        ulb_set_fullness(&mut m, -1.0);
        assert_eq!(m.fullness, 0.0);
    }

    #[test]
    fn test_projection_set() {
        let mut m = new_upper_lip_body_morph();
        ulb_set_projection(&mut m, 0.7);
        assert!((m.projection - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_vermilion_height_set() {
        let mut m = new_upper_lip_body_morph();
        ulb_set_vermilion_height(&mut m, 0.9);
        assert!((m.vermilion_height - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_roll_negative() {
        let mut m = new_upper_lip_body_morph();
        ulb_set_roll(&mut m, -0.5);
        assert!((m.roll - (-0.5)).abs() < 1e-6);
    }

    #[test]
    fn test_roll_clamp() {
        let mut m = new_upper_lip_body_morph();
        ulb_set_roll(&mut m, 3.0);
        assert_eq!(m.roll, 1.0);
    }

    #[test]
    fn test_volume_estimate_positive() {
        let m = new_upper_lip_body_morph();
        assert!(ulb_volume_estimate(&m) > 0.0);
    }

    #[test]
    fn test_json_keys() {
        let m = new_upper_lip_body_morph();
        let s = upper_lip_body_morph_to_json(&m);
        assert!(s.contains("vermilion_height"));
    }

    #[test]
    fn test_clone() {
        let m = new_upper_lip_body_morph();
        let m2 = m.clone();
        assert!((m2.fullness - m.fullness).abs() < 1e-6);
    }
}
