// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Neck forward control — forward head posture / cervical lordosis.

use std::f32::consts::FRAC_PI_3;

/// Configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NeckForwardConfig {
    /// Maximum forward tilt angle in radians.
    pub max_angle_rad: f32,
    /// Whether to compensate thoracic kyphosis.
    pub thoracic_compensate: bool,
}

impl Default for NeckForwardConfig {
    fn default() -> Self {
        NeckForwardConfig {
            max_angle_rad: FRAC_PI_3,
            thoracic_compensate: false,
        }
    }
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NeckForwardState {
    /// Forward tilt in `[0.0, 1.0]`.
    forward: f32,
    /// Lateral bend in `[-1.0, 1.0]`.
    lateral: f32,
    /// Protrusion (anterior shift) in `[0.0, 1.0]`.
    protrusion: f32,
    config: NeckForwardConfig,
}

/// Default config.
pub fn default_neck_forward_config() -> NeckForwardConfig {
    NeckForwardConfig::default()
}

/// New neutral state.
pub fn new_neck_forward_state(config: NeckForwardConfig) -> NeckForwardState {
    NeckForwardState {
        forward: 0.0,
        lateral: 0.0,
        protrusion: 0.0,
        config,
    }
}

/// Set forward tilt.
pub fn nfc_set_forward(state: &mut NeckForwardState, v: f32) {
    state.forward = v.clamp(0.0, 1.0);
}

/// Set lateral bend.
pub fn nfc_set_lateral(state: &mut NeckForwardState, v: f32) {
    state.lateral = v.clamp(-1.0, 1.0);
}

/// Set protrusion.
pub fn nfc_set_protrusion(state: &mut NeckForwardState, v: f32) {
    state.protrusion = v.clamp(0.0, 1.0);
}

/// Reset.
pub fn nfc_reset(state: &mut NeckForwardState) {
    state.forward = 0.0;
    state.lateral = 0.0;
    state.protrusion = 0.0;
}

/// True when neutral.
pub fn nfc_is_neutral(state: &NeckForwardState) -> bool {
    state.forward < 1e-5 && state.lateral.abs() < 1e-5 && state.protrusion < 1e-5
}

/// Compute forward tilt angle in radians.
pub fn nfc_angle_rad(state: &NeckForwardState) -> f32 {
    state.forward * state.config.max_angle_rad
}

/// Morph weights: `[forward, lateral_norm, protrusion]`.
pub fn nfc_to_weights(state: &NeckForwardState) -> [f32; 3] {
    [
        state.forward,
        (state.lateral * 0.5 + 0.5).clamp(0.0, 1.0),
        state.protrusion,
    ]
}

/// Blend.
pub fn nfc_blend(a: &NeckForwardState, b: &NeckForwardState, t: f32) -> NeckForwardState {
    let t = t.clamp(0.0, 1.0);
    NeckForwardState {
        forward: a.forward + (b.forward - a.forward) * t,
        lateral: a.lateral + (b.lateral - a.lateral) * t,
        protrusion: a.protrusion + (b.protrusion - a.protrusion) * t,
        config: a.config.clone(),
    }
}

/// Serialise.
pub fn nfc_to_json(state: &NeckForwardState) -> String {
    format!(
        r#"{{"forward":{:.4},"lateral":{:.4},"protrusion":{:.4}}}"#,
        state.forward, state.lateral, state.protrusion
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> NeckForwardState {
        new_neck_forward_state(default_neck_forward_config())
    }

    #[test]
    fn neutral_on_creation() {
        assert!(nfc_is_neutral(&make()));
    }

    #[test]
    fn set_forward_clamps() {
        let mut s = make();
        nfc_set_forward(&mut s, 3.0);
        assert!((s.forward - 1.0).abs() < 1e-5);
    }

    #[test]
    fn reset_clears() {
        let mut s = make();
        nfc_set_forward(&mut s, 0.8);
        nfc_reset(&mut s);
        assert!(nfc_is_neutral(&s));
    }

    #[test]
    fn angle_positive_when_forward() {
        let mut s = make();
        nfc_set_forward(&mut s, 0.5);
        assert!(nfc_angle_rad(&s) > 0.0);
    }

    #[test]
    fn weights_in_range() {
        let s = make();
        for v in nfc_to_weights(&s) {
            assert!((0.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn blend_midpoint() {
        let mut b = make();
        nfc_set_forward(&mut b, 1.0);
        let m = nfc_blend(&make(), &b, 0.5);
        assert!((m.forward - 0.5).abs() < 1e-5);
    }

    #[test]
    fn lateral_clamped_positive() {
        let mut s = make();
        nfc_set_lateral(&mut s, 5.0);
        assert!((s.lateral - 1.0).abs() < 1e-5);
    }

    #[test]
    fn json_has_forward() {
        assert!(nfc_to_json(&make()).contains("forward"));
    }

    #[test]
    fn blend_at_zero_is_a() {
        let a = make();
        let r = nfc_blend(&a, &make(), 0.0);
        assert!((r.forward - a.forward).abs() < 1e-5);
    }
}
