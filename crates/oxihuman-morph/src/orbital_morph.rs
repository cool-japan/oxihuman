// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Orbital (eye socket) shape morph — rim shape, tilt, and depth control.

/// Orbital morph configuration.
#[derive(Debug, Clone)]
pub struct OrbitalMorph {
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    pub tilt: f32,
    pub rim_prominence: f32,
}

impl OrbitalMorph {
    pub fn new() -> Self {
        Self {
            width: 0.5,
            height: 0.5,
            depth: 0.5,
            tilt: 0.0,
            rim_prominence: 0.5,
        }
    }
}

impl Default for OrbitalMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new orbital morph.
pub fn new_orbital_morph() -> OrbitalMorph {
    OrbitalMorph::new()
}

/// Set orbital width.
pub fn orb_set_width(m: &mut OrbitalMorph, v: f32) {
    m.width = v.clamp(0.0, 1.0);
}

/// Set orbital height.
pub fn orb_set_height(m: &mut OrbitalMorph, v: f32) {
    m.height = v.clamp(0.0, 1.0);
}

/// Set orbital depth (how deep the eye socket is).
pub fn orb_set_depth(m: &mut OrbitalMorph, v: f32) {
    m.depth = v.clamp(0.0, 1.0);
}

/// Set orbital tilt (-1 = medial down, 0 = horizontal, 1 = lateral down).
pub fn orb_set_tilt(m: &mut OrbitalMorph, v: f32) {
    m.tilt = v.clamp(-1.0, 1.0);
}

/// Set supraorbital rim prominence.
pub fn orb_set_rim_prominence(m: &mut OrbitalMorph, v: f32) {
    m.rim_prominence = v.clamp(0.0, 1.0);
}

/// Aperture area estimate (ellipse approximation, normalized).
pub fn orb_aperture_area(m: &OrbitalMorph) -> f32 {
    m.width * m.height * std::f32::consts::FRAC_PI_4
}

/// Serialize to JSON-like string.
pub fn orbital_morph_to_json(m: &OrbitalMorph) -> String {
    format!(
        r#"{{"width":{:.4},"height":{:.4},"depth":{:.4},"tilt":{:.4},"rim_prominence":{:.4}}}"#,
        m.width, m.height, m.depth, m.tilt, m.rim_prominence
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_orbital_morph();
        assert!((m.width - 0.5).abs() < 1e-6);
        assert_eq!(m.tilt, 0.0);
    }

    #[test]
    fn test_width_clamp() {
        let mut m = new_orbital_morph();
        orb_set_width(&mut m, 5.0);
        assert_eq!(m.width, 1.0);
    }

    #[test]
    fn test_height_set() {
        let mut m = new_orbital_morph();
        orb_set_height(&mut m, 0.7);
        assert!((m.height - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_depth_set() {
        let mut m = new_orbital_morph();
        orb_set_depth(&mut m, 0.8);
        assert!((m.depth - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_tilt_clamp() {
        let mut m = new_orbital_morph();
        orb_set_tilt(&mut m, 2.0);
        assert_eq!(m.tilt, 1.0);
    }

    #[test]
    fn test_rim_prominence_set() {
        let mut m = new_orbital_morph();
        orb_set_rim_prominence(&mut m, 0.9);
        assert!((m.rim_prominence - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_aperture_area_positive() {
        let m = new_orbital_morph();
        assert!(orb_aperture_area(&m) > 0.0);
    }

    #[test]
    fn test_json_keys() {
        let m = new_orbital_morph();
        let s = orbital_morph_to_json(&m);
        assert!(s.contains("rim_prominence"));
    }

    #[test]
    fn test_clone() {
        let m = new_orbital_morph();
        let m2 = m.clone();
        assert!((m2.depth - m.depth).abs() < 1e-6);
    }
}
