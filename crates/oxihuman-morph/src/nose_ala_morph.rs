// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Nose ala (wing) shape morph — controls lateral alar width, flare, and curvature.

/// Nose ala morph configuration.
#[derive(Debug, Clone)]
pub struct NoseAlaMorph {
    pub width: f32,
    pub flare: f32,
    pub curvature: f32,
    pub thickness: f32,
    pub attachment_height: f32,
}

impl NoseAlaMorph {
    pub fn new() -> Self {
        Self {
            width: 0.5,
            flare: 0.0,
            curvature: 0.5,
            thickness: 0.5,
            attachment_height: 0.5,
        }
    }
}

impl Default for NoseAlaMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new nose ala morph.
pub fn new_nose_ala_morph() -> NoseAlaMorph {
    NoseAlaMorph::new()
}

/// Set alar width (0 = narrow, 1 = wide).
pub fn nala_set_width(m: &mut NoseAlaMorph, v: f32) {
    m.width = v.clamp(0.0, 1.0);
}

/// Set alar flare angle (-1 = inward, 0 = neutral, 1 = outward).
pub fn nala_set_flare(m: &mut NoseAlaMorph, v: f32) {
    m.flare = v.clamp(-1.0, 1.0);
}

/// Set alar curvature (0 = flat, 1 = highly curved).
pub fn nala_set_curvature(m: &mut NoseAlaMorph, v: f32) {
    m.curvature = v.clamp(0.0, 1.0);
}

/// Set alar skin thickness.
pub fn nala_set_thickness(m: &mut NoseAlaMorph, v: f32) {
    m.thickness = v.clamp(0.0, 1.0);
}

/// Estimate projected alar surface area (normalized).
pub fn nala_surface_area(m: &NoseAlaMorph) -> f32 {
    m.width * m.curvature * std::f32::consts::FRAC_PI_4
}

/// Serialize to JSON-like string.
pub fn nose_ala_morph_to_json(m: &NoseAlaMorph) -> String {
    format!(
        r#"{{"width":{:.4},"flare":{:.4},"curvature":{:.4},"thickness":{:.4},"attachment_height":{:.4}}}"#,
        m.width, m.flare, m.curvature, m.thickness, m.attachment_height
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_nose_ala_morph();
        assert!((m.width - 0.5).abs() < 1e-6);
        assert_eq!(m.flare, 0.0);
    }

    #[test]
    fn test_width_clamp_high() {
        let mut m = new_nose_ala_morph();
        nala_set_width(&mut m, 2.0);
        assert_eq!(m.width, 1.0);
    }

    #[test]
    fn test_width_clamp_low() {
        let mut m = new_nose_ala_morph();
        nala_set_width(&mut m, -1.0);
        assert_eq!(m.width, 0.0);
    }

    #[test]
    fn test_flare_clamp() {
        let mut m = new_nose_ala_morph();
        nala_set_flare(&mut m, 3.0);
        assert_eq!(m.flare, 1.0);
    }

    #[test]
    fn test_flare_negative_clamp() {
        let mut m = new_nose_ala_morph();
        nala_set_flare(&mut m, -3.0);
        assert_eq!(m.flare, -1.0);
    }

    #[test]
    fn test_curvature_set() {
        let mut m = new_nose_ala_morph();
        nala_set_curvature(&mut m, 0.8);
        assert!((m.curvature - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_thickness_set() {
        let mut m = new_nose_ala_morph();
        nala_set_thickness(&mut m, 0.3);
        assert!((m.thickness - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_surface_area_positive() {
        let m = new_nose_ala_morph();
        assert!(nala_surface_area(&m) > 0.0);
    }

    #[test]
    fn test_json_contains_keys() {
        let m = new_nose_ala_morph();
        let s = nose_ala_morph_to_json(&m);
        assert!(s.contains("curvature"));
    }

    #[test]
    fn test_clone() {
        let m = new_nose_ala_morph();
        let m2 = m.clone();
        assert!((m2.width - m.width).abs() < 1e-6);
    }
}
