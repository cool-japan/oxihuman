// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Age-based facial morphing system.
//!
//! Models a continuous age parameter in `[0.0, 1.0]` where `0.0` represents a
//! child and `1.0` represents an elderly individual.  The system outputs morph
//! target weights that drive the deformation engine through the full life-span
//! progression.

#![allow(dead_code)]

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Type aliases
// ---------------------------------------------------------------------------

/// Map of morph-target name → blend weight produced by age evaluation.
pub type AgeMorphWeights = HashMap<String, f32>;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Broad life-stage groups used for categorisation and blending.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum AgeGroup {
    /// age ∈ [0.0, 0.17)
    Child,
    /// age ∈ [0.17, 0.30)
    Teen,
    /// age ∈ [0.30, 0.50)
    Young,
    /// age ∈ [0.50, 0.67)
    Middle,
    /// age ∈ [0.67, 0.83)
    Senior,
    /// age ∈ [0.83, 1.0]
    Elderly,
}

impl AgeGroup {
    /// Return the canonical display name for this group.
    pub fn name(self) -> &'static str {
        match self {
            AgeGroup::Child => "Child",
            AgeGroup::Teen => "Teen",
            AgeGroup::Young => "Young Adult",
            AgeGroup::Middle => "Middle Age",
            AgeGroup::Senior => "Senior",
            AgeGroup::Elderly => "Elderly",
        }
    }
}

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

/// Tuning parameters for age-morph behaviour.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AgeMorphConfig {
    /// Scale factor for wrinkle intensity (default 1.0).
    pub wrinkle_scale: f32,
    /// Scale factor for volume-loss intensity (default 1.0).
    pub volume_loss_scale: f32,
    /// Scale factor for skin-roughness intensity (default 1.0).
    pub roughness_scale: f32,
    /// Scale factor for ptosis (eyelid droop, default 1.0).
    pub ptosis_scale: f32,
    /// Scale factor for jowl sagging (default 1.0).
    pub jowl_scale: f32,
}

/// Runtime state for a single character's age morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AgeMorphState {
    /// Normalised age value in `[0.0, 1.0]`.
    age: f32,
    /// Active tuning configuration.
    config: AgeMorphConfig,
}

// ---------------------------------------------------------------------------
// Config helpers
// ---------------------------------------------------------------------------

/// Return an [`AgeMorphConfig`] with all scales set to 1.0.
pub fn default_age_config() -> AgeMorphConfig {
    AgeMorphConfig {
        wrinkle_scale: 1.0,
        volume_loss_scale: 1.0,
        roughness_scale: 1.0,
        ptosis_scale: 1.0,
        jowl_scale: 1.0,
    }
}

// ---------------------------------------------------------------------------
// State construction / mutation
// ---------------------------------------------------------------------------

/// Create a new [`AgeMorphState`] at a given age value (clamped to `[0, 1]`).
pub fn new_age_morph_state(age: f32, config: AgeMorphConfig) -> AgeMorphState {
    AgeMorphState {
        age: age.clamp(0.0, 1.0),
        config,
    }
}

/// Update the age value (clamped to `[0.0, 1.0]`).
pub fn set_age(state: &mut AgeMorphState, age: f32) {
    state.age = age.clamp(0.0, 1.0);
}

/// Read the current age value.
pub fn age_value(state: &AgeMorphState) -> f32 {
    state.age
}

/// Reset the age back to 0.0 (child) and all config scales to 1.0.
pub fn reset_age_morph(state: &mut AgeMorphState) {
    state.age = 0.0;
    state.config = default_age_config();
}

// ---------------------------------------------------------------------------
// Classification
// ---------------------------------------------------------------------------

/// Map an age float in `[0.0, 1.0]` to an [`AgeGroup`].
pub fn age_group_from_value(age: f32) -> AgeGroup {
    let a = age.clamp(0.0, 1.0);
    if a < 0.17 {
        AgeGroup::Child
    } else if a < 0.30 {
        AgeGroup::Teen
    } else if a < 0.50 {
        AgeGroup::Young
    } else if a < 0.67 {
        AgeGroup::Middle
    } else if a < 0.83 {
        AgeGroup::Senior
    } else {
        AgeGroup::Elderly
    }
}

/// Return the display name for the age group at the given value.
pub fn age_group_name(age: f32) -> &'static str {
    age_group_from_value(age).name()
}

// ---------------------------------------------------------------------------
// Per-feature age curves
// ---------------------------------------------------------------------------

/// Wrinkle intensity at `age` ∈ `[0.0, 1.0]`, scaled by config.
///
/// Wrinkles begin appearing at Young and increase monotonically.
pub fn age_wrinkle_intensity(state: &AgeMorphState) -> f32 {
    let t = ((state.age - 0.3) / 0.7).clamp(0.0, 1.0);
    (t * t * state.config.wrinkle_scale).clamp(0.0, 1.0)
}

