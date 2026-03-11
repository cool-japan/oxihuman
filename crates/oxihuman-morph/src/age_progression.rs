#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Age-related morph progression (young → old).

/// Weights describing how age manifests on a mesh.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct AgeProgression {
    /// Normalised age value in [0, 1] (0 = newborn, 1 = very old).
    pub age: f32,
    /// Blend weight for wrinkle morphs.
    pub wrinkle_weight: f32,
    /// Blend weight for volume-loss morphs.
    pub volume_loss: f32,
    /// Blend weight for tissue-sag morphs.
    pub sag_factor: f32,
}

/// Returns a default `AgeProgression` representing a young adult (~age 25).
#[allow(dead_code)]
pub fn default_age_progression() -> AgeProgression {
    AgeProgression {
        age: 0.25,
        wrinkle_weight: 0.0,
        volume_loss: 0.0,
        sag_factor: 0.0,
    }
}

/// Derive `AgeProgression` weights from a normalised age value in [0, 1].
///
/// * 0.0 – 0.2 : child (no aging morphs)
/// * 0.2 – 0.5 : young adult (mild morphs)
/// * 0.5 – 0.8 : middle-aged (moderate morphs)
/// * 0.8 – 1.0 : elderly (strong morphs)
#[allow(dead_code)]
pub fn age_to_weights(age: f32) -> AgeProgression {
    let age_c = age.clamp(0.0, 1.0);
    let t = ((age_c - 0.2) / 0.8).clamp(0.0, 1.0);
    AgeProgression {
        age: age_c,
        wrinkle_weight: t * t,
        volume_loss: t * 0.8,
        sag_factor: t * t * 0.6,
    }
}

/// Apply the `AgeProgression` weights to a mutable morph-weight slice.
///
/// Expects at least 3 elements: `[wrinkle, volume_loss, sag]`.
#[allow(dead_code)]
pub fn apply_age_progression(weights: &mut [f32], ap: &AgeProgression) {
    if !weights.is_empty() {
        weights[0] = ap.wrinkle_weight;
    }
    if weights.len() >= 2 {
        weights[1] = ap.volume_loss;
    }
    if weights.len() >= 3 {
        weights[2] = ap.sag_factor;
    }
}

/// Linearly blend two `AgeProgression` structs.
#[allow(dead_code)]
pub fn age_blend(a: &AgeProgression, b: &AgeProgression, t: f32) -> AgeProgression {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: f32, y: f32| x + (y - x) * t;
    AgeProgression {
        age: lerp(a.age, b.age),
        wrinkle_weight: lerp(a.wrinkle_weight, b.wrinkle_weight),
        volume_loss: lerp(a.volume_loss, b.volume_loss),
        sag_factor: lerp(a.sag_factor, b.sag_factor),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_age_progression_young() {
        let ap = default_age_progression();
        assert!((ap.age - 0.25).abs() < 1e-6);
        assert!((ap.wrinkle_weight).abs() < 1e-6);
    }

    #[test]
    fn age_to_weights_young_no_wrinkles() {
        let ap = age_to_weights(0.1);
        assert!(ap.wrinkle_weight < 1e-6);
        assert!(ap.volume_loss < 1e-6);
    }

    #[test]
    fn age_to_weights_elderly_high_wrinkles() {
        let ap = age_to_weights(1.0);
        assert!(ap.wrinkle_weight > 0.5);
        assert!(ap.volume_loss > 0.5);
    }

    #[test]
    fn age_to_weights_clamps_below_zero() {
        let ap = age_to_weights(-5.0);
        assert!((0.0..=1.0).contains(&ap.age));
    }

    #[test]
    fn age_to_weights_clamps_above_one() {
        let ap = age_to_weights(2.0);
        assert!((0.0..=1.0).contains(&ap.age));
    }

    #[test]
    fn apply_age_progression_sets_weights() {
        let ap = age_to_weights(1.0);
        let mut w = vec![0.0_f32; 3];
        apply_age_progression(&mut w, &ap);
        assert!((w[0] - ap.wrinkle_weight).abs() < 1e-6);
        assert!((w[1] - ap.volume_loss).abs() < 1e-6);
        assert!((w[2] - ap.sag_factor).abs() < 1e-6);
    }

    #[test]
    fn apply_age_progression_short_slice_no_panic() {
        let ap = age_to_weights(1.0);
        let mut w = vec![0.0_f32; 1];
        apply_age_progression(&mut w, &ap);
        assert!((w[0] - ap.wrinkle_weight).abs() < 1e-6);
    }

    #[test]
    fn age_blend_at_zero_returns_a() {
        let a = age_to_weights(0.2);
        let b = age_to_weights(0.8);
        let r = age_blend(&a, &b, 0.0);
        assert!((r.age - a.age).abs() < 1e-6);
    }

    #[test]
    fn age_blend_at_one_returns_b() {
        let a = age_to_weights(0.2);
        let b = age_to_weights(0.8);
        let r = age_blend(&a, &b, 1.0);
        assert!((r.age - b.age).abs() < 1e-6);
    }

    #[test]
    fn age_blend_midpoint() {
        let a = age_to_weights(0.0);
        let b = age_to_weights(1.0);
        let r = age_blend(&a, &b, 0.5);
        assert!((r.age - 0.5).abs() < 1e-5);
    }

    #[test]
    fn age_blend_clamps_t() {
        let a = age_to_weights(0.2);
        let b = age_to_weights(0.8);
        let r_over = age_blend(&a, &b, 2.0);
        let r_one = age_blend(&a, &b, 1.0);
        assert!((r_over.age - r_one.age).abs() < 1e-6);
    }
}
