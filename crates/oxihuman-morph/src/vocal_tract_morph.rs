// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Vocal tract shape morph — controls overall tract length, lip rounding, and constriction profile.

/// Vocal tract morph configuration.
#[derive(Debug, Clone)]
pub struct VocalTractMorph {
    pub tract_length: f32,
    pub lip_rounding: f32,
    pub mid_constriction: f32,
    pub back_constriction: f32,
}

impl VocalTractMorph {
    pub fn new() -> Self {
        Self {
            tract_length: 0.5,
            lip_rounding: 0.0,
            mid_constriction: 0.0,
            back_constriction: 0.0,
        }
    }
}

impl Default for VocalTractMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new vocal tract morph.
pub fn new_vocal_tract_morph() -> VocalTractMorph {
    VocalTractMorph::new()
}

/// Set normalised tract length.
pub fn vocal_tract_set_length(m: &mut VocalTractMorph, v: f32) {
    m.tract_length = v.clamp(0.0, 1.0);
}

/// Set lip rounding (0 = spread, 1 = rounded).
pub fn vocal_tract_set_lip_rounding(m: &mut VocalTractMorph, v: f32) {
    m.lip_rounding = v.clamp(0.0, 1.0);
}

/// Set mid-tract constriction.
pub fn vocal_tract_set_mid_constriction(m: &mut VocalTractMorph, v: f32) {
    m.mid_constriction = v.clamp(0.0, 1.0);
}

/// Set back-tract constriction.
pub fn vocal_tract_set_back_constriction(m: &mut VocalTractMorph, v: f32) {
    m.back_constriction = v.clamp(0.0, 1.0);
}

/// Average constriction across mid and back regions.
pub fn vocal_tract_mean_constriction(m: &VocalTractMorph) -> f32 {
    (m.mid_constriction + m.back_constriction) * 0.5
}

/// Serialize to JSON-like string.
pub fn vocal_tract_morph_to_json(m: &VocalTractMorph) -> String {
    format!(
        r#"{{"tract_length":{:.4},"lip_rounding":{:.4},"mid_constriction":{:.4},"back_constriction":{:.4}}}"#,
        m.tract_length, m.lip_rounding, m.mid_constriction, m.back_constriction
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_vocal_tract_morph();
        assert_eq!(m.lip_rounding, 0.0);
    }

    #[test]
    fn test_length() {
        let mut m = new_vocal_tract_morph();
        vocal_tract_set_length(&mut m, 0.8);
        assert!((m.tract_length - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_lip_rounding() {
        let mut m = new_vocal_tract_morph();
        vocal_tract_set_lip_rounding(&mut m, 0.9);
        assert!((m.lip_rounding - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_mid_constriction_clamp() {
        let mut m = new_vocal_tract_morph();
        vocal_tract_set_mid_constriction(&mut m, 5.0);
        assert_eq!(m.mid_constriction, 1.0);
    }

    #[test]
    fn test_back_constriction() {
        let mut m = new_vocal_tract_morph();
        vocal_tract_set_back_constriction(&mut m, 0.4);
        assert!((m.back_constriction - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_mean_constriction_zero() {
        let m = new_vocal_tract_morph();
        assert_eq!(vocal_tract_mean_constriction(&m), 0.0);
    }

    #[test]
    fn test_mean_constriction_value() {
        let mut m = new_vocal_tract_morph();
        vocal_tract_set_mid_constriction(&mut m, 0.4);
        vocal_tract_set_back_constriction(&mut m, 0.8);
        let mean = vocal_tract_mean_constriction(&m);
        assert!((mean - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_json_keys() {
        let m = new_vocal_tract_morph();
        let s = vocal_tract_morph_to_json(&m);
        assert!(s.contains("back_constriction"));
    }

    #[test]
    fn test_clone() {
        let m = new_vocal_tract_morph();
        let m2 = m.clone();
        assert!((m2.tract_length - m.tract_length).abs() < 1e-6);
    }
}
