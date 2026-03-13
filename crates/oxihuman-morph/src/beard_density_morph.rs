// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Beard and facial hair density morph control.

/// Beard density morph configuration.
#[derive(Debug, Clone)]
pub struct BeardDensityMorph {
    pub density: f32,
    pub coarseness: f32,
    pub coverage: f32,
}

impl BeardDensityMorph {
    pub fn new() -> Self {
        Self {
            density: 0.5,
            coarseness: 0.5,
            coverage: 0.5,
        }
    }
}

impl Default for BeardDensityMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new beard density morph.
pub fn new_beard_density_morph() -> BeardDensityMorph {
    BeardDensityMorph::new()
}

/// Set hair strand density.
pub fn beard_density_set_density(morph: &mut BeardDensityMorph, density: f32) {
    morph.density = density.clamp(0.0, 1.0);
}

/// Set coarseness of hair strands.
pub fn beard_density_set_coarseness(morph: &mut BeardDensityMorph, coarseness: f32) {
    morph.coarseness = coarseness.clamp(0.0, 1.0);
}

/// Set facial coverage area in normalized range [0, 1].
pub fn beard_density_set_coverage(morph: &mut BeardDensityMorph, coverage: f32) {
    morph.coverage = coverage.clamp(0.0, 1.0);
}

/// Compute approximate visual thickness from density and coarseness.
pub fn beard_density_visual_thickness(morph: &BeardDensityMorph) -> f32 {
    (morph.density * 0.6 + morph.coarseness * 0.4).clamp(0.0, 1.0)
}

/// Serialize to JSON-like string.
pub fn beard_density_morph_to_json(morph: &BeardDensityMorph) -> String {
    format!(
        r#"{{"density":{:.4},"coarseness":{:.4},"coverage":{:.4}}}"#,
        morph.density, morph.coarseness, morph.coverage
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_beard_density_morph();
        assert!((m.density - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_density_clamp() {
        let mut m = new_beard_density_morph();
        beard_density_set_density(&mut m, 3.0);
        assert_eq!(m.density, 1.0);
    }

    #[test]
    fn test_coarseness() {
        let mut m = new_beard_density_morph();
        beard_density_set_coarseness(&mut m, 0.9);
        assert!((m.coarseness - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_coverage() {
        let mut m = new_beard_density_morph();
        beard_density_set_coverage(&mut m, 0.2);
        assert!((m.coverage - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_visual_thickness_range() {
        let m = new_beard_density_morph();
        let t = beard_density_visual_thickness(&m);
        assert!((0.0..=1.0).contains(&t));
    }

    #[test]
    fn test_json() {
        let m = new_beard_density_morph();
        let s = beard_density_morph_to_json(&m);
        assert!(s.contains("coarseness"));
    }

    #[test]
    fn test_clone() {
        let m = new_beard_density_morph();
        let m2 = m.clone();
        assert!((m2.coverage - m.coverage).abs() < 1e-6);
    }

    #[test]
    fn test_coverage_clamp_low() {
        let mut m = new_beard_density_morph();
        beard_density_set_coverage(&mut m, -1.0);
        assert_eq!(m.coverage, 0.0);
    }

    #[test]
    fn test_default_trait() {
        let m: BeardDensityMorph = Default::default();
        assert!((m.coarseness - 0.5).abs() < 1e-6);
    }
}
