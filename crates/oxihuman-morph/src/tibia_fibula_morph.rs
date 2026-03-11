// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Tibia/fibula proportions morph — lower leg bone geometry control.

/// Tibia/fibula morph configuration.
#[derive(Debug, Clone)]
pub struct TibiaFibulaMorph {
    pub length: f32,
    pub tibial_torsion: f32,
    pub fibula_offset: f32,
    pub malleolus_width: f32,
}

impl TibiaFibulaMorph {
    pub fn new() -> Self {
        Self {
            length: 0.5,
            tibial_torsion: 0.5,
            fibula_offset: 0.5,
            malleolus_width: 0.5,
        }
    }
}

impl Default for TibiaFibulaMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new tibia/fibula morph.
pub fn new_tibia_fibula_morph() -> TibiaFibulaMorph {
    TibiaFibulaMorph::new()
}

/// Set lower leg length.
pub fn tf_set_length(m: &mut TibiaFibulaMorph, v: f32) {
    m.length = v.clamp(0.0, 1.0);
}

/// Set tibial torsion (0 = internal, 0.5 = neutral, 1 = external).
pub fn tf_set_tibial_torsion(m: &mut TibiaFibulaMorph, v: f32) {
    m.tibial_torsion = v.clamp(0.0, 1.0);
}

/// Set fibula lateral offset.
pub fn tf_set_fibula_offset(m: &mut TibiaFibulaMorph, v: f32) {
    m.fibula_offset = v.clamp(0.0, 1.0);
}

/// Set bimalleolar width.
pub fn tf_set_malleolus_width(m: &mut TibiaFibulaMorph, v: f32) {
    m.malleolus_width = v.clamp(0.0, 1.0);
}

/// Foot progression angle heuristic from tibial torsion.
pub fn tf_foot_progression(m: &TibiaFibulaMorph) -> f32 {
    (m.tibial_torsion - 0.5) * 2.0 /* -1 = toe-in, 1 = toe-out */
}

/// Serialize to JSON-like string.
pub fn tibia_fibula_morph_to_json(m: &TibiaFibulaMorph) -> String {
    format!(
        r#"{{"length":{:.4},"tibial_torsion":{:.4},"fibula_offset":{:.4},"malleolus_width":{:.4}}}"#,
        m.length, m.tibial_torsion, m.fibula_offset, m.malleolus_width
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_tibia_fibula_morph();
        assert!((m.length - 0.5).abs() < 1e-6);
        assert!((m.tibial_torsion - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_length_clamp() {
        let mut m = new_tibia_fibula_morph();
        tf_set_length(&mut m, 3.0);
        assert_eq!(m.length, 1.0);
    }

    #[test]
    fn test_torsion_set() {
        let mut m = new_tibia_fibula_morph();
        tf_set_tibial_torsion(&mut m, 0.8);
        assert!((m.tibial_torsion - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_fibula_offset_set() {
        let mut m = new_tibia_fibula_morph();
        tf_set_fibula_offset(&mut m, 0.6);
        assert!((m.fibula_offset - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_malleolus_clamp() {
        let mut m = new_tibia_fibula_morph();
        tf_set_malleolus_width(&mut m, 2.0);
        assert_eq!(m.malleolus_width, 1.0);
    }

    #[test]
    fn test_foot_progression_neutral() {
        let m = new_tibia_fibula_morph();
        assert!(tf_foot_progression(&m).abs() < 1e-5); /* neutral = 0 */
    }

    #[test]
    fn test_foot_progression_toe_out() {
        let mut m = new_tibia_fibula_morph();
        tf_set_tibial_torsion(&mut m, 1.0);
        assert!(tf_foot_progression(&m) > 0.0); /* external = toe-out */
    }

    #[test]
    fn test_json_keys() {
        let m = new_tibia_fibula_morph();
        let s = tibia_fibula_morph_to_json(&m);
        assert!(s.contains("tibial_torsion"));
    }

    #[test]
    fn test_clone() {
        let m = new_tibia_fibula_morph();
        let m2 = m.clone();
        assert!((m2.malleolus_width - m.malleolus_width).abs() < 1e-6);
    }
}
