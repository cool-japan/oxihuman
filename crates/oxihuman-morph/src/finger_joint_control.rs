// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Finger joint morph control: adjusts finger bend and spread.

use std::f32::consts::PI;

/// Which finger.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Finger {
    Thumb,
    Index,
    Middle,
    Ring,
    Pinky,
}

/// Runtime state for finger joints.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FingerJointState {
    pub curl: [f32; 5],
    pub spread: [f32; 5],
    pub stiffness: f32,
}

#[allow(dead_code)]
pub fn new_finger_joint_state() -> FingerJointState {
    FingerJointState {
        curl: [0.0; 5],
        spread: [0.0; 5],
        stiffness: 0.5,
    }
}

#[allow(dead_code)]
pub fn fj_finger_index(finger: Finger) -> usize {
    match finger {
        Finger::Thumb => 0,
        Finger::Index => 1,
        Finger::Middle => 2,
        Finger::Ring => 3,
        Finger::Pinky => 4,
    }
}

#[allow(dead_code)]
pub fn fj_set_curl(state: &mut FingerJointState, finger: Finger, v: f32) {
    let idx = fj_finger_index(finger);
    state.curl[idx] = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn fj_set_spread(state: &mut FingerJointState, finger: Finger, v: f32) {
    let idx = fj_finger_index(finger);
    state.spread[idx] = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn fj_set_all_curl(state: &mut FingerJointState, v: f32) {
    let clamped = v.clamp(0.0, 1.0);
    #[allow(clippy::needless_range_loop)]
    for i in 0..5 {
        state.curl[i] = clamped;
    }
}

#[allow(dead_code)]
pub fn fj_set_stiffness(state: &mut FingerJointState, v: f32) {
    state.stiffness = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn fj_reset(state: &mut FingerJointState) {
    *state = new_finger_joint_state();
}

#[allow(dead_code)]
pub fn fj_curl_angle(state: &FingerJointState, finger: Finger) -> f32 {
    let idx = fj_finger_index(finger);
    state.curl[idx] * PI * 0.5
}

#[allow(dead_code)]
pub fn fj_to_json(state: &FingerJointState) -> String {
    format!(
        r#"{{"curl":[{:.4},{:.4},{:.4},{:.4},{:.4}],"stiffness":{:.4}}}"#,
        state.curl[0], state.curl[1], state.curl[2], state.curl[3], state.curl[4], state.stiffness
    )
}

#[allow(dead_code)]
pub fn fj_blend(a: &FingerJointState, b: &FingerJointState, t: f32) -> FingerJointState {
    let t = t.clamp(0.0, 1.0);
    let mut result = new_finger_joint_state();
    #[allow(clippy::needless_range_loop)]
    for i in 0..5 {
        result.curl[i] = a.curl[i] + (b.curl[i] - a.curl[i]) * t;
        result.spread[i] = a.spread[i] + (b.spread[i] - a.spread[i]) * t;
    }
    result.stiffness = a.stiffness + (b.stiffness - a.stiffness) * t;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_state() {
        let s = new_finger_joint_state();
        assert!(s.curl[0].abs() < 1e-6);
        assert!((s.stiffness - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_finger_index() {
        assert_eq!(fj_finger_index(Finger::Thumb), 0);
        assert_eq!(fj_finger_index(Finger::Pinky), 4);
    }

    #[test]
    fn test_set_curl() {
        let mut s = new_finger_joint_state();
        fj_set_curl(&mut s, Finger::Index, 0.8);
        assert!((s.curl[1] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_curl_clamps() {
        let mut s = new_finger_joint_state();
        fj_set_curl(&mut s, Finger::Middle, 5.0);
        assert!((s.curl[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_spread() {
        let mut s = new_finger_joint_state();
        fj_set_spread(&mut s, Finger::Ring, -0.5);
        assert!((s.spread[3] + 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_all_curl() {
        let mut s = new_finger_joint_state();
        fj_set_all_curl(&mut s, 0.7);
        for c in &s.curl {
            assert!((*c - 0.7).abs() < 1e-6);
        }
    }

    #[test]
    fn test_reset() {
        let mut s = new_finger_joint_state();
        fj_set_curl(&mut s, Finger::Thumb, 0.9);
        fj_reset(&mut s);
        assert!(s.curl[0].abs() < 1e-6);
    }

    #[test]
    fn test_curl_angle() {
        let mut s = new_finger_joint_state();
        fj_set_curl(&mut s, Finger::Index, 1.0);
        let angle = fj_curl_angle(&s, Finger::Index);
        assert!((angle - PI * 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let s = new_finger_joint_state();
        let j = fj_to_json(&s);
        assert!(j.contains("curl"));
        assert!(j.contains("stiffness"));
    }

    #[test]
    fn test_blend() {
        let a = new_finger_joint_state();
        let mut b = new_finger_joint_state();
        b.curl[0] = 1.0;
        let mid = fj_blend(&a, &b, 0.5);
        assert!((mid.curl[0] - 0.5).abs() < 1e-6);
    }
}
