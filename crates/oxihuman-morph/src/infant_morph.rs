// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Infant/baby body proportion morph.

/// Configuration for the infant morph.
#[derive(Debug, Clone)]
pub struct InfantMorphConfig {
    /// Head-to-body ratio multiplier (infants ~1/4 body height).
    pub head_ratio: f32,
    /// Limb shortening factor relative to adult.
    pub limb_scale: f32,
    /// Belly roundness.
    pub belly_roundness: f32,
}

impl Default for InfantMorphConfig {
    fn default() -> Self {
        InfantMorphConfig {
            head_ratio: 1.0,
            limb_scale: 0.45,
            belly_roundness: 0.8,
        }
    }
}

/// State for the infant body morph.
#[derive(Debug, Clone)]
pub struct InfantMorph {
    /// Age in months (0–24).
    pub age_months: f32,
    pub config: InfantMorphConfig,
    pub enabled: bool,
}

/// Create a new infant morph at birth (0 months).
pub fn new_infant_morph() -> InfantMorph {
    InfantMorph {
        age_months: 0.0,
        config: InfantMorphConfig::default(),
        enabled: true,
    }
}

/// Set age in months (clamped to 0–24).
pub fn im_set_age(m: &mut InfantMorph, months: f32) {
    m.age_months = months.clamp(0.0, 24.0);
}

/// Normalised morphing weight (0 = newborn, 1 = 24 months).
pub fn im_weight(m: &InfantMorph) -> f32 {
    m.age_months / 24.0
}

/// Head scale factor (infants have proportionally larger heads).
pub fn im_head_scale(m: &InfantMorph) -> f32 {
    let t = im_weight(m);
    m.config.head_ratio * (1.0 - 0.3 * t)
}

/// Limb length scale relative to adult norm.
pub fn im_limb_scale(m: &InfantMorph) -> f32 {
    let t = im_weight(m);
    m.config.limb_scale + 0.1 * t
}

/// Serialise to JSON string.
pub fn im_to_json(m: &InfantMorph) -> String {
    format!(
        r#"{{"age_months":{:.1},"enabled":{},"head_scale":{:.3}}}"#,
        m.age_months,
        m.enabled,
        im_head_scale(m)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_newborn() {
        let m = new_infant_morph();
        assert!((m.age_months - 0.0).abs() < 1e-6 /* newborn age */);
    }

    #[test]
    fn set_age_clamps_upper() {
        let mut m = new_infant_morph();
        im_set_age(&mut m, 100.0);
        assert!((m.age_months - 24.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn weight_at_24_months_is_one() {
        let mut m = new_infant_morph();
        im_set_age(&mut m, 24.0);
        assert!((im_weight(&m) - 1.0).abs() < 1e-6 /* full weight */);
    }

    #[test]
    fn head_scale_decreases_with_age() {
        let mut m = new_infant_morph();
        im_set_age(&mut m, 0.0);
        let h0 = im_head_scale(&m);
        im_set_age(&mut m, 24.0);
        let h24 = im_head_scale(&m);
        assert!(h0 > h24 /* head relatively larger at birth */);
    }

    #[test]
    fn limb_scale_increases_with_age() {
        let mut m = new_infant_morph();
        im_set_age(&mut m, 0.0);
        let l0 = im_limb_scale(&m);
        im_set_age(&mut m, 24.0);
        let l24 = im_limb_scale(&m);
        assert!(l24 > l0 /* limbs grow with age */);
    }

    #[test]
    fn to_json_has_age() {
        let mut m = new_infant_morph();
        im_set_age(&mut m, 6.0);
        assert!(im_to_json(&m).contains("6.0") /* json has age */);
    }

    #[test]
    fn enabled_toggle() {
        let mut m = new_infant_morph();
        m.enabled = false;
        assert!(!m.enabled /* disabled */);
    }

    #[test]
    fn belly_roundness_in_config() {
        let m = new_infant_morph();
        assert!(m.config.belly_roundness > 0.0 /* non-zero belly roundness */);
    }
}
