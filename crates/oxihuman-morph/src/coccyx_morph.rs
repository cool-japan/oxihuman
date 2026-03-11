// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Coccyx shape morph — adjusts tailbone angle and segment count influence.

/// Coccyx morph configuration.
#[derive(Debug, Clone)]
pub struct CoccyxMorph {
    pub flexion: f32,
    pub length: f32,
    pub deviation: f32,
    pub prominence: f32,
}

impl CoccyxMorph {
    pub fn new() -> Self {
        Self {
            flexion: 0.5,
            length: 0.5,
            deviation: 0.0,
            prominence: 0.3,
        }
    }
}

impl Default for CoccyxMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new coccyx morph.
pub fn new_coccyx_morph() -> CoccyxMorph {
    CoccyxMorph::new()
}

/// Set coccyx flexion (0 = extended, 1 = fully flexed anteriorly).
pub fn coc_set_flexion(m: &mut CoccyxMorph, v: f32) {
    m.flexion = v.clamp(0.0, 1.0);
}

/// Set coccyx length relative to sacrum.
pub fn coc_set_length(m: &mut CoccyxMorph, v: f32) {
    m.length = v.clamp(0.0, 1.0);
}

/// Set lateral deviation (-1 = left, 0 = center, 1 = right).
pub fn coc_set_deviation(m: &mut CoccyxMorph, v: f32) {
    m.deviation = v.clamp(-1.0, 1.0);
}

/// Set dorsal prominence.
pub fn coc_set_prominence(m: &mut CoccyxMorph, v: f32) {
    m.prominence = v.clamp(0.0, 1.0);
}

/// Effective tip displacement from neutral (combines length and flexion).
pub fn coc_tip_displacement(m: &CoccyxMorph) -> f32 {
    m.length * (0.5 + m.flexion * 0.5)
}

/// Serialize to JSON-like string.
pub fn coccyx_morph_to_json(m: &CoccyxMorph) -> String {
    format!(
        r#"{{"flexion":{:.4},"length":{:.4},"deviation":{:.4},"prominence":{:.4}}}"#,
        m.flexion, m.length, m.deviation, m.prominence
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_coccyx_morph();
        assert!((m.flexion - 0.5).abs() < 1e-6);
        assert_eq!(m.deviation, 0.0);
    }

    #[test]
    fn test_flexion_clamp() {
        let mut m = new_coccyx_morph();
        coc_set_flexion(&mut m, 5.0);
        assert_eq!(m.flexion, 1.0);
    }

    #[test]
    fn test_length_set() {
        let mut m = new_coccyx_morph();
        coc_set_length(&mut m, 0.8);
        assert!((m.length - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_deviation_clamp() {
        let mut m = new_coccyx_morph();
        coc_set_deviation(&mut m, -3.0);
        assert_eq!(m.deviation, -1.0);
    }

    #[test]
    fn test_prominence_clamp() {
        let mut m = new_coccyx_morph();
        coc_set_prominence(&mut m, 2.0);
        assert_eq!(m.prominence, 1.0);
    }

    #[test]
    fn test_tip_displacement_positive() {
        let m = new_coccyx_morph();
        assert!(coc_tip_displacement(&m) > 0.0);
    }

    #[test]
    fn test_tip_displacement_max() {
        let mut m = new_coccyx_morph();
        coc_set_length(&mut m, 1.0);
        coc_set_flexion(&mut m, 1.0);
        assert!((coc_tip_displacement(&m) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_json_keys() {
        let m = new_coccyx_morph();
        let s = coccyx_morph_to_json(&m);
        assert!(s.contains("deviation"));
    }

    #[test]
    fn test_clone() {
        let m = new_coccyx_morph();
        let m2 = m.clone();
        assert!((m2.prominence - m.prominence).abs() < 1e-6);
    }
}
