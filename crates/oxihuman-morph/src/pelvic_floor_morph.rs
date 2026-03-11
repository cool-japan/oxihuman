// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Pelvic floor position morph — controls perineal plane and levator tension.

/// Pelvic floor morph configuration.
#[derive(Debug, Clone)]
pub struct PelvicFloorMorph {
    pub descent: f32,
    pub levator_tension: f32,
    pub perineal_body: f32,
    pub contraction: f32,
}

impl PelvicFloorMorph {
    pub fn new() -> Self {
        Self {
            descent: 0.0,
            levator_tension: 0.5,
            perineal_body: 0.5,
            contraction: 0.0,
        }
    }
}

impl Default for PelvicFloorMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new pelvic floor morph.
pub fn new_pelvic_floor_morph() -> PelvicFloorMorph {
    PelvicFloorMorph::new()
}

/// Set pelvic floor descent (prolapse direction, 0 = normal, 1 = maximal descent).
pub fn pelvic_floor_set_descent(m: &mut PelvicFloorMorph, v: f32) {
    m.descent = v.clamp(0.0, 1.0);
}

/// Set levator ani muscle tension.
pub fn pelvic_floor_set_levator_tension(m: &mut PelvicFloorMorph, v: f32) {
    m.levator_tension = v.clamp(0.0, 1.0);
}

/// Set perineal body prominence.
pub fn pelvic_floor_set_perineal_body(m: &mut PelvicFloorMorph, v: f32) {
    m.perineal_body = v.clamp(0.0, 1.0);
}

/// Set overall contraction level.
pub fn pelvic_floor_set_contraction(m: &mut PelvicFloorMorph, v: f32) {
    m.contraction = v.clamp(0.0, 1.0);
}

/// Effective floor elevation (contraction lifts, descent lowers).
pub fn pelvic_floor_elevation(m: &PelvicFloorMorph) -> f32 {
    (m.contraction - m.descent).clamp(-1.0, 1.0)
}

/// Serialize to JSON-like string.
pub fn pelvic_floor_morph_to_json(m: &PelvicFloorMorph) -> String {
    format!(
        r#"{{"descent":{:.4},"levator_tension":{:.4},"perineal_body":{:.4},"contraction":{:.4}}}"#,
        m.descent, m.levator_tension, m.perineal_body, m.contraction
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_pelvic_floor_morph();
        assert_eq!(m.descent, 0.0);
        assert_eq!(m.contraction, 0.0);
    }

    #[test]
    fn test_descent_clamp() {
        let mut m = new_pelvic_floor_morph();
        pelvic_floor_set_descent(&mut m, 3.0);
        assert_eq!(m.descent, 1.0);
    }

    #[test]
    fn test_levator_tension() {
        let mut m = new_pelvic_floor_morph();
        pelvic_floor_set_levator_tension(&mut m, 0.8);
        assert!((m.levator_tension - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_perineal_body() {
        let mut m = new_pelvic_floor_morph();
        pelvic_floor_set_perineal_body(&mut m, 0.7);
        assert!((m.perineal_body - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_contraction_clamp() {
        let mut m = new_pelvic_floor_morph();
        pelvic_floor_set_contraction(&mut m, -1.0);
        assert_eq!(m.contraction, 0.0);
    }

    #[test]
    fn test_elevation_neutral() {
        let m = new_pelvic_floor_morph();
        assert_eq!(pelvic_floor_elevation(&m), 0.0);
    }

    #[test]
    fn test_elevation_contracted() {
        let mut m = new_pelvic_floor_morph();
        pelvic_floor_set_contraction(&mut m, 0.8);
        assert!(pelvic_floor_elevation(&m) > 0.0); /* lifted */
    }

    #[test]
    fn test_json_keys() {
        let m = new_pelvic_floor_morph();
        let s = pelvic_floor_morph_to_json(&m);
        assert!(s.contains("levator_tension"));
    }

    #[test]
    fn test_clone() {
        let m = new_pelvic_floor_morph();
        let m2 = m.clone();
        assert!((m2.levator_tension - m.levator_tension).abs() < 1e-6);
    }
}
