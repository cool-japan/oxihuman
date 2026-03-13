// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Soft palate (velum) shape morph — controls raise, tension, and curvature.

/// Soft palate morph configuration.
#[derive(Debug, Clone)]
pub struct SoftPalateMorph {
    pub raise: f32,
    pub tension: f32,
    pub curvature: f32,
    pub width: f32,
}

impl SoftPalateMorph {
    pub fn new() -> Self {
        Self {
            raise: 0.0,
            tension: 0.5,
            curvature: 0.3,
            width: 0.5,
        }
    }
}

impl Default for SoftPalateMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new soft palate morph.
pub fn new_soft_palate_morph() -> SoftPalateMorph {
    SoftPalateMorph::new()
}

/// Set velum raise (0 = lowered, 1 = fully raised, nasal cavity closed).
pub fn soft_palate_set_raise(m: &mut SoftPalateMorph, v: f32) {
    m.raise = v.clamp(0.0, 1.0);
}

/// Set muscular tension of the velum.
pub fn soft_palate_set_tension(m: &mut SoftPalateMorph, v: f32) {
    m.tension = v.clamp(0.0, 1.0);
}

/// Set posterior curvature of the soft palate.
pub fn soft_palate_set_curvature(m: &mut SoftPalateMorph, v: f32) {
    m.curvature = v.clamp(0.0, 1.0);
}

/// Set palatal width.
pub fn soft_palate_set_width(m: &mut SoftPalateMorph, v: f32) {
    m.width = v.clamp(0.0, 1.0);
}

/// Returns true when velum seals the nasopharynx (raise near 1.0).
pub fn soft_palate_is_sealed(m: &SoftPalateMorph) -> bool {
    m.raise >= 0.9
}

/// Serialize to JSON-like string.
pub fn soft_palate_morph_to_json(m: &SoftPalateMorph) -> String {
    format!(
        r#"{{"raise":{:.4},"tension":{:.4},"curvature":{:.4},"width":{:.4}}}"#,
        m.raise, m.tension, m.curvature, m.width
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_soft_palate_morph();
        assert_eq!(m.raise, 0.0);
        assert!(!soft_palate_is_sealed(&m));
    }

    #[test]
    fn test_set_raise() {
        let mut m = new_soft_palate_morph();
        soft_palate_set_raise(&mut m, 0.95);
        assert!(soft_palate_is_sealed(&m)); /* velum seals nasopharynx */
    }

    #[test]
    fn test_raise_clamp() {
        let mut m = new_soft_palate_morph();
        soft_palate_set_raise(&mut m, 3.0);
        assert_eq!(m.raise, 1.0);
    }

    #[test]
    fn test_tension() {
        let mut m = new_soft_palate_morph();
        soft_palate_set_tension(&mut m, 0.8);
        assert!((m.tension - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_curvature_clamp() {
        let mut m = new_soft_palate_morph();
        soft_palate_set_curvature(&mut m, -0.5);
        assert_eq!(m.curvature, 0.0);
    }

    #[test]
    fn test_width() {
        let mut m = new_soft_palate_morph();
        soft_palate_set_width(&mut m, 0.7);
        assert!((m.width - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_json_keys() {
        let m = new_soft_palate_morph();
        let s = soft_palate_morph_to_json(&m);
        assert!(s.contains("curvature"));
    }

    #[test]
    fn test_clone() {
        let m = new_soft_palate_morph();
        let m2 = m.clone();
        assert!((m2.tension - m.tension).abs() < 1e-6);
    }

    #[test]
    fn test_not_sealed_midway() {
        let mut m = new_soft_palate_morph();
        soft_palate_set_raise(&mut m, 0.5);
        assert!(!soft_palate_is_sealed(&m));
    }
}
