// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Brow-tail (lateral end) raise/lower and angle control.

/// Side selector for brow tail.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrowTailSide {
    Left,
    Right,
    Both,
}

/// Runtime state for brow-tail positions.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct BrowTailState {
    /// Tail raise left  (0 = neutral, 1 = max raise, -1 = max lower).
    pub raise_left: f32,
    /// Tail raise right (0 = neutral, 1 = max raise, -1 = max lower).
    pub raise_right: f32,
    /// Angular tilt of the tail (positive = upward tilt, degrees).
    pub angle_left: f32,
    pub angle_right: f32,
}

/// Configuration limits.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct BrowTailConfig {
    pub max_raise: f32,
    pub max_lower: f32,
    pub max_angle_deg: f32,
}

impl Default for BrowTailConfig {
    fn default() -> Self {
        Self {
            max_raise: 1.0,
            max_lower: 1.0,
            max_angle_deg: 30.0,
        }
    }
}

impl Default for BrowTailState {
    fn default() -> Self {
        Self {
            raise_left: 0.0,
            raise_right: 0.0,
            angle_left: 0.0,
            angle_right: 0.0,
        }
    }
}

#[allow(dead_code)]
pub fn new_brow_tail_state() -> BrowTailState {
    BrowTailState::default()
}

#[allow(dead_code)]
pub fn default_brow_tail_config() -> BrowTailConfig {
    BrowTailConfig::default()
}

#[allow(dead_code)]
pub fn bt_set_raise(state: &mut BrowTailState, cfg: &BrowTailConfig, side: BrowTailSide, v: f32) {
    let v = v.clamp(-cfg.max_lower, cfg.max_raise);
    match side {
        BrowTailSide::Left => state.raise_left = v,
        BrowTailSide::Right => state.raise_right = v,
        BrowTailSide::Both => {
            state.raise_left = v;
            state.raise_right = v;
        }
    }
}

#[allow(dead_code)]
pub fn bt_set_angle(state: &mut BrowTailState, cfg: &BrowTailConfig, side: BrowTailSide, deg: f32) {
    let d = deg.clamp(-cfg.max_angle_deg, cfg.max_angle_deg);
    match side {
        BrowTailSide::Left => state.angle_left = d,
        BrowTailSide::Right => state.angle_right = d,
        BrowTailSide::Both => {
            state.angle_left = d;
            state.angle_right = d;
        }
    }
}

#[allow(dead_code)]
pub fn bt_reset(state: &mut BrowTailState) {
    *state = BrowTailState::default();
}

#[allow(dead_code)]
pub fn bt_symmetry(state: &BrowTailState) -> f32 {
    1.0 - (state.raise_left - state.raise_right).abs().min(1.0)
}

#[allow(dead_code)]
pub fn bt_blend(a: &BrowTailState, b: &BrowTailState, t: f32) -> BrowTailState {
    let t = t.clamp(0.0, 1.0);
    BrowTailState {
        raise_left: a.raise_left + (b.raise_left - a.raise_left) * t,
        raise_right: a.raise_right + (b.raise_right - a.raise_right) * t,
        angle_left: a.angle_left + (b.angle_left - a.angle_left) * t,
        angle_right: a.angle_right + (b.angle_right - a.angle_right) * t,
    }
}

#[allow(dead_code)]
pub fn bt_to_morph_weights(state: &BrowTailState) -> [f32; 4] {
    [
        state.raise_left,
        state.raise_right,
        state.angle_left / 30.0,
        state.angle_right / 30.0,
    ]
}

#[allow(dead_code)]
pub fn bt_is_neutral(state: &BrowTailState) -> bool {
    state.raise_left.abs() < 1e-4
        && state.raise_right.abs() < 1e-4
        && state.angle_left.abs() < 1e-4
        && state.angle_right.abs() < 1e-4
}

#[allow(dead_code)]
pub fn bt_to_json(state: &BrowTailState) -> String {
    format!(
        "{{\"raise_left\":{:.4},\"raise_right\":{:.4},\"angle_left\":{:.4},\"angle_right\":{:.4}}}",
        state.raise_left, state.raise_right, state.angle_left, state.angle_right
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_neutral() {
        assert!(bt_is_neutral(&new_brow_tail_state()));
    }

    #[test]
    fn set_raise_clamps_max() {
        let mut s = new_brow_tail_state();
        let cfg = default_brow_tail_config();
        bt_set_raise(&mut s, &cfg, BrowTailSide::Left, 2.0);
        assert!(s.raise_left <= cfg.max_raise);
    }

    #[test]
    fn set_raise_clamps_min() {
        let mut s = new_brow_tail_state();
        let cfg = default_brow_tail_config();
        bt_set_raise(&mut s, &cfg, BrowTailSide::Right, -5.0);
        assert!(s.raise_right >= -cfg.max_lower);
    }

    #[test]
    fn both_side_sets_both() {
        let mut s = new_brow_tail_state();
        let cfg = default_brow_tail_config();
        bt_set_raise(&mut s, &cfg, BrowTailSide::Both, 0.5);
        assert!((s.raise_left - 0.5).abs() < 1e-5);
        assert!((s.raise_right - 0.5).abs() < 1e-5);
    }

    #[test]
    fn reset_clears_state() {
        let mut s = new_brow_tail_state();
        let cfg = default_brow_tail_config();
        bt_set_raise(&mut s, &cfg, BrowTailSide::Both, 0.8);
        bt_reset(&mut s);
        assert!(bt_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let mut a = new_brow_tail_state();
        let mut b = new_brow_tail_state();
        let cfg = default_brow_tail_config();
        bt_set_raise(&mut a, &cfg, BrowTailSide::Left, 0.0);
        bt_set_raise(&mut b, &cfg, BrowTailSide::Left, 1.0);
        let mid = bt_blend(&a, &b, 0.5);
        assert!((mid.raise_left - 0.5).abs() < 1e-4);
    }

    #[test]
    fn symmetry_one_when_equal() {
        let s = new_brow_tail_state();
        assert!((bt_symmetry(&s) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn morph_weights_len() {
        let s = new_brow_tail_state();
        assert_eq!(bt_to_morph_weights(&s).len(), 4);
    }

    #[test]
    fn angle_clamp() {
        let mut s = new_brow_tail_state();
        let cfg = default_brow_tail_config();
        bt_set_angle(&mut s, &cfg, BrowTailSide::Left, 999.0);
        assert!(s.angle_left <= cfg.max_angle_deg);
    }

    #[test]
    fn json_output_not_empty() {
        let s = new_brow_tail_state();
        assert!(!bt_to_json(&s).is_empty());
    }
}
