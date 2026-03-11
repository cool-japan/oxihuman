// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Body twist control — rotational twist of torso around vertical axis.

use std::f32::consts::PI;

/// Configuration for body twist.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyTwistConfig {
    pub max_angle_rad: f32,
}

/// Runtime state for body twist.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyTwistState {
    pub upper_twist_rad: f32,
    pub lower_twist_rad: f32,
}

#[allow(dead_code)]
pub fn default_body_twist_config() -> BodyTwistConfig {
    BodyTwistConfig { max_angle_rad: PI }
}

#[allow(dead_code)]
pub fn new_body_twist_state() -> BodyTwistState {
    BodyTwistState {
        upper_twist_rad: 0.0,
        lower_twist_rad: 0.0,
    }
}

#[allow(dead_code)]
pub fn btwist_set_upper(state: &mut BodyTwistState, cfg: &BodyTwistConfig, v: f32) {
    state.upper_twist_rad = v.clamp(-cfg.max_angle_rad, cfg.max_angle_rad);
}

#[allow(dead_code)]
pub fn btwist_set_lower(state: &mut BodyTwistState, cfg: &BodyTwistConfig, v: f32) {
    state.lower_twist_rad = v.clamp(-cfg.max_angle_rad, cfg.max_angle_rad);
}

#[allow(dead_code)]
pub fn btwist_reset(state: &mut BodyTwistState) {
    *state = new_body_twist_state();
}

#[allow(dead_code)]
pub fn btwist_is_neutral(state: &BodyTwistState) -> bool {
    state.upper_twist_rad.abs() < 1e-6 && state.lower_twist_rad.abs() < 1e-6
}

#[allow(dead_code)]
pub fn btwist_total_angle_rad(state: &BodyTwistState) -> f32 {
    state.upper_twist_rad - state.lower_twist_rad
}

#[allow(dead_code)]
pub fn btwist_blend(a: &BodyTwistState, b: &BodyTwistState, t: f32) -> BodyTwistState {
    let t = t.clamp(0.0, 1.0);
    BodyTwistState {
        upper_twist_rad: a.upper_twist_rad + (b.upper_twist_rad - a.upper_twist_rad) * t,
        lower_twist_rad: a.lower_twist_rad + (b.lower_twist_rad - a.lower_twist_rad) * t,
    }
}

#[allow(dead_code)]
pub fn btwist_to_weights(state: &BodyTwistState) -> Vec<(String, f32)> {
    let norm = 1.0 / PI;
    vec![
        ("body_twist_upper".to_string(), state.upper_twist_rad * norm),
        ("body_twist_lower".to_string(), state.lower_twist_rad * norm),
    ]
}

#[allow(dead_code)]
pub fn btwist_to_json(state: &BodyTwistState) -> String {
    format!(
        r#"{{"upper_twist_rad":{:.4},"lower_twist_rad":{:.4}}}"#,
        state.upper_twist_rad, state.lower_twist_rad
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_pi() {
        let cfg = default_body_twist_config();
        assert!((cfg.max_angle_rad - PI).abs() < 1e-6);
    }

    #[test]
    fn new_state_neutral() {
        let s = new_body_twist_state();
        assert!(btwist_is_neutral(&s));
    }

    #[test]
    fn set_upper_clamps() {
        let cfg = default_body_twist_config();
        let mut s = new_body_twist_state();
        btwist_set_upper(&mut s, &cfg, 10.0);
        assert!((s.upper_twist_rad - PI).abs() < 1e-6);
    }

    #[test]
    fn set_upper_negative_clamps() {
        let cfg = default_body_twist_config();
        let mut s = new_body_twist_state();
        btwist_set_upper(&mut s, &cfg, -10.0);
        assert!((s.upper_twist_rad + PI).abs() < 1e-6);
    }

    #[test]
    fn set_lower() {
        let cfg = default_body_twist_config();
        let mut s = new_body_twist_state();
        btwist_set_lower(&mut s, &cfg, 0.5);
        assert!((s.lower_twist_rad - 0.5).abs() < 1e-6);
    }

    #[test]
    fn total_angle() {
        let cfg = default_body_twist_config();
        let mut s = new_body_twist_state();
        btwist_set_upper(&mut s, &cfg, 1.0);
        btwist_set_lower(&mut s, &cfg, -0.5);
        assert!((btwist_total_angle_rad(&s) - 1.5).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_body_twist_config();
        let mut s = new_body_twist_state();
        btwist_set_upper(&mut s, &cfg, 0.8);
        btwist_reset(&mut s);
        assert!(btwist_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let a = new_body_twist_state();
        let cfg = default_body_twist_config();
        let mut b = new_body_twist_state();
        btwist_set_upper(&mut b, &cfg, 1.0);
        let m = btwist_blend(&a, &b, 0.5);
        assert!((m.upper_twist_rad - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_weights_len() {
        let s = new_body_twist_state();
        assert_eq!(btwist_to_weights(&s).len(), 2);
    }

    #[test]
    fn to_json_contains_fields() {
        let s = new_body_twist_state();
        let j = btwist_to_json(&s);
        assert!(j.contains("upper_twist_rad"));
        assert!(j.contains("lower_twist_rad"));
    }
}
