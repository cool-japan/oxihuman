// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Uvula shape morph — controls length, width, and elevation.

/// Uvula morph configuration.
#[derive(Debug, Clone)]
pub struct UvulaMorph {
    pub length: f32,
    pub width: f32,
    pub elevation: f32,
    pub tip_bulge: f32,
}

impl UvulaMorph {
    pub fn new() -> Self {
        Self {
            length: 0.5,
            width: 0.5,
            elevation: 0.0,
            tip_bulge: 0.3,
        }
    }
}

impl Default for UvulaMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new uvula morph.
pub fn new_uvula_morph() -> UvulaMorph {
    UvulaMorph::new()
}

/// Set uvula length.
pub fn uvula_set_length(m: &mut UvulaMorph, v: f32) {
    m.length = v.clamp(0.0, 1.0);
}

/// Set uvula width at base.
pub fn uvula_set_width(m: &mut UvulaMorph, v: f32) {
    m.width = v.clamp(0.0, 1.0);
}

/// Set uvula elevation angle (0 = hanging freely).
pub fn uvula_set_elevation(m: &mut UvulaMorph, v: f32) {
    m.elevation = v.clamp(0.0, 1.0);
}

/// Set tip bulge prominence.
pub fn uvula_set_tip_bulge(m: &mut UvulaMorph, v: f32) {
    m.tip_bulge = v.clamp(0.0, 1.0);
}

/// Compute approximate surface area in normalised units.
pub fn uvula_surface_area(m: &UvulaMorph) -> f32 {
    /* simple ellipse approximation */
    std::f32::consts::PI * m.width * 0.5 * m.length
}

/// Serialize to JSON-like string.
pub fn uvula_morph_to_json(m: &UvulaMorph) -> String {
    format!(
        r#"{{"length":{:.4},"width":{:.4},"elevation":{:.4},"tip_bulge":{:.4}}}"#,
        m.length, m.width, m.elevation, m.tip_bulge
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_uvula_morph();
        assert!((m.length - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_length() {
        let mut m = new_uvula_morph();
        uvula_set_length(&mut m, 0.9);
        assert!((m.length - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_length_clamp() {
        let mut m = new_uvula_morph();
        uvula_set_length(&mut m, 2.0);
        assert_eq!(m.length, 1.0);
    }

    #[test]
    fn test_set_width() {
        let mut m = new_uvula_morph();
        uvula_set_width(&mut m, 0.3);
        assert!((m.width - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_set_elevation() {
        let mut m = new_uvula_morph();
        uvula_set_elevation(&mut m, 0.6);
        assert!((m.elevation - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_surface_area_positive() {
        let m = new_uvula_morph();
        assert!(uvula_surface_area(&m) > 0.0);
    }

    #[test]
    fn test_tip_bulge_clamp() {
        let mut m = new_uvula_morph();
        uvula_set_tip_bulge(&mut m, -1.0);
        assert_eq!(m.tip_bulge, 0.0);
    }

    #[test]
    fn test_json_keys() {
        let m = new_uvula_morph();
        let s = uvula_morph_to_json(&m);
        assert!(s.contains("tip_bulge"));
    }

    #[test]
    fn test_clone() {
        let m = new_uvula_morph();
        let m2 = m.clone();
        assert!((m2.elevation - m.elevation).abs() < 1e-6);
    }
}
