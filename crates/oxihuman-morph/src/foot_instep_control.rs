// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Foot instep (dorsal arch height) control.

use std::f32::consts::FRAC_PI_4;

/// Foot side.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FootInstepSide {
    Left,
    Right,
}

/// Config.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct FootInstepConfig {
    pub max_arch_m: f32,
}

impl Default for FootInstepConfig {
    fn default() -> Self {
        Self { max_arch_m: 0.014 }
    }
}

/// State.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct FootInstepState {
    pub left_arch: f32,
    pub right_arch: f32,
}

#[allow(dead_code)]
pub fn new_foot_instep_state() -> FootInstepState {
    FootInstepState::default()
}

#[allow(dead_code)]
pub fn default_foot_instep_config() -> FootInstepConfig {
    FootInstepConfig::default()
}

#[allow(dead_code)]
pub fn fi_set_arch(state: &mut FootInstepState, side: FootInstepSide, v: f32) {
    let v = v.clamp(0.0, 1.0);
    match side {
        FootInstepSide::Left => state.left_arch = v,
        FootInstepSide::Right => state.right_arch = v,
    }
}

#[allow(dead_code)]
pub fn fi_set_both(state: &mut FootInstepState, v: f32) {
    let v = v.clamp(0.0, 1.0);
    state.left_arch = v;
    state.right_arch = v;
}

#[allow(dead_code)]
pub fn fi_reset(state: &mut FootInstepState) {
    *state = FootInstepState::default();
}

#[allow(dead_code)]
pub fn fi_is_neutral(state: &FootInstepState) -> bool {
    state.left_arch < 1e-4 && state.right_arch < 1e-4
}

#[allow(dead_code)]
pub fn fi_average_arch(state: &FootInstepState) -> f32 {
    (state.left_arch + state.right_arch) * 0.5
}

/// Arch angle in radians.
#[allow(dead_code)]
pub fn fi_arch_angle_rad(state: &FootInstepState, side: FootInstepSide) -> f32 {
    let v = match side {
        FootInstepSide::Left => state.left_arch,
        FootInstepSide::Right => state.right_arch,
    };
    v * FRAC_PI_4 * 0.5
}

#[allow(dead_code)]
pub fn fi_to_weights(state: &FootInstepState, cfg: &FootInstepConfig) -> [f32; 2] {
    [
        state.left_arch * cfg.max_arch_m,
        state.right_arch * cfg.max_arch_m,
    ]
}

#[allow(dead_code)]
pub fn fi_blend(a: &FootInstepState, b: &FootInstepState, t: f32) -> FootInstepState {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    FootInstepState {
        left_arch: a.left_arch * inv + b.left_arch * t,
        right_arch: a.right_arch * inv + b.right_arch * t,
    }
}

#[allow(dead_code)]
pub fn fi_to_json(state: &FootInstepState) -> String {
    format!(
        "{{\"left_arch\":{:.4},\"right_arch\":{:.4}}}",
        state.left_arch, state.right_arch
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_neutral() {
        assert!(fi_is_neutral(&new_foot_instep_state()));
    }

    #[test]
    fn set_clamps_high() {
        let mut s = new_foot_instep_state();
        fi_set_arch(&mut s, FootInstepSide::Left, 10.0);
        assert!((s.left_arch - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_clamps_low() {
        let mut s = new_foot_instep_state();
        fi_set_arch(&mut s, FootInstepSide::Right, -5.0);
        assert!(s.right_arch < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let mut s = new_foot_instep_state();
        fi_set_both(&mut s, 0.7);
        fi_reset(&mut s);
        assert!(fi_is_neutral(&s));
    }

    #[test]
    fn average_arch_correct() {
        let mut s = new_foot_instep_state();
        fi_set_arch(&mut s, FootInstepSide::Left, 0.4);
        fi_set_arch(&mut s, FootInstepSide::Right, 0.6);
        assert!((fi_average_arch(&s) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn arch_angle_positive() {
        let mut s = new_foot_instep_state();
        fi_set_arch(&mut s, FootInstepSide::Left, 1.0);
        assert!(fi_arch_angle_rad(&s, FootInstepSide::Left) > 0.0);
    }

    #[test]
    fn weights_scale_correctly() {
        let cfg = default_foot_instep_config();
        let mut s = new_foot_instep_state();
        fi_set_both(&mut s, 1.0);
        let w = fi_to_weights(&s, &cfg);
        assert!((w[0] - cfg.max_arch_m).abs() < 1e-6);
    }

    #[test]
    fn blend_midpoint() {
        let mut b = new_foot_instep_state();
        fi_set_both(&mut b, 1.0);
        let r = fi_blend(&new_foot_instep_state(), &b, 0.5);
        assert!((r.left_arch - 0.5).abs() < 1e-5);
    }

    #[test]
    fn json_has_keys() {
        let j = fi_to_json(&new_foot_instep_state());
        assert!(j.contains("left_arch") && j.contains("right_arch"));
    }

    #[test]
    fn set_both_equal() {
        let mut s = new_foot_instep_state();
        fi_set_both(&mut s, 0.3);
        assert!((s.left_arch - s.right_arch).abs() < 1e-6);
    }
}
