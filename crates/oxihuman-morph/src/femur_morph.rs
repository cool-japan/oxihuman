// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Femur length/angle morph — controls thigh bone proportions and neck angle.

/// Femur morph configuration.
#[derive(Debug, Clone)]
pub struct FemurMorph {
    pub length: f32,
    pub neck_angle: f32,
    pub anteversion: f32,
    pub condyle_width: f32,
}

impl FemurMorph {
    pub fn new() -> Self {
        Self {
            length: 0.5,
            neck_angle: 0.5,
            anteversion: 0.5,
            condyle_width: 0.5,
        }
    }
}

impl Default for FemurMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new femur morph.
pub fn new_femur_morph() -> FemurMorph {
    FemurMorph::new()
}

/// Set femur length (relative to lower limb).
pub fn fem_set_length(m: &mut FemurMorph, v: f32) {
    m.length = v.clamp(0.0, 1.0);
}

/// Set neck-shaft angle (0 = varus, 0.5 = neutral ~130°, 1 = valgus).
pub fn fem_set_neck_angle(m: &mut FemurMorph, v: f32) {
    m.neck_angle = v.clamp(0.0, 1.0);
}

/// Set femoral anteversion (forward torsion of neck, 0 = retroversion, 1 = max anteversion).
pub fn fem_set_anteversion(m: &mut FemurMorph, v: f32) {
    m.anteversion = v.clamp(0.0, 1.0);
}

/// Set medial/lateral condyle width.
pub fn fem_set_condyle_width(m: &mut FemurMorph, v: f32) {
    m.condyle_width = v.clamp(0.0, 1.0);
}

/// Knee Q-angle contribution from neck and anteversion.
pub fn fem_q_angle(m: &FemurMorph) -> f32 {
    (m.neck_angle * 0.6 + m.anteversion * 0.4).clamp(0.0, 1.0)
}

/// Serialize to JSON-like string.
pub fn femur_morph_to_json(m: &FemurMorph) -> String {
    format!(
        r#"{{"length":{:.4},"neck_angle":{:.4},"anteversion":{:.4},"condyle_width":{:.4}}}"#,
        m.length, m.neck_angle, m.anteversion, m.condyle_width
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_femur_morph();
        assert!((m.length - 0.5).abs() < 1e-6);
        assert!((m.neck_angle - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_length_clamp() {
        let mut m = new_femur_morph();
        fem_set_length(&mut m, 3.0);
        assert_eq!(m.length, 1.0);
    }

    #[test]
    fn test_neck_angle_clamp() {
        let mut m = new_femur_morph();
        fem_set_neck_angle(&mut m, -0.5);
        assert_eq!(m.neck_angle, 0.0);
    }

    #[test]
    fn test_anteversion_set() {
        let mut m = new_femur_morph();
        fem_set_anteversion(&mut m, 0.8);
        assert!((m.anteversion - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_condyle_set() {
        let mut m = new_femur_morph();
        fem_set_condyle_width(&mut m, 0.7);
        assert!((m.condyle_width - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_q_angle_range() {
        let m = new_femur_morph();
        let q = fem_q_angle(&m);
        assert!((0.0..=1.0).contains(&q));
    }

    #[test]
    fn test_q_angle_increases() {
        let mut m = new_femur_morph();
        let q0 = fem_q_angle(&m);
        fem_set_neck_angle(&mut m, 1.0);
        fem_set_anteversion(&mut m, 1.0);
        let q1 = fem_q_angle(&m);
        assert!(q1 > q0);
    }

    #[test]
    fn test_json_keys() {
        let m = new_femur_morph();
        let s = femur_morph_to_json(&m);
        assert!(s.contains("anteversion"));
    }

    #[test]
    fn test_clone() {
        let m = new_femur_morph();
        let m2 = m.clone();
        assert!((m2.condyle_width - m.condyle_width).abs() < 1e-6);
    }
}
