// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Iris diameter morph control.

/// Iris size morph configuration.
#[derive(Debug, Clone)]
pub struct IrisSizeMorph {
    pub diameter: f32,
    pub left_diameter: f32,
    pub right_diameter: f32,
    pub anisocoria: f32,
}

impl IrisSizeMorph {
    pub fn new() -> Self {
        Self {
            diameter: 0.5,
            left_diameter: 0.5,
            right_diameter: 0.5,
            anisocoria: 0.0,
        }
    }
}

impl Default for IrisSizeMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new iris size morph.
pub fn new_iris_size_morph() -> IrisSizeMorph {
    IrisSizeMorph::new()
}

/// Set symmetric iris diameter in normalized range.
pub fn iris_size_set_diameter(morph: &mut IrisSizeMorph, diameter: f32) {
    let d = diameter.clamp(0.0, 1.0);
    morph.diameter = d;
    morph.left_diameter = d;
    morph.right_diameter = d;
    morph.anisocoria = 0.0;
}

/// Set left-eye iris diameter independently.
pub fn iris_size_set_left(morph: &mut IrisSizeMorph, diameter: f32) {
    morph.left_diameter = diameter.clamp(0.0, 1.0);
    morph.anisocoria = (morph.left_diameter - morph.right_diameter).abs();
}

/// Set right-eye iris diameter independently.
pub fn iris_size_set_right(morph: &mut IrisSizeMorph, diameter: f32) {
    morph.right_diameter = diameter.clamp(0.0, 1.0);
    morph.anisocoria = (morph.left_diameter - morph.right_diameter).abs();
}

/// Compute mean iris diameter.
pub fn iris_size_mean(morph: &IrisSizeMorph) -> f32 {
    (morph.left_diameter + morph.right_diameter) / 2.0
}

/// Serialize to JSON-like string.
pub fn iris_size_morph_to_json(morph: &IrisSizeMorph) -> String {
    format!(
        r#"{{"diameter":{:.4},"left_diameter":{:.4},"right_diameter":{:.4},"anisocoria":{:.4}}}"#,
        morph.diameter, morph.left_diameter, morph.right_diameter, morph.anisocoria
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_iris_size_morph();
        assert!((m.diameter - 0.5).abs() < 1e-6);
        assert_eq!(m.anisocoria, 0.0);
    }

    #[test]
    fn test_set_diameter_symmetric() {
        let mut m = new_iris_size_morph();
        iris_size_set_diameter(&mut m, 0.7);
        assert!((m.left_diameter - 0.7).abs() < 1e-6);
        assert!((m.right_diameter - 0.7).abs() < 1e-6);
        assert_eq!(m.anisocoria, 0.0);
    }

    #[test]
    fn test_set_left() {
        let mut m = new_iris_size_morph();
        iris_size_set_left(&mut m, 0.8);
        assert!((m.left_diameter - 0.8).abs() < 1e-6);
        assert!(m.anisocoria > 0.0);
    }

    #[test]
    fn test_set_right() {
        let mut m = new_iris_size_morph();
        iris_size_set_right(&mut m, 0.3);
        assert!((m.right_diameter - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_mean_diameter() {
        let mut m = new_iris_size_morph();
        iris_size_set_left(&mut m, 0.6);
        iris_size_set_right(&mut m, 0.4);
        let mean = iris_size_mean(&m);
        assert!((mean - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_json() {
        let m = new_iris_size_morph();
        let s = iris_size_morph_to_json(&m);
        assert!(s.contains("anisocoria"));
    }

    #[test]
    fn test_clone() {
        let m = new_iris_size_morph();
        let m2 = m.clone();
        assert!((m2.diameter - m.diameter).abs() < 1e-6);
    }

    #[test]
    fn test_diameter_clamp() {
        let mut m = new_iris_size_morph();
        iris_size_set_diameter(&mut m, 5.0);
        assert_eq!(m.diameter, 1.0);
    }

    #[test]
    fn test_default_trait() {
        let m: IrisSizeMorph = Default::default();
        assert!((m.right_diameter - 0.5).abs() < 1e-6);
    }
}
