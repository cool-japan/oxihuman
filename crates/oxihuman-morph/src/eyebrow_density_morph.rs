// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Eyebrow hair density morph control.

/// Eyebrow density morph configuration.
#[derive(Debug, Clone)]
pub struct EyebrowDensityMorph {
    pub density: f32,
    pub fullness: f32,
    pub gap_fill: f32,
}

impl EyebrowDensityMorph {
    pub fn new() -> Self {
        Self {
            density: 0.5,
            fullness: 0.5,
            gap_fill: 0.0,
        }
    }
}

impl Default for EyebrowDensityMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new eyebrow density morph.
pub fn new_eyebrow_density_morph() -> EyebrowDensityMorph {
    EyebrowDensityMorph::new()
}

/// Set hair strand density for eyebrows.
pub fn ebrow_density_set_density(morph: &mut EyebrowDensityMorph, density: f32) {
    morph.density = density.clamp(0.0, 1.0);
}

/// Set perceived fullness of the eyebrow.
pub fn ebrow_density_set_fullness(morph: &mut EyebrowDensityMorph, fullness: f32) {
    morph.fullness = fullness.clamp(0.0, 1.0);
}

/// Set gap fill factor between strands.
pub fn ebrow_density_set_gap_fill(morph: &mut EyebrowDensityMorph, gap_fill: f32) {
    morph.gap_fill = gap_fill.clamp(0.0, 1.0);
}

/// Serialize to JSON-like string.
pub fn eyebrow_density_morph_to_json(morph: &EyebrowDensityMorph) -> String {
    format!(
        r#"{{"density":{:.4},"fullness":{:.4},"gap_fill":{:.4}}}"#,
        morph.density, morph.fullness, morph.gap_fill
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_eyebrow_density_morph();
        assert!((m.density - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_density_clamp_high() {
        let mut m = new_eyebrow_density_morph();
        ebrow_density_set_density(&mut m, 2.0);
        assert_eq!(m.density, 1.0);
    }

    #[test]
    fn test_density_clamp_low() {
        let mut m = new_eyebrow_density_morph();
        ebrow_density_set_density(&mut m, -0.5);
        assert_eq!(m.density, 0.0);
    }

    #[test]
    fn test_fullness() {
        let mut m = new_eyebrow_density_morph();
        ebrow_density_set_fullness(&mut m, 0.8);
        assert!((m.fullness - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_gap_fill() {
        let mut m = new_eyebrow_density_morph();
        ebrow_density_set_gap_fill(&mut m, 0.4);
        assert!((m.gap_fill - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_json() {
        let m = new_eyebrow_density_morph();
        let s = eyebrow_density_morph_to_json(&m);
        assert!(s.contains("density"));
        assert!(s.contains("fullness"));
    }

    #[test]
    fn test_clone() {
        let m = new_eyebrow_density_morph();
        let m2 = m.clone();
        assert!((m2.fullness - m.fullness).abs() < 1e-6);
    }

    #[test]
    fn test_default_trait() {
        let m: EyebrowDensityMorph = Default::default();
        assert!((m.gap_fill - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_json_gap_fill() {
        let m = new_eyebrow_density_morph();
        let s = eyebrow_density_morph_to_json(&m);
        assert!(s.contains("gap_fill"));
    }
}
