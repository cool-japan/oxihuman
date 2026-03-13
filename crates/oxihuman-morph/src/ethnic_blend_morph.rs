// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Ethnic feature blend morph stub.

/// An ethnic feature set with associated morph weights.
#[derive(Debug, Clone)]
pub struct EthnicFeatureSet {
    pub name: String,
    pub morph_weights: Vec<f32>,
}

/// Ethnic blend morph controller.
#[derive(Debug, Clone)]
pub struct EthnicBlendMorph {
    pub feature_sets: Vec<EthnicFeatureSet>,
    pub blend_weights: Vec<f32>,
    pub morph_count: usize,
    pub enabled: bool,
}

impl EthnicBlendMorph {
    pub fn new(morph_count: usize) -> Self {
        EthnicBlendMorph {
            feature_sets: Vec::new(),
            blend_weights: Vec::new(),
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new ethnic blend morph controller.
pub fn new_ethnic_blend_morph(morph_count: usize) -> EthnicBlendMorph {
    EthnicBlendMorph::new(morph_count)
}

/// Add an ethnic feature set.
pub fn ebmrph_add_feature_set(morph: &mut EthnicBlendMorph, feature_set: EthnicFeatureSet) {
    morph.blend_weights.push(0.0);
    morph.feature_sets.push(feature_set);
}

/// Set the blend weight for a feature set.
pub fn ebmrph_set_blend_weight(morph: &mut EthnicBlendMorph, idx: usize, weight: f32) {
    if idx < morph.blend_weights.len() {
        morph.blend_weights[idx] = weight.clamp(0.0, 1.0);
    }
}

/// Evaluate blended morph weights (stub: zeroed).
pub fn ebmrph_evaluate(morph: &EthnicBlendMorph) -> Vec<f32> {
    /* Stub: returns zeroed output */
    vec![0.0; morph.morph_count]
}

/// Return feature set count.
pub fn ebmrph_feature_count(morph: &EthnicBlendMorph) -> usize {
    morph.feature_sets.len()
}

/// Enable or disable.
pub fn ebmrph_set_enabled(morph: &mut EthnicBlendMorph, enabled: bool) {
    morph.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn ebmrph_to_json(morph: &EthnicBlendMorph) -> String {
    format!(
        r#"{{"morph_count":{},"feature_sets":{},"enabled":{}}}"#,
        morph.morph_count,
        morph.feature_sets.len(),
        morph.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_morph_count() {
        let m = new_ethnic_blend_morph(16);
        assert_eq!(m.morph_count, 16 /* morph count must match */,);
    }

    #[test]
    fn test_no_features_initially() {
        let m = new_ethnic_blend_morph(8);
        assert_eq!(
            ebmrph_feature_count(&m),
            0, /* no feature sets initially */
        );
    }

    #[test]
    fn test_add_feature_set() {
        let mut m = new_ethnic_blend_morph(8);
        ebmrph_add_feature_set(
            &mut m,
            EthnicFeatureSet {
                name: "set_a".into(),
                morph_weights: vec![0.0; 8],
            },
        );
        assert_eq!(
            ebmrph_feature_count(&m),
            1, /* one feature set after add */
        );
    }

    #[test]
    fn test_blend_weight_init_zero() {
        let mut m = new_ethnic_blend_morph(4);
        ebmrph_add_feature_set(
            &mut m,
            EthnicFeatureSet {
                name: "x".into(),
                morph_weights: vec![0.0; 4],
            },
        );
        assert!((m.blend_weights[0]).abs() < 1e-6, /* initial blend weight must be 0 */);
    }

    #[test]
    fn test_set_blend_weight() {
        let mut m = new_ethnic_blend_morph(4);
        ebmrph_add_feature_set(
            &mut m,
            EthnicFeatureSet {
                name: "x".into(),
                morph_weights: vec![0.0; 4],
            },
        );
        ebmrph_set_blend_weight(&mut m, 0, 0.7);
        assert!((m.blend_weights[0] - 0.7).abs() < 1e-5, /* weight must be set */);
    }

    #[test]
    fn test_blend_weight_clamped() {
        let mut m = new_ethnic_blend_morph(4);
        ebmrph_add_feature_set(
            &mut m,
            EthnicFeatureSet {
                name: "x".into(),
                morph_weights: vec![0.0; 4],
            },
        );
        ebmrph_set_blend_weight(&mut m, 0, 2.0);
        assert!((m.blend_weights[0] - 1.0).abs() < 1e-6, /* weight clamped to 1.0 */);
    }

    #[test]
    fn test_evaluate_output_length() {
        let m = new_ethnic_blend_morph(10);
        let out = ebmrph_evaluate(&m);
        assert_eq!(
            out.len(),
            10, /* output length must match morph_count */
        );
    }

    #[test]
    fn test_set_enabled() {
        let mut m = new_ethnic_blend_morph(4);
        ebmrph_set_enabled(&mut m, false);
        assert!(!m.enabled /* must be disabled */,);
    }

    #[test]
    fn test_to_json_contains_morph_count() {
        let m = new_ethnic_blend_morph(5);
        let j = ebmrph_to_json(&m);
        assert!(j.contains("\"morph_count\""), /* json must contain morph_count */);
    }

    #[test]
    fn test_enabled_default() {
        let m = new_ethnic_blend_morph(1);
        assert!(m.enabled /* must be enabled by default */,);
    }

    #[test]
    fn test_out_of_bounds_set_ignored() {
        let mut m = new_ethnic_blend_morph(4);
        ebmrph_set_blend_weight(&mut m, 99, 1.0);
        assert_eq!(
            ebmrph_feature_count(&m),
            0, /* out of bounds set must be ignored */
        );
    }
}
