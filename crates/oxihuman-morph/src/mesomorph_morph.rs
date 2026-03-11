// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Mesomorph body type morph — medium, muscular build.

/// Configuration for the mesomorph morph.
#[derive(Debug, Clone)]
pub struct MesomorphConfig {
    pub chest_width: f32,
    pub muscle_tone: f32,
    pub waist_ratio: f32,
}

impl Default for MesomorphConfig {
    fn default() -> Self {
        MesomorphConfig {
            chest_width: 0.7,
            muscle_tone: 0.6,
            waist_ratio: 0.75,
        }
    }
}

/// State for the mesomorph morph.
#[derive(Debug, Clone)]
pub struct MesomorphMorph {
    pub intensity: f32,
    pub config: MesomorphConfig,
    pub enabled: bool,
}

/// Create a new mesomorph morph.
pub fn new_mesomorph_morph() -> MesomorphMorph {
    MesomorphMorph {
        intensity: 0.0,
        config: MesomorphConfig::default(),
        enabled: true,
    }
}

/// Set intensity [0, 1].
pub fn meso_set_intensity(m: &mut MesomorphMorph, v: f32) {
    m.intensity = v.clamp(0.0, 1.0);
}

/// Chest width delta.
pub fn meso_chest_delta(m: &MesomorphMorph) -> f32 {
    m.intensity * m.config.chest_width
}

/// Muscle tone weight.
pub fn meso_muscle_tone(m: &MesomorphMorph) -> f32 {
    m.intensity * m.config.muscle_tone
}

/// Waist-to-hip ratio parameter.
pub fn meso_waist_ratio(m: &MesomorphMorph) -> f32 {
    1.0 - m.intensity * (1.0 - m.config.waist_ratio)
}

/// Serialise to JSON.
pub fn meso_to_json(m: &MesomorphMorph) -> String {
    format!(
        r#"{{"intensity":{:.3},"muscle_tone":{:.3},"enabled":{}}}"#,
        m.intensity,
        meso_muscle_tone(m),
        m.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_zero() {
        let m = new_mesomorph_morph();
        assert!((m.intensity - 0.0).abs() < 1e-6 /* zero intensity */);
    }

    #[test]
    fn clamp_upper() {
        let mut m = new_mesomorph_morph();
        meso_set_intensity(&mut m, 5.0);
        assert!((m.intensity - 1.0).abs() < 1e-6 /* clamped to 1 */);
    }

    #[test]
    fn chest_delta_increases() {
        let mut m = new_mesomorph_morph();
        meso_set_intensity(&mut m, 1.0);
        assert!(meso_chest_delta(&m) > 0.0 /* positive delta */);
    }

    #[test]
    fn muscle_tone_zero_at_zero() {
        let m = new_mesomorph_morph();
        assert!((meso_muscle_tone(&m) - 0.0).abs() < 1e-6 /* zero tone */);
    }

    #[test]
    fn waist_ratio_at_zero_is_one() {
        let m = new_mesomorph_morph();
        assert!((meso_waist_ratio(&m) - 1.0).abs() < 1e-6 /* neutral ratio */);
    }

    #[test]
    fn json_contains_field() {
        let mut m = new_mesomorph_morph();
        meso_set_intensity(&mut m, 0.9);
        assert!(meso_to_json(&m).contains("0.900") /* in json */);
    }

    #[test]
    fn enabled_default() {
        let m = new_mesomorph_morph();
        assert!(m.enabled /* enabled */);
    }

    #[test]
    fn config_chest_positive() {
        let m = new_mesomorph_morph();
        assert!(m.config.chest_width > 0.0 /* positive */);
    }
}
