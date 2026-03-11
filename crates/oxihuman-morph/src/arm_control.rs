// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Arm and forearm proportion morphs including upper arm, forearm length, muscle mass, and elbow angle.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArmConfig {
    pub upper_arm_length: f32,
    pub forearm_length: f32,
    pub muscle_mass: f32,
    pub elbow_angle: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArmState {
    pub left: ArmConfig,
    pub right: ArmConfig,
    pub symmetric: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArmMorphWeights {
    pub upper_length_weight: f32,
    pub forearm_length_weight: f32,
    pub muscle_weight: f32,
    pub elbow_weight: f32,
}

#[allow(dead_code)]
pub fn default_arm_config() -> ArmConfig {
    ArmConfig {
        upper_arm_length: 1.0,
        forearm_length: 1.0,
        muscle_mass: 0.5,
        elbow_angle: 0.0,
    }
}

#[allow(dead_code)]
pub fn new_arm_state() -> ArmState {
    ArmState {
        left: default_arm_config(),
        right: default_arm_config(),
        symmetric: true,
    }
}

#[allow(dead_code)]
pub fn compute_arm_weights(state: &ArmState) -> ArmMorphWeights {
    let upper = (state.left.upper_arm_length + state.right.upper_arm_length) * 0.5;
    let forearm = (state.left.forearm_length + state.right.forearm_length) * 0.5;
    let muscle = (state.left.muscle_mass + state.right.muscle_mass) * 0.5;
    let elbow = (state.left.elbow_angle + state.right.elbow_angle) * 0.5;
    ArmMorphWeights {
        upper_length_weight: upper.clamp(0.0, 2.0),
        forearm_length_weight: forearm.clamp(0.0, 2.0),
        muscle_weight: muscle.clamp(0.0, 1.0),
        elbow_weight: elbow.clamp(-1.0, 1.0),
    }
}

#[allow(dead_code)]
pub fn set_upper_arm_length(state: &mut ArmState, left_side: bool, v: f32) {
    let v = v.clamp(0.0, 2.0);
    if left_side {
        state.left.upper_arm_length = v;
        if state.symmetric {
            state.right.upper_arm_length = v;
        }
    } else {
        state.right.upper_arm_length = v;
        if state.symmetric {
            state.left.upper_arm_length = v;
        }
    }
}

#[allow(dead_code)]
pub fn set_forearm_length(state: &mut ArmState, left_side: bool, v: f32) {
    let v = v.clamp(0.0, 2.0);
    if left_side {
        state.left.forearm_length = v;
        if state.symmetric {
            state.right.forearm_length = v;
        }
    } else {
        state.right.forearm_length = v;
        if state.symmetric {
            state.left.forearm_length = v;
        }
    }
}

#[allow(dead_code)]
pub fn set_arm_muscle(state: &mut ArmState, v: f32) {
    let v = v.clamp(0.0, 1.0);
    state.left.muscle_mass = v;
    state.right.muscle_mass = v;
}

#[allow(dead_code)]
pub fn set_elbow_angle(state: &mut ArmState, left_side: bool, v: f32) {
    let v = v.clamp(-1.0, 1.0);
    if left_side {
        state.left.elbow_angle = v;
        if state.symmetric {
            state.right.elbow_angle = v;
        }
    } else {
        state.right.elbow_angle = v;
        if state.symmetric {
            state.left.elbow_angle = v;
        }
    }
}

#[allow(dead_code)]
pub fn symmetrize_arms(state: &mut ArmState) {
    let upper_avg = (state.left.upper_arm_length + state.right.upper_arm_length) * 0.5;
    state.left.upper_arm_length = upper_avg;
    state.right.upper_arm_length = upper_avg;
    let forearm_avg = (state.left.forearm_length + state.right.forearm_length) * 0.5;
    state.left.forearm_length = forearm_avg;
    state.right.forearm_length = forearm_avg;
    let muscle_avg = (state.left.muscle_mass + state.right.muscle_mass) * 0.5;
    state.left.muscle_mass = muscle_avg;
    state.right.muscle_mass = muscle_avg;
    let elbow_avg = (state.left.elbow_angle + state.right.elbow_angle) * 0.5;
    state.left.elbow_angle = elbow_avg;
    state.right.elbow_angle = elbow_avg;
    state.symmetric = true;
}

#[allow(dead_code)]
pub fn arm_state_to_json(state: &ArmState) -> String {
    format!(
        "{{\"left\":{{\"upper_arm_length\":{},\"forearm_length\":{},\"muscle_mass\":{},\"elbow_angle\":{}}},\
\"right\":{{\"upper_arm_length\":{},\"forearm_length\":{},\"muscle_mass\":{},\"elbow_angle\":{}}},\
\"symmetric\":{}}}",
        state.left.upper_arm_length,
        state.left.forearm_length,
        state.left.muscle_mass,
        state.left.elbow_angle,
        state.right.upper_arm_length,
        state.right.forearm_length,
        state.right.muscle_mass,
        state.right.elbow_angle,
        state.symmetric,
    )
}

#[allow(dead_code)]
pub fn blend_arm_states(a: &ArmState, b: &ArmState, t: f32) -> ArmState {
    let t = t.clamp(0.0, 1.0);
    let s = 1.0 - t;
    ArmState {
        left: ArmConfig {
            upper_arm_length: a.left.upper_arm_length * s + b.left.upper_arm_length * t,
            forearm_length: a.left.forearm_length * s + b.left.forearm_length * t,
            muscle_mass: a.left.muscle_mass * s + b.left.muscle_mass * t,
            elbow_angle: a.left.elbow_angle * s + b.left.elbow_angle * t,
        },
        right: ArmConfig {
            upper_arm_length: a.right.upper_arm_length * s + b.right.upper_arm_length * t,
            forearm_length: a.right.forearm_length * s + b.right.forearm_length * t,
            muscle_mass: a.right.muscle_mass * s + b.right.muscle_mass * t,
            elbow_angle: a.right.elbow_angle * s + b.right.elbow_angle * t,
        },
        symmetric: a.symmetric && b.symmetric,
    }
}

#[allow(dead_code)]
pub fn reset_arm_state(state: &mut ArmState) {
    state.left = default_arm_config();
    state.right = default_arm_config();
    state.symmetric = true;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_arm_config() {
        let cfg = default_arm_config();
        assert!((cfg.upper_arm_length - 1.0).abs() < 1e-6);
        assert!((cfg.forearm_length - 1.0).abs() < 1e-6);
        assert!((cfg.muscle_mass - 0.5).abs() < 1e-6);
        assert!((cfg.elbow_angle - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_arm_state_symmetric() {
        let s = new_arm_state();
        assert!(s.symmetric);
        assert!((s.left.upper_arm_length - 1.0).abs() < 1e-6);
        assert!((s.right.upper_arm_length - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_upper_arm_length_symmetric() {
        let mut s = new_arm_state();
        set_upper_arm_length(&mut s, true, 1.5);
        assert!((s.left.upper_arm_length - 1.5).abs() < 1e-6);
        assert!((s.right.upper_arm_length - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_upper_arm_length_clamped() {
        let mut s = new_arm_state();
        s.symmetric = false;
        set_upper_arm_length(&mut s, true, 5.0);
        assert!((s.left.upper_arm_length - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_forearm_length() {
        let mut s = new_arm_state();
        s.symmetric = false;
        set_forearm_length(&mut s, false, 0.7);
        assert!((s.right.forearm_length - 0.7).abs() < 1e-6);
        assert!((s.left.forearm_length - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_arm_muscle_clamped() {
        let mut s = new_arm_state();
        set_arm_muscle(&mut s, 2.0);
        assert!((s.left.muscle_mass - 1.0).abs() < 1e-6);
        assert!((s.right.muscle_mass - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_elbow_angle() {
        let mut s = new_arm_state();
        set_elbow_angle(&mut s, true, 0.3);
        assert!((s.left.elbow_angle - 0.3).abs() < 1e-6);
        assert!((s.right.elbow_angle - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_symmetrize_arms() {
        let mut s = new_arm_state();
        s.symmetric = false;
        s.left.upper_arm_length = 1.4;
        s.right.upper_arm_length = 0.6;
        symmetrize_arms(&mut s);
        assert!((s.left.upper_arm_length - 1.0).abs() < 1e-6);
        assert!((s.right.upper_arm_length - 1.0).abs() < 1e-6);
        assert!(s.symmetric);
    }

    #[test]
    fn test_compute_arm_weights() {
        let mut s = new_arm_state();
        set_upper_arm_length(&mut s, true, 1.2);
        let w = compute_arm_weights(&s);
        assert!((w.upper_length_weight - 1.2).abs() < 1e-6);
    }

    #[test]
    fn test_arm_state_to_json() {
        let s = new_arm_state();
        let json = arm_state_to_json(&s);
        assert!(json.contains("upper_arm_length"));
        assert!(json.contains("symmetric"));
    }

    #[test]
    fn test_blend_arm_states() {
        let a = new_arm_state();
        let mut b = new_arm_state();
        b.left.upper_arm_length = 2.0;
        b.right.upper_arm_length = 2.0;
        let blended = blend_arm_states(&a, &b, 0.5);
        assert!((blended.left.upper_arm_length - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_reset_arm_state() {
        let mut s = new_arm_state();
        set_arm_muscle(&mut s, 1.0);
        reset_arm_state(&mut s);
        assert!((s.left.muscle_mass - 0.5).abs() < 1e-6);
    }
}
