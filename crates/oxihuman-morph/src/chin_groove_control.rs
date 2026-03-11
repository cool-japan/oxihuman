// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Chin groove (mentolabial sulcus) depth and width control.

use std::f32::consts::FRAC_PI_4;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChinGrooveConfig {
    pub max_depth: f32,
    pub max_width: f32,
}

impl Default for ChinGrooveConfig {
    fn default() -> Self {
        Self {
            max_depth: 1.0,
            max_width: 1.0,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChinGrooveState {
    pub depth: f32,
    pub width: f32,
    pub config: ChinGrooveConfig,
}

#[allow(dead_code)]
pub fn default_chin_groove_config() -> ChinGrooveConfig {
    ChinGrooveConfig::default()
}

#[allow(dead_code)]
pub fn new_chin_groove_state(config: ChinGrooveConfig) -> ChinGrooveState {
    ChinGrooveState {
        depth: 0.0,
        width: 0.0,
        config,
    }
}

#[allow(dead_code)]
pub fn cg_set_depth(state: &mut ChinGrooveState, v: f32) {
    state.depth = v.clamp(0.0, state.config.max_depth);
}

#[allow(dead_code)]
pub fn cg_set_width(state: &mut ChinGrooveState, v: f32) {
    state.width = v.clamp(0.0, state.config.max_width);
}

#[allow(dead_code)]
pub fn cg_reset(state: &mut ChinGrooveState) {
    state.depth = 0.0;
    state.width = 0.0;
}

#[allow(dead_code)]
pub fn cg_is_neutral(state: &ChinGrooveState) -> bool {
    state.depth.abs() < 1e-6 && state.width.abs() < 1e-6
}

#[allow(dead_code)]
pub fn cg_groove_area(state: &ChinGrooveState) -> f32 {
    state.depth * state.width
}

#[allow(dead_code)]
pub fn cg_angle_rad(state: &ChinGrooveState) -> f32 {
    state.depth * FRAC_PI_4
}

#[allow(dead_code)]
pub fn cg_to_weights(state: &ChinGrooveState) -> [f32; 2] {
    let nd = if state.config.max_depth > 1e-9 {
        state.depth / state.config.max_depth
    } else {
        0.0
    };
    let nw = if state.config.max_width > 1e-9 {
        state.width / state.config.max_width
    } else {
        0.0
    };
    [nd, nw]
}

#[allow(dead_code)]
pub fn cg_blend(a: &ChinGrooveState, b: &ChinGrooveState, t: f32) -> [f32; 2] {
    let t = t.clamp(0.0, 1.0);
    [
        a.depth * (1.0 - t) + b.depth * t,
        a.width * (1.0 - t) + b.width * t,
    ]
}

#[allow(dead_code)]
pub fn cg_to_json(state: &ChinGrooveState) -> String {
    format!(
        "{{\"depth\":{:.4},\"width\":{:.4}}}",
        state.depth, state.width
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_neutral() {
        assert!(cg_is_neutral(&new_chin_groove_state(
            default_chin_groove_config()
        )));
    }
    #[test]
    fn set_depth_clamps() {
        let mut s = new_chin_groove_state(default_chin_groove_config());
        cg_set_depth(&mut s, 9.9);
        assert!((0.0..=1.0).contains(&s.depth));
    }
    #[test]
    fn set_width_clamps() {
        let mut s = new_chin_groove_state(default_chin_groove_config());
        cg_set_width(&mut s, -1.0);
        assert!(s.width.abs() < 1e-6);
    }
    #[test]
    fn reset_zeroes() {
        let mut s = new_chin_groove_state(default_chin_groove_config());
        cg_set_depth(&mut s, 0.5);
        cg_reset(&mut s);
        assert!(cg_is_neutral(&s));
    }
    #[test]
    fn groove_area_product() {
        let mut s = new_chin_groove_state(default_chin_groove_config());
        cg_set_depth(&mut s, 0.5);
        cg_set_width(&mut s, 0.4);
        assert!((cg_groove_area(&s) - 0.2).abs() < 1e-5);
    }
    #[test]
    fn angle_nonneg() {
        let s = new_chin_groove_state(default_chin_groove_config());
        assert!(cg_angle_rad(&s) >= 0.0);
    }
    #[test]
    fn to_weights_at_max() {
        let mut s = new_chin_groove_state(default_chin_groove_config());
        cg_set_depth(&mut s, 1.0);
        assert!((cg_to_weights(&s)[0] - 1.0).abs() < 1e-5);
    }
    #[test]
    fn blend_at_one_is_b() {
        let mut b = new_chin_groove_state(default_chin_groove_config());
        let a = new_chin_groove_state(default_chin_groove_config());
        cg_set_depth(&mut b, 0.6);
        let w = cg_blend(&a, &b, 1.0);
        assert!((w[0] - 0.6).abs() < 1e-5);
    }
    #[test]
    fn to_json_has_depth() {
        assert!(
            cg_to_json(&new_chin_groove_state(default_chin_groove_config())).contains("\"depth\"")
        );
    }
    #[test]
    fn to_weights_range() {
        let mut s = new_chin_groove_state(default_chin_groove_config());
        cg_set_depth(&mut s, 0.5);
        let w = cg_to_weights(&s);
        assert!(w[0] >= 0.0 && w[0] <= 1.0);
    }
}
