// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Nasal septum deviation morph — controls septal deflection and columellar angulation.

/// Nasal septum morph configuration.
#[derive(Debug, Clone)]
pub struct NasalSeptumMorph {
    pub deviation: f32,
    pub thickness: f32,
    pub caudal_angle: f32,
    pub dorsal_height: f32,
    pub perforation: f32,
}

impl NasalSeptumMorph {
    pub fn new() -> Self {
        Self {
            deviation: 0.0,
            thickness: 0.5,
            caudal_angle: 0.0,
            dorsal_height: 0.5,
            perforation: 0.0,
        }
    }
}

impl Default for NasalSeptumMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new nasal septum morph.
pub fn new_nasal_septum_morph() -> NasalSeptumMorph {
    NasalSeptumMorph::new()
}

/// Set deviation angle (-1 = left, 0 = straight, 1 = right).
pub fn nsept_set_deviation(m: &mut NasalSeptumMorph, v: f32) {
    m.deviation = v.clamp(-1.0, 1.0);
}

/// Set septal thickness.
pub fn nsept_set_thickness(m: &mut NasalSeptumMorph, v: f32) {
    m.thickness = v.clamp(0.0, 1.0);
}

/// Set caudal septal angle (columellar-septal relationship).
pub fn nsept_set_caudal_angle(m: &mut NasalSeptumMorph, v: f32) {
    m.caudal_angle = v.clamp(-1.0, 1.0);
}

/// Set dorsal septal height (contributes to nasal dorsum profile).
pub fn nsept_set_dorsal_height(m: &mut NasalSeptumMorph, v: f32) {
    m.dorsal_height = v.clamp(0.0, 1.0);
}

/// Compute absolute deviation magnitude.
pub fn nsept_deviation_magnitude(m: &NasalSeptumMorph) -> f32 {
    m.deviation.abs()
}

/// Serialize to JSON-like string.
pub fn nasal_septum_morph_to_json(m: &NasalSeptumMorph) -> String {
    format!(
        r#"{{"deviation":{:.4},"thickness":{:.4},"caudal_angle":{:.4},"dorsal_height":{:.4},"perforation":{:.4}}}"#,
        m.deviation, m.thickness, m.caudal_angle, m.dorsal_height, m.perforation
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_nasal_septum_morph();
        assert_eq!(m.deviation, 0.0);
        assert!((m.thickness - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_deviation_clamp_positive() {
        let mut m = new_nasal_septum_morph();
        nsept_set_deviation(&mut m, 3.0);
        assert_eq!(m.deviation, 1.0);
    }

    #[test]
    fn test_deviation_clamp_negative() {
        let mut m = new_nasal_septum_morph();
        nsept_set_deviation(&mut m, -3.0);
        assert_eq!(m.deviation, -1.0);
    }

    #[test]
    fn test_thickness_set() {
        let mut m = new_nasal_septum_morph();
        nsept_set_thickness(&mut m, 0.6);
        assert!((m.thickness - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_caudal_angle_set() {
        let mut m = new_nasal_septum_morph();
        nsept_set_caudal_angle(&mut m, 0.4);
        assert!((m.caudal_angle - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_dorsal_height_clamp() {
        let mut m = new_nasal_septum_morph();
        nsept_set_dorsal_height(&mut m, 2.0);
        assert_eq!(m.dorsal_height, 1.0);
    }

    #[test]
    fn test_deviation_magnitude_zero() {
        let m = new_nasal_septum_morph();
        assert_eq!(nsept_deviation_magnitude(&m), 0.0);
    }

    #[test]
    fn test_deviation_magnitude_negative() {
        let mut m = new_nasal_septum_morph();
        nsept_set_deviation(&mut m, -0.6);
        assert!((nsept_deviation_magnitude(&m) - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_json_keys() {
        let m = new_nasal_septum_morph();
        let s = nasal_septum_morph_to_json(&m);
        assert!(s.contains("dorsal_height"));
    }

    #[test]
    fn test_clone() {
        let m = new_nasal_septum_morph();
        let m2 = m.clone();
        assert_eq!(m2.deviation, m.deviation);
    }
}
