// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Hand width control — transverse palm width scaling.

/// Configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HandWidthConfig {
    pub max_width: f32,
}

impl Default for HandWidthConfig {
    fn default() -> Self {
        HandWidthConfig { max_width: 1.0 }
    }
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HandWidthState {
    left: f32,
    right: f32,
    /// Finger-spread contribution in `[0.0, 1.0]`.
    finger_spread: f32,
    config: HandWidthConfig,
}

/// Default config.
pub fn default_hand_width_config() -> HandWidthConfig {
    HandWidthConfig::default()
}

/// New neutral state.
pub fn new_hand_width_state(config: HandWidthConfig) -> HandWidthState {
    HandWidthState {
        left: 0.5,
        right: 0.5,
        finger_spread: 0.0,
        config,
    }
}

/// Set width for one hand.
pub fn hwc_set_left(state: &mut HandWidthState, v: f32) {
    state.left = v.clamp(0.0, 1.0);
}

/// Set width for right hand.
pub fn hwc_set_right(state: &mut HandWidthState, v: f32) {
    state.right = v.clamp(0.0, 1.0);
}

/// Set both hands.
pub fn hwc_set_both(state: &mut HandWidthState, v: f32) {
    let v = v.clamp(0.0, 1.0);
    state.left = v;
    state.right = v;
}

/// Set finger spread.
pub fn hwc_set_finger_spread(state: &mut HandWidthState, v: f32) {
    state.finger_spread = v.clamp(0.0, 1.0);
}

/// Reset to neutral (0.5).
pub fn hwc_reset(state: &mut HandWidthState) {
    state.left = 0.5;
    state.right = 0.5;
    state.finger_spread = 0.0;
}

/// True when neutral.
pub fn hwc_is_neutral(state: &HandWidthState) -> bool {
    (state.left - 0.5).abs() < 1e-5
        && (state.right - 0.5).abs() < 1e-5
        && state.finger_spread < 1e-5
}

/// Asymmetry between hands.
pub fn hwc_asymmetry(state: &HandWidthState) -> f32 {
    (state.left - state.right).abs()
}

/// Average width including finger spread contribution.
pub fn hwc_effective_width(state: &HandWidthState) -> f32 {
    let base = (state.left + state.right) * 0.5;
    (base + state.finger_spread * 0.2).clamp(0.0, 1.0)
}

/// Morph weights: `[left, right, finger_spread]`.
pub fn hwc_to_weights(state: &HandWidthState) -> [f32; 3] {
    [state.left, state.right, state.finger_spread]
}

/// Blend.
pub fn hwc_blend(a: &HandWidthState, b: &HandWidthState, t: f32) -> HandWidthState {
    let t = t.clamp(0.0, 1.0);
    HandWidthState {
        left: a.left + (b.left - a.left) * t,
        right: a.right + (b.right - a.right) * t,
        finger_spread: a.finger_spread + (b.finger_spread - a.finger_spread) * t,
        config: a.config.clone(),
    }
}

/// Serialise.
pub fn hwc_to_json(state: &HandWidthState) -> String {
    format!(
        r#"{{"left":{:.4},"right":{:.4},"finger_spread":{:.4}}}"#,
        state.left, state.right, state.finger_spread
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> HandWidthState {
        new_hand_width_state(default_hand_width_config())
    }

    #[test]
    fn neutral_on_creation() {
        assert!(hwc_is_neutral(&make()));
    }

    #[test]
    fn set_both_equal() {
        let mut s = make();
        hwc_set_both(&mut s, 0.8);
        assert!((s.left - s.right).abs() < 1e-5);
    }

    #[test]
    fn reset_restores_neutral() {
        let mut s = make();
        hwc_set_both(&mut s, 0.1);
        hwc_reset(&mut s);
        assert!(hwc_is_neutral(&s));
    }

    #[test]
    fn asymmetry_zero_equal() {
        let mut s = make();
        hwc_set_both(&mut s, 0.7);
        assert!(hwc_asymmetry(&s) < 1e-5);
    }

    #[test]
    fn effective_width_in_range() {
        let s = make();
        assert!((0.0..=1.0).contains(&hwc_effective_width(&s)));
    }

    #[test]
    fn weights_in_range() {
        let s = make();
        for v in hwc_to_weights(&s) {
            assert!((0.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn blend_midpoint() {
        let mut a = make();
        let mut b = make();
        hwc_set_both(&mut a, 0.0);
        hwc_set_both(&mut b, 1.0);
        let m = hwc_blend(&a, &b, 0.5);
        assert!((m.left - 0.5).abs() < 1e-5);
    }

    #[test]
    fn json_has_left() {
        assert!(hwc_to_json(&make()).contains("left"));
    }

    #[test]
    fn finger_spread_clamped() {
        let mut s = make();
        hwc_set_finger_spread(&mut s, 2.0);
        assert!((s.finger_spread - 1.0).abs() < 1e-5);
    }
}
