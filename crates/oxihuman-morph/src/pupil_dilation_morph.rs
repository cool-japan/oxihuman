// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Pupil size/dilation morph control — models light response and emotional state.

/// Pupil dilation morph configuration.
#[derive(Debug, Clone)]
pub struct PupilDilationMorph {
    pub dilation: f32,
    pub left_dilation: f32,
    pub right_dilation: f32,
    pub light_response: f32,
}

impl PupilDilationMorph {
    pub fn new() -> Self {
        Self {
            dilation: 0.5,
            left_dilation: 0.5,
            right_dilation: 0.5,
            light_response: 0.5,
        }
    }
}

impl Default for PupilDilationMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new pupil dilation morph.
pub fn new_pupil_dilation_morph() -> PupilDilationMorph {
    PupilDilationMorph::new()
}

/// Set symmetric pupil dilation.
pub fn pupil_set_dilation(morph: &mut PupilDilationMorph, dilation: f32) {
    let d = dilation.clamp(0.0, 1.0);
    morph.dilation = d;
    morph.left_dilation = d;
    morph.right_dilation = d;
}

/// Set left pupil dilation independently.
pub fn pupil_set_left(morph: &mut PupilDilationMorph, dilation: f32) {
    morph.left_dilation = dilation.clamp(0.0, 1.0);
}

/// Set right pupil dilation independently.
pub fn pupil_set_right(morph: &mut PupilDilationMorph, dilation: f32) {
    morph.right_dilation = dilation.clamp(0.0, 1.0);
}

/// Apply simulated light-response: bright light → pupil contracts.
pub fn pupil_apply_light_response(morph: &mut PupilDilationMorph, luminance: f32) {
    let lum = luminance.clamp(0.0, 1.0);
    let response = 1.0 - lum * morph.light_response;
    let d = response.clamp(0.0, 1.0);
    morph.left_dilation = d;
    morph.right_dilation = d;
    morph.dilation = d;
}

/// Serialize to JSON-like string.
pub fn pupil_dilation_morph_to_json(morph: &PupilDilationMorph) -> String {
    format!(
        r#"{{"dilation":{:.4},"left_dilation":{:.4},"right_dilation":{:.4},"light_response":{:.4}}}"#,
        morph.dilation, morph.left_dilation, morph.right_dilation, morph.light_response
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_pupil_dilation_morph();
        assert!((m.dilation - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_dilation_symmetric() {
        let mut m = new_pupil_dilation_morph();
        pupil_set_dilation(&mut m, 0.8);
        assert!((m.left_dilation - 0.8).abs() < 1e-6);
        assert!((m.right_dilation - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_left_independent() {
        let mut m = new_pupil_dilation_morph();
        pupil_set_left(&mut m, 0.9);
        assert!((m.left_dilation - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_set_right_independent() {
        let mut m = new_pupil_dilation_morph();
        pupil_set_right(&mut m, 0.2);
        assert!((m.right_dilation - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_light_response_bright() {
        let mut m = new_pupil_dilation_morph();
        pupil_apply_light_response(&mut m, 1.0); /* full bright */
        assert!(m.dilation < 0.6); /* contracted */
    }

    #[test]
    fn test_light_response_dark() {
        let mut m = new_pupil_dilation_morph();
        pupil_apply_light_response(&mut m, 0.0); /* full dark */
        assert!((m.dilation - 1.0).abs() < 1e-6); /* dilated */
    }

    #[test]
    fn test_json() {
        let m = new_pupil_dilation_morph();
        let s = pupil_dilation_morph_to_json(&m);
        assert!(s.contains("light_response"));
    }

    #[test]
    fn test_dilation_clamp() {
        let mut m = new_pupil_dilation_morph();
        pupil_set_dilation(&mut m, 3.0);
        assert_eq!(m.dilation, 1.0);
    }

    #[test]
    fn test_clone() {
        let m = new_pupil_dilation_morph();
        let m2 = m.clone();
        assert!((m2.light_response - m.light_response).abs() < 1e-6);
    }
}
