// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Gender morphology blend system: manages masculinity/femininity morph weights.

#![allow(dead_code)]

// ── Types ────────────────────────────────────────────────────────────────────

/// Configuration for gender morphology blending.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GenderMorphConfig {
    /// Clamp output weights to 0..1.
    pub clamp_output: bool,
    /// Number of morph features driven by gender.
    pub feature_count: usize,
    /// Names of the morph targets controlled by this system.
    pub target_names: Vec<String>,
}

/// Runtime state for gender morphology.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GenderMorphState {
    pub config: GenderMorphConfig,
    /// Masculinity value 0..1.
    pub masculinity: f32,
    /// Femininity value 0..1.
    pub femininity: f32,
    /// Computed morph weights per target.
    pub morph_weights: Vec<f32>,
}

/// Combined gender feature vector type alias.
pub type GenderFeatureVec = Vec<f32>;

/// Morph weight map type alias.
pub type MorphWeightMap = Vec<(String, f32)>;

// ── Constructors ─────────────────────────────────────────────────────────────

/// Create a default `GenderMorphConfig`.
#[allow(dead_code)]
pub fn default_gender_config() -> GenderMorphConfig {
    let target_names = vec![
        "gender_jaw_width".to_string(),
        "gender_brow_ridge".to_string(),
        "gender_cheekbone".to_string(),
        "gender_lip_fullness".to_string(),
        "gender_nose_width".to_string(),
        "gender_shoulder_width".to_string(),
        "gender_hip_width".to_string(),
        "gender_chest".to_string(),
    ];
    let feature_count = target_names.len();
    GenderMorphConfig {
        clamp_output: true,
        feature_count,
        target_names,
    }
}

/// Create a new `GenderMorphState` from a config.
#[allow(dead_code)]
pub fn new_gender_morph_state(config: GenderMorphConfig) -> GenderMorphState {
    let n = config.feature_count;
    GenderMorphState {
        config,
        masculinity: 0.5,
        femininity: 0.5,
        morph_weights: vec![0.0; n],
    }
}

// ── Setters ──────────────────────────────────────────────────────────────────

/// Set masculinity (0..1), automatically clamped.
#[allow(dead_code)]
pub fn set_masculinity(state: &mut GenderMorphState, value: f32) {
    state.masculinity = value.clamp(0.0, 1.0);
}

/// Set femininity (0..1), automatically clamped.
#[allow(dead_code)]
pub fn set_femininity(state: &mut GenderMorphState, value: f32) {
    state.femininity = value.clamp(0.0, 1.0);
}

// ── Queries ──────────────────────────────────────────────────────────────────

/// Return the blend factor between masculine (0.0) and feminine (1.0).
#[allow(dead_code)]
pub fn gender_blend_factor(state: &GenderMorphState) -> f32 {
    let sum = state.masculinity + state.femininity;
    if sum < f32::EPSILON {
        0.5
    } else {
        state.femininity / sum
    }
}

/// Compute morph weights for each target based on current gender blend.
#[allow(dead_code)]
pub fn gender_to_morph_weights(state: &GenderMorphState) -> MorphWeightMap {
    let blend = gender_blend_factor(state);
    // Masculine targets peak at blend=0, feminine at blend=1.
    // For the first half of targets we treat them as masculine-leaning,
    // the second half as feminine-leaning.
    let n = state.config.feature_count;
    let mut out = Vec::with_capacity(n);
    for (i, name) in state.config.target_names.iter().enumerate() {
        let w = if i < n / 2 {
            1.0 - blend
        } else {
            blend
        };
        let w = if state.config.clamp_output {
            w.clamp(0.0, 1.0)
        } else {
            w
        };
        out.push((name.clone(), w));
    }
    out
}

/// Blend two gender states by `t` (0 = a, 1 = b).
#[allow(dead_code)]
pub fn blend_gender_states(a: &GenderMorphState, b: &GenderMorphState, t: f32) -> GenderMorphState {
    let t = t.clamp(0.0, 1.0);
    let mut out = a.clone();
    out.masculinity = a.masculinity + (b.masculinity - a.masculinity) * t;
    out.femininity = a.femininity + (b.femininity - a.femininity) * t;
    out
}

/// Normalize masculinity + femininity so they sum to 1.0.
#[allow(dead_code)]
pub fn normalize_gender_blend(state: &mut GenderMorphState) {
    let sum = state.masculinity + state.femininity;
    if sum > f32::EPSILON {
        state.masculinity /= sum;
        state.femininity /= sum;
    }
}

/// Return a feature vector [masculinity, femininity, blend_factor, symmetry_factor].
#[allow(dead_code)]
pub fn gender_feature_vector(state: &GenderMorphState) -> GenderFeatureVec {
    vec![
        state.masculinity,
        state.femininity,
        gender_blend_factor(state),
        gender_symmetry_factor(state),
    ]
}

/// Reset gender morph to neutral (both 0.5).
#[allow(dead_code)]
pub fn reset_gender_morph(state: &mut GenderMorphState) {
    state.masculinity = 0.5;
    state.femininity = 0.5;
    for w in &mut state.morph_weights {
        *w = 0.0;
    }
}

