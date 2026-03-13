// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Jaw clench (masseter bulge) control — bilateral and unilateral clenching.

use std::f32::consts::PI;

/// Config.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct JawClenchConfig {
    pub max_bulge_m: f32,
}

impl Default for JawClenchConfig {
    fn default() -> Self {
        Self { max_bulge_m: 0.007 }
    }
}

/// State.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct JawClenchState {
    /// Left masseter clench, 0..=1.
    pub left: f32,
    /// Right masseter clench, 0..=1.
    pub right: f32,
}

#[allow(dead_code)]
pub fn new_jaw_clench_state() -> JawClenchState {
    JawClenchState::default()
}

#[allow(dead_code)]
pub fn default_jaw_clench_config() -> JawClenchConfig {
    JawClenchConfig::default()
}

/// Clench side enum.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClenchSide {
    Left,
    Right,
}

#[allow(dead_code)]
pub fn jcl_set(state: &mut JawClenchState, side: ClenchSide, v: f32) {
    let v = v.clamp(0.0, 1.0);
    match side {
        ClenchSide::Left => state.left = v,
        ClenchSide::Right => state.right = v,
    }
}

#[allow(dead_code)]
pub fn jcl_set_both(state: &mut JawClenchState, v: f32) {
    let v = v.clamp(0.0, 1.0);
    state.left = v;
    state.right = v;
}

#[allow(dead_code)]
pub fn jcl_reset(state: &mut JawClenchState) {
    *state = JawClenchState::default();
}

#[allow(dead_code)]
pub fn jcl_is_neutral(state: &JawClenchState) -> bool {
    state.left < 1e-4 && state.right < 1e-4
}

#[allow(dead_code)]
pub fn jcl_asymmetry(state: &JawClenchState) -> f32 {
    (state.left - state.right).abs()
}

/// Bite force approximation (0..=1, normalised).
#[allow(dead_code)]
pub fn jcl_bite_force(state: &JawClenchState) -> f32 {
    ((state.left + state.right) * 0.5).min(1.0)
}

/// Temporal muscle pressure angle in radians.
#[allow(dead_code)]
pub fn jcl_temporal_angle_rad(state: &JawClenchState) -> f32 {
    jcl_bite_force(state) * (PI / 8.0)
}

#[allow(dead_code)]
pub fn jcl_to_weights(state: &JawClenchState, cfg: &JawClenchConfig) -> [f32; 2] {
    [state.left * cfg.max_bulge_m, state.right * cfg.max_bulge_m]
}

#[allow(dead_code)]
pub fn jcl_blend(a: &JawClenchState, b: &JawClenchState, t: f32) -> JawClenchState {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    JawClenchState {
        left: a.left * inv + b.left * t,
        right: a.right * inv + b.right * t,
    }
}

#[allow(dead_code)]
pub fn jcl_to_json(state: &JawClenchState) -> String {
    format!(
        "{{\"left\":{:.4},\"right\":{:.4}}}",
        state.left, state.right
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_neutral() {
        assert!(jcl_is_neutral(&new_jaw_clench_state()));
    }

    #[test]
    fn set_clamps_high() {
        let mut s = new_jaw_clench_state();
        jcl_set(&mut s, ClenchSide::Left, 5.0);
        assert!((s.left - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_clamps_low() {
        let mut s = new_jaw_clench_state();
        jcl_set(&mut s, ClenchSide::Right, -1.0);
        assert!(s.right < 1e-6);
    }

    #[test]
    fn set_both_equal() {
        let mut s = new_jaw_clench_state();
        jcl_set_both(&mut s, 0.6);
        assert!((s.left - s.right).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let mut s = new_jaw_clench_state();
        jcl_set_both(&mut s, 0.9);
        jcl_reset(&mut s);
        assert!(jcl_is_neutral(&s));
    }

    #[test]
    fn bite_force_average() {
        let mut s = new_jaw_clench_state();
        jcl_set_both(&mut s, 1.0);
        assert!((jcl_bite_force(&s) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn temporal_angle_positive_when_clenched() {
        let mut s = new_jaw_clench_state();
        jcl_set_both(&mut s, 1.0);
        assert!(jcl_temporal_angle_rad(&s) > 0.0);
    }

    #[test]
    fn weights_proportional() {
        let cfg = default_jaw_clench_config();
        let mut s = new_jaw_clench_state();
        jcl_set_both(&mut s, 1.0);
        let w = jcl_to_weights(&s, &cfg);
        assert!((w[0] - cfg.max_bulge_m).abs() < 1e-6);
    }

    #[test]
    fn blend_midpoint() {
        let mut b = new_jaw_clench_state();
        jcl_set_both(&mut b, 1.0);
        let r = jcl_blend(&new_jaw_clench_state(), &b, 0.5);
        assert!((r.left - 0.5).abs() < 1e-5);
    }

    #[test]
    fn json_has_keys() {
        let j = jcl_to_json(&new_jaw_clench_state());
        assert!(j.contains("left") && j.contains("right"));
    }
}
