// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Thyroid cartilage shape morph — controls prominence and angle of the thyroid cartilage.

/// Thyroid cartilage morph configuration.
#[derive(Debug, Clone)]
pub struct ThyroidCartilageMorph {
    pub prominence: f32,
    pub angle: f32,
    pub width: f32,
    pub height: f32,
}

impl ThyroidCartilageMorph {
    pub fn new() -> Self {
        Self {
            prominence: 0.3,
            angle: 0.5,
            width: 0.5,
            height: 0.5,
        }
    }
}

impl Default for ThyroidCartilageMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new thyroid cartilage morph.
pub fn new_thyroid_cartilage_morph() -> ThyroidCartilageMorph {
    ThyroidCartilageMorph::new()
}

/// Set thyroid prominence ("Adam's apple").
pub fn thyroid_set_prominence(m: &mut ThyroidCartilageMorph, v: f32) {
    m.prominence = v.clamp(0.0, 1.0);
}

/// Set laminar angle (smaller angle = more prominent).
pub fn thyroid_set_angle(m: &mut ThyroidCartilageMorph, v: f32) {
    m.angle = v.clamp(0.0, 1.0);
}

/// Set cartilage width.
pub fn thyroid_set_width(m: &mut ThyroidCartilageMorph, v: f32) {
    m.width = v.clamp(0.0, 1.0);
}

/// Set cartilage height.
pub fn thyroid_set_height(m: &mut ThyroidCartilageMorph, v: f32) {
    m.height = v.clamp(0.0, 1.0);
}

/// Serialize to JSON-like string.
pub fn thyroid_cartilage_morph_to_json(m: &ThyroidCartilageMorph) -> String {
    format!(
        r#"{{"prominence":{:.4},"angle":{:.4},"width":{:.4},"height":{:.4}}}"#,
        m.prominence, m.angle, m.width, m.height
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_thyroid_cartilage_morph();
        assert!((m.prominence - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_prominence_clamp() {
        let mut m = new_thyroid_cartilage_morph();
        thyroid_set_prominence(&mut m, 2.0);
        assert_eq!(m.prominence, 1.0);
    }

    #[test]
    fn test_set_angle() {
        let mut m = new_thyroid_cartilage_morph();
        thyroid_set_angle(&mut m, 0.2);
        assert!((m.angle - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_set_width() {
        let mut m = new_thyroid_cartilage_morph();
        thyroid_set_width(&mut m, 0.7);
        assert!((m.width - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_height() {
        let mut m = new_thyroid_cartilage_morph();
        thyroid_set_height(&mut m, 0.6);
        assert!((m.height - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_angle_clamp_low() {
        let mut m = new_thyroid_cartilage_morph();
        thyroid_set_angle(&mut m, -1.0);
        assert_eq!(m.angle, 0.0);
    }

    #[test]
    fn test_json_keys() {
        let m = new_thyroid_cartilage_morph();
        let s = thyroid_cartilage_morph_to_json(&m);
        assert!(s.contains("prominence"));
    }

    #[test]
    fn test_clone() {
        let m = new_thyroid_cartilage_morph();
        let m2 = m.clone();
        assert!((m2.angle - m.angle).abs() < 1e-6);
    }

    #[test]
    fn test_width_zero() {
        let mut m = new_thyroid_cartilage_morph();
        thyroid_set_width(&mut m, 0.0);
        assert_eq!(m.width, 0.0);
    }
}
