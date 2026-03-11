// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Sternum control — sternal length and manubrium prominence.

use std::f32::consts::FRAC_PI_4;

/// Configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SternumConfig {
    /// Reference tilt angle for xiphoid process.
    pub xiphoid_ref_rad: f32,
}

impl Default for SternumConfig {
    fn default() -> Self {
        SternumConfig {
            xiphoid_ref_rad: FRAC_PI_4,
        }
    }
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SternumState {
    /// Sternal length in `[0.0, 1.0]`.
    length: f32,
    /// Manubrium protrusion in `[0.0, 1.0]`.
    manubrium: f32,
    /// Xiphoid angle in `[-1.0, 1.0]` (positive = flared).
    xiphoid_angle: f32,
    config: SternumConfig,
}

/// Default config.
pub fn default_sternum_config() -> SternumConfig {
    SternumConfig::default()
}

/// New neutral state.
pub fn new_sternum_state(config: SternumConfig) -> SternumState {
    SternumState {
        length: 0.5,
        manubrium: 0.0,
        xiphoid_angle: 0.0,
        config,
    }
}

/// Set sternal length.
pub fn stc_set_length(state: &mut SternumState, v: f32) {
    state.length = v.clamp(0.0, 1.0);
}

/// Set manubrium protrusion.
pub fn stc_set_manubrium(state: &mut SternumState, v: f32) {
    state.manubrium = v.clamp(0.0, 1.0);
}

/// Set xiphoid angle.
pub fn stc_set_xiphoid_angle(state: &mut SternumState, v: f32) {
    state.xiphoid_angle = v.clamp(-1.0, 1.0);
}

/// Reset.
pub fn stc_reset(state: &mut SternumState) {
    state.length = 0.5;
    state.manubrium = 0.0;
    state.xiphoid_angle = 0.0;
}

/// True when neutral.
pub fn stc_is_neutral(state: &SternumState) -> bool {
    (state.length - 0.5).abs() < 1e-5 && state.manubrium < 1e-5 && state.xiphoid_angle.abs() < 1e-5
}

/// Xiphoid angle in radians.
pub fn stc_xiphoid_angle_rad(state: &SternumState) -> f32 {
    state.xiphoid_angle * state.config.xiphoid_ref_rad
}

/// Morph weights: `[length, manubrium, xiphoid_norm]`.
pub fn stc_to_weights(state: &SternumState) -> [f32; 3] {
    [
        state.length,
        state.manubrium,
        (state.xiphoid_angle * 0.5 + 0.5).clamp(0.0, 1.0),
    ]
}

/// Blend.
pub fn stc_blend(a: &SternumState, b: &SternumState, t: f32) -> SternumState {
    let t = t.clamp(0.0, 1.0);
    SternumState {
        length: a.length + (b.length - a.length) * t,
        manubrium: a.manubrium + (b.manubrium - a.manubrium) * t,
        xiphoid_angle: a.xiphoid_angle + (b.xiphoid_angle - a.xiphoid_angle) * t,
        config: a.config.clone(),
    }
}

/// Serialise.
pub fn stc_to_json(state: &SternumState) -> String {
    format!(
        r#"{{"length":{:.4},"manubrium":{:.4},"xiphoid_angle":{:.4}}}"#,
        state.length, state.manubrium, state.xiphoid_angle
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> SternumState {
        new_sternum_state(default_sternum_config())
    }

    #[test]
    fn neutral_on_creation() {
        assert!(stc_is_neutral(&make()));
    }

    #[test]
    fn set_length_clamps() {
        let mut s = make();
        stc_set_length(&mut s, 5.0);
        assert!((s.length - 1.0).abs() < 1e-5);
    }

    #[test]
    fn reset_restores_neutral() {
        let mut s = make();
        stc_set_manubrium(&mut s, 0.8);
        stc_reset(&mut s);
        assert!(stc_is_neutral(&s));
    }

    #[test]
    fn xiphoid_angle_rad_computation() {
        let mut s = make();
        stc_set_xiphoid_angle(&mut s, 1.0);
        assert!(stc_xiphoid_angle_rad(&s) > 0.0);
    }

    #[test]
    fn weights_in_range() {
        let s = make();
        for v in stc_to_weights(&s) {
            assert!((0.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn blend_midpoint() {
        let mut a = make();
        let mut b = make();
        stc_set_length(&mut a, 0.0);
        stc_set_length(&mut b, 1.0);
        let m = stc_blend(&a, &b, 0.5);
        assert!((m.length - 0.5).abs() < 1e-5);
    }

    #[test]
    fn blend_at_zero_is_a() {
        let a = make();
        let r = stc_blend(&a, &make(), 0.0);
        assert!((r.length - a.length).abs() < 1e-5);
    }

    #[test]
    fn json_has_length() {
        assert!(stc_to_json(&make()).contains("length"));
    }

    #[test]
    fn xiphoid_clamped_negative() {
        let mut s = make();
        stc_set_xiphoid_angle(&mut s, -5.0);
        assert!((s.xiphoid_angle + 1.0).abs() < 1e-5);
    }
}