/// Apply gender blend to external proportion params (returns scale factors).
/// Returns `[shoulder_scale, hip_scale, chest_scale]`.
#[allow(dead_code)]
pub fn apply_gender_to_proportions(state: &GenderMorphState) -> [f32; 3] {
    let blend = gender_blend_factor(state);
    // 0 = fully masculine, 1 = fully feminine
    let shoulder = 1.0 + (1.0 - blend) * 0.15;
    let hip = 1.0 + blend * 0.20;
    let chest = 1.0 + blend * 0.25;
    [shoulder, hip, chest]
}

/// Return how symmetric the gender blend is (1.0 = perfectly balanced 50/50).
#[allow(dead_code)]
pub fn gender_symmetry_factor(state: &GenderMorphState) -> f32 {
    let diff = (state.masculinity - state.femininity).abs();
    1.0 - diff.min(1.0)
}

/// Return a human-readable name for the current gender blend.
#[allow(dead_code)]
pub fn gender_morph_name(state: &GenderMorphState) -> &'static str {
    let blend = gender_blend_factor(state);
    if blend < 0.2 {
        "masculine"
    } else if blend > 0.8 {
        "feminine"
    } else {
        "androgynous"
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state() -> GenderMorphState {
        new_gender_morph_state(default_gender_config())
    }

    #[test]
    fn test_default_config_has_targets() {
        let cfg = default_gender_config();
        assert!(!cfg.target_names.is_empty());
        assert_eq!(cfg.feature_count, cfg.target_names.len());
    }

    #[test]
    fn test_new_state_neutral() {
        let s = make_state();
        assert!((s.masculinity - 0.5).abs() < 1e-6);
        assert!((s.femininity - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_masculinity_clamp() {
        let mut s = make_state();
        set_masculinity(&mut s, 2.0);
        assert!((s.masculinity - 1.0).abs() < 1e-6);
        set_masculinity(&mut s, -1.0);
        assert!(s.masculinity.abs() < 1e-6);
    }

    #[test]
    fn test_set_femininity_clamp() {
        let mut s = make_state();
        set_femininity(&mut s, -5.0);
        assert!(s.femininity.abs() < 1e-6);
        set_femininity(&mut s, 5.0);
        assert!((s.femininity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_gender_blend_factor_neutral() {
        let s = make_state();
        let f = gender_blend_factor(&s);
        assert!((f - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_gender_blend_factor_pure_masc() {
        let mut s = make_state();
        s.masculinity = 1.0;
        s.femininity = 0.0;
        assert!(gender_blend_factor(&s) < 1e-6);
    }

    #[test]
    fn test_gender_blend_factor_pure_fem() {
        let mut s = make_state();
        s.masculinity = 0.0;
        s.femininity = 1.0;
        assert!((gender_blend_factor(&s) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_gender_to_morph_weights_count() {
        let s = make_state();
        let w = gender_to_morph_weights(&s);
        assert_eq!(w.len(), s.config.feature_count);
    }

    #[test]
    fn test_gender_to_morph_weights_clamped() {
        let s = make_state();
        for (_, w) in gender_to_morph_weights(&s) {
            assert!((0.0..=1.0).contains(&w));
        }
    }

    #[test]
    fn test_blend_gender_states() {
        let mut a = make_state();
        let mut b = make_state();
        a.masculinity = 1.0;
        a.femininity = 0.0;
        b.masculinity = 0.0;
        b.femininity = 1.0;
        let mid = blend_gender_states(&a, &b, 0.5);
        assert!((mid.masculinity - 0.5).abs() < 1e-5);
        assert!((mid.femininity - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_gender_blend() {
        let mut s = make_state();
        s.masculinity = 2.0;
        s.femininity = 2.0;
        normalize_gender_blend(&mut s);
        assert!((s.masculinity + s.femininity - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_gender_feature_vector_len() {
        let s = make_state();
        let v = gender_feature_vector(&s);
        assert_eq!(v.len(), 4);
    }

    #[test]
    fn test_reset_gender_morph() {
        let mut s = make_state();
        s.masculinity = 0.9;
        s.femininity = 0.1;
        reset_gender_morph(&mut s);
        assert!((s.masculinity - 0.5).abs() < 1e-6);
        assert!((s.femininity - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_apply_gender_to_proportions() {
        let s = make_state();
        let [sh, hip, ch] = apply_gender_to_proportions(&s);
        assert!(sh > 1.0);
        assert!(hip > 1.0);
        assert!(ch > 1.0);
    }

    #[test]
    fn test_gender_symmetry_factor_neutral() {
        let s = make_state();
        assert!((gender_symmetry_factor(&s) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_gender_symmetry_factor_extreme() {
        let mut s = make_state();
        s.masculinity = 1.0;
        s.femininity = 0.0;
        assert!(gender_symmetry_factor(&s) < 1e-5);
    }

    #[test]
    fn test_gender_morph_name_masculine() {
        let mut s = make_state();
        s.masculinity = 1.0;
        s.femininity = 0.0;
        assert_eq!(gender_morph_name(&s), "masculine");
    }

    #[test]
    fn test_gender_morph_name_feminine() {
        let mut s = make_state();
        s.masculinity = 0.0;
        s.femininity = 1.0;
        assert_eq!(gender_morph_name(&s), "feminine");
    }

    #[test]
    fn test_gender_morph_name_androgynous() {
        let s = make_state();
        assert_eq!(gender_morph_name(&s), "androgynous");
    }
}
