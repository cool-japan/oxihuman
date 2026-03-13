// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Gonion (jaw angle) morph — controls the posterior-inferior corner of the mandible.

/// Gonion morph configuration.
#[derive(Debug, Clone)]
pub struct GonionMorph {
    pub gonial_angle: f32,
    pub flare: f32,
    pub prominence: f32,
    pub rounding: f32,
    pub asymmetry: f32,
}

impl GonionMorph {
    pub fn new() -> Self {
        Self {
            gonial_angle: 0.5,
            flare: 0.0,
            prominence: 0.5,
            rounding: 0.5,
            asymmetry: 0.0,
        }
    }
}

impl Default for GonionMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new gonion morph.
pub fn new_gonion_morph() -> GonionMorph {
    GonionMorph::new()
}

/// Set gonial angle (0 = very acute ~90°, 1 = obtuse ~140°).
pub fn gon_set_gonial_angle(m: &mut GonionMorph, v: f32) {
    m.gonial_angle = v.clamp(0.0, 1.0);
}

/// Set lateral jaw flare.
pub fn gon_set_flare(m: &mut GonionMorph, v: f32) {
    m.flare = v.clamp(-1.0, 1.0);
}

/// Set prominence of the gonion landmark.
pub fn gon_set_prominence(m: &mut GonionMorph, v: f32) {
    m.prominence = v.clamp(0.0, 1.0);
}

/// Set corner rounding (0 = sharp, 1 = smooth).
pub fn gon_set_rounding(m: &mut GonionMorph, v: f32) {
    m.rounding = v.clamp(0.0, 1.0);
}

/// Compute jaw angle in degrees (approximate linear mapping 90–140°).
pub fn gon_angle_degrees(m: &GonionMorph) -> f32 {
    90.0 + m.gonial_angle * 50.0
}

/// Serialize to JSON-like string.
pub fn gonion_morph_to_json(m: &GonionMorph) -> String {
    format!(
        r#"{{"gonial_angle":{:.4},"flare":{:.4},"prominence":{:.4},"rounding":{:.4},"asymmetry":{:.4}}}"#,
        m.gonial_angle, m.flare, m.prominence, m.rounding, m.asymmetry
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_gonion_morph();
        assert!((m.gonial_angle - 0.5).abs() < 1e-6);
        assert_eq!(m.flare, 0.0);
    }

    #[test]
    fn test_gonial_angle_clamp() {
        let mut m = new_gonion_morph();
        gon_set_gonial_angle(&mut m, 2.0);
        assert_eq!(m.gonial_angle, 1.0);
    }

    #[test]
    fn test_gonial_angle_set() {
        let mut m = new_gonion_morph();
        gon_set_gonial_angle(&mut m, 0.7);
        assert!((m.gonial_angle - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_flare_clamp_negative() {
        let mut m = new_gonion_morph();
        gon_set_flare(&mut m, -2.0);
        assert_eq!(m.flare, -1.0);
    }

    #[test]
    fn test_prominence_set() {
        let mut m = new_gonion_morph();
        gon_set_prominence(&mut m, 0.8);
        assert!((m.prominence - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_rounding_clamp() {
        let mut m = new_gonion_morph();
        gon_set_rounding(&mut m, 3.0);
        assert_eq!(m.rounding, 1.0);
    }

    #[test]
    fn test_angle_degrees_range() {
        let m = new_gonion_morph();
        let deg = gon_angle_degrees(&m);
        assert!((90.0..=140.0).contains(&deg));
    }

    #[test]
    fn test_angle_degrees_min() {
        let mut m = new_gonion_morph();
        gon_set_gonial_angle(&mut m, 0.0);
        assert!((gon_angle_degrees(&m) - 90.0).abs() < 1e-4);
    }

    #[test]
    fn test_json_keys() {
        let m = new_gonion_morph();
        let s = gonion_morph_to_json(&m);
        assert!(s.contains("gonial_angle"));
    }

    #[test]
    fn test_clone() {
        let m = new_gonion_morph();
        let m2 = m.clone();
        assert!((m2.gonial_angle - m.gonial_angle).abs() < 1e-6);
    }
}
