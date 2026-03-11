// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Jaw twist control — mandibular rotation around the vertical axis.

use std::f32::consts::FRAC_PI_8;

/// Configuration for jaw twist.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JawTwistConfig {
    pub max_twist_rad: f32,
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JawTwistState {
    pub twist_rad: f32,
    pub lateral_shift: f32,
}

#[allow(dead_code)]
pub fn default_jaw_twist_config() -> JawTwistConfig {
    JawTwistConfig {
        max_twist_rad: FRAC_PI_8,
    }
}

#[allow(dead_code)]
pub fn new_jaw_twist_state() -> JawTwistState {
    JawTwistState {
        twist_rad: 0.0,
        lateral_shift: 0.0,
    }
}

#[allow(dead_code)]
pub fn jtwist_set_twist(state: &mut JawTwistState, cfg: &JawTwistConfig, v: f32) {
    state.twist_rad = v.clamp(-cfg.max_twist_rad, cfg.max_twist_rad);
}

#[allow(dead_code)]
pub fn jtwist_set_lateral(state: &mut JawTwistState, v: f32) {
    state.lateral_shift = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn jtwist_reset(state: &mut JawTwistState) {
    *state = new_jaw_twist_state();
}

#[allow(dead_code)]
pub fn jtwist_is_neutral(state: &JawTwistState) -> bool {
    state.twist_rad.abs() < 1e-6 && state.lateral_shift.abs() < 1e-6
}

#[allow(dead_code)]
pub fn jtwist_total_displacement(state: &JawTwistState) -> f32 {
    state.twist_rad.abs() + state.lateral_shift.abs()
}

#[allow(dead_code)]
pub fn jtwist_blend(a: &JawTwistState, b: &JawTwistState, t: f32) -> JawTwistState {
    let t = t.clamp(0.0, 1.0);
    JawTwistState {
        twist_rad: a.twist_rad + (b.twist_rad - a.twist_rad) * t,
        lateral_shift: a.lateral_shift + (b.lateral_shift - a.lateral_shift) * t,
    }
}

#[allow(dead_code)]
pub fn jtwist_to_weights(state: &JawTwistState) -> Vec<(String, f32)> {
    let norm = 1.0 / FRAC_PI_8;
    vec![
        ("jaw_twist".to_string(), state.twist_rad * norm),
        ("jaw_lateral_shift".to_string(), state.lateral_shift),
    ]
}

#[allow(dead_code)]
pub fn jtwist_to_json(state: &JawTwistState) -> String {
    format!(
        r#"{{"twist_rad":{:.4},"lateral_shift":{:.4}}}"#,
        state.twist_rad, state.lateral_shift
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_jaw_twist_config();
        assert!((cfg.max_twist_rad - FRAC_PI_8).abs() < 1e-6);
    }

    #[test]
    fn new_state_neutral() {
        let s = new_jaw_twist_state();
        assert!(jtwist_is_neutral(&s));
    }

    #[test]
    fn set_twist_clamps() {
        let cfg = default_jaw_twist_config();
        let mut s = new_jaw_twist_state();
        jtwist_set_twist(&mut s, &cfg, 10.0);
        assert!((s.twist_rad - FRAC_PI_8).abs() < 1e-6);
    }

    #[test]
    fn set_twist_negative() {
        let cfg = default_jaw_twist_config();
        let mut s = new_jaw_twist_state();
        jtwist_set_twist(&mut s, &cfg, -FRAC_PI_8);
        assert!((s.twist_rad + FRAC_PI_8).abs() < 1e-6);
    }

    #[test]
    fn set_lateral() {
        let mut s = new_jaw_twist_state();
        jtwist_set_lateral(&mut s, 0.3);
        assert!((s.lateral_shift - 0.3).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_jaw_twist_config();
        let mut s = new_jaw_twist_state();
        jtwist_set_twist(&mut s, &cfg, 0.2);
        jtwist_reset(&mut s);
        assert!(jtwist_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let a = new_jaw_twist_state();
        let cfg = default_jaw_twist_config();
        let mut b = new_jaw_twist_state();
        jtwist_set_twist(&mut b, &cfg, FRAC_PI_8);
        let m = jtwist_blend(&a, &b, 0.5);
        assert!((m.twist_rad - FRAC_PI_8 * 0.5).abs() < 1e-5);
    }

    #[test]
    fn to_weights_count() {
        let s = new_jaw_twist_state();
        assert_eq!(jtwist_to_weights(&s).len(), 2);
    }

    #[test]
    fn total_displacement_positive() {
        let cfg = default_jaw_twist_config();
        let mut s = new_jaw_twist_state();
        jtwist_set_twist(&mut s, &cfg, 0.2);
        assert!(jtwist_total_displacement(&s) > 0.0);
    }

    #[test]
    fn to_json_fields() {
        let s = new_jaw_twist_state();
        let j = jtwist_to_json(&s);
        assert!(j.contains("twist_rad"));
    }
}
