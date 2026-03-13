// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Freckle distribution morph stub.

/// Freckle distribution pattern.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrecklePattern {
    Sparse,
    Moderate,
    Dense,
    Solar,
}

/// Freckle morph controller.
#[derive(Debug, Clone)]
pub struct FreckileMorph {
    pub pattern: FrecklePattern,
    pub density: f32,
    pub size: f32,
    pub morph_count: usize,
    pub enabled: bool,
}

impl FreckileMorph {
    pub fn new(morph_count: usize) -> Self {
        FreckileMorph {
            pattern: FrecklePattern::Sparse,
            density: 0.3,
            size: 0.5,
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new freckle morph.
pub fn new_freckle_morph(morph_count: usize) -> FreckileMorph {
    FreckileMorph::new(morph_count)
}

/// Set freckle pattern.
pub fn fkm_set_pattern(morph: &mut FreckileMorph, pattern: FrecklePattern) {
    morph.pattern = pattern;
}

/// Set freckle density.
pub fn fkm_set_density(morph: &mut FreckileMorph, density: f32) {
    morph.density = density.clamp(0.0, 1.0);
}

/// Set freckle size factor.
pub fn fkm_set_size(morph: &mut FreckileMorph, size: f32) {
    morph.size = size.clamp(0.0, 1.0);
}

/// Evaluate morph weights (stub: uniform from density).
pub fn fkm_evaluate(morph: &FreckileMorph) -> Vec<f32> {
    /* Stub: uniform weight from density */
    if !morph.enabled || morph.morph_count == 0 {
        return vec![];
    }
    vec![morph.density; morph.morph_count]
}

/// Enable or disable.
pub fn fkm_set_enabled(morph: &mut FreckileMorph, enabled: bool) {
    morph.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn fkm_to_json(morph: &FreckileMorph) -> String {
    let pat = match morph.pattern {
        FrecklePattern::Sparse => "sparse",
        FrecklePattern::Moderate => "moderate",
        FrecklePattern::Dense => "dense",
        FrecklePattern::Solar => "solar",
    };
    format!(
        r#"{{"pattern":"{}","density":{},"size":{},"morph_count":{},"enabled":{}}}"#,
        pat, morph.density, morph.size, morph.morph_count, morph.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_pattern() {
        let m = new_freckle_morph(4);
        assert_eq!(
            m.pattern,
            FrecklePattern::Sparse /* default pattern must be Sparse */
        );
    }

    #[test]
    fn test_set_pattern() {
        let mut m = new_freckle_morph(4);
        fkm_set_pattern(&mut m, FrecklePattern::Dense);
        assert_eq!(
            m.pattern,
            FrecklePattern::Dense /* pattern must be set */
        );
    }

    #[test]
    fn test_density_clamp() {
        let mut m = new_freckle_morph(4);
        fkm_set_density(&mut m, 1.5);
        assert!((m.density - 1.0).abs() < 1e-6 /* density clamped to 1.0 */);
    }

    #[test]
    fn test_size_clamp() {
        let mut m = new_freckle_morph(4);
        fkm_set_size(&mut m, -0.5);
        assert!(m.size.abs() < 1e-6 /* size clamped to 0.0 */);
    }

    #[test]
    fn test_evaluate_length() {
        let m = new_freckle_morph(5);
        assert_eq!(
            fkm_evaluate(&m).len(),
            5 /* output must match morph_count */
        );
    }

    #[test]
    fn test_evaluate_disabled() {
        let mut m = new_freckle_morph(4);
        fkm_set_enabled(&mut m, false);
        assert!(fkm_evaluate(&m).is_empty() /* disabled must return empty */);
    }

    #[test]
    fn test_to_json_has_pattern() {
        let m = new_freckle_morph(4);
        let j = fkm_to_json(&m);
        assert!(j.contains("\"pattern\"") /* JSON must have pattern */);
    }

    #[test]
    fn test_enabled_default() {
        let m = new_freckle_morph(4);
        assert!(m.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_evaluate_matches_density() {
        let mut m = new_freckle_morph(3);
        fkm_set_density(&mut m, 0.6);
        let out = fkm_evaluate(&m);
        assert!((out[0] - 0.6).abs() < 1e-5 /* evaluate must match density */);
    }
}
