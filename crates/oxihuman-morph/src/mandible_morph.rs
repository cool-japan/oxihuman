// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Mandible (jawbone) shape morph — body width, ramus height, gonial angle.

/// Mandible morph configuration.
#[derive(Debug, Clone)]
pub struct MandibleMorph {
    pub body_width: f32,
    pub ramus_height: f32,
    pub gonial_angle: f32,
    pub symphysis_height: f32,
}

impl MandibleMorph {
    pub fn new() -> Self {
        Self {
            body_width: 0.5,
            ramus_height: 0.5,
            gonial_angle: 0.5,
            symphysis_height: 0.5,
        }
    }
}

impl Default for MandibleMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new mandible morph.
pub fn new_mandible_morph() -> MandibleMorph {
    MandibleMorph::new()
}

/// Set mandibular body width.
pub fn mand_set_body_width(m: &mut MandibleMorph, v: f32) {
    m.body_width = v.clamp(0.0, 1.0);
}

/// Set ramus height.
pub fn mand_set_ramus_height(m: &mut MandibleMorph, v: f32) {
    m.ramus_height = v.clamp(0.0, 1.0);
}

/// Set gonial angle (0 = obtuse/open, 0.5 = normal ~120°, 1 = acute/closed).
pub fn mand_set_gonial_angle(m: &mut MandibleMorph, v: f32) {
    m.gonial_angle = v.clamp(0.0, 1.0);
}

/// Set chin height at symphysis.
pub fn mand_set_symphysis_height(m: &mut MandibleMorph, v: f32) {
    m.symphysis_height = v.clamp(0.0, 1.0);
}

/// Jaw squareness heuristic (wide body + low gonial angle = square).
pub fn mand_squareness(m: &MandibleMorph) -> f32 {
    (m.body_width * (1.0 - m.gonial_angle * 0.5)).clamp(0.0, 1.0)
}

/// Serialize to JSON-like string.
pub fn mandible_morph_to_json(m: &MandibleMorph) -> String {
    format!(
        r#"{{"body_width":{:.4},"ramus_height":{:.4},"gonial_angle":{:.4},"symphysis_height":{:.4}}}"#,
        m.body_width, m.ramus_height, m.gonial_angle, m.symphysis_height
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_mandible_morph();
        assert!((m.body_width - 0.5).abs() < 1e-6);
        assert!((m.gonial_angle - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_body_width_clamp() {
        let mut m = new_mandible_morph();
        mand_set_body_width(&mut m, 5.0);
        assert_eq!(m.body_width, 1.0);
    }

    #[test]
    fn test_ramus_height_set() {
        let mut m = new_mandible_morph();
        mand_set_ramus_height(&mut m, 0.8);
        assert!((m.ramus_height - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_gonial_angle_clamp() {
        let mut m = new_mandible_morph();
        mand_set_gonial_angle(&mut m, -0.1);
        assert_eq!(m.gonial_angle, 0.0);
    }

    #[test]
    fn test_symphysis_set() {
        let mut m = new_mandible_morph();
        mand_set_symphysis_height(&mut m, 0.7);
        assert!((m.symphysis_height - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_squareness_range() {
        let m = new_mandible_morph();
        let s = mand_squareness(&m);
        assert!((0.0..=1.0).contains(&s));
    }

    #[test]
    fn test_squareness_decreases_with_angle() {
        let mut m = new_mandible_morph();
        let s0 = mand_squareness(&m);
        mand_set_gonial_angle(&mut m, 1.0);
        let s1 = mand_squareness(&m);
        assert!(s1 < s0); /* more open angle = less square */
    }

    #[test]
    fn test_json_keys() {
        let m = new_mandible_morph();
        let s = mandible_morph_to_json(&m);
        assert!(s.contains("symphysis_height"));
    }

    #[test]
    fn test_clone() {
        let m = new_mandible_morph();
        let m2 = m.clone();
        assert!((m2.ramus_height - m.ramus_height).abs() < 1e-6);
    }
}
