// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Skull shape/proportion morph — controls cranial vault dimensions and proportions.

/// Skull morph configuration.
#[derive(Debug, Clone)]
pub struct SkullMorph {
    pub cranial_length: f32,
    pub cranial_width: f32,
    pub cranial_height: f32,
    pub frontal_slope: f32,
    pub occipital_projection: f32,
}

impl SkullMorph {
    pub fn new() -> Self {
        Self {
            cranial_length: 0.5,
            cranial_width: 0.5,
            cranial_height: 0.5,
            frontal_slope: 0.5,
            occipital_projection: 0.5,
        }
    }
}

impl Default for SkullMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new skull morph.
pub fn new_skull_morph() -> SkullMorph {
    SkullMorph::new()
}

/// Set cranial length (anterior-posterior).
pub fn skl_set_cranial_length(m: &mut SkullMorph, v: f32) {
    m.cranial_length = v.clamp(0.0, 1.0);
}

/// Set cranial width (bizygomatic).
pub fn skl_set_cranial_width(m: &mut SkullMorph, v: f32) {
    m.cranial_width = v.clamp(0.0, 1.0);
}

/// Set cranial height (basion-bregma).
pub fn skl_set_cranial_height(m: &mut SkullMorph, v: f32) {
    m.cranial_height = v.clamp(0.0, 1.0);
}

/// Set frontal bone slope (0 = receding forehead, 1 = vertical).
pub fn skl_set_frontal_slope(m: &mut SkullMorph, v: f32) {
    m.frontal_slope = v.clamp(0.0, 1.0);
}

/// Set occipital bone projection.
pub fn skl_set_occipital_projection(m: &mut SkullMorph, v: f32) {
    m.occipital_projection = v.clamp(0.0, 1.0);
}

/// Cranial index (width/length ratio * 100, normalized to 0-1 here).
pub fn skl_cranial_index(m: &SkullMorph) -> f32 {
    if m.cranial_length < 1e-6 {
        return 0.0;
    }
    (m.cranial_width / m.cranial_length).clamp(0.0, 2.0) / 2.0
}

/// Serialize to JSON-like string.
pub fn skull_morph_to_json(m: &SkullMorph) -> String {
    format!(
        r#"{{"cranial_length":{:.4},"cranial_width":{:.4},"cranial_height":{:.4},"frontal_slope":{:.4},"occipital_projection":{:.4}}}"#,
        m.cranial_length,
        m.cranial_width,
        m.cranial_height,
        m.frontal_slope,
        m.occipital_projection
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_skull_morph();
        assert!((m.cranial_length - 0.5).abs() < 1e-6);
        assert!((m.cranial_width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_length_clamp() {
        let mut m = new_skull_morph();
        skl_set_cranial_length(&mut m, 3.0);
        assert_eq!(m.cranial_length, 1.0);
    }

    #[test]
    fn test_width_set() {
        let mut m = new_skull_morph();
        skl_set_cranial_width(&mut m, 0.7);
        assert!((m.cranial_width - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_height_set() {
        let mut m = new_skull_morph();
        skl_set_cranial_height(&mut m, 0.6);
        assert!((m.cranial_height - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_frontal_slope_clamp() {
        let mut m = new_skull_morph();
        skl_set_frontal_slope(&mut m, -1.0);
        assert_eq!(m.frontal_slope, 0.0);
    }

    #[test]
    fn test_occipital_projection_set() {
        let mut m = new_skull_morph();
        skl_set_occipital_projection(&mut m, 0.9);
        assert!((m.occipital_projection - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_cranial_index_neutral() {
        let m = new_skull_morph(); /* width == length == 0.5 => index = 0.5 */
        let ci = skl_cranial_index(&m);
        assert!((ci - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_json_keys() {
        let m = new_skull_morph();
        let s = skull_morph_to_json(&m);
        assert!(s.contains("occipital_projection"));
    }

    #[test]
    fn test_clone() {
        let m = new_skull_morph();
        let m2 = m.clone();
        assert!((m2.frontal_slope - m.frontal_slope).abs() < 1e-6);
    }
}
