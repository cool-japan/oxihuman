// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Skin translucency morph control — back-lighting transmission effect.

/// Skin translucency morph configuration.
#[derive(Debug, Clone)]
pub struct SkinTranslucencyMorph {
    pub translucency: f32,
    pub thin_skin_factor: f32,
    pub vein_visibility: f32,
}

impl SkinTranslucencyMorph {
    pub fn new() -> Self {
        Self {
            translucency: 0.4,
            thin_skin_factor: 0.3,
            vein_visibility: 0.1,
        }
    }
}

impl Default for SkinTranslucencyMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new skin translucency morph.
pub fn new_skin_translucency_morph() -> SkinTranslucencyMorph {
    SkinTranslucencyMorph::new()
}

/// Set overall translucency level.
pub fn skin_trans_set_translucency(morph: &mut SkinTranslucencyMorph, translucency: f32) {
    morph.translucency = translucency.clamp(0.0, 1.0);
}

/// Set thin-skin factor (ear/hand regions).
pub fn skin_trans_set_thin_skin(morph: &mut SkinTranslucencyMorph, factor: f32) {
    morph.thin_skin_factor = factor.clamp(0.0, 1.0);
}

/// Set vein visibility through skin.
pub fn skin_trans_set_vein_visibility(morph: &mut SkinTranslucencyMorph, visibility: f32) {
    morph.vein_visibility = visibility.clamp(0.0, 1.0);
}

/// Compute light bleed factor from translucency and thin-skin.
pub fn skin_trans_light_bleed(morph: &SkinTranslucencyMorph) -> f32 {
    (morph.translucency * 0.7 + morph.thin_skin_factor * 0.3).clamp(0.0, 1.0)
}

/// Serialize to JSON-like string.
pub fn skin_translucency_morph_to_json(morph: &SkinTranslucencyMorph) -> String {
    format!(
        r#"{{"translucency":{:.4},"thin_skin_factor":{:.4},"vein_visibility":{:.4}}}"#,
        morph.translucency, morph.thin_skin_factor, morph.vein_visibility
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_skin_translucency_morph();
        assert!((m.translucency - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_translucency_clamp() {
        let mut m = new_skin_translucency_morph();
        skin_trans_set_translucency(&mut m, 2.0);
        assert_eq!(m.translucency, 1.0);
    }

    #[test]
    fn test_thin_skin_set() {
        let mut m = new_skin_translucency_morph();
        skin_trans_set_thin_skin(&mut m, 0.7);
        assert!((m.thin_skin_factor - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_vein_visibility() {
        let mut m = new_skin_translucency_morph();
        skin_trans_set_vein_visibility(&mut m, 0.5);
        assert!((m.vein_visibility - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_light_bleed_range() {
        let m = new_skin_translucency_morph();
        let lb = skin_trans_light_bleed(&m);
        assert!((0.0..=1.0).contains(&lb));
    }

    #[test]
    fn test_json() {
        let m = new_skin_translucency_morph();
        let s = skin_translucency_morph_to_json(&m);
        assert!(s.contains("thin_skin_factor"));
    }

    #[test]
    fn test_clone() {
        let m = new_skin_translucency_morph();
        let m2 = m.clone();
        assert!((m2.vein_visibility - m.vein_visibility).abs() < 1e-6);
    }

    #[test]
    fn test_vein_clamp_low() {
        let mut m = new_skin_translucency_morph();
        skin_trans_set_vein_visibility(&mut m, -0.5);
        assert_eq!(m.vein_visibility, 0.0);
    }

    #[test]
    fn test_default_trait() {
        let m: SkinTranslucencyMorph = Default::default();
        assert!((m.thin_skin_factor - 0.3).abs() < 1e-6);
    }
}