/// Skin roughness at `age`, scaled by config.
///
/// Roughness is low in childhood and increases steadily.
pub fn age_skin_roughness(state: &AgeMorphState) -> f32 {
    let base = (state.age * 0.8 + 0.1).clamp(0.0, 1.0);
    (base * state.config.roughness_scale).clamp(0.0, 1.0)
}

/// Soft-tissue volume loss factor, scaled by config.
///
/// Volume loss begins in Middle age and is maximum at Elderly.
pub fn age_volume_loss(state: &AgeMorphState) -> f32 {
    let t = ((state.age - 0.5) / 0.5).clamp(0.0, 1.0);
    (t * state.config.volume_loss_scale).clamp(0.0, 1.0)
}

/// Eyelid ptosis (droop) factor, scaled by config.
///
/// Ptosis increases from Senior onwards.
pub fn age_ptosis(state: &AgeMorphState) -> f32 {
    let t = ((state.age - 0.67) / 0.33).clamp(0.0, 1.0);
    (t * state.config.ptosis_scale).clamp(0.0, 1.0)
}

/// Jowl sagging factor, scaled by config.
///
/// Jowl sagging increases from Middle age onwards.
pub fn age_jowl_factor(state: &AgeMorphState) -> f32 {
    let t = ((state.age - 0.5) / 0.5).clamp(0.0, 1.0);
    (t * t * state.config.jowl_scale).clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// Morph weight map
// ---------------------------------------------------------------------------

/// Produce a full [`AgeMorphWeights`] map from the current state.
///
/// Keys: `"wrinkle"`, `"roughness"`, `"volume_loss"`, `"ptosis"`, `"jowl"`,
/// `"age_raw"`.
pub fn age_to_morph_weights(state: &AgeMorphState) -> AgeMorphWeights {
    let mut map = HashMap::new();
    map.insert("wrinkle".to_string(), age_wrinkle_intensity(state));
    map.insert("roughness".to_string(), age_skin_roughness(state));
    map.insert("volume_loss".to_string(), age_volume_loss(state));
    map.insert("ptosis".to_string(), age_ptosis(state));
    map.insert("jowl".to_string(), age_jowl_factor(state));
    map.insert("age_raw".to_string(), state.age);
    map
}

// ---------------------------------------------------------------------------
// Blending
// ---------------------------------------------------------------------------

/// Linear blend between two [`AgeMorphState`] values.
///
/// `t = 0.0` → `a`, `t = 1.0` → `b`.  Config is taken from `a`.
pub fn blend_age_states(a: &AgeMorphState, b: &AgeMorphState, t: f32) -> AgeMorphState {
    let t = t.clamp(0.0, 1.0);
    let age = a.age + (b.age - a.age) * t;
    AgeMorphState {
        age: age.clamp(0.0, 1.0),
        config: a.config.clone(),
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state(age: f32) -> AgeMorphState {
        new_age_morph_state(age, default_age_config())
    }

    // 1
    #[test]
    fn default_config_scales_are_one() {
        let c = default_age_config();
        assert!((c.wrinkle_scale - 1.0).abs() < 1e-6);
        assert!((c.roughness_scale - 1.0).abs() < 1e-6);
        assert!((c.volume_loss_scale - 1.0).abs() < 1e-6);
        assert!((c.ptosis_scale - 1.0).abs() < 1e-6);
        assert!((c.jowl_scale - 1.0).abs() < 1e-6);
    }

    // 2
    #[test]
    fn new_state_clamps_age() {
        let s = make_state(2.5);
        assert!((age_value(&s) - 1.0).abs() < 1e-6);
        let s2 = make_state(-1.0);
        assert!((age_value(&s2) - 0.0).abs() < 1e-6);
    }

    // 3
    #[test]
    fn set_age_clamps_range() {
        let mut s = make_state(0.5);
        set_age(&mut s, 99.0);
        assert!((age_value(&s) - 1.0).abs() < 1e-6);
        set_age(&mut s, -5.0);
        assert!((age_value(&s) - 0.0).abs() < 1e-6);
    }

    // 4
    #[test]
    fn reset_age_morph_resets_to_child() {
        let mut s = make_state(0.9);
        s.config.wrinkle_scale = 2.0;
        reset_age_morph(&mut s);
        assert!((age_value(&s) - 0.0).abs() < 1e-6);
        assert!((s.config.wrinkle_scale - 1.0).abs() < 1e-6);
    }

    // 5
    #[test]
    fn age_group_from_value_child() {
        assert_eq!(age_group_from_value(0.0), AgeGroup::Child);
        assert_eq!(age_group_from_value(0.16), AgeGroup::Child);
    }

    // 6
    #[test]
    fn age_group_from_value_teen() {
        assert_eq!(age_group_from_value(0.17), AgeGroup::Teen);
    }

    // 7
    #[test]
    fn age_group_from_value_young() {
        assert_eq!(age_group_from_value(0.30), AgeGroup::Young);
    }

    // 8
    #[test]
    fn age_group_from_value_middle() {
        assert_eq!(age_group_from_value(0.50), AgeGroup::Middle);
    }

    // 9
    #[test]
    fn age_group_from_value_senior() {
        assert_eq!(age_group_from_value(0.67), AgeGroup::Senior);
    }

    // 10
    #[test]
    fn age_group_from_value_elderly() {
        assert_eq!(age_group_from_value(1.0), AgeGroup::Elderly);
    }

    // 11
    #[test]
    fn age_group_name_strings() {
        assert_eq!(AgeGroup::Child.name(), "Child");
        assert_eq!(AgeGroup::Teen.name(), "Teen");
        assert_eq!(AgeGroup::Young.name(), "Young Adult");
        assert_eq!(AgeGroup::Middle.name(), "Middle Age");
        assert_eq!(AgeGroup::Senior.name(), "Senior");
        assert_eq!(AgeGroup::Elderly.name(), "Elderly");
    }

    // 12
    #[test]
    fn wrinkle_intensity_child_is_zero() {
        let s = make_state(0.0);
        assert!((age_wrinkle_intensity(&s) - 0.0).abs() < 1e-6);
    }

    // 13
    #[test]
    fn wrinkle_intensity_elderly_is_near_one() {
        let s = make_state(1.0);
        assert!(age_wrinkle_intensity(&s) > 0.9);
    }

    // 14
    #[test]
    fn volume_loss_child_is_zero() {
        let s = make_state(0.0);
        assert!((age_volume_loss(&s) - 0.0).abs() < 1e-6);
    }

    // 15
    #[test]
    fn volume_loss_elderly_is_near_one() {
        let s = make_state(1.0);
        assert!(age_volume_loss(&s) > 0.9);
    }

    // 16
    #[test]
    fn ptosis_child_is_zero() {
        let s = make_state(0.0);
        assert!((age_ptosis(&s) - 0.0).abs() < 1e-6);
    }

    // 17
    #[test]
    fn ptosis_elderly_is_near_one() {
        let s = make_state(1.0);
        assert!(age_ptosis(&s) > 0.9);
    }

    // 18
    #[test]
    fn jowl_factor_young_is_zero() {
        let s = make_state(0.3);
        assert!((age_jowl_factor(&s) - 0.0).abs() < 1e-6);
    }

    // 19
    #[test]
    fn age_to_morph_weights_has_all_keys() {
        let s = make_state(0.5);
        let w = age_to_morph_weights(&s);
        for key in &["wrinkle", "roughness", "volume_loss", "ptosis", "jowl", "age_raw"] {
            assert!(w.contains_key(*key), "missing key: {}", key);
        }
    }

    // 20
    #[test]
    fn age_to_morph_weights_age_raw_matches() {
        let s = make_state(0.42);
        let w = age_to_morph_weights(&s);
        assert!((w["age_raw"] - 0.42).abs() < 1e-6);
    }

    // 21
    #[test]
    fn blend_age_states_midpoint() {
        let a = make_state(0.0);
        let b = make_state(1.0);
        let mid = blend_age_states(&a, &b, 0.5);
        assert!((age_value(&mid) - 0.5).abs() < 1e-5);
    }

    // 22
    #[test]
    fn blend_age_states_at_zero_is_a() {
        let a = make_state(0.2);
        let b = make_state(0.8);
        let r = blend_age_states(&a, &b, 0.0);
        assert!((age_value(&r) - 0.2).abs() < 1e-5);
    }

    // 23
    #[test]
    fn blend_age_states_at_one_is_b() {
        let a = make_state(0.2);
        let b = make_state(0.8);
        let r = blend_age_states(&a, &b, 1.0);
        assert!((age_value(&r) - 0.8).abs() < 1e-5);
    }

    // 24
    #[test]
    fn skin_roughness_increases_with_age() {
        let young = make_state(0.1);
        let old = make_state(0.9);
        assert!(age_skin_roughness(&old) > age_skin_roughness(&young));
    }

    // 25
    #[test]
    fn all_weights_in_unit_range() {
        for i in 0..=10 {
            let s = make_state(i as f32 / 10.0);
            let w = age_to_morph_weights(&s);
            for (k, v) in &w {
                if k != "age_raw" {
                    assert!(*v >= 0.0 && *v <= 1.0, "key {} out of range: {}", k, v);
                }
            }
        }
    }
}
