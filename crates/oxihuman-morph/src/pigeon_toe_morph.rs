// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Pigeon toe (in-toeing) foot rotation morph.

/// Pigeon toe configuration.
#[derive(Debug, Clone)]
pub struct PigeonToeMorphConfig {
    pub intoeing_angle: f32,
    pub femoral_anteversion: f32,
    pub metatarsus_adductus: f32,
}

impl Default for PigeonToeMorphConfig {
    fn default() -> Self {
        Self {
            intoeing_angle: 0.0,
            femoral_anteversion: 0.0,
            metatarsus_adductus: 0.0,
        }
    }
}

/// Pigeon toe morph state.
#[derive(Debug, Clone)]
pub struct PigeonToeMorph {
    pub config: PigeonToeMorphConfig,
    pub intensity: f32,
    pub enabled: bool,
}

impl PigeonToeMorph {
    pub fn new() -> Self {
        Self {
            config: PigeonToeMorphConfig::default(),
            intensity: 0.0,
            enabled: true,
        }
    }
}

impl Default for PigeonToeMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new PigeonToeMorph.
pub fn new_pigeon_toe_morph() -> PigeonToeMorph {
    PigeonToeMorph::new()
}

/// Set inward toe angle factor (0.0–1.0).
pub fn pigeon_toe_set_intoeing(morph: &mut PigeonToeMorph, v: f32) {
    morph.config.intoeing_angle = v.clamp(0.0, 1.0);
}

/// Set femoral anteversion component.
pub fn pigeon_toe_set_femoral(morph: &mut PigeonToeMorph, v: f32) {
    morph.config.femoral_anteversion = v.clamp(0.0, 1.0);
}

/// Set metatarsus adductus component.
pub fn pigeon_toe_set_metatarsus(morph: &mut PigeonToeMorph, v: f32) {
    morph.config.metatarsus_adductus = v.clamp(0.0, 1.0);
}

/// Compute total inward rotation in normalized units.
pub fn pigeon_toe_total_rotation(morph: &PigeonToeMorph) -> f32 {
    morph.intensity
        * (morph.config.intoeing_angle
            + morph.config.femoral_anteversion * 0.5
            + morph.config.metatarsus_adductus * 0.3)
}

/// Serialize to JSON.
pub fn pigeon_toe_to_json(morph: &PigeonToeMorph) -> String {
    format!(
        r#"{{"intensity":{},"intoeing_angle":{},"femoral_anteversion":{},"metatarsus_adductus":{}}}"#,
        morph.intensity,
        morph.config.intoeing_angle,
        morph.config.femoral_anteversion,
        morph.config.metatarsus_adductus,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = new_pigeon_toe_morph();
        assert!((m.config.intoeing_angle - 0.0).abs() < 1e-6 /* default */);
    }

    #[test]
    fn test_intoeing_clamp() {
        let mut m = new_pigeon_toe_morph();
        pigeon_toe_set_intoeing(&mut m, 5.0);
        assert!((m.config.intoeing_angle - 1.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_femoral() {
        let mut m = new_pigeon_toe_morph();
        pigeon_toe_set_femoral(&mut m, 0.4);
        assert!((m.config.femoral_anteversion - 0.4).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_metatarsus() {
        let mut m = new_pigeon_toe_morph();
        pigeon_toe_set_metatarsus(&mut m, 0.2);
        assert!((m.config.metatarsus_adductus - 0.2).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_total_rotation_zero() {
        let m = new_pigeon_toe_morph();
        assert!((pigeon_toe_total_rotation(&m) - 0.0).abs() < 1e-6 /* zero */);
    }

    #[test]
    fn test_total_rotation_nonzero() {
        let mut m = new_pigeon_toe_morph();
        pigeon_toe_set_intoeing(&mut m, 1.0);
        m.intensity = 1.0;
        assert!(pigeon_toe_total_rotation(&m) > 0.0 /* positive */);
    }

    #[test]
    fn test_json_key() {
        let m = new_pigeon_toe_morph();
        let j = pigeon_toe_to_json(&m);
        assert!(j.contains("intoeing_angle") /* key */);
    }

    #[test]
    fn test_default_enabled() {
        let m = PigeonToeMorph::default();
        assert!(m.enabled /* enabled */);
    }

    #[test]
    fn test_clone() {
        let m = new_pigeon_toe_morph();
        let c = m.clone();
        assert!((c.intensity - m.intensity).abs() < 1e-6 /* equal */);
    }
}
