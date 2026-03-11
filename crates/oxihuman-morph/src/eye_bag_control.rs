// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Under-eye and periorbital feature morphology controls.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeBagConfig {
    pub bag_depth: f32,
    pub bag_width: f32,
    pub puffiness: f32,
    pub dark_circle_intensity: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeBagState {
    pub puffiness_l: f32,
    pub puffiness_r: f32,
    pub depth_l: f32,
    pub depth_r: f32,
    pub tear_trough: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyeBagWeights {
    pub puff_l: f32,
    pub puff_r: f32,
    pub bag_l: f32,
    pub bag_r: f32,
    pub tear_trough: f32,
}

#[allow(dead_code)]
pub fn default_eye_bag_config() -> EyeBagConfig {
    EyeBagConfig {
        bag_depth: 0.5,
        bag_width: 0.5,
        puffiness: 0.5,
        dark_circle_intensity: 0.3,
    }
}

#[allow(dead_code)]
pub fn new_eye_bag_state() -> EyeBagState {
    EyeBagState {
        puffiness_l: 0.0,
        puffiness_r: 0.0,
        depth_l: 0.0,
        depth_r: 0.0,
        tear_trough: 0.0,
    }
}

#[allow(dead_code)]
pub fn set_puffiness(state: &mut EyeBagState, left: f32, right: f32) {
    state.puffiness_l = left.clamp(0.0, 1.0);
    state.puffiness_r = right.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_bag_depth(state: &mut EyeBagState, left: f32, right: f32) {
    state.depth_l = left.clamp(0.0, 1.0);
    state.depth_r = right.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_tear_trough(state: &mut EyeBagState, amount: f32) {
    state.tear_trough = amount.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_eye_bag_weights(state: &EyeBagState, cfg: &EyeBagConfig) -> EyeBagWeights {
    let puff_scale = cfg.puffiness;
    let depth_scale = cfg.bag_depth;

    EyeBagWeights {
        puff_l: (state.puffiness_l * puff_scale).clamp(0.0, 1.0),
        puff_r: (state.puffiness_r * puff_scale).clamp(0.0, 1.0),
        bag_l: (state.depth_l * depth_scale).clamp(0.0, 1.0),
        bag_r: (state.depth_r * depth_scale).clamp(0.0, 1.0),
        tear_trough: (state.tear_trough * cfg.bag_width).clamp(0.0, 1.0),
    }
}

#[allow(dead_code)]
pub fn blend_eye_bags(a: &EyeBagState, b: &EyeBagState, t: f32) -> EyeBagState {
    let t = t.clamp(0.0, 1.0);
    let s = 1.0 - t;
    EyeBagState {
        puffiness_l: a.puffiness_l * s + b.puffiness_l * t,
        puffiness_r: a.puffiness_r * s + b.puffiness_r * t,
        depth_l: a.depth_l * s + b.depth_l * t,
        depth_r: a.depth_r * s + b.depth_r * t,
        tear_trough: a.tear_trough * s + b.tear_trough * t,
    }
}

#[allow(dead_code)]
pub fn reset_eye_bags(state: &mut EyeBagState) {
    *state = new_eye_bag_state();
}

#[allow(dead_code)]
pub fn symmetrize_eye_bags(state: &mut EyeBagState) {
    let puff_avg = (state.puffiness_l + state.puffiness_r) * 0.5;
    let depth_avg = (state.depth_l + state.depth_r) * 0.5;
    state.puffiness_l = puff_avg;
    state.puffiness_r = puff_avg;
    state.depth_l = depth_avg;
    state.depth_r = depth_avg;
}

#[allow(dead_code)]
pub fn eye_bag_state_to_json(state: &EyeBagState) -> String {
    format!(
        r#"{{"puffiness_l":{:.4},"puffiness_r":{:.4},"depth_l":{:.4},"depth_r":{:.4},"tear_trough":{:.4}}}"#,
        state.puffiness_l,
        state.puffiness_r,
        state.depth_l,
        state.depth_r,
        state.tear_trough
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_eye_bag_config() {
        let cfg = default_eye_bag_config();
        assert!(cfg.bag_depth > 0.0);
        assert!(cfg.puffiness > 0.0);
    }

    #[test]
    fn test_new_eye_bag_state() {
        let s = new_eye_bag_state();
        assert_eq!(s.puffiness_l, 0.0);
        assert_eq!(s.depth_l, 0.0);
        assert_eq!(s.tear_trough, 0.0);
    }

    #[test]
    fn test_set_puffiness_clamp() {
        let mut s = new_eye_bag_state();
        set_puffiness(&mut s, 1.5, -0.5);
        assert_eq!(s.puffiness_l, 1.0);
        assert_eq!(s.puffiness_r, 0.0);
    }

    #[test]
    fn test_set_bag_depth() {
        let mut s = new_eye_bag_state();
        set_bag_depth(&mut s, 0.6, 0.8);
        assert!((s.depth_l - 0.6).abs() < 1e-5);
        assert!((s.depth_r - 0.8).abs() < 1e-5);
    }

    #[test]
    fn test_compute_eye_bag_weights_range() {
        let cfg = default_eye_bag_config();
        let mut s = new_eye_bag_state();
        set_puffiness(&mut s, 0.7, 0.3);
        set_bag_depth(&mut s, 0.5, 0.5);
        set_tear_trough(&mut s, 0.4);
        let w = compute_eye_bag_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.puff_l));
        assert!((0.0..=1.0).contains(&w.puff_r));
        assert!((0.0..=1.0).contains(&w.bag_l));
        assert!((0.0..=1.0).contains(&w.bag_r));
        assert!((0.0..=1.0).contains(&w.tear_trough));
    }

    #[test]
    fn test_blend_eye_bags() {
        let a = new_eye_bag_state();
        let mut b = new_eye_bag_state();
        b.puffiness_l = 1.0;
        let blended = blend_eye_bags(&a, &b, 0.5);
        assert!((blended.puffiness_l - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_symmetrize_eye_bags() {
        let mut s = new_eye_bag_state();
        set_puffiness(&mut s, 0.8, 0.2);
        symmetrize_eye_bags(&mut s);
        assert!((s.puffiness_l - 0.5).abs() < 1e-5);
        assert!((s.puffiness_r - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_reset_eye_bags() {
        let mut s = new_eye_bag_state();
        set_puffiness(&mut s, 0.9, 0.9);
        reset_eye_bags(&mut s);
        assert_eq!(s.puffiness_l, 0.0);
    }

    #[test]
    fn test_eye_bag_state_to_json() {
        let s = new_eye_bag_state();
        let json = eye_bag_state_to_json(&s);
        assert!(json.contains("puffiness_l"));
        assert!(json.contains("tear_trough"));
    }

    #[test]
    fn test_set_tear_trough_clamp() {
        let mut s = new_eye_bag_state();
        set_tear_trough(&mut s, 2.0);
        assert_eq!(s.tear_trough, 1.0);
    }
}
