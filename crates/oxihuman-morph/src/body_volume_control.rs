// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Body volume control — overall body mass / volume morph parameter.

use std::f32::consts::PI;

/// Configuration for body volume scaling.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyVolumeConfig {
    /// Minimum allowed volume scale (default 0.5).
    pub min_scale: f32,
    /// Maximum allowed volume scale (default 2.0).
    pub max_scale: f32,
    /// Exponent applied to the normalised parameter before output.
    pub exponent: f32,
}

impl Default for BodyVolumeConfig {
    fn default() -> Self {
        BodyVolumeConfig {
            min_scale: 0.5,
            max_scale: 2.0,
            exponent: 1.0,
        }
    }
}

/// Runtime state for body volume control.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyVolumeState {
    /// Normalised volume parameter in `[0.0, 1.0]`.
    volume: f32,
    /// Separate chest contribution in `[0.0, 1.0]`.
    chest: f32,
    /// Separate abdomen contribution in `[0.0, 1.0]`.
    abdomen: f32,
    config: BodyVolumeConfig,
}

/// Morph weights produced by body volume evaluation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyVolumeWeights {
    pub overall: f32,
    pub chest: f32,
    pub abdomen: f32,
    pub limbs: f32,
}

/// Create a new [`BodyVolumeState`] with neutral settings.
pub fn new_body_volume_state(config: BodyVolumeConfig) -> BodyVolumeState {
    BodyVolumeState {
        volume: 0.5,
        chest: 0.5,
        abdomen: 0.5,
        config,
    }
}

/// Return a default [`BodyVolumeConfig`].
pub fn default_body_volume_config() -> BodyVolumeConfig {
    BodyVolumeConfig::default()
}

/// Set the overall volume parameter (clamped to `[0.0, 1.0]`).
pub fn bvc_set_volume(state: &mut BodyVolumeState, v: f32) {
    state.volume = v.clamp(0.0, 1.0);
}

/// Set the chest-specific volume parameter.
pub fn bvc_set_chest(state: &mut BodyVolumeState, v: f32) {
    state.chest = v.clamp(0.0, 1.0);
}

/// Set the abdomen-specific volume parameter.
pub fn bvc_set_abdomen(state: &mut BodyVolumeState, v: f32) {
    state.abdomen = v.clamp(0.0, 1.0);
}

/// Reset all parameters to neutral (0.5).
pub fn bvc_reset(state: &mut BodyVolumeState) {
    state.volume = 0.5;
    state.chest = 0.5;
    state.abdomen = 0.5;
}

/// Return true if all parameters are at neutral.
pub fn bvc_is_neutral(state: &BodyVolumeState) -> bool {
    (state.volume - 0.5).abs() < 1e-5
        && (state.chest - 0.5).abs() < 1e-5
        && (state.abdomen - 0.5).abs() < 1e-5
}

/// Evaluate the body volume morph weights from the current state.
pub fn bvc_to_weights(state: &BodyVolumeState) -> BodyVolumeWeights {
    let scale = |x: f32| x.powf(state.config.exponent);
    BodyVolumeWeights {
        overall: scale(state.volume),
        chest: scale(state.chest),
        abdomen: scale(state.abdomen),
        limbs: scale((state.chest + state.abdomen) * 0.5),
    }
}

/// Blend between two states by `t ∈ [0.0, 1.0]`.
pub fn bvc_blend(a: &BodyVolumeState, b: &BodyVolumeState, t: f32) -> BodyVolumeState {
    let t = t.clamp(0.0, 1.0);
    BodyVolumeState {
        volume: a.volume + (b.volume - a.volume) * t,
        chest: a.chest + (b.chest - a.chest) * t,
        abdomen: a.abdomen + (b.abdomen - a.abdomen) * t,
        config: a.config.clone(),
    }
}

/// Estimate a rough spherical volume from the state (arbitrary units).
///
/// Uses `(4/3)π r³` where `r` is derived from overall scale.
pub fn bvc_estimated_volume(state: &BodyVolumeState) -> f32 {
    let w = bvc_to_weights(state);
    let r = w.overall * 0.5 + 0.5; // map [0,1] → [0.5, 1.0]
    (4.0 / 3.0) * PI * r * r * r
}

/// Serialise to a simple JSON-like string.
pub fn bvc_to_json(state: &BodyVolumeState) -> String {
    format!(
        r#"{{"volume":{:.4},"chest":{:.4},"abdomen":{:.4}}}"#,
        state.volume, state.chest, state.abdomen
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> BodyVolumeState {
        new_body_volume_state(default_body_volume_config())
    }

    #[test]
    fn neutral_on_creation() {
        let s = make();
        assert!(bvc_is_neutral(&s));
    }

    #[test]
    fn set_volume_clamps_high() {
        let mut s = make();
        bvc_set_volume(&mut s, 5.0);
        let w = bvc_to_weights(&s);
        assert!((0.0..=1.0).contains(&w.overall));
    }

    #[test]
    fn set_volume_clamps_low() {
        let mut s = make();
        bvc_set_volume(&mut s, -3.0);
        let w = bvc_to_weights(&s);
        assert!((0.0..=1.0).contains(&w.overall));
    }

    #[test]
    fn reset_restores_neutral() {
        let mut s = make();
        bvc_set_volume(&mut s, 0.9);
        bvc_set_chest(&mut s, 0.1);
        bvc_reset(&mut s);
        assert!(bvc_is_neutral(&s));
    }

    #[test]
    fn weights_in_unit_range() {
        let mut s = make();
        bvc_set_volume(&mut s, 0.8);
        bvc_set_chest(&mut s, 0.3);
        bvc_set_abdomen(&mut s, 0.6);
        let w = bvc_to_weights(&s);
        assert!((0.0..=1.0).contains(&w.overall));
        assert!((0.0..=1.0).contains(&w.chest));
        assert!((0.0..=1.0).contains(&w.abdomen));
        assert!((0.0..=1.0).contains(&w.limbs));
    }

    #[test]
    fn blend_midpoint() {
        let mut a = make();
        let mut b = make();
        bvc_set_volume(&mut a, 0.0);
        bvc_set_volume(&mut b, 1.0);
        let mid = bvc_blend(&a, &b, 0.5);
        assert!((mid.volume - 0.5).abs() < 1e-5);
    }

    #[test]
    fn blend_at_zero_is_a() {
        let a = make();
        let b = make();
        let r = bvc_blend(&a, &b, 0.0);
        assert!((r.volume - a.volume).abs() < 1e-5);
    }

    #[test]
    fn estimated_volume_positive() {
        let s = make();
        assert!(bvc_estimated_volume(&s) > 0.0);
    }

    #[test]
    fn json_contains_volume_key() {
        let s = make();
        assert!(bvc_to_json(&s).contains("volume"));
    }

    #[test]
    fn exponent_changes_output() {
        let mut cfg = default_body_volume_config();
        cfg.exponent = 2.0;
        let s = new_body_volume_state(cfg);
        let w = bvc_to_weights(&s);
        // 0.5^2 = 0.25
        assert!((w.overall - 0.25).abs() < 1e-5);
    }
}
