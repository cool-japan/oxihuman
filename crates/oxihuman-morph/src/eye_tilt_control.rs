// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Eye-tilt (canthal tilt / palpebral axis angle) control.

use std::f32::consts::FRAC_PI_6;

/// Side.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EyeTiltSide {
    Left,
    Right,
    Both,
}

/// State.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct EyeTiltState {
    /// Tilt angle in degrees.  Positive = lateral corner up (positive canthal tilt).
    pub tilt_left_deg: f32,
    pub tilt_right_deg: f32,
}

/// Config.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct EyeTiltConfig {
    pub max_tilt_deg: f32,
}

impl Default for EyeTiltConfig {
    fn default() -> Self {
        Self { max_tilt_deg: 20.0 }
    }
}
impl Default for EyeTiltState {
    fn default() -> Self {
        Self {
            tilt_left_deg: 0.0,
            tilt_right_deg: 0.0,
        }
    }
}

#[allow(dead_code)]
pub fn new_eye_tilt_state() -> EyeTiltState {
    EyeTiltState::default()
}

#[allow(dead_code)]
pub fn default_eye_tilt_config() -> EyeTiltConfig {
    EyeTiltConfig::default()
}

#[allow(dead_code)]
pub fn et_set_tilt(state: &mut EyeTiltState, cfg: &EyeTiltConfig, side: EyeTiltSide, deg: f32) {
    let d = deg.clamp(-cfg.max_tilt_deg, cfg.max_tilt_deg);
    match side {
        EyeTiltSide::Left => state.tilt_left_deg = d,
        EyeTiltSide::Right => state.tilt_right_deg = d,
        EyeTiltSide::Both => {
            state.tilt_left_deg = d;
            state.tilt_right_deg = d;
        }
    }
}

#[allow(dead_code)]
pub fn et_reset(state: &mut EyeTiltState) {
    *state = EyeTiltState::default();
}

#[allow(dead_code)]
pub fn et_is_neutral(state: &EyeTiltState) -> bool {
    state.tilt_left_deg.abs() < 1e-4 && state.tilt_right_deg.abs() < 1e-4
}

#[allow(dead_code)]
pub fn et_blend(a: &EyeTiltState, b: &EyeTiltState, t: f32) -> EyeTiltState {
    let t = t.clamp(0.0, 1.0);
    EyeTiltState {
        tilt_left_deg: a.tilt_left_deg + (b.tilt_left_deg - a.tilt_left_deg) * t,
        tilt_right_deg: a.tilt_right_deg + (b.tilt_right_deg - a.tilt_right_deg) * t,
    }
}

#[allow(dead_code)]
pub fn et_tilt_rad_left(state: &EyeTiltState) -> f32 {
    state.tilt_left_deg.to_radians()
}

#[allow(dead_code)]
pub fn et_tilt_rad_right(state: &EyeTiltState) -> f32 {
    state.tilt_right_deg.to_radians()
}

#[allow(dead_code)]
pub fn et_asymmetry(state: &EyeTiltState) -> f32 {
    (state.tilt_left_deg - state.tilt_right_deg).abs()
}

/// Reference angle constant used internally (30° = PI/6).
#[allow(dead_code)]
pub fn et_reference_angle_rad() -> f32 {
    FRAC_PI_6
}

#[allow(dead_code)]
pub fn et_to_weights(state: &EyeTiltState, max_deg: f32) -> [f32; 2] {
    if max_deg < 1e-6 {
        return [0.0, 0.0];
    }
    [
        state.tilt_left_deg / max_deg,
        state.tilt_right_deg / max_deg,
    ]
}

#[allow(dead_code)]
pub fn et_to_json(state: &EyeTiltState) -> String {
    format!(
        "{{\"tilt_left_deg\":{:.4},\"tilt_right_deg\":{:.4}}}",
        state.tilt_left_deg, state.tilt_right_deg
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_neutral() {
        assert!(et_is_neutral(&new_eye_tilt_state()));
    }

    #[test]
    fn set_tilt_clamp_positive() {
        let mut s = new_eye_tilt_state();
        let cfg = default_eye_tilt_config();
        et_set_tilt(&mut s, &cfg, EyeTiltSide::Left, 999.0);
        assert!(s.tilt_left_deg <= cfg.max_tilt_deg);
    }

    #[test]
    fn set_tilt_clamp_negative() {
        let mut s = new_eye_tilt_state();
        let cfg = default_eye_tilt_config();
        et_set_tilt(&mut s, &cfg, EyeTiltSide::Right, -999.0);
        assert!(s.tilt_right_deg >= -cfg.max_tilt_deg);
    }

    #[test]
    fn both_sides() {
        let mut s = new_eye_tilt_state();
        let cfg = default_eye_tilt_config();
        et_set_tilt(&mut s, &cfg, EyeTiltSide::Both, 10.0);
        assert!((s.tilt_left_deg - s.tilt_right_deg).abs() < 1e-5);
    }

    #[test]
    fn reset_clears() {
        let mut s = new_eye_tilt_state();
        let cfg = default_eye_tilt_config();
        et_set_tilt(&mut s, &cfg, EyeTiltSide::Both, 5.0);
        et_reset(&mut s);
        assert!(et_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let cfg = default_eye_tilt_config();
        let mut a = new_eye_tilt_state();
        let mut b = new_eye_tilt_state();
        et_set_tilt(&mut a, &cfg, EyeTiltSide::Left, 0.0);
        et_set_tilt(&mut b, &cfg, EyeTiltSide::Left, 10.0);
        let m = et_blend(&a, &b, 0.5);
        assert!((m.tilt_left_deg - 5.0).abs() < 1e-4);
    }

    #[test]
    fn rad_conversion() {
        let mut s = new_eye_tilt_state();
        let cfg = default_eye_tilt_config();
        et_set_tilt(&mut s, &cfg, EyeTiltSide::Left, 10.0);
        let rad = et_tilt_rad_left(&s);
        assert!((rad - 10f32.to_radians()).abs() < 1e-5);
    }

    #[test]
    fn asymmetry_zero_symmetric() {
        assert!((et_asymmetry(&new_eye_tilt_state())).abs() < 1e-5);
    }

    #[test]
    fn weights_len() {
        assert_eq!(et_to_weights(&new_eye_tilt_state(), 20.0).len(), 2);
    }

    #[test]
    fn json_not_empty() {
        assert!(!et_to_json(&new_eye_tilt_state()).is_empty());
    }
}
