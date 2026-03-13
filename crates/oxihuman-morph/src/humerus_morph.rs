// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Humerus length/shape morph — controls upper arm bone proportions.

/// Humerus morph configuration.
#[derive(Debug, Clone)]
pub struct HumerusMorph {
    pub length: f32,
    pub head_size: f32,
    pub shaft_curvature: f32,
    pub epicondyle_width: f32,
}

impl HumerusMorph {
    pub fn new() -> Self {
        Self {
            length: 0.5,
            head_size: 0.5,
            shaft_curvature: 0.0,
            epicondyle_width: 0.5,
        }
    }
}

impl Default for HumerusMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new humerus morph.
pub fn new_humerus_morph() -> HumerusMorph {
    HumerusMorph::new()
}

/// Set humerus length (relative to total arm length).
pub fn hum_set_length(m: &mut HumerusMorph, v: f32) {
    m.length = v.clamp(0.0, 1.0);
}

/// Set humeral head size.
pub fn hum_set_head_size(m: &mut HumerusMorph, v: f32) {
    m.head_size = v.clamp(0.0, 1.0);
}

/// Set shaft anterior curvature (0 = straight, 1 = maximum bow).
pub fn hum_set_shaft_curvature(m: &mut HumerusMorph, v: f32) {
    m.shaft_curvature = v.clamp(0.0, 1.0);
}

/// Set epicondyle width (elbow breadth contribution).
pub fn hum_set_epicondyle_width(m: &mut HumerusMorph, v: f32) {
    m.epicondyle_width = v.clamp(0.0, 1.0);
}

/// Elbow breadth heuristic combining epicondyle and length.
pub fn hum_elbow_breadth(m: &HumerusMorph) -> f32 {
    m.epicondyle_width * (0.8 + m.length * 0.2)
}

/// Serialize to JSON-like string.
pub fn humerus_morph_to_json(m: &HumerusMorph) -> String {
    format!(
        r#"{{"length":{:.4},"head_size":{:.4},"shaft_curvature":{:.4},"epicondyle_width":{:.4}}}"#,
        m.length, m.head_size, m.shaft_curvature, m.epicondyle_width
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_humerus_morph();
        assert!((m.length - 0.5).abs() < 1e-6);
        assert_eq!(m.shaft_curvature, 0.0);
    }

    #[test]
    fn test_length_clamp() {
        let mut m = new_humerus_morph();
        hum_set_length(&mut m, 2.0);
        assert_eq!(m.length, 1.0);
    }

    #[test]
    fn test_head_size_set() {
        let mut m = new_humerus_morph();
        hum_set_head_size(&mut m, 0.8);
        assert!((m.head_size - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_curvature_clamp() {
        let mut m = new_humerus_morph();
        hum_set_shaft_curvature(&mut m, -1.0);
        assert_eq!(m.shaft_curvature, 0.0);
    }

    #[test]
    fn test_epicondyle_set() {
        let mut m = new_humerus_morph();
        hum_set_epicondyle_width(&mut m, 0.7);
        assert!((m.epicondyle_width - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_elbow_breadth_positive() {
        let m = new_humerus_morph();
        assert!(hum_elbow_breadth(&m) > 0.0);
    }

    #[test]
    fn test_elbow_breadth_scales_with_epicondyle() {
        let mut m = new_humerus_morph();
        let b0 = hum_elbow_breadth(&m);
        hum_set_epicondyle_width(&mut m, 1.0);
        let b1 = hum_elbow_breadth(&m);
        assert!(b1 > b0);
    }

    #[test]
    fn test_json_keys() {
        let m = new_humerus_morph();
        let s = humerus_morph_to_json(&m);
        assert!(s.contains("epicondyle_width"));
    }

    #[test]
    fn test_clone() {
        let m = new_humerus_morph();
        let m2 = m.clone();
        assert!((m2.head_size - m.head_size).abs() < 1e-6);
    }
}
