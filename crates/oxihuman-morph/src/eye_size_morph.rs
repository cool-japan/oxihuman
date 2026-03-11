// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Eye aperture/size morph stub.

/// Which eye to target.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EyeSide {
    Left,
    Right,
    Both,
}

/// Eye size morph controller.
#[derive(Debug, Clone)]
pub struct EyeSizeMorph {
    pub side: EyeSide,
    pub aperture: f32,
    pub width: f32,
    pub height: f32,
    pub tilt: f32,
    pub morph_count: usize,
    pub enabled: bool,
}

impl EyeSizeMorph {
    pub fn new(morph_count: usize) -> Self {
        EyeSizeMorph {
            side: EyeSide::Both,
            aperture: 0.5,
            width: 0.5,
            height: 0.5,
            tilt: 0.0,
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new eye size morph controller.
pub fn new_eye_size_morph(morph_count: usize) -> EyeSizeMorph {
    EyeSizeMorph::new(morph_count)
}

/// Set target side.
pub fn esm_set_side(morph: &mut EyeSizeMorph, side: EyeSide) {
    morph.side = side;
}

/// Set eye aperture (openness).
pub fn esm_set_aperture(morph: &mut EyeSizeMorph, aperture: f32) {
    morph.aperture = aperture.clamp(0.0, 1.0);
}

/// Set eye width.
pub fn esm_set_width(morph: &mut EyeSizeMorph, width: f32) {
    morph.width = width.clamp(0.0, 1.0);
}

/// Set eye height.
pub fn esm_set_height(morph: &mut EyeSizeMorph, height: f32) {
    morph.height = height.clamp(0.0, 1.0);
}

/// Set eye tilt angle (normalized).
pub fn esm_set_tilt(morph: &mut EyeSizeMorph, tilt: f32) {
    morph.tilt = tilt.clamp(-1.0, 1.0);
}

/// Evaluate morph weights (stub: aperture × size average).
pub fn esm_evaluate(morph: &EyeSizeMorph) -> Vec<f32> {
    /* Stub: combined aperture/width/height average */
    if !morph.enabled || morph.morph_count == 0 {
        return vec![];
    }
    let w = (morph.aperture + morph.width + morph.height) / 3.0;
    vec![w.clamp(0.0, 1.0); morph.morph_count]
}

/// Enable or disable.
pub fn esm_set_enabled(morph: &mut EyeSizeMorph, enabled: bool) {
    morph.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn esm_to_json(morph: &EyeSizeMorph) -> String {
    let side = match morph.side {
        EyeSide::Left => "left",
        EyeSide::Right => "right",
        EyeSide::Both => "both",
    };
    format!(
        r#"{{"side":"{}","aperture":{},"width":{},"height":{},"tilt":{},"enabled":{}}}"#,
        side, morph.aperture, morph.width, morph.height, morph.tilt, morph.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_side() {
        let m = new_eye_size_morph(4);
        assert_eq!(m.side, EyeSide::Both /* default side must be Both */);
    }

    #[test]
    fn test_set_side() {
        let mut m = new_eye_size_morph(4);
        esm_set_side(&mut m, EyeSide::Left);
        assert_eq!(m.side, EyeSide::Left /* side must be set */);
    }

    #[test]
    fn test_aperture_clamped() {
        let mut m = new_eye_size_morph(4);
        esm_set_aperture(&mut m, 2.0);
        assert!((m.aperture - 1.0).abs() < 1e-6 /* aperture clamped to 1.0 */);
    }

    #[test]
    fn test_width_clamped() {
        let mut m = new_eye_size_morph(4);
        esm_set_width(&mut m, -0.5);
        assert!((m.width).abs() < 1e-6 /* width clamped to 0.0 */);
    }

    #[test]
    fn test_height_clamped() {
        let mut m = new_eye_size_morph(4);
        esm_set_height(&mut m, 1.5);
        assert!((m.height - 1.0).abs() < 1e-6 /* height clamped to 1.0 */);
    }

    #[test]
    fn test_tilt_clamped() {
        let mut m = new_eye_size_morph(4);
        esm_set_tilt(&mut m, 3.0);
        assert!((m.tilt - 1.0).abs() < 1e-6 /* tilt clamped to 1.0 */);
    }

    #[test]
    fn test_evaluate_length() {
        let m = new_eye_size_morph(5);
        assert_eq!(
            esm_evaluate(&m).len(),
            5 /* output must match morph_count */
        );
    }

    #[test]
    fn test_evaluate_disabled() {
        let mut m = new_eye_size_morph(4);
        esm_set_enabled(&mut m, false);
        assert!(esm_evaluate(&m).is_empty() /* disabled must return empty */);
    }

    #[test]
    fn test_to_json_has_side() {
        let m = new_eye_size_morph(4);
        let j = esm_to_json(&m);
        assert!(j.contains("\"side\"") /* JSON must have side */);
    }

    #[test]
    fn test_enabled_default() {
        let m = new_eye_size_morph(4);
        assert!(m.enabled /* must be enabled by default */);
    }
}
