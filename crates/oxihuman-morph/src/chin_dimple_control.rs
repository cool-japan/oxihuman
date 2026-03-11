// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Chin-dimple (cleft chin) depth and width control.

use std::f32::consts::FRAC_PI_4;

/// Chin dimple state.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ChinDimpleState {
    /// Dimple depth (0 = no dimple, 1 = maximum).
    pub depth: f32,
    /// Dimple width factor (1.0 = standard, 0 = point, 2 = wide).
    pub width: f32,
    /// Vertical offset from chin tip centre (normalised, -1..1).
    pub vertical_offset: f32,
}

/// Config.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ChinDimpleConfig {
    pub max_depth: f32,
    pub max_width: f32,
}

impl Default for ChinDimpleConfig {
    fn default() -> Self {
        Self {
            max_depth: 1.0,
            max_width: 2.0,
        }
    }
}

impl Default for ChinDimpleState {
    fn default() -> Self {
        Self {
            depth: 0.0,
            width: 1.0,
            vertical_offset: 0.0,
        }
    }
}

#[allow(dead_code)]
pub fn new_chin_dimple_state() -> ChinDimpleState {
    ChinDimpleState::default()
}

#[allow(dead_code)]
pub fn default_chin_dimple_config() -> ChinDimpleConfig {
    ChinDimpleConfig::default()
}

#[allow(dead_code)]
pub fn cd_set_depth(state: &mut ChinDimpleState, cfg: &ChinDimpleConfig, v: f32) {
    state.depth = v.clamp(0.0, cfg.max_depth);
}

#[allow(dead_code)]
pub fn cd_set_width(state: &mut ChinDimpleState, cfg: &ChinDimpleConfig, v: f32) {
    state.width = v.clamp(0.1, cfg.max_width);
}

#[allow(dead_code)]
pub fn cd_set_vertical_offset(state: &mut ChinDimpleState, v: f32) {
    state.vertical_offset = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn cd_reset(state: &mut ChinDimpleState) {
    *state = ChinDimpleState::default();
}

#[allow(dead_code)]
pub fn cd_is_neutral(state: &ChinDimpleState) -> bool {
    state.depth < 1e-4
}

#[allow(dead_code)]
pub fn cd_blend(a: &ChinDimpleState, b: &ChinDimpleState, t: f32) -> ChinDimpleState {
    let t = t.clamp(0.0, 1.0);
    ChinDimpleState {
        depth: a.depth + (b.depth - a.depth) * t,
        width: a.width + (b.width - a.width) * t,
        vertical_offset: a.vertical_offset + (b.vertical_offset - a.vertical_offset) * t,
    }
}

/// Compute approximate dimple area in normalised units.
#[allow(dead_code)]
pub fn cd_area(state: &ChinDimpleState) -> f32 {
    // Ellipse-like: depth * width * pi/4
    state.depth * state.width * FRAC_PI_4
}

#[allow(dead_code)]
pub fn cd_to_weights(state: &ChinDimpleState) -> [f32; 3] {
    [state.depth, state.width - 1.0, state.vertical_offset]
}

#[allow(dead_code)]
pub fn cd_to_json(state: &ChinDimpleState) -> String {
    format!(
        "{{\"depth\":{:.4},\"width\":{:.4},\"v_offset\":{:.4}}}",
        state.depth, state.width, state.vertical_offset
    )
}

#[allow(dead_code)]
pub fn cd_effective_depth(state: &ChinDimpleState) -> f32 {
    state.depth * state.width.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_neutral() {
        assert!(cd_is_neutral(&new_chin_dimple_state()));
    }

    #[test]
    fn depth_clamp_max() {
        let mut s = new_chin_dimple_state();
        let cfg = default_chin_dimple_config();
        cd_set_depth(&mut s, &cfg, 5.0);
        assert!(s.depth <= cfg.max_depth);
    }

    #[test]
    fn depth_not_negative() {
        let mut s = new_chin_dimple_state();
        let cfg = default_chin_dimple_config();
        cd_set_depth(&mut s, &cfg, -1.0);
        assert!(s.depth >= 0.0);
    }

    #[test]
    fn width_min_clamp() {
        let mut s = new_chin_dimple_state();
        let cfg = default_chin_dimple_config();
        cd_set_width(&mut s, &cfg, 0.0);
        assert!(s.width >= 0.1);
    }

    #[test]
    fn reset_neutral() {
        let mut s = new_chin_dimple_state();
        let cfg = default_chin_dimple_config();
        cd_set_depth(&mut s, &cfg, 0.8);
        cd_reset(&mut s);
        assert!(cd_is_neutral(&s));
    }

    #[test]
    fn blend_half() {
        let cfg = default_chin_dimple_config();
        let mut a = new_chin_dimple_state();
        let mut b = new_chin_dimple_state();
        cd_set_depth(&mut a, &cfg, 0.0);
        cd_set_depth(&mut b, &cfg, 1.0);
        let m = cd_blend(&a, &b, 0.5);
        assert!((m.depth - 0.5).abs() < 1e-4);
    }

    #[test]
    fn area_zero_when_no_depth() {
        let s = new_chin_dimple_state();
        assert!(cd_area(&s).abs() < 1e-5);
    }

    #[test]
    fn weights_len() {
        assert_eq!(cd_to_weights(&new_chin_dimple_state()).len(), 3);
    }

    #[test]
    fn json_contains_depth() {
        assert!(cd_to_json(&new_chin_dimple_state()).contains("depth"));
    }

    #[test]
    fn effective_depth_positive() {
        let mut s = new_chin_dimple_state();
        let cfg = default_chin_dimple_config();
        cd_set_depth(&mut s, &cfg, 0.5);
        assert!(cd_effective_depth(&s) > 0.0);
    }
}
