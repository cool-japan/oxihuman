// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Endomorph body type morph — rounder, higher body-fat build.

/// Configuration for the endomorph morph.
#[derive(Debug, Clone)]
pub struct EndomorphConfig {
    pub belly_fullness: f32,
    pub limb_girth: f32,
    pub face_roundness: f32,
}

impl Default for EndomorphConfig {
    fn default() -> Self {
        EndomorphConfig {
            belly_fullness: 0.8,
            limb_girth: 0.6,
            face_roundness: 0.7,
        }
    }
}

/// State for the endomorph morph.
#[derive(Debug, Clone)]
pub struct EndomorphMorph {
    pub intensity: f32,
    pub config: EndomorphConfig,
    pub enabled: bool,
}

/// Create a new endomorph morph.
pub fn new_endomorph_morph() -> EndomorphMorph {
    EndomorphMorph {
        intensity: 0.0,
        config: EndomorphConfig::default(),
        enabled: true,
    }
}

/// Set intensity [0, 1].
pub fn endo_set_intensity(m: &mut EndomorphMorph, v: f32) {
    m.intensity = v.clamp(0.0, 1.0);
}

/// Belly fullness weight.
pub fn endo_belly_weight(m: &EndomorphMorph) -> f32 {
    m.intensity * m.config.belly_fullness
}

/// Limb girth weight.
pub fn endo_limb_girth(m: &EndomorphMorph) -> f32 {
    m.intensity * m.config.limb_girth
}

/// Face roundness weight.
pub fn endo_face_roundness(m: &EndomorphMorph) -> f32 {
    m.intensity * m.config.face_roundness
}

/// Serialise to JSON.
pub fn endo_to_json(m: &EndomorphMorph) -> String {
    format!(
        r#"{{"intensity":{:.3},"belly":{:.3},"enabled":{}}}"#,
        m.intensity,
        endo_belly_weight(m),
        m.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_zero() {
        let m = new_endomorph_morph();
        assert!((m.intensity - 0.0).abs() < 1e-6 /* zero */);
    }

    #[test]
    fn clamp_to_one() {
        let mut m = new_endomorph_morph();
        endo_set_intensity(&mut m, 2.0);
        assert!((m.intensity - 1.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn belly_weight_at_full() {
        let mut m = new_endomorph_morph();
        endo_set_intensity(&mut m, 1.0);
        assert!((endo_belly_weight(&m) - m.config.belly_fullness).abs() < 1e-6 /* correct */);
    }

    #[test]
    fn limb_girth_zero_at_zero() {
        let m = new_endomorph_morph();
        assert!((endo_limb_girth(&m) - 0.0).abs() < 1e-6 /* zero */);
    }

    #[test]
    fn face_roundness_increasing() {
        let mut m = new_endomorph_morph();
        endo_set_intensity(&mut m, 0.5);
        let r5 = endo_face_roundness(&m);
        endo_set_intensity(&mut m, 1.0);
        let r10 = endo_face_roundness(&m);
        assert!(r10 > r5 /* more round at higher intensity */);
    }

    #[test]
    fn json_has_belly() {
        let mut m = new_endomorph_morph();
        endo_set_intensity(&mut m, 0.6);
        assert!(endo_to_json(&m).contains("belly") /* json has belly key */);
    }

    #[test]
    fn enabled_default() {
        let m = new_endomorph_morph();
        assert!(m.enabled /* enabled */);
    }

    #[test]
    fn config_limb_girth_positive() {
        let m = new_endomorph_morph();
        assert!(m.config.limb_girth > 0.0 /* valid */);
    }
}
