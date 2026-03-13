// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Foot-width (forefoot / heel) proportioning control.

/// Side.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FootSide {
    Left,
    Right,
    Both,
}

/// State.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct FootWidthState {
    pub forefoot_left: f32,
    pub forefoot_right: f32,
    pub heel_left: f32,
    pub heel_right: f32,
    pub arch_height: f32,
}

/// Config.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct FootWidthConfig {
    pub max_width: f32,
    pub max_arch: f32,
}

impl Default for FootWidthConfig {
    fn default() -> Self {
        Self {
            max_width: 1.0,
            max_arch: 1.0,
        }
    }
}
impl Default for FootWidthState {
    fn default() -> Self {
        Self {
            forefoot_left: 0.5,
            forefoot_right: 0.5,
            heel_left: 0.5,
            heel_right: 0.5,
            arch_height: 0.0,
        }
    }
}

#[allow(dead_code)]
pub fn new_foot_width_state() -> FootWidthState {
    FootWidthState::default()
}

#[allow(dead_code)]
pub fn default_foot_width_config() -> FootWidthConfig {
    FootWidthConfig::default()
}

#[allow(dead_code)]
pub fn fw_set_forefoot(state: &mut FootWidthState, cfg: &FootWidthConfig, side: FootSide, v: f32) {
    let v = v.clamp(0.0, cfg.max_width);
    match side {
        FootSide::Left => state.forefoot_left = v,
        FootSide::Right => state.forefoot_right = v,
        FootSide::Both => {
            state.forefoot_left = v;
            state.forefoot_right = v;
        }
    }
}

#[allow(dead_code)]
pub fn fw_set_heel(state: &mut FootWidthState, cfg: &FootWidthConfig, side: FootSide, v: f32) {
    let v = v.clamp(0.0, cfg.max_width);
    match side {
        FootSide::Left => state.heel_left = v,
        FootSide::Right => state.heel_right = v,
        FootSide::Both => {
            state.heel_left = v;
            state.heel_right = v;
        }
    }
}

#[allow(dead_code)]
pub fn fw_set_arch(state: &mut FootWidthState, cfg: &FootWidthConfig, v: f32) {
    state.arch_height = v.clamp(0.0, cfg.max_arch);
}

#[allow(dead_code)]
pub fn fw_reset(state: &mut FootWidthState) {
    *state = FootWidthState::default();
}

#[allow(dead_code)]
pub fn fw_blend(a: &FootWidthState, b: &FootWidthState, t: f32) -> FootWidthState {
    let t = t.clamp(0.0, 1.0);
    FootWidthState {
        forefoot_left: a.forefoot_left + (b.forefoot_left - a.forefoot_left) * t,
        forefoot_right: a.forefoot_right + (b.forefoot_right - a.forefoot_right) * t,
        heel_left: a.heel_left + (b.heel_left - a.heel_left) * t,
        heel_right: a.heel_right + (b.heel_right - a.heel_right) * t,
        arch_height: a.arch_height + (b.arch_height - a.arch_height) * t,
    }
}

#[allow(dead_code)]
pub fn fw_symmetry(state: &FootWidthState) -> f32 {
    1.0 - (state.forefoot_left - state.forefoot_right).abs().min(1.0)
}

#[allow(dead_code)]
pub fn fw_to_weights(state: &FootWidthState) -> [f32; 5] {
    [
        state.forefoot_left,
        state.forefoot_right,
        state.heel_left,
        state.heel_right,
        state.arch_height,
    ]
}

#[allow(dead_code)]
pub fn fw_to_json(state: &FootWidthState) -> String {
    format!(
        "{{\"ff_l\":{:.4},\"ff_r\":{:.4},\"heel_l\":{:.4},\"heel_r\":{:.4},\"arch\":{:.4}}}",
        state.forefoot_left,
        state.forefoot_right,
        state.heel_left,
        state.heel_right,
        state.arch_height
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_half_width() {
        let s = new_foot_width_state();
        assert!((s.forefoot_left - 0.5).abs() < 1e-5);
    }

    #[test]
    fn clamp_max() {
        let mut s = new_foot_width_state();
        let cfg = default_foot_width_config();
        fw_set_forefoot(&mut s, &cfg, FootSide::Left, 5.0);
        assert!(s.forefoot_left <= cfg.max_width);
    }

    #[test]
    fn clamp_min() {
        let mut s = new_foot_width_state();
        let cfg = default_foot_width_config();
        fw_set_heel(&mut s, &cfg, FootSide::Right, -1.0);
        assert!(s.heel_right >= 0.0);
    }

    #[test]
    fn both_sides_set() {
        let mut s = new_foot_width_state();
        let cfg = default_foot_width_config();
        fw_set_forefoot(&mut s, &cfg, FootSide::Both, 0.8);
        assert!((s.forefoot_left - s.forefoot_right).abs() < 1e-5);
    }

    #[test]
    fn reset_default() {
        let mut s = new_foot_width_state();
        let cfg = default_foot_width_config();
        fw_set_arch(&mut s, &cfg, 0.9);
        fw_reset(&mut s);
        assert!((s.arch_height).abs() < 1e-5);
    }

    #[test]
    fn blend_half() {
        let cfg = default_foot_width_config();
        let mut a = new_foot_width_state();
        let mut b = new_foot_width_state();
        fw_set_forefoot(&mut a, &cfg, FootSide::Left, 0.0);
        fw_set_forefoot(&mut b, &cfg, FootSide::Left, 1.0);
        let m = fw_blend(&a, &b, 0.5);
        assert!((m.forefoot_left - 0.5).abs() < 1e-4);
    }

    #[test]
    fn symmetry_equal() {
        let s = new_foot_width_state();
        assert!((fw_symmetry(&s) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn weights_len() {
        assert_eq!(fw_to_weights(&new_foot_width_state()).len(), 5);
    }

    #[test]
    fn json_has_arch() {
        assert!(fw_to_json(&new_foot_width_state()).contains("arch"));
    }

    #[test]
    fn arch_clamp() {
        let mut s = new_foot_width_state();
        let cfg = default_foot_width_config();
        fw_set_arch(&mut s, &cfg, -5.0);
        assert!(s.arch_height >= 0.0);
    }
}
