// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Fingernail shape morph stub.

/// Nail shape style.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NailShape {
    Square,
    Oval,
    Round,
    Almond,
    Stiletto,
    Coffin,
}

/// Nail morph controller.
#[derive(Debug, Clone)]
pub struct NailMorph {
    pub shape: NailShape,
    pub length: f32,
    pub thickness: f32,
    pub curvature: f32,
    pub morph_count: usize,
    pub enabled: bool,
}

impl NailMorph {
    pub fn new(morph_count: usize) -> Self {
        NailMorph {
            shape: NailShape::Square,
            length: 0.5,
            thickness: 0.5,
            curvature: 0.3,
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new nail morph controller.
pub fn new_nail_morph(morph_count: usize) -> NailMorph {
    NailMorph::new(morph_count)
}

/// Set nail shape.
pub fn nm_set_shape(morph: &mut NailMorph, shape: NailShape) {
    morph.shape = shape;
}

/// Set nail length.
pub fn nm_set_length(morph: &mut NailMorph, length: f32) {
    morph.length = length.clamp(0.0, 1.0);
}

/// Set nail thickness.
pub fn nm_set_thickness(morph: &mut NailMorph, thickness: f32) {
    morph.thickness = thickness.clamp(0.0, 1.0);
}

/// Set nail curvature.
pub fn nm_set_curvature(morph: &mut NailMorph, curvature: f32) {
    morph.curvature = curvature.clamp(0.0, 1.0);
}

/// Evaluate morph weights (stub: length-driven).
pub fn nm_evaluate(morph: &NailMorph) -> Vec<f32> {
    /* Stub: weight driven by length and curvature */
    if !morph.enabled || morph.morph_count == 0 {
        return vec![];
    }
    let w = (morph.length + morph.curvature * 0.5) / 1.5;
    vec![w.clamp(0.0, 1.0); morph.morph_count]
}

/// Enable or disable.
pub fn nm_set_enabled(morph: &mut NailMorph, enabled: bool) {
    morph.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn nm_to_json(morph: &NailMorph) -> String {
    let shape = match morph.shape {
        NailShape::Square => "square",
        NailShape::Oval => "oval",
        NailShape::Round => "round",
        NailShape::Almond => "almond",
        NailShape::Stiletto => "stiletto",
        NailShape::Coffin => "coffin",
    };
    format!(
        r#"{{"shape":"{}","length":{},"thickness":{},"curvature":{},"enabled":{}}}"#,
        shape, morph.length, morph.thickness, morph.curvature, morph.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_shape() {
        let m = new_nail_morph(4);
        assert_eq!(
            m.shape,
            NailShape::Square /* default shape must be Square */
        );
    }

    #[test]
    fn test_set_shape() {
        let mut m = new_nail_morph(4);
        nm_set_shape(&mut m, NailShape::Almond);
        assert_eq!(m.shape, NailShape::Almond /* shape must be set */);
    }

    #[test]
    fn test_length_clamped() {
        let mut m = new_nail_morph(4);
        nm_set_length(&mut m, 2.0);
        assert!((m.length - 1.0).abs() < 1e-6 /* length clamped to 1.0 */);
    }

    #[test]
    fn test_thickness_clamped() {
        let mut m = new_nail_morph(4);
        nm_set_thickness(&mut m, -1.0);
        assert!((m.thickness).abs() < 1e-6 /* thickness clamped to 0.0 */);
    }

    #[test]
    fn test_curvature_clamped() {
        let mut m = new_nail_morph(4);
        nm_set_curvature(&mut m, 5.0);
        assert!((m.curvature - 1.0).abs() < 1e-6 /* curvature clamped to 1.0 */);
    }

    #[test]
    fn test_evaluate_length() {
        let m = new_nail_morph(6);
        assert_eq!(
            nm_evaluate(&m).len(),
            6 /* output must match morph_count */
        );
    }

    #[test]
    fn test_evaluate_disabled() {
        let mut m = new_nail_morph(4);
        nm_set_enabled(&mut m, false);
        assert!(nm_evaluate(&m).is_empty() /* disabled must return empty */);
    }

    #[test]
    fn test_to_json_has_shape() {
        let m = new_nail_morph(4);
        let j = nm_to_json(&m);
        assert!(j.contains("\"shape\"") /* JSON must have shape */);
    }

    #[test]
    fn test_enabled_default() {
        let m = new_nail_morph(4);
        assert!(m.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_evaluate_in_range() {
        let m = new_nail_morph(2);
        let out = nm_evaluate(&m);
        assert!(out[0] >= 0.0 && out[0] <= 1.0 /* evaluated weight must be in [0, 1] */);
    }
}
