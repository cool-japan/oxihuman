// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Jaw rest control — rest-position gap and muscle tone of the jaw.

use std::f32::consts::PI;

/// Configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JawRestConfig {
    /// Maximum rest-gap angle in radians (informational).
    pub max_gap_rad: f32,
    /// Whether natural micro-relaxation is enabled.
    pub micro_relax: bool,
}

impl Default for JawRestConfig {
    fn default() -> Self {
        JawRestConfig {
            max_gap_rad: PI / 12.0,
            micro_relax: true,
        }
    }
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JawRestState {
    /// Rest-gap in `[0.0, 1.0]` (0 = fully closed, 1 = max rest opening).
    gap: f32,
    /// Lateral shift in `[-1.0, 1.0]`.
    lateral: f32,
    /// Muscle relaxation in `[0.0, 1.0]`.
    relaxation: f32,
    config: JawRestConfig,
}

/// Default config.
pub fn default_jaw_rest_config() -> JawRestConfig {
    JawRestConfig::default()
}

/// New neutral state (gap = 0.1, fully relaxed).
pub fn new_jaw_rest_state(config: JawRestConfig) -> JawRestState {
    JawRestState {
        gap: 0.1,
        lateral: 0.0,
        relaxation: 0.5,
        config,
    }
}

/// Set rest gap.
pub fn jr_set_gap(state: &mut JawRestState, v: f32) {
    state.gap = v.clamp(0.0, 1.0);
}

/// Set lateral shift.
pub fn jr_set_lateral(state: &mut JawRestState, v: f32) {
    state.lateral = v.clamp(-1.0, 1.0);
}

/// Set muscle relaxation.
pub fn jr_set_relaxation(state: &mut JawRestState, v: f32) {
    state.relaxation = v.clamp(0.0, 1.0);
}

/// Reset.
pub fn jr_reset(state: &mut JawRestState) {
    state.gap = 0.1;
    state.lateral = 0.0;
    state.relaxation = 0.5;
}

/// True when neutral (gap = 0.1, lateral = 0.0, relaxation = 0.5).
pub fn jr_is_neutral(state: &JawRestState) -> bool {
    (state.gap - 0.1).abs() < 1e-5
        && state.lateral.abs() < 1e-5
        && (state.relaxation - 0.5).abs() < 1e-5
}

/// Compute the rest-gap angle in radians.
pub fn jr_gap_rad(state: &JawRestState) -> f32 {
    state.gap * state.config.max_gap_rad
}

/// Morph weights: `[gap, lateral_norm, relaxation]`.
pub fn jr_to_weights(state: &JawRestState) -> [f32; 3] {
    [
        state.gap,
        (state.lateral * 0.5 + 0.5).clamp(0.0, 1.0),
        state.relaxation,
    ]
}

/// Blend.
pub fn jr_blend(a: &JawRestState, b: &JawRestState, t: f32) -> JawRestState {
    let t = t.clamp(0.0, 1.0);
    JawRestState {
        gap: a.gap + (b.gap - a.gap) * t,
        lateral: a.lateral + (b.lateral - a.lateral) * t,
        relaxation: a.relaxation + (b.relaxation - a.relaxation) * t,
        config: a.config.clone(),
    }
}

/// Serialise.
pub fn jr_to_json(state: &JawRestState) -> String {
    format!(
        r#"{{"gap":{:.4},"lateral":{:.4},"relaxation":{:.4}}}"#,
        state.gap, state.lateral, state.relaxation
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> JawRestState {
        new_jaw_rest_state(default_jaw_rest_config())
    }

    #[test]
    fn neutral_on_creation() {
        assert!(jr_is_neutral(&make()));
    }

    #[test]
    fn gap_clamped_high() {
        let mut s = make();
        jr_set_gap(&mut s, 5.0);
        assert!((s.gap - 1.0).abs() < 1e-5);
    }

    #[test]
    fn lateral_clamped_negative() {
        let mut s = make();
        jr_set_lateral(&mut s, -5.0);
        assert!((s.lateral + 1.0).abs() < 1e-5);
    }

    #[test]
    fn reset_restores_neutral() {
        let mut s = make();
        jr_set_gap(&mut s, 0.9);
        jr_reset(&mut s);
        assert!(jr_is_neutral(&s));
    }

    #[test]
    fn gap_rad_positive() {
        let mut s = make();
        jr_set_gap(&mut s, 0.5);
        assert!(jr_gap_rad(&s) > 0.0);
    }

    #[test]
    fn weights_in_range() {
        let s = make();
        for v in jr_to_weights(&s) {
            assert!((0.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn blend_midpoint() {
        let mut b = make();
        jr_set_gap(&mut b, 1.0);
        let m = jr_blend(&make(), &b, 0.5);
        // gap starts at 0.1, target 1.0, midpoint is ~0.55
        assert!(m.gap > 0.1 && m.gap < 1.0);
    }

    #[test]
    fn json_has_gap() {
        assert!(jr_to_json(&make()).contains("gap"));
    }

    #[test]
    fn blend_at_zero_is_a() {
        let a = make();
        let r = jr_blend(&a, &make(), 0.0);
        assert!((r.gap - a.gap).abs() < 1e-5);
    }
}
