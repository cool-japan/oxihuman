// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Hair volume and puffiness morph control.

/// Hair volume morph configuration.
#[derive(Debug, Clone)]
pub struct HairVolumeMorph {
    pub volume: f32,
    pub crown_lift: f32,
    pub side_puff: f32,
    pub root_lift: f32,
}

impl HairVolumeMorph {
    pub fn new() -> Self {
        Self {
            volume: 0.5,
            crown_lift: 0.3,
            side_puff: 0.3,
            root_lift: 0.2,
        }
    }
}

impl Default for HairVolumeMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new hair volume morph.
pub fn new_hair_volume_morph() -> HairVolumeMorph {
    HairVolumeMorph::new()
}

/// Set overall hair volume in normalized range.
pub fn hair_volume_set_volume(morph: &mut HairVolumeMorph, volume: f32) {
    morph.volume = volume.clamp(0.0, 1.0);
}

/// Set crown lift factor (vertical uplift at the crown).
pub fn hair_volume_set_crown_lift(morph: &mut HairVolumeMorph, lift: f32) {
    morph.crown_lift = lift.clamp(0.0, 1.0);
}

/// Set side puffiness factor.
pub fn hair_volume_set_side_puff(morph: &mut HairVolumeMorph, puff: f32) {
    morph.side_puff = puff.clamp(0.0, 1.0);
}

/// Compute overall fullness as weighted average.
pub fn hair_volume_fullness(morph: &HairVolumeMorph) -> f32 {
    (morph.volume * 0.5 + morph.crown_lift * 0.3 + morph.side_puff * 0.2).clamp(0.0, 1.0)
}

/// Serialize to JSON-like string.
pub fn hair_volume_morph_to_json(morph: &HairVolumeMorph) -> String {
    format!(
        r#"{{"volume":{:.4},"crown_lift":{:.4},"side_puff":{:.4},"root_lift":{:.4}}}"#,
        morph.volume, morph.crown_lift, morph.side_puff, morph.root_lift
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_hair_volume_morph();
        assert!((m.volume - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_volume_clamp() {
        let mut m = new_hair_volume_morph();
        hair_volume_set_volume(&mut m, 3.0);
        assert_eq!(m.volume, 1.0);
    }

    #[test]
    fn test_crown_lift() {
        let mut m = new_hair_volume_morph();
        hair_volume_set_crown_lift(&mut m, 0.9);
        assert!((m.crown_lift - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_side_puff() {
        let mut m = new_hair_volume_morph();
        hair_volume_set_side_puff(&mut m, 0.4);
        assert!((m.side_puff - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_fullness_range() {
        let m = new_hair_volume_morph();
        let f = hair_volume_fullness(&m);
        assert!((0.0..=1.0).contains(&f));
    }

    #[test]
    fn test_json() {
        let m = new_hair_volume_morph();
        let s = hair_volume_morph_to_json(&m);
        assert!(s.contains("crown_lift"));
    }

    #[test]
    fn test_clone() {
        let m = new_hair_volume_morph();
        let m2 = m.clone();
        assert!((m2.root_lift - m.root_lift).abs() < 1e-6);
    }

    #[test]
    fn test_crown_clamp_low() {
        let mut m = new_hair_volume_morph();
        hair_volume_set_crown_lift(&mut m, -1.0);
        assert_eq!(m.crown_lift, 0.0);
    }

    #[test]
    fn test_default_trait() {
        let m: HairVolumeMorph = Default::default();
        assert!((m.side_puff - 0.3).abs() < 1e-6);
    }
}
