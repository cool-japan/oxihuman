// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Skin specularity and gloss morph control.

/// Skin gloss morph configuration.
#[derive(Debug, Clone)]
pub struct SkinGlossMorph {
    pub specularity: f32,
    pub roughness: f32,
    pub oiliness: f32,
    pub highlight_sharpness: f32,
}

impl SkinGlossMorph {
    pub fn new() -> Self {
        Self {
            specularity: 0.3,
            roughness: 0.6,
            oiliness: 0.2,
            highlight_sharpness: 0.5,
        }
    }
}

impl Default for SkinGlossMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new skin gloss morph.
pub fn new_skin_gloss_morph() -> SkinGlossMorph {
    SkinGlossMorph::new()
}

/// Set specularity intensity.
pub fn skin_gloss_set_specularity(morph: &mut SkinGlossMorph, specularity: f32) {
    morph.specularity = specularity.clamp(0.0, 1.0);
}

/// Set roughness (inverse of gloss).
pub fn skin_gloss_set_roughness(morph: &mut SkinGlossMorph, roughness: f32) {
    morph.roughness = roughness.clamp(0.0, 1.0);
}

/// Set oiliness (sebum simulation).
pub fn skin_gloss_set_oiliness(morph: &mut SkinGlossMorph, oiliness: f32) {
    morph.oiliness = oiliness.clamp(0.0, 1.0);
}

/// Compute effective gloss as specularity * (1 - roughness).
pub fn skin_gloss_effective(morph: &SkinGlossMorph) -> f32 {
    (morph.specularity * (1.0 - morph.roughness)).clamp(0.0, 1.0)
}

/// Serialize to JSON-like string.
pub fn skin_gloss_morph_to_json(morph: &SkinGlossMorph) -> String {
    format!(
        r#"{{"specularity":{:.4},"roughness":{:.4},"oiliness":{:.4},"highlight_sharpness":{:.4}}}"#,
        morph.specularity, morph.roughness, morph.oiliness, morph.highlight_sharpness
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_skin_gloss_morph();
        assert!((m.specularity - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_specularity_clamp() {
        let mut m = new_skin_gloss_morph();
        skin_gloss_set_specularity(&mut m, 2.0);
        assert_eq!(m.specularity, 1.0);
    }

    #[test]
    fn test_roughness_set() {
        let mut m = new_skin_gloss_morph();
        skin_gloss_set_roughness(&mut m, 0.2);
        assert!((m.roughness - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_oiliness_set() {
        let mut m = new_skin_gloss_morph();
        skin_gloss_set_oiliness(&mut m, 0.8);
        assert!((m.oiliness - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_effective_gloss_range() {
        let m = new_skin_gloss_morph();
        let g = skin_gloss_effective(&m);
        assert!((0.0..=1.0).contains(&g));
    }

    #[test]
    fn test_effective_zero_roughness() {
        let mut m = new_skin_gloss_morph();
        skin_gloss_set_roughness(&mut m, 0.0);
        skin_gloss_set_specularity(&mut m, 1.0);
        assert!((skin_gloss_effective(&m) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_json() {
        let m = new_skin_gloss_morph();
        let s = skin_gloss_morph_to_json(&m);
        assert!(s.contains("oiliness"));
    }

    #[test]
    fn test_clone() {
        let m = new_skin_gloss_morph();
        let m2 = m.clone();
        assert!((m2.highlight_sharpness - m.highlight_sharpness).abs() < 1e-6);
    }

    #[test]
    fn test_default_trait() {
        let m: SkinGlossMorph = Default::default();
        assert!((m.roughness - 0.6).abs() < 1e-6);
    }
}
