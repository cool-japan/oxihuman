// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Diaphragm position morph — controls dome height, descent, and excursion range.

/// Diaphragm morph configuration.
#[derive(Debug, Clone)]
pub struct DiaphragmMorph {
    pub dome_height: f32,
    pub descent: f32,
    pub excursion_range: f32,
    pub contraction: f32,
}

impl DiaphragmMorph {
    pub fn new() -> Self {
        Self {
            dome_height: 0.5,
            descent: 0.0,
            excursion_range: 0.5,
            contraction: 0.0,
        }
    }
}

impl Default for DiaphragmMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new diaphragm morph.
pub fn new_diaphragm_morph() -> DiaphragmMorph {
    DiaphragmMorph::new()
}

/// Set diaphragm dome height.
pub fn diaphragm_set_dome_height(m: &mut DiaphragmMorph, v: f32) {
    m.dome_height = v.clamp(0.0, 1.0);
}

/// Set inspiratory descent (contraction lowers dome).
pub fn diaphragm_set_descent(m: &mut DiaphragmMorph, v: f32) {
    m.descent = v.clamp(0.0, 1.0);
}

/// Set excursion range during breathing cycle.
pub fn diaphragm_set_excursion_range(m: &mut DiaphragmMorph, v: f32) {
    m.excursion_range = v.clamp(0.0, 1.0);
}

/// Set muscle contraction level.
pub fn diaphragm_set_contraction(m: &mut DiaphragmMorph, v: f32) {
    m.contraction = v.clamp(0.0, 1.0);
}

/// Effective dome position after descent.
pub fn diaphragm_effective_dome(m: &DiaphragmMorph) -> f32 {
    (m.dome_height - m.descent * 0.5).max(0.0)
}

/// Serialize to JSON-like string.
pub fn diaphragm_morph_to_json(m: &DiaphragmMorph) -> String {
    format!(
        r#"{{"dome_height":{:.4},"descent":{:.4},"excursion_range":{:.4},"contraction":{:.4}}}"#,
        m.dome_height, m.descent, m.excursion_range, m.contraction
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_diaphragm_morph();
        assert!((m.dome_height - 0.5).abs() < 1e-6);
        assert_eq!(m.descent, 0.0);
    }

    #[test]
    fn test_dome_height_clamp() {
        let mut m = new_diaphragm_morph();
        diaphragm_set_dome_height(&mut m, 2.0);
        assert_eq!(m.dome_height, 1.0);
    }

    #[test]
    fn test_descent() {
        let mut m = new_diaphragm_morph();
        diaphragm_set_descent(&mut m, 0.6);
        assert!((m.descent - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_effective_dome_neutral() {
        let m = new_diaphragm_morph();
        assert!((diaphragm_effective_dome(&m) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_effective_dome_lowered() {
        let mut m = new_diaphragm_morph();
        diaphragm_set_descent(&mut m, 1.0);
        assert!(diaphragm_effective_dome(&m) < 0.5); /* dome lowered */
    }

    #[test]
    fn test_excursion_range() {
        let mut m = new_diaphragm_morph();
        diaphragm_set_excursion_range(&mut m, 0.8);
        assert!((m.excursion_range - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_contraction_clamp() {
        let mut m = new_diaphragm_morph();
        diaphragm_set_contraction(&mut m, -1.0);
        assert_eq!(m.contraction, 0.0);
    }

    #[test]
    fn test_json_keys() {
        let m = new_diaphragm_morph();
        let s = diaphragm_morph_to_json(&m);
        assert!(s.contains("excursion_range"));
    }

    #[test]
    fn test_clone() {
        let m = new_diaphragm_morph();
        let m2 = m.clone();
        assert!((m2.dome_height - m.dome_height).abs() < 1e-6);
    }
}
