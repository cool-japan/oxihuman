// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Hand vein morph — controls prominence and pattern of dorsal hand veins.

/// Configuration for hand vein control.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HandVeinConfig {
    pub max_prominence: f32,
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HandVeinState {
    pub left_prominence: f32,
    pub right_prominence: f32,
    pub left_branching: f32,
    pub right_branching: f32,
}

#[allow(dead_code)]
pub fn default_hand_vein_config() -> HandVeinConfig {
    HandVeinConfig {
        max_prominence: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_hand_vein_state() -> HandVeinState {
    HandVeinState {
        left_prominence: 0.0,
        right_prominence: 0.0,
        left_branching: 0.0,
        right_branching: 0.0,
    }
}

#[allow(dead_code)]
pub fn hv_set_left(state: &mut HandVeinState, cfg: &HandVeinConfig, v: f32) {
    state.left_prominence = v.clamp(0.0, cfg.max_prominence);
}

#[allow(dead_code)]
pub fn hv_set_right(state: &mut HandVeinState, cfg: &HandVeinConfig, v: f32) {
    state.right_prominence = v.clamp(0.0, cfg.max_prominence);
}

#[allow(dead_code)]
pub fn hv_set_both(state: &mut HandVeinState, cfg: &HandVeinConfig, v: f32) {
    let clamped = v.clamp(0.0, cfg.max_prominence);
    state.left_prominence = clamped;
    state.right_prominence = clamped;
}

#[allow(dead_code)]
pub fn hv_set_branching(state: &mut HandVeinState, left: f32, right: f32) {
    state.left_branching = left.clamp(0.0, 1.0);
    state.right_branching = right.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn hv_reset(state: &mut HandVeinState) {
    *state = new_hand_vein_state();
}

#[allow(dead_code)]
pub fn hv_is_neutral(state: &HandVeinState) -> bool {
    let vals = [
        state.left_prominence,
        state.right_prominence,
        state.left_branching,
        state.right_branching,
    ];
    !vals.is_empty() && vals.iter().all(|v| v.abs() < 1e-6)
}

#[allow(dead_code)]
pub fn hv_average_prominence(state: &HandVeinState) -> f32 {
    (state.left_prominence + state.right_prominence) * 0.5
}

#[allow(dead_code)]
pub fn hv_symmetry(state: &HandVeinState) -> f32 {
    (state.left_prominence - state.right_prominence).abs()
}

#[allow(dead_code)]
pub fn hv_blend(a: &HandVeinState, b: &HandVeinState, t: f32) -> HandVeinState {
    let t = t.clamp(0.0, 1.0);
    HandVeinState {
        left_prominence: a.left_prominence + (b.left_prominence - a.left_prominence) * t,
        right_prominence: a.right_prominence + (b.right_prominence - a.right_prominence) * t,
        left_branching: a.left_branching + (b.left_branching - a.left_branching) * t,
        right_branching: a.right_branching + (b.right_branching - a.right_branching) * t,
    }
}

#[allow(dead_code)]
pub fn hv_to_weights(state: &HandVeinState) -> Vec<(String, f32)> {
    vec![
        ("hand_vein_l".to_string(), state.left_prominence),
        ("hand_vein_r".to_string(), state.right_prominence),
        ("hand_vein_branch_l".to_string(), state.left_branching),
        ("hand_vein_branch_r".to_string(), state.right_branching),
    ]
}

#[allow(dead_code)]
pub fn hv_to_json(state: &HandVeinState) -> String {
    format!(
        r#"{{"left_prominence":{:.4},"right_prominence":{:.4},"left_branching":{:.4},"right_branching":{:.4}}}"#,
        state.left_prominence, state.right_prominence, state.left_branching, state.right_branching
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_hand_vein_config();
        assert!((cfg.max_prominence - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_state_neutral() {
        let s = new_hand_vein_state();
        assert!(hv_is_neutral(&s));
    }

    #[test]
    fn set_left_clamps() {
        let cfg = default_hand_vein_config();
        let mut s = new_hand_vein_state();
        hv_set_left(&mut s, &cfg, 5.0);
        assert!((s.left_prominence - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_both_equal() {
        let cfg = default_hand_vein_config();
        let mut s = new_hand_vein_state();
        hv_set_both(&mut s, &cfg, 0.6);
        assert!((hv_symmetry(&s)).abs() < 1e-6);
    }

    #[test]
    fn set_branching() {
        let mut s = new_hand_vein_state();
        hv_set_branching(&mut s, 0.3, 0.7);
        assert!((s.left_branching - 0.3).abs() < 1e-6);
        assert!((s.right_branching - 0.7).abs() < 1e-6);
    }

    #[test]
    fn average_prominence() {
        let cfg = default_hand_vein_config();
        let mut s = new_hand_vein_state();
        hv_set_left(&mut s, &cfg, 0.4);
        hv_set_right(&mut s, &cfg, 0.6);
        assert!((hv_average_prominence(&s) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_hand_vein_config();
        let mut s = new_hand_vein_state();
        hv_set_both(&mut s, &cfg, 0.8);
        hv_reset(&mut s);
        assert!(hv_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let a = new_hand_vein_state();
        let cfg = default_hand_vein_config();
        let mut b = new_hand_vein_state();
        hv_set_both(&mut b, &cfg, 1.0);
        let mid = hv_blend(&a, &b, 0.5);
        assert!((mid.left_prominence - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_weights_count() {
        let s = new_hand_vein_state();
        assert_eq!(hv_to_weights(&s).len(), 4);
    }

    #[test]
    fn to_json_fields() {
        let s = new_hand_vein_state();
        let j = hv_to_json(&s);
        assert!(j.contains("left_prominence"));
        assert!(j.contains("right_branching"));
    }
}
