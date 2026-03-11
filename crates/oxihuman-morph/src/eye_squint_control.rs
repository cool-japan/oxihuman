// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Eye squint control — orbital compression / narrowing of the eye aperture.

use std::f32::consts::FRAC_PI_8;

/// Configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeSquintConfig {
    /// Reference angle for the squint shape.
    pub ref_angle_rad: f32,
    /// Whether to apply asymmetric lower-lid weighting.
    pub lower_lid_bias: bool,
}

impl Default for EyeSquintConfig {
    fn default() -> Self {
        EyeSquintConfig {
            ref_angle_rad: FRAC_PI_8,
            lower_lid_bias: false,
        }
    }
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeSquintState {
    left: f32,
    right: f32,
    /// Inner-corner contribution in `[0.0, 1.0]`.
    inner: f32,
    config: EyeSquintConfig,
}

/// Default config.
pub fn default_eye_squint_config() -> EyeSquintConfig {
    EyeSquintConfig::default()
}

/// New neutral state.
pub fn new_eye_squint_state(config: EyeSquintConfig) -> EyeSquintState {
    EyeSquintState {
        left: 0.0,
        right: 0.0,
        inner: 0.0,
        config,
    }
}

/// Set left squint.
pub fn esq_set_left(state: &mut EyeSquintState, v: f32) {
    state.left = v.clamp(0.0, 1.0);
}

/// Set right squint.
pub fn esq_set_right(state: &mut EyeSquintState, v: f32) {
    state.right = v.clamp(0.0, 1.0);
}

/// Set both sides.
pub fn esq_set_both(state: &mut EyeSquintState, v: f32) {
    let v = v.clamp(0.0, 1.0);
    state.left = v;
    state.right = v;
}

/// Set inner-corner contribution.
pub fn esq_set_inner(state: &mut EyeSquintState, v: f32) {
    state.inner = v.clamp(0.0, 1.0);
}

/// Reset.
pub fn esq_reset(state: &mut EyeSquintState) {
    state.left = 0.0;
    state.right = 0.0;
    state.inner = 0.0;
}

/// True when neutral.
pub fn esq_is_neutral(state: &EyeSquintState) -> bool {
    state.left < 1e-5 && state.right < 1e-5
}

/// Asymmetry between sides.
pub fn esq_asymmetry(state: &EyeSquintState) -> f32 {
    (state.left - state.right).abs()
}

/// Average squint across both eyes.
pub fn esq_average(state: &EyeSquintState) -> f32 {
    (state.left + state.right) * 0.5
}

/// Compute the orbital compression angle in radians (approximation).
pub fn esq_compression_angle(state: &EyeSquintState) -> f32 {
    esq_average(state) * state.config.ref_angle_rad
}

/// Morph weights: `[left, right, inner, avg]`.
pub fn esq_to_weights(state: &EyeSquintState) -> [f32; 4] {
    [state.left, state.right, state.inner, esq_average(state)]
}

/// Blend.
pub fn esq_blend(a: &EyeSquintState, b: &EyeSquintState, t: f32) -> EyeSquintState {
    let t = t.clamp(0.0, 1.0);
    EyeSquintState {
        left: a.left + (b.left - a.left) * t,
        right: a.right + (b.right - a.right) * t,
        inner: a.inner + (b.inner - a.inner) * t,
        config: a.config.clone(),
    }
}

/// Serialise.
pub fn esq_to_json(state: &EyeSquintState) -> String {
    format!(
        r#"{{"left":{:.4},"right":{:.4},"inner":{:.4}}}"#,
        state.left, state.right, state.inner
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> EyeSquintState {
        new_eye_squint_state(default_eye_squint_config())
    }

    #[test]
    fn neutral_on_creation() {
        assert!(esq_is_neutral(&make()));
    }

    #[test]
    fn set_left() {
        let mut s = make();
        esq_set_left(&mut s, 0.5);
        assert!((s.left - 0.5).abs() < 1e-5);
    }

    #[test]
    fn set_both_syncs() {
        let mut s = make();
        esq_set_both(&mut s, 0.7);
        assert!((s.left - s.right).abs() < 1e-5);
    }

    #[test]
    fn reset_clears() {
        let mut s = make();
        esq_set_both(&mut s, 1.0);
        esq_reset(&mut s);
        assert!(esq_is_neutral(&s));
    }

    #[test]
    fn asymmetry_zero_equal() {
        let mut s = make();
        esq_set_both(&mut s, 0.5);
        assert!(esq_asymmetry(&s) < 1e-5);
    }

    #[test]
    fn compression_angle_positive() {
        let mut s = make();
        esq_set_both(&mut s, 0.5);
        assert!(esq_compression_angle(&s) > 0.0);
    }

    #[test]
    fn blend_midpoint() {
        let mut b = make();
        esq_set_both(&mut b, 1.0);
        let m = esq_blend(&make(), &b, 0.5);
        assert!((m.left - 0.5).abs() < 1e-5);
    }

    #[test]
    fn weights_in_range() {
        let mut s = make();
        esq_set_both(&mut s, 0.8);
        for v in esq_to_weights(&s) {
            assert!((0.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn json_has_left() {
        assert!(esq_to_json(&make()).contains("left"));
    }
}
