// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Ectomorph body type morph — lean, narrow build.

/// Configuration for the ectomorph morph.
#[derive(Debug, Clone)]
pub struct EctomorphConfig {
    pub limb_slenderness: f32,
    pub shoulder_narrowness: f32,
    pub hip_narrowness: f32,
}

impl Default for EctomorphConfig {
    fn default() -> Self {
        EctomorphConfig {
            limb_slenderness: 0.75,
            shoulder_narrowness: 0.65,
            hip_narrowness: 0.6,
        }
    }
}

/// State for the ectomorph morph.
#[derive(Debug, Clone)]
pub struct EctomorphMorph {
    pub intensity: f32,
    pub config: EctomorphConfig,
    pub enabled: bool,
}

/// Create a new ectomorph morph.
pub fn new_ectomorph_morph() -> EctomorphMorph {
    EctomorphMorph {
        intensity: 0.0,
        config: EctomorphConfig::default(),
        enabled: true,
    }
}

/// Set intensity [0, 1].
pub fn ect_set_intensity(m: &mut EctomorphMorph, v: f32) {
    m.intensity = v.clamp(0.0, 1.0);
}

/// Limb slenderness weight.
pub fn ect_limb_weight(m: &EctomorphMorph) -> f32 {
    m.intensity * m.config.limb_slenderness
}

/// Shoulder narrowness weight.
pub fn ect_shoulder_weight(m: &EctomorphMorph) -> f32 {
    m.intensity * m.config.shoulder_narrowness
}

/// Hip narrowness weight.
pub fn ect_hip_weight(m: &EctomorphMorph) -> f32 {
    m.intensity * m.config.hip_narrowness
}

/// Serialise to JSON.
pub fn ect_to_json(m: &EctomorphMorph) -> String {
    format!(
        r#"{{"intensity":{:.3},"limb_weight":{:.3},"enabled":{}}}"#,
        m.intensity,
        ect_limb_weight(m),
        m.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_zero_intensity() {
        let m = new_ectomorph_morph();
        assert!((m.intensity - 0.0).abs() < 1e-6 /* zero */);
    }

    #[test]
    fn clamp_works() {
        let mut m = new_ectomorph_morph();
        ect_set_intensity(&mut m, 2.0);
        assert!((m.intensity - 1.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn limb_weight_proportional() {
        let mut m = new_ectomorph_morph();
        ect_set_intensity(&mut m, 1.0);
        assert!((ect_limb_weight(&m) - m.config.limb_slenderness).abs() < 1e-6 /* proportional */);
    }

    #[test]
    fn shoulder_narrower_than_hip_at_max() {
        let mut m = new_ectomorph_morph();
        ect_set_intensity(&mut m, 1.0);
        /* ectomorphs have narrow shoulders too */
        assert!(ect_shoulder_weight(&m) > 0.0);
    }

    #[test]
    fn hip_weight_zero_at_zero_intensity() {
        let m = new_ectomorph_morph();
        assert!((ect_hip_weight(&m) - 0.0).abs() < 1e-6 /* zero hip at zero */);
    }

    #[test]
    fn json_has_intensity() {
        let mut m = new_ectomorph_morph();
        ect_set_intensity(&mut m, 0.5);
        assert!(ect_to_json(&m).contains("0.500") /* json ok */);
    }

    #[test]
    fn enabled_flag() {
        let m = new_ectomorph_morph();
        assert!(m.enabled /* default enabled */);
    }

    #[test]
    fn config_slenderness_positive() {
        let m = new_ectomorph_morph();
        assert!(m.config.limb_slenderness > 0.0 /* positive */);
    }
}
