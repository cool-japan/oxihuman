// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Arytenoid cartilage shape morph — controls adduction, rotation, and tilt.

/// Arytenoid morph configuration.
#[derive(Debug, Clone)]
pub struct ArytenoidMorph {
    pub adduction: f32,
    pub rotation: f32,
    pub tilt: f32,
    pub left_right_asymmetry: f32,
}

impl ArytenoidMorph {
    pub fn new() -> Self {
        Self {
            adduction: 0.0,
            rotation: 0.0,
            tilt: 0.0,
            left_right_asymmetry: 0.0,
        }
    }
}

impl Default for ArytenoidMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new arytenoid morph.
pub fn new_arytenoid_morph() -> ArytenoidMorph {
    ArytenoidMorph::new()
}

/// Set adduction (0 = abducted/open, 1 = fully adducted/closed).
pub fn arytenoid_set_adduction(m: &mut ArytenoidMorph, v: f32) {
    m.adduction = v.clamp(0.0, 1.0);
}

/// Set medial rotation of arytenoids.
pub fn arytenoid_set_rotation(m: &mut ArytenoidMorph, v: f32) {
    m.rotation = v.clamp(-1.0, 1.0);
}

/// Set anterior tilt (cricoid-arytenoid joint).
pub fn arytenoid_set_tilt(m: &mut ArytenoidMorph, v: f32) {
    m.tilt = v.clamp(-1.0, 1.0);
}

/// Set left-right asymmetry.
pub fn arytenoid_set_asymmetry(m: &mut ArytenoidMorph, v: f32) {
    m.left_right_asymmetry = v.clamp(-1.0, 1.0);
}

/// Returns true when vocal folds are in phonation position.
pub fn arytenoid_is_phonating(m: &ArytenoidMorph) -> bool {
    m.adduction >= 0.7
}

/// Serialize to JSON-like string.
pub fn arytenoid_morph_to_json(m: &ArytenoidMorph) -> String {
    format!(
        r#"{{"adduction":{:.4},"rotation":{:.4},"tilt":{:.4},"asymmetry":{:.4}}}"#,
        m.adduction, m.rotation, m.tilt, m.left_right_asymmetry
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_arytenoid_morph();
        assert_eq!(m.adduction, 0.0);
        assert!(!arytenoid_is_phonating(&m));
    }

    #[test]
    fn test_phonating() {
        let mut m = new_arytenoid_morph();
        arytenoid_set_adduction(&mut m, 0.9);
        assert!(arytenoid_is_phonating(&m));
    }

    #[test]
    fn test_adduction_clamp() {
        let mut m = new_arytenoid_morph();
        arytenoid_set_adduction(&mut m, 2.0);
        assert_eq!(m.adduction, 1.0);
    }

    #[test]
    fn test_rotation() {
        let mut m = new_arytenoid_morph();
        arytenoid_set_rotation(&mut m, -0.3);
        assert!((m.rotation + 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_tilt_clamp() {
        let mut m = new_arytenoid_morph();
        arytenoid_set_tilt(&mut m, -2.0);
        assert_eq!(m.tilt, -1.0);
    }

    #[test]
    fn test_asymmetry() {
        let mut m = new_arytenoid_morph();
        arytenoid_set_asymmetry(&mut m, 0.2);
        assert!((m.left_right_asymmetry - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_json_keys() {
        let m = new_arytenoid_morph();
        let s = arytenoid_morph_to_json(&m);
        assert!(s.contains("adduction"));
    }

    #[test]
    fn test_clone() {
        let m = new_arytenoid_morph();
        let m2 = m.clone();
        assert!((m2.adduction - m.adduction).abs() < 1e-6);
    }

    #[test]
    fn test_not_phonating_at_threshold() {
        let mut m = new_arytenoid_morph();
        arytenoid_set_adduction(&mut m, 0.69);
        assert!(!arytenoid_is_phonating(&m));
    }
}
