// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Neck crease / horizontal neck fold control.

/// Crease tier (top = just below jaw, bottom = near clavicle).
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreaseTier {
    Top,
    Middle,
    Bottom,
}

/// State.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct NeckCreaseState {
    pub depth_top: f32,
    pub depth_middle: f32,
    pub depth_bottom: f32,
    /// Vertical spread of creases (0 = tight, 1 = spread).
    pub vertical_spread: f32,
}

/// Config.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct NeckCreaseConfig {
    pub max_depth: f32,
}

impl Default for NeckCreaseConfig {
    fn default() -> Self {
        Self { max_depth: 1.0 }
    }
}
impl Default for NeckCreaseState {
    fn default() -> Self {
        Self {
            depth_top: 0.0,
            depth_middle: 0.0,
            depth_bottom: 0.0,
            vertical_spread: 0.5,
        }
    }
}

#[allow(dead_code)]
pub fn new_neck_crease_state() -> NeckCreaseState {
    NeckCreaseState::default()
}

#[allow(dead_code)]
pub fn default_neck_crease_config() -> NeckCreaseConfig {
    NeckCreaseConfig::default()
}

#[allow(dead_code)]
pub fn nc_set_depth(state: &mut NeckCreaseState, cfg: &NeckCreaseConfig, tier: CreaseTier, v: f32) {
    let v = v.clamp(0.0, cfg.max_depth);
    match tier {
        CreaseTier::Top => state.depth_top = v,
        CreaseTier::Middle => state.depth_middle = v,
        CreaseTier::Bottom => state.depth_bottom = v,
    }
}

#[allow(dead_code)]
pub fn nc_set_spread(state: &mut NeckCreaseState, v: f32) {
    state.vertical_spread = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn nc_reset(state: &mut NeckCreaseState) {
    *state = NeckCreaseState::default();
}

#[allow(dead_code)]
pub fn nc_is_neutral(state: &NeckCreaseState) -> bool {
    state.depth_top < 1e-4 && state.depth_middle < 1e-4 && state.depth_bottom < 1e-4
}

#[allow(dead_code)]
pub fn nc_blend(a: &NeckCreaseState, b: &NeckCreaseState, t: f32) -> NeckCreaseState {
    let t = t.clamp(0.0, 1.0);
    NeckCreaseState {
        depth_top: a.depth_top + (b.depth_top - a.depth_top) * t,
        depth_middle: a.depth_middle + (b.depth_middle - a.depth_middle) * t,
        depth_bottom: a.depth_bottom + (b.depth_bottom - a.depth_bottom) * t,
        vertical_spread: a.vertical_spread + (b.vertical_spread - a.vertical_spread) * t,
    }
}

#[allow(dead_code)]
pub fn nc_average_depth(state: &NeckCreaseState) -> f32 {
    (state.depth_top + state.depth_middle + state.depth_bottom) / 3.0
}

#[allow(dead_code)]
pub fn nc_to_weights(state: &NeckCreaseState) -> [f32; 4] {
    [
        state.depth_top,
        state.depth_middle,
        state.depth_bottom,
        state.vertical_spread,
    ]
}

#[allow(dead_code)]
pub fn nc_to_json(state: &NeckCreaseState) -> String {
    format!(
        "{{\"top\":{:.4},\"mid\":{:.4},\"bot\":{:.4},\"spread\":{:.4}}}",
        state.depth_top, state.depth_middle, state.depth_bottom, state.vertical_spread
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_neutral() {
        assert!(nc_is_neutral(&new_neck_crease_state()));
    }

    #[test]
    fn depth_clamps_max() {
        let mut s = new_neck_crease_state();
        let cfg = default_neck_crease_config();
        nc_set_depth(&mut s, &cfg, CreaseTier::Top, 5.0);
        assert!(s.depth_top <= cfg.max_depth);
    }

    #[test]
    fn depth_not_negative() {
        let mut s = new_neck_crease_state();
        let cfg = default_neck_crease_config();
        nc_set_depth(&mut s, &cfg, CreaseTier::Middle, -1.0);
        assert!(s.depth_middle >= 0.0);
    }

    #[test]
    fn bottom_tier() {
        let mut s = new_neck_crease_state();
        let cfg = default_neck_crease_config();
        nc_set_depth(&mut s, &cfg, CreaseTier::Bottom, 0.7);
        assert!((s.depth_bottom - 0.7).abs() < 1e-5);
    }

    #[test]
    fn reset_neutral() {
        let mut s = new_neck_crease_state();
        let cfg = default_neck_crease_config();
        nc_set_depth(&mut s, &cfg, CreaseTier::Top, 0.5);
        nc_reset(&mut s);
        assert!(nc_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let cfg = default_neck_crease_config();
        let mut a = new_neck_crease_state();
        let mut b = new_neck_crease_state();
        nc_set_depth(&mut a, &cfg, CreaseTier::Top, 0.0);
        nc_set_depth(&mut b, &cfg, CreaseTier::Top, 1.0);
        let m = nc_blend(&a, &b, 0.5);
        assert!((m.depth_top - 0.5).abs() < 1e-4);
    }

    #[test]
    fn average_depth_zero() {
        assert!((nc_average_depth(&new_neck_crease_state())).abs() < 1e-5);
    }

    #[test]
    fn spread_clamp() {
        let mut s = new_neck_crease_state();
        nc_set_spread(&mut s, 5.0);
        assert!(s.vertical_spread <= 1.0);
    }

    #[test]
    fn weights_len() {
        assert_eq!(nc_to_weights(&new_neck_crease_state()).len(), 4);
    }

    #[test]
    fn json_has_spread() {
        assert!(nc_to_json(&new_neck_crease_state()).contains("spread"));
    }
}
