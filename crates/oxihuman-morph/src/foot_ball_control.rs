// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Foot ball control — metatarsal region width and padding.

/// Configuration for foot ball control.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FootBallConfig {
    pub max_width: f32,
}

/// Side selector.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FootBallSide {
    Left,
    Right,
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FootBallState {
    pub left_width: f32,
    pub right_width: f32,
    pub padding: f32,
}

#[allow(dead_code)]
pub fn default_foot_ball_config() -> FootBallConfig {
    FootBallConfig { max_width: 1.0 }
}

#[allow(dead_code)]
pub fn new_foot_ball_state() -> FootBallState {
    FootBallState {
        left_width: 0.0,
        right_width: 0.0,
        padding: 0.0,
    }
}

#[allow(dead_code)]
pub fn fb_set_width(state: &mut FootBallState, cfg: &FootBallConfig, side: FootBallSide, v: f32) {
    let clamped = v.clamp(0.0, cfg.max_width);
    match side {
        FootBallSide::Left => state.left_width = clamped,
        FootBallSide::Right => state.right_width = clamped,
    }
}

#[allow(dead_code)]
pub fn fb_set_both(state: &mut FootBallState, cfg: &FootBallConfig, v: f32) {
    let clamped = v.clamp(0.0, cfg.max_width);
    state.left_width = clamped;
    state.right_width = clamped;
}

#[allow(dead_code)]
pub fn fb_set_padding(state: &mut FootBallState, v: f32) {
    state.padding = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn fb_reset(state: &mut FootBallState) {
    *state = new_foot_ball_state();
}

#[allow(dead_code)]
pub fn fb_is_neutral(state: &FootBallState) -> bool {
    state.left_width.abs() < 1e-6 && state.right_width.abs() < 1e-6 && state.padding.abs() < 1e-6
}

#[allow(dead_code)]
pub fn fb_average_width(state: &FootBallState) -> f32 {
    (state.left_width + state.right_width) * 0.5
}

#[allow(dead_code)]
pub fn fb_symmetry(state: &FootBallState) -> f32 {
    (state.left_width - state.right_width).abs()
}

#[allow(dead_code)]
pub fn fb_blend(a: &FootBallState, b: &FootBallState, t: f32) -> FootBallState {
    let t = t.clamp(0.0, 1.0);
    FootBallState {
        left_width: a.left_width + (b.left_width - a.left_width) * t,
        right_width: a.right_width + (b.right_width - a.right_width) * t,
        padding: a.padding + (b.padding - a.padding) * t,
    }
}

#[allow(dead_code)]
pub fn fb_to_weights(state: &FootBallState) -> Vec<(String, f32)> {
    vec![
        ("foot_ball_width_l".to_string(), state.left_width),
        ("foot_ball_width_r".to_string(), state.right_width),
        ("foot_ball_padding".to_string(), state.padding),
    ]
}

#[allow(dead_code)]
pub fn fb_to_json(state: &FootBallState) -> String {
    format!(
        r#"{{"left_width":{:.4},"right_width":{:.4},"padding":{:.4}}}"#,
        state.left_width, state.right_width, state.padding
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_foot_ball_config();
        assert!((cfg.max_width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_state_neutral() {
        let s = new_foot_ball_state();
        assert!(fb_is_neutral(&s));
    }

    #[test]
    fn set_width_left() {
        let cfg = default_foot_ball_config();
        let mut s = new_foot_ball_state();
        fb_set_width(&mut s, &cfg, FootBallSide::Left, 0.5);
        assert!((s.left_width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn set_width_clamps() {
        let cfg = default_foot_ball_config();
        let mut s = new_foot_ball_state();
        fb_set_width(&mut s, &cfg, FootBallSide::Right, 2.0);
        assert!((s.right_width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_both_symmetric() {
        let cfg = default_foot_ball_config();
        let mut s = new_foot_ball_state();
        fb_set_both(&mut s, &cfg, 0.6);
        assert!(fb_symmetry(&s) < 1e-6);
    }

    #[test]
    fn set_padding() {
        let mut s = new_foot_ball_state();
        fb_set_padding(&mut s, 0.4);
        assert!((s.padding - 0.4).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_foot_ball_config();
        let mut s = new_foot_ball_state();
        fb_set_both(&mut s, &cfg, 0.8);
        fb_reset(&mut s);
        assert!(fb_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let a = new_foot_ball_state();
        let cfg = default_foot_ball_config();
        let mut b = new_foot_ball_state();
        fb_set_both(&mut b, &cfg, 1.0);
        let m = fb_blend(&a, &b, 0.5);
        assert!((m.left_width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_weights_count() {
        let s = new_foot_ball_state();
        assert_eq!(fb_to_weights(&s).len(), 3);
    }

    #[test]
    fn to_json_fields() {
        let s = new_foot_ball_state();
        let j = fb_to_json(&s);
        assert!(j.contains("left_width"));
    }
}
