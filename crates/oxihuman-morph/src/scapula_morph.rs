// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Scapula shape morph — controls scapular size, winging, and spine prominence.

/// Scapula morph configuration.
#[derive(Debug, Clone)]
pub struct ScapulaMorph {
    pub size: f32,
    pub winging: f32,
    pub spine_prominence: f32,
    pub rotation: f32,
}

impl ScapulaMorph {
    pub fn new() -> Self {
        Self {
            size: 0.5,
            winging: 0.0,
            spine_prominence: 0.5,
            rotation: 0.0,
        }
    }
}

impl Default for ScapulaMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new scapula morph.
pub fn new_scapula_morph() -> ScapulaMorph {
    ScapulaMorph::new()
}

/// Set scapula overall size.
pub fn scap_set_size(m: &mut ScapulaMorph, v: f32) {
    m.size = v.clamp(0.0, 1.0);
}

/// Set scapular winging (0 = flat against thorax, 1 = fully winged).
pub fn scap_set_winging(m: &mut ScapulaMorph, v: f32) {
    m.winging = v.clamp(0.0, 1.0);
}

/// Set spine of scapula prominence.
pub fn scap_set_spine_prominence(m: &mut ScapulaMorph, v: f32) {
    m.spine_prominence = v.clamp(0.0, 1.0);
}

/// Set upward/downward rotation bias (-1 = downward, 0 = neutral, 1 = upward).
pub fn scap_set_rotation(m: &mut ScapulaMorph, v: f32) {
    m.rotation = v.clamp(-1.0, 1.0);
}

/// Surface visibility score (winging makes scapula more visible dorsally).
pub fn scap_visibility(m: &ScapulaMorph) -> f32 {
    (m.spine_prominence * 0.6 + m.winging * 0.4).clamp(0.0, 1.0)
}

/// Serialize to JSON-like string.
pub fn scapula_morph_to_json(m: &ScapulaMorph) -> String {
    format!(
        r#"{{"size":{:.4},"winging":{:.4},"spine_prominence":{:.4},"rotation":{:.4}}}"#,
        m.size, m.winging, m.spine_prominence, m.rotation
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_scapula_morph();
        assert_eq!(m.winging, 0.0);
        assert!((m.size - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_size_clamp() {
        let mut m = new_scapula_morph();
        scap_set_size(&mut m, 5.0);
        assert_eq!(m.size, 1.0);
    }

    #[test]
    fn test_winging_set() {
        let mut m = new_scapula_morph();
        scap_set_winging(&mut m, 0.6);
        assert!((m.winging - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_spine_prominence_set() {
        let mut m = new_scapula_morph();
        scap_set_spine_prominence(&mut m, 0.9);
        assert!((m.spine_prominence - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_rotation_clamp() {
        let mut m = new_scapula_morph();
        scap_set_rotation(&mut m, 3.0);
        assert_eq!(m.rotation, 1.0);
    }

    #[test]
    fn test_visibility_range() {
        let m = new_scapula_morph();
        let v = scap_visibility(&m);
        assert!((0.0..=1.0).contains(&v));
    }

    #[test]
    fn test_visibility_increases_with_winging() {
        let mut m = new_scapula_morph();
        let v0 = scap_visibility(&m);
        scap_set_winging(&mut m, 1.0);
        let v1 = scap_visibility(&m);
        assert!(v1 > v0);
    }

    #[test]
    fn test_json_keys() {
        let m = new_scapula_morph();
        let s = scapula_morph_to_json(&m);
        assert!(s.contains("spine_prominence"));
    }

    #[test]
    fn test_clone() {
        let m = new_scapula_morph();
        let m2 = m.clone();
        assert!((m2.size - m.size).abs() < 1e-6);
    }
}
