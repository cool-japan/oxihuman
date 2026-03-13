#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Blend of ethnic facial features.

/// A named set of ethnic feature morph weights.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EthnicFeatureSet {
    pub set_id: u32,
    pub name: String,
    pub weights: Vec<f32>,
}

/// A weighted collection of `EthnicFeatureSet` values.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct EthnicBlend {
    pub sets: Vec<(EthnicFeatureSet, f32)>,
}

/// Create an empty `EthnicBlend`.
#[allow(dead_code)]
pub fn new_ethnic_blend() -> EthnicBlend {
    EthnicBlend::default()
}

/// Add an ethnic feature set with a given influence weight.
#[allow(dead_code)]
pub fn add_ethnic_set(
    blend: &mut EthnicBlend,
    id: u32,
    name: &str,
    weights: Vec<f32>,
    influence: f32,
) {
    let efs = EthnicFeatureSet {
        set_id: id,
        name: name.to_string(),
        weights,
    };
    blend.sets.push((efs, influence.max(0.0)));
}

/// Compute the blended morph weight vector by summing influence-weighted sets.
///
/// Returns an empty `Vec` if `blend.sets` is empty.
#[allow(dead_code)]
pub fn compute_ethnic_blend(blend: &EthnicBlend) -> Vec<f32> {
    if blend.sets.is_empty() {
        return Vec::new();
    }
    let max_len = blend.sets.iter().map(|(s, _)| s.weights.len()).max().unwrap_or(0);
    if max_len == 0 {
        return Vec::new();
    }
    let mut result = vec![0.0_f32; max_len];
    let total_inf: f32 = blend.sets.iter().map(|(_, inf)| inf).sum();
    let denom = if total_inf > 1e-9 { total_inf } else { 1.0 };
    for (efs, inf) in &blend.sets {
        let factor = inf / denom;
        for (i, &w) in efs.weights.iter().enumerate() {
            result[i] += w * factor;
        }
    }
    result
}

/// Normalise all influence values in `blend` so they sum to 1.0.
#[allow(dead_code)]
pub fn normalize_influences(blend: &mut EthnicBlend) {
    let total: f32 = blend.sets.iter().map(|(_, inf)| *inf).sum();
    if total < 1e-9 {
        return;
    }
    for (_, inf) in blend.sets.iter_mut() {
        *inf /= total;
    }
}

/// Return the number of ethnic sets in the blend.
#[allow(dead_code)]
pub fn set_count(blend: &EthnicBlend) -> usize {
    blend.sets.len()
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_blend_is_empty() {
        let b = new_ethnic_blend();
        assert_eq!(set_count(&b), 0);
    }

    #[test]
    fn add_ethnic_set_increments_count() {
        let mut b = new_ethnic_blend();
        add_ethnic_set(&mut b, 1, "SetA", vec![0.5, 0.5], 1.0);
        assert_eq!(set_count(&b), 1);
    }

    #[test]
    fn compute_blend_empty_returns_empty() {
        let b = new_ethnic_blend();
        assert!(compute_ethnic_blend(&b).is_empty());
    }

    #[test]
    fn compute_blend_single_set_identity() {
        let mut b = new_ethnic_blend();
        add_ethnic_set(&mut b, 1, "A", vec![0.4, 0.6], 1.0);
        let result = compute_ethnic_blend(&b);
        assert!((result[0] - 0.4).abs() < 1e-5);
        assert!((result[1] - 0.6).abs() < 1e-5);
    }

    #[test]
    fn compute_blend_two_equal_sets() {
        let mut b = new_ethnic_blend();
        add_ethnic_set(&mut b, 1, "A", vec![1.0, 0.0], 1.0);
        add_ethnic_set(&mut b, 2, "B", vec![0.0, 1.0], 1.0);
        let result = compute_ethnic_blend(&b);
        assert!((result[0] - 0.5).abs() < 1e-5);
        assert!((result[1] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn normalize_influences_sums_to_one() {
        let mut b = new_ethnic_blend();
        add_ethnic_set(&mut b, 1, "A", vec![1.0], 2.0);
        add_ethnic_set(&mut b, 2, "B", vec![0.0], 3.0);
        normalize_influences(&mut b);
        let total: f32 = b.sets.iter().map(|(_, inf)| inf).sum();
        assert!((total - 1.0).abs() < 1e-5);
    }

    #[test]
    fn normalize_influences_noop_when_empty() {
        let mut b = new_ethnic_blend();
        normalize_influences(&mut b); // must not panic
    }

    #[test]
    fn add_ethnic_set_negative_influence_clamped() {
        let mut b = new_ethnic_blend();
        add_ethnic_set(&mut b, 1, "A", vec![0.5], -5.0);
        assert!((b.sets[0].1 - 0.0).abs() < 1e-6);
    }

    #[test]
    fn set_count_multiple() {
        let mut b = new_ethnic_blend();
        for i in 0..5u32 {
            add_ethnic_set(&mut b, i, "x", vec![], 1.0);
        }
        assert_eq!(set_count(&b), 5);
    }

    #[test]
    fn compute_blend_unequal_weights() {
        let mut b = new_ethnic_blend();
        add_ethnic_set(&mut b, 1, "A", vec![1.0], 3.0);
        add_ethnic_set(&mut b, 2, "B", vec![0.0], 1.0);
        let result = compute_ethnic_blend(&b);
        assert!((result[0] - 0.75).abs() < 1e-5);
    }
}
