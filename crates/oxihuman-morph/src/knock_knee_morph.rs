// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Knock knee (genu valgum) leg morph.

/// Knock knee configuration.
#[derive(Debug, Clone)]
pub struct KnockKneeMorphConfig {
    pub valgus_angle: f32,
    pub tibial_torsion: f32,
    pub ankle_eversion: f32,
}

impl Default for KnockKneeMorphConfig {
    fn default() -> Self {
        Self {
            valgus_angle: 0.0,
            tibial_torsion: 0.0,
            ankle_eversion: 0.0,
        }
    }
}

/// Knock knee morph state.
#[derive(Debug, Clone)]
pub struct KnockKneeMorph {
    pub config: KnockKneeMorphConfig,
    pub intensity: f32,
    pub symmetric: bool,
}

impl KnockKneeMorph {
    pub fn new() -> Self {
        Self {
            config: KnockKneeMorphConfig::default(),
            intensity: 0.0,
            symmetric: true,
        }
    }
}

impl Default for KnockKneeMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new KnockKneeMorph.
pub fn new_knock_knee_morph() -> KnockKneeMorph {
    KnockKneeMorph::new()
}

/// Set valgus angle factor (0.0–1.0).
pub fn knock_knee_set_valgus(morph: &mut KnockKneeMorph, v: f32) {
    morph.config.valgus_angle = v.clamp(0.0, 1.0);
}

/// Set tibial torsion factor.
pub fn knock_knee_set_torsion(morph: &mut KnockKneeMorph, v: f32) {
    morph.config.tibial_torsion = v.clamp(0.0, 1.0);
}

/// Set ankle eversion factor.
pub fn knock_knee_set_eversion(morph: &mut KnockKneeMorph, v: f32) {
    morph.config.ankle_eversion = v.clamp(0.0, 1.0);
}

/// Effective knee separation distance factor (higher = more knock-kneed).
pub fn knock_knee_separation(morph: &KnockKneeMorph) -> f32 {
    morph.intensity * morph.config.valgus_angle
}

/// Serialize to JSON.
pub fn knock_knee_to_json(morph: &KnockKneeMorph) -> String {
    format!(
        r#"{{"intensity":{},"valgus_angle":{},"tibial_torsion":{},"ankle_eversion":{}}}"#,
        morph.intensity,
        morph.config.valgus_angle,
        morph.config.tibial_torsion,
        morph.config.ankle_eversion,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = new_knock_knee_morph();
        assert!(m.symmetric /* symmetric by default */);
    }

    #[test]
    fn test_valgus_clamp() {
        let mut m = new_knock_knee_morph();
        knock_knee_set_valgus(&mut m, 5.0);
        assert!((m.config.valgus_angle - 1.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_torsion() {
        let mut m = new_knock_knee_morph();
        knock_knee_set_torsion(&mut m, 0.4);
        assert!((m.config.tibial_torsion - 0.4).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_eversion() {
        let mut m = new_knock_knee_morph();
        knock_knee_set_eversion(&mut m, 0.3);
        assert!((m.config.ankle_eversion - 0.3).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_separation_zero() {
        let m = new_knock_knee_morph();
        assert!((knock_knee_separation(&m) - 0.0).abs() < 1e-6 /* zero */);
    }

    #[test]
    fn test_separation_nonzero() {
        let mut m = new_knock_knee_morph();
        knock_knee_set_valgus(&mut m, 0.8);
        m.intensity = 1.0;
        assert!(knock_knee_separation(&m) > 0.0 /* positive */);
    }

    #[test]
    fn test_json_key() {
        let m = new_knock_knee_morph();
        let j = knock_knee_to_json(&m);
        assert!(j.contains("valgus_angle") /* key */);
    }

    #[test]
    fn test_default_trait() {
        let m = KnockKneeMorph::default();
        assert!(m.symmetric /* symmetric */);
    }

    #[test]
    fn test_clone() {
        let m = new_knock_knee_morph();
        let c = m.clone();
        assert!((c.intensity - m.intensity).abs() < 1e-6 /* equal */);
    }
}
