// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Shoulder pad control — deltoid bulk and acromion prominence.

/// Configuration for shoulder pad.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShoulderPadConfig {
    pub max_bulk: f32,
}

/// Side selector.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShoulderPadSide {
    Left,
    Right,
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShoulderPadState {
    pub left_bulk: f32,
    pub right_bulk: f32,
    pub acromion_prominence: f32,
}

#[allow(dead_code)]
pub fn default_shoulder_pad_config() -> ShoulderPadConfig {
    ShoulderPadConfig { max_bulk: 1.0 }
}

#[allow(dead_code)]
pub fn new_shoulder_pad_state() -> ShoulderPadState {
    ShoulderPadState {
        left_bulk: 0.0,
        right_bulk: 0.0,
        acromion_prominence: 0.0,
    }
}

#[allow(dead_code)]
pub fn spad_set_bulk(
    state: &mut ShoulderPadState,
    cfg: &ShoulderPadConfig,
    side: ShoulderPadSide,
    v: f32,
) {
    let clamped = v.clamp(0.0, cfg.max_bulk);
    match side {
        ShoulderPadSide::Left => state.left_bulk = clamped,
        ShoulderPadSide::Right => state.right_bulk = clamped,
    }
}

#[allow(dead_code)]
pub fn spad_set_both(state: &mut ShoulderPadState, cfg: &ShoulderPadConfig, v: f32) {
    let clamped = v.clamp(0.0, cfg.max_bulk);
    state.left_bulk = clamped;
    state.right_bulk = clamped;
}

#[allow(dead_code)]
pub fn spad_set_acromion(state: &mut ShoulderPadState, v: f32) {
    state.acromion_prominence = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn spad_reset(state: &mut ShoulderPadState) {
    *state = new_shoulder_pad_state();
}

#[allow(dead_code)]
pub fn spad_is_neutral(state: &ShoulderPadState) -> bool {
    state.left_bulk.abs() < 1e-6
        && state.right_bulk.abs() < 1e-6
        && state.acromion_prominence.abs() < 1e-6
}

#[allow(dead_code)]
pub fn spad_average_bulk(state: &ShoulderPadState) -> f32 {
    (state.left_bulk + state.right_bulk) * 0.5
}

#[allow(dead_code)]
pub fn spad_symmetry(state: &ShoulderPadState) -> f32 {
    (state.left_bulk - state.right_bulk).abs()
}

#[allow(dead_code)]
pub fn spad_blend(a: &ShoulderPadState, b: &ShoulderPadState, t: f32) -> ShoulderPadState {
    let t = t.clamp(0.0, 1.0);
    ShoulderPadState {
        left_bulk: a.left_bulk + (b.left_bulk - a.left_bulk) * t,
        right_bulk: a.right_bulk + (b.right_bulk - a.right_bulk) * t,
        acromion_prominence: a.acromion_prominence
            + (b.acromion_prominence - a.acromion_prominence) * t,
    }
}

#[allow(dead_code)]
pub fn spad_to_weights(state: &ShoulderPadState) -> Vec<(String, f32)> {
    vec![
        ("shoulder_pad_bulk_l".to_string(), state.left_bulk),
        ("shoulder_pad_bulk_r".to_string(), state.right_bulk),
        ("acromion_prominence".to_string(), state.acromion_prominence),
    ]
}

#[allow(dead_code)]
pub fn spad_to_json(state: &ShoulderPadState) -> String {
    format!(
        r#"{{"left_bulk":{:.4},"right_bulk":{:.4},"acromion":{:.4}}}"#,
        state.left_bulk, state.right_bulk, state.acromion_prominence
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_shoulder_pad_config();
        assert!((cfg.max_bulk - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_state_neutral() {
        let s = new_shoulder_pad_state();
        assert!(spad_is_neutral(&s));
    }

    #[test]
    fn set_bulk_left() {
        let cfg = default_shoulder_pad_config();
        let mut s = new_shoulder_pad_state();
        spad_set_bulk(&mut s, &cfg, ShoulderPadSide::Left, 0.6);
        assert!((s.left_bulk - 0.6).abs() < 1e-6);
    }

    #[test]
    fn set_bulk_clamps() {
        let cfg = default_shoulder_pad_config();
        let mut s = new_shoulder_pad_state();
        spad_set_bulk(&mut s, &cfg, ShoulderPadSide::Right, 5.0);
        assert!((s.right_bulk - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_both_symmetric() {
        let cfg = default_shoulder_pad_config();
        let mut s = new_shoulder_pad_state();
        spad_set_both(&mut s, &cfg, 0.4);
        assert!(spad_symmetry(&s) < 1e-6);
    }

    #[test]
    fn set_acromion() {
        let mut s = new_shoulder_pad_state();
        spad_set_acromion(&mut s, 0.7);
        assert!((s.acromion_prominence - 0.7).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_shoulder_pad_config();
        let mut s = new_shoulder_pad_state();
        spad_set_both(&mut s, &cfg, 0.8);
        spad_reset(&mut s);
        assert!(spad_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let a = new_shoulder_pad_state();
        let cfg = default_shoulder_pad_config();
        let mut b = new_shoulder_pad_state();
        spad_set_both(&mut b, &cfg, 1.0);
        let m = spad_blend(&a, &b, 0.5);
        assert!((m.left_bulk - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_weights_count() {
        let s = new_shoulder_pad_state();
        assert_eq!(spad_to_weights(&s).len(), 3);
    }

    #[test]
    fn to_json_fields() {
        let s = new_shoulder_pad_state();
        let j = spad_to_json(&s);
        assert!(j.contains("left_bulk"));
    }
}
