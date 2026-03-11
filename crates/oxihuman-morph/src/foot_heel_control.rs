// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Foot heel control — heel pad thickness and calcaneus prominence.

/// Which foot.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FootSide {
    Left,
    Right,
}

/// Configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FootHeelConfig {
    pub max_pad: f32,
}

impl Default for FootHeelConfig {
    fn default() -> Self {
        FootHeelConfig { max_pad: 1.0 }
    }
}

/// State per foot.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FootHeelEntry {
    pub pad: f32,
    pub calcaneus: f32,
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FootHeelState {
    left: FootHeelEntry,
    right: FootHeelEntry,
    config: FootHeelConfig,
}

/// Default config.
pub fn default_foot_heel_config() -> FootHeelConfig {
    FootHeelConfig::default()
}

/// New neutral state.
pub fn new_foot_heel_state(config: FootHeelConfig) -> FootHeelState {
    FootHeelState {
        left: FootHeelEntry {
            pad: 0.5,
            calcaneus: 0.0,
        },
        right: FootHeelEntry {
            pad: 0.5,
            calcaneus: 0.0,
        },
        config,
    }
}

fn entry_mut(state: &mut FootHeelState, side: FootSide) -> &mut FootHeelEntry {
    match side {
        FootSide::Left => &mut state.left,
        FootSide::Right => &mut state.right,
    }
}

fn entry_ref(state: &FootHeelState, side: FootSide) -> &FootHeelEntry {
    match side {
        FootSide::Left => &state.left,
        FootSide::Right => &state.right,
    }
}

/// Set heel pad thickness for a side.
pub fn fhc_set_pad(state: &mut FootHeelState, side: FootSide, v: f32) {
    entry_mut(state, side).pad = v.clamp(0.0, 1.0);
}

/// Set calcaneus prominence for a side.
pub fn fhc_set_calcaneus(state: &mut FootHeelState, side: FootSide, v: f32) {
    entry_mut(state, side).calcaneus = v.clamp(0.0, 1.0);
}

/// Set both sides to the same pad value.
pub fn fhc_set_both_pad(state: &mut FootHeelState, v: f32) {
    let v = v.clamp(0.0, 1.0);
    state.left.pad = v;
    state.right.pad = v;
}

/// Reset.
pub fn fhc_reset(state: &mut FootHeelState) {
    state.left = FootHeelEntry {
        pad: 0.5,
        calcaneus: 0.0,
    };
    state.right = FootHeelEntry {
        pad: 0.5,
        calcaneus: 0.0,
    };
}

/// True if both heels are at neutral.
pub fn fhc_is_neutral(state: &FootHeelState) -> bool {
    (state.left.pad - 0.5).abs() < 1e-5
        && (state.right.pad - 0.5).abs() < 1e-5
        && state.left.calcaneus < 1e-5
        && state.right.calcaneus < 1e-5
}

/// Pad value for one side.
pub fn fhc_pad(state: &FootHeelState, side: FootSide) -> f32 {
    entry_ref(state, side).pad
}

/// Asymmetry of pad thickness.
pub fn fhc_pad_asymmetry(state: &FootHeelState) -> f32 {
    (state.left.pad - state.right.pad).abs()
}

/// Morph weights as flat array: `[left_pad, right_pad, left_cal, right_cal]`.
pub fn fhc_to_weights(state: &FootHeelState) -> [f32; 4] {
    [
        state.left.pad,
        state.right.pad,
        state.left.calcaneus,
        state.right.calcaneus,
    ]
}

/// Serialise.
pub fn fhc_to_json(state: &FootHeelState) -> String {
    format!(
        r#"{{"left_pad":{:.4},"right_pad":{:.4},"left_cal":{:.4},"right_cal":{:.4}}}"#,
        state.left.pad, state.right.pad, state.left.calcaneus, state.right.calcaneus
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> FootHeelState {
        new_foot_heel_state(default_foot_heel_config())
    }

    #[test]
    fn neutral_on_creation() {
        assert!(fhc_is_neutral(&make()));
    }

    #[test]
    fn set_pad_clamps_high() {
        let mut s = make();
        fhc_set_pad(&mut s, FootSide::Left, 10.0);
        assert!((s.left.pad - 1.0).abs() < 1e-5);
    }

    #[test]
    fn set_both_pads_equal() {
        let mut s = make();
        fhc_set_both_pad(&mut s, 0.7);
        assert!((s.left.pad - s.right.pad).abs() < 1e-5);
    }

    #[test]
    fn reset_restores_neutral() {
        let mut s = make();
        fhc_set_pad(&mut s, FootSide::Left, 1.0);
        fhc_reset(&mut s);
        assert!(fhc_is_neutral(&s));
    }

    #[test]
    fn pad_asymmetry_zero_when_equal() {
        let mut s = make();
        fhc_set_both_pad(&mut s, 0.6);
        assert!(fhc_pad_asymmetry(&s) < 1e-5);
    }

    #[test]
    fn weights_in_range() {
        let s = make();
        for v in fhc_to_weights(&s) {
            assert!((0.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn json_has_left_pad() {
        assert!(fhc_to_json(&make()).contains("left_pad"));
    }

    #[test]
    fn calcaneus_clamped() {
        let mut s = make();
        fhc_set_calcaneus(&mut s, FootSide::Right, -1.0);
        assert!(s.right.calcaneus >= 0.0);
    }

    #[test]
    fn pad_returns_correct_side() {
        let mut s = make();
        fhc_set_pad(&mut s, FootSide::Right, 0.3);
        assert!((fhc_pad(&s, FootSide::Right) - 0.3).abs() < 1e-5);
    }
}
