// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Radius/ulna proportions morph — forearm bone ratio and curvature control.

/// Radius/ulna morph configuration.
#[derive(Debug, Clone)]
pub struct RadiusUlnaMorph {
    pub length: f32,
    pub radius_ratio: f32,
    pub bowing: f32,
    pub styloid_prominence: f32,
}

impl RadiusUlnaMorph {
    pub fn new() -> Self {
        Self {
            length: 0.5,
            radius_ratio: 0.5,
            bowing: 0.0,
            styloid_prominence: 0.5,
        }
    }
}

impl Default for RadiusUlnaMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new radius/ulna morph.
pub fn new_radius_ulna_morph() -> RadiusUlnaMorph {
    RadiusUlnaMorph::new()
}

/// Set total forearm length.
pub fn ru_set_length(m: &mut RadiusUlnaMorph, v: f32) {
    m.length = v.clamp(0.0, 1.0);
}

/// Set radius-to-ulna width ratio (0 = thin radius, 1 = thick radius).
pub fn ru_set_radius_ratio(m: &mut RadiusUlnaMorph, v: f32) {
    m.radius_ratio = v.clamp(0.0, 1.0);
}

/// Set lateral bowing (0 = straight, 1 = maximum bow).
pub fn ru_set_bowing(m: &mut RadiusUlnaMorph, v: f32) {
    m.bowing = v.clamp(0.0, 1.0);
}

/// Set styloid process prominence.
pub fn ru_set_styloid_prominence(m: &mut RadiusUlnaMorph, v: f32) {
    m.styloid_prominence = v.clamp(0.0, 1.0);
}

/// Effective wrist width estimate.
pub fn ru_wrist_width(m: &RadiusUlnaMorph) -> f32 {
    (0.4 + m.radius_ratio * 0.3 + m.styloid_prominence * 0.3).clamp(0.0, 1.0)
}

/// Serialize to JSON-like string.
pub fn radius_ulna_morph_to_json(m: &RadiusUlnaMorph) -> String {
    format!(
        r#"{{"length":{:.4},"radius_ratio":{:.4},"bowing":{:.4},"styloid_prominence":{:.4}}}"#,
        m.length, m.radius_ratio, m.bowing, m.styloid_prominence
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_radius_ulna_morph();
        assert_eq!(m.bowing, 0.0);
        assert!((m.radius_ratio - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_length_clamp() {
        let mut m = new_radius_ulna_morph();
        ru_set_length(&mut m, 3.0);
        assert_eq!(m.length, 1.0);
    }

    #[test]
    fn test_ratio_set() {
        let mut m = new_radius_ulna_morph();
        ru_set_radius_ratio(&mut m, 0.8);
        assert!((m.radius_ratio - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_bowing_clamp() {
        let mut m = new_radius_ulna_morph();
        ru_set_bowing(&mut m, -1.0);
        assert_eq!(m.bowing, 0.0);
    }

    #[test]
    fn test_styloid_set() {
        let mut m = new_radius_ulna_morph();
        ru_set_styloid_prominence(&mut m, 0.9);
        assert!((m.styloid_prominence - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_wrist_width_range() {
        let m = new_radius_ulna_morph();
        let w = ru_wrist_width(&m);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_wrist_width_increases() {
        let mut m = new_radius_ulna_morph();
        let w0 = ru_wrist_width(&m);
        ru_set_styloid_prominence(&mut m, 1.0);
        let w1 = ru_wrist_width(&m);
        assert!(w1 > w0);
    }

    #[test]
    fn test_json_keys() {
        let m = new_radius_ulna_morph();
        let s = radius_ulna_morph_to_json(&m);
        assert!(s.contains("styloid_prominence"));
    }

    #[test]
    fn test_clone() {
        let m = new_radius_ulna_morph();
        let m2 = m.clone();
        assert!((m2.length - m.length).abs() < 1e-6);
    }
}
