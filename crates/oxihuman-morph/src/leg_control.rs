// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Leg and calf proportion morphs including thigh, calf length, girth, and knee angle.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LegConfig {
    pub thigh_length: f32,
    pub calf_length: f32,
    pub thigh_girth: f32,
    pub knee_angle: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LegState {
    pub left: LegConfig,
    pub right: LegConfig,
    pub symmetric: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LegMorphWeights {
    pub thigh_length_weight: f32,
    pub calf_length_weight: f32,
    pub thigh_girth_weight: f32,
    pub knee_weight: f32,
}

#[allow(dead_code)]
pub fn default_leg_config() -> LegConfig {
    LegConfig {
        thigh_length: 1.0,
        calf_length: 1.0,
        thigh_girth: 0.5,
        knee_angle: 0.0,
    }
}

#[allow(dead_code)]
pub fn new_leg_state() -> LegState {
    LegState {
        left: default_leg_config(),
        right: default_leg_config(),
        symmetric: true,
    }
}

#[allow(dead_code)]
pub fn compute_leg_weights(state: &LegState) -> LegMorphWeights {
    let thigh = (state.left.thigh_length + state.right.thigh_length) * 0.5;
    let calf = (state.left.calf_length + state.right.calf_length) * 0.5;
    let girth = (state.left.thigh_girth + state.right.thigh_girth) * 0.5;
    let knee = (state.left.knee_angle + state.right.knee_angle) * 0.5;
    LegMorphWeights {
        thigh_length_weight: thigh.clamp(0.0, 2.0),
        calf_length_weight: calf.clamp(0.0, 2.0),
        thigh_girth_weight: girth.clamp(0.0, 1.0),
        knee_weight: knee.clamp(-1.0, 1.0),
    }
}

#[allow(dead_code)]
pub fn set_thigh_length(state: &mut LegState, left_side: bool, v: f32) {
    let v = v.clamp(0.0, 2.0);
    if left_side {
        state.left.thigh_length = v;
        if state.symmetric {
            state.right.thigh_length = v;
        }
    } else {
        state.right.thigh_length = v;
        if state.symmetric {
            state.left.thigh_length = v;
        }
    }
}

#[allow(dead_code)]
pub fn set_calf_length(state: &mut LegState, left_side: bool, v: f32) {
    let v = v.clamp(0.0, 2.0);
    if left_side {
        state.left.calf_length = v;
        if state.symmetric {
            state.right.calf_length = v;
        }
    } else {
        state.right.calf_length = v;
        if state.symmetric {
            state.left.calf_length = v;
        }
    }
}

#[allow(dead_code)]
pub fn set_thigh_girth(state: &mut LegState, v: f32) {
    let v = v.clamp(0.0, 1.0);
    state.left.thigh_girth = v;
    state.right.thigh_girth = v;
}

#[allow(dead_code)]
pub fn set_knee_angle(state: &mut LegState, left_side: bool, v: f32) {
    let v = v.clamp(-1.0, 1.0);
    if left_side {
        state.left.knee_angle = v;
        if state.symmetric {
            state.right.knee_angle = v;
        }
    } else {
        state.right.knee_angle = v;
        if state.symmetric {
            state.left.knee_angle = v;
        }
    }
}

#[allow(dead_code)]
pub fn symmetrize_legs(state: &mut LegState) {
    let thigh_avg = (state.left.thigh_length + state.right.thigh_length) * 0.5;
    state.left.thigh_length = thigh_avg;
    state.right.thigh_length = thigh_avg;
    let calf_avg = (state.left.calf_length + state.right.calf_length) * 0.5;
    state.left.calf_length = calf_avg;
    state.right.calf_length = calf_avg;
    let girth_avg = (state.left.thigh_girth + state.right.thigh_girth) * 0.5;
    state.left.thigh_girth = girth_avg;
    state.right.thigh_girth = girth_avg;
    let knee_avg = (state.left.knee_angle + state.right.knee_angle) * 0.5;
    state.left.knee_angle = knee_avg;
    state.right.knee_angle = knee_avg;
    state.symmetric = true;
}

#[allow(dead_code)]
pub fn leg_state_to_json(state: &LegState) -> String {
    format!(
        "{{\"left\":{{\"thigh_length\":{},\"calf_length\":{},\"thigh_girth\":{},\"knee_angle\":{}}},\
\"right\":{{\"thigh_length\":{},\"calf_length\":{},\"thigh_girth\":{},\"knee_angle\":{}}},\
\"symmetric\":{}}}",
        state.left.thigh_length,
        state.left.calf_length,
        state.left.thigh_girth,
        state.left.knee_angle,
        state.right.thigh_length,
        state.right.calf_length,
        state.right.thigh_girth,
        state.right.knee_angle,
        state.symmetric,
    )
}

#[allow(dead_code)]
pub fn blend_leg_states(a: &LegState, b: &LegState, t: f32) -> LegState {
    let t = t.clamp(0.0, 1.0);
    let s = 1.0 - t;
    LegState {
        left: LegConfig {
            thigh_length: a.left.thigh_length * s + b.left.thigh_length * t,
            calf_length: a.left.calf_length * s + b.left.calf_length * t,
            thigh_girth: a.left.thigh_girth * s + b.left.thigh_girth * t,
            knee_angle: a.left.knee_angle * s + b.left.knee_angle * t,
        },
        right: LegConfig {
            thigh_length: a.right.thigh_length * s + b.right.thigh_length * t,
            calf_length: a.right.calf_length * s + b.right.calf_length * t,
            thigh_girth: a.right.thigh_girth * s + b.right.thigh_girth * t,
            knee_angle: a.right.knee_angle * s + b.right.knee_angle * t,
        },
        symmetric: a.symmetric && b.symmetric,
    }
}

#[allow(dead_code)]
pub fn reset_leg_state(state: &mut LegState) {
    state.left = default_leg_config();
    state.right = default_leg_config();
    state.symmetric = true;
}

#[allow(dead_code)]
pub fn total_leg_length(cfg: &LegConfig) -> f32 {
    cfg.thigh_length + cfg.calf_length
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_leg_config() {
        let cfg = default_leg_config();
        assert!((cfg.thigh_length - 1.0).abs() < 1e-6);
        assert!((cfg.calf_length - 1.0).abs() < 1e-6);
        assert!((cfg.thigh_girth - 0.5).abs() < 1e-6);
        assert!((cfg.knee_angle - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_leg_state_symmetric() {
        let s = new_leg_state();
        assert!(s.symmetric);
        assert!((s.left.thigh_length - 1.0).abs() < 1e-6);
        assert!((s.right.calf_length - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_thigh_length_symmetric() {
        let mut s = new_leg_state();
        set_thigh_length(&mut s, true, 1.3);
        assert!((s.left.thigh_length - 1.3).abs() < 1e-6);
        assert!((s.right.thigh_length - 1.3).abs() < 1e-6);
    }

    #[test]
    fn test_set_thigh_length_clamped() {
        let mut s = new_leg_state();
        s.symmetric = false;
        set_thigh_length(&mut s, false, 9.9);
        assert!((s.right.thigh_length - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_calf_length_asymmetric() {
        let mut s = new_leg_state();
        s.symmetric = false;
        set_calf_length(&mut s, true, 0.8);
        assert!((s.left.calf_length - 0.8).abs() < 1e-6);
        assert!((s.right.calf_length - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_thigh_girth_clamped() {
        let mut s = new_leg_state();
        set_thigh_girth(&mut s, 1.5);
        assert!((s.left.thigh_girth - 1.0).abs() < 1e-6);
        assert!((s.right.thigh_girth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_knee_angle() {
        let mut s = new_leg_state();
        set_knee_angle(&mut s, false, -0.4);
        assert!((s.right.knee_angle - (-0.4)).abs() < 1e-6);
        assert!((s.left.knee_angle - (-0.4)).abs() < 1e-6);
    }

    #[test]
    fn test_symmetrize_legs() {
        let mut s = new_leg_state();
        s.symmetric = false;
        s.left.thigh_length = 1.8;
        s.right.thigh_length = 0.2;
        symmetrize_legs(&mut s);
        assert!((s.left.thigh_length - 1.0).abs() < 1e-6);
        assert!((s.right.thigh_length - 1.0).abs() < 1e-6);
        assert!(s.symmetric);
    }

    #[test]
    fn test_compute_leg_weights() {
        let mut s = new_leg_state();
        set_thigh_length(&mut s, true, 1.5);
        let w = compute_leg_weights(&s);
        assert!((w.thigh_length_weight - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_leg_state_to_json() {
        let s = new_leg_state();
        let json = leg_state_to_json(&s);
        assert!(json.contains("thigh_length"));
        assert!(json.contains("symmetric"));
    }

    #[test]
    fn test_blend_leg_states() {
        let a = new_leg_state();
        let mut b = new_leg_state();
        b.left.calf_length = 2.0;
        b.right.calf_length = 2.0;
        let blended = blend_leg_states(&a, &b, 0.5);
        assert!((blended.left.calf_length - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_reset_leg_state() {
        let mut s = new_leg_state();
        set_thigh_girth(&mut s, 1.0);
        reset_leg_state(&mut s);
        assert!((s.left.thigh_girth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_total_leg_length() {
        let cfg = default_leg_config();
        let total = total_leg_length(&cfg);
        assert!((total - 2.0).abs() < 1e-6);
    }
}
