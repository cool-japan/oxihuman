// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Pharynx shape morph — controls constriction, length, and wall tension.

/// Pharynx morph configuration.
#[derive(Debug, Clone)]
pub struct PharynxMorph {
    pub constriction: f32,
    pub length: f32,
    pub wall_tension: f32,
    pub epiglottis_tilt: f32,
}

impl PharynxMorph {
    pub fn new() -> Self {
        Self {
            constriction: 0.0,
            length: 0.5,
            wall_tension: 0.4,
            epiglottis_tilt: 0.0,
        }
    }
}

impl Default for PharynxMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new pharynx morph.
pub fn new_pharynx_morph() -> PharynxMorph {
    PharynxMorph::new()
}

/// Set pharyngeal constriction (0 = open, 1 = fully constricted).
pub fn pharynx_set_constriction(m: &mut PharynxMorph, v: f32) {
    m.constriction = v.clamp(0.0, 1.0);
}

/// Set pharynx length scaling.
pub fn pharynx_set_length(m: &mut PharynxMorph, v: f32) {
    m.length = v.clamp(0.0, 1.0);
}

/// Set posterior wall tension.
pub fn pharynx_set_wall_tension(m: &mut PharynxMorph, v: f32) {
    m.wall_tension = v.clamp(0.0, 1.0);
}

/// Set epiglottis tilt into pharynx.
pub fn pharynx_set_epiglottis_tilt(m: &mut PharynxMorph, v: f32) {
    m.epiglottis_tilt = v.clamp(0.0, 1.0);
}

/// Effective cross-sectional area (inverse of constriction).
pub fn pharynx_cross_section(m: &PharynxMorph) -> f32 {
    (1.0 - m.constriction).max(0.0)
}

/// Serialize to JSON-like string.
pub fn pharynx_morph_to_json(m: &PharynxMorph) -> String {
    format!(
        r#"{{"constriction":{:.4},"length":{:.4},"wall_tension":{:.4},"epiglottis_tilt":{:.4}}}"#,
        m.constriction, m.length, m.wall_tension, m.epiglottis_tilt
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_pharynx_morph();
        assert_eq!(m.constriction, 0.0);
    }

    #[test]
    fn test_constriction_clamp() {
        let mut m = new_pharynx_morph();
        pharynx_set_constriction(&mut m, 2.0);
        assert_eq!(m.constriction, 1.0);
    }

    #[test]
    fn test_cross_section_open() {
        let m = new_pharynx_morph();
        assert!((pharynx_cross_section(&m) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cross_section_closed() {
        let mut m = new_pharynx_morph();
        pharynx_set_constriction(&mut m, 1.0);
        assert_eq!(pharynx_cross_section(&m), 0.0);
    }

    #[test]
    fn test_length() {
        let mut m = new_pharynx_morph();
        pharynx_set_length(&mut m, 0.7);
        assert!((m.length - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_wall_tension() {
        let mut m = new_pharynx_morph();
        pharynx_set_wall_tension(&mut m, 0.9);
        assert!((m.wall_tension - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_epiglottis_tilt() {
        let mut m = new_pharynx_morph();
        pharynx_set_epiglottis_tilt(&mut m, 0.5);
        assert!((m.epiglottis_tilt - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_json_keys() {
        let m = new_pharynx_morph();
        let s = pharynx_morph_to_json(&m);
        assert!(s.contains("epiglottis_tilt"));
    }

    #[test]
    fn test_clone() {
        let m = new_pharynx_morph();
        let m2 = m.clone();
        assert!((m2.length - m.length).abs() < 1e-6);
    }
}
