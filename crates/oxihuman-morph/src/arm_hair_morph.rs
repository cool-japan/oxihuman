// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Arm and body hair density morph control.

/// Arm hair density morph configuration.
#[derive(Debug, Clone)]
pub struct ArmHairMorph {
    pub density: f32,
    pub length: f32,
    pub darkness: f32,
    pub forearm_coverage: f32,
    pub upper_arm_coverage: f32,
}

impl ArmHairMorph {
    pub fn new() -> Self {
        Self {
            density: 0.3,
            length: 0.2,
            darkness: 0.4,
            forearm_coverage: 0.5,
            upper_arm_coverage: 0.3,
        }
    }
}

impl Default for ArmHairMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new arm hair morph.
pub fn new_arm_hair_morph() -> ArmHairMorph {
    ArmHairMorph::new()
}

/// Set hair strand density uniformly.
pub fn arm_hair_set_density(morph: &mut ArmHairMorph, density: f32) {
    morph.density = density.clamp(0.0, 1.0);
}

/// Set average hair length in normalized range.
pub fn arm_hair_set_length(morph: &mut ArmHairMorph, length: f32) {
    morph.length = length.clamp(0.0, 1.0);
}

/// Set hair darkness pigmentation level.
pub fn arm_hair_set_darkness(morph: &mut ArmHairMorph, darkness: f32) {
    morph.darkness = darkness.clamp(0.0, 1.0);
}

/// Compute overall visibility as density * darkness.
pub fn arm_hair_visibility(morph: &ArmHairMorph) -> f32 {
    (morph.density * morph.darkness).clamp(0.0, 1.0)
}

/// Serialize to JSON-like string.
pub fn arm_hair_morph_to_json(morph: &ArmHairMorph) -> String {
    format!(
        r#"{{"density":{:.4},"length":{:.4},"darkness":{:.4},"forearm_coverage":{:.4},"upper_arm_coverage":{:.4}}}"#,
        morph.density,
        morph.length,
        morph.darkness,
        morph.forearm_coverage,
        morph.upper_arm_coverage
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_arm_hair_morph();
        assert!((m.density - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_density_clamp_high() {
        let mut m = new_arm_hair_morph();
        arm_hair_set_density(&mut m, 2.0);
        assert_eq!(m.density, 1.0);
    }

    #[test]
    fn test_length_set() {
        let mut m = new_arm_hair_morph();
        arm_hair_set_length(&mut m, 0.5);
        assert!((m.length - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_darkness_set() {
        let mut m = new_arm_hair_morph();
        arm_hair_set_darkness(&mut m, 0.8);
        assert!((m.darkness - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_visibility_range() {
        let m = new_arm_hair_morph();
        let v = arm_hair_visibility(&m);
        assert!((0.0..=1.0).contains(&v));
    }

    #[test]
    fn test_json_contains_forearm() {
        let m = new_arm_hair_morph();
        let s = arm_hair_morph_to_json(&m);
        assert!(s.contains("forearm_coverage"));
    }

    #[test]
    fn test_clone() {
        let m = new_arm_hair_morph();
        let m2 = m.clone();
        assert!((m2.upper_arm_coverage - m.upper_arm_coverage).abs() < 1e-6);
    }

    #[test]
    fn test_darkness_clamp_low() {
        let mut m = new_arm_hair_morph();
        arm_hair_set_darkness(&mut m, -1.0);
        assert_eq!(m.darkness, 0.0);
    }

    #[test]
    fn test_default_trait() {
        let m: ArmHairMorph = Default::default();
        assert!((m.length - 0.2).abs() < 1e-6);
    }
}
