// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Finger curl and splay morph controls for hand poses.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Finger {
    Thumb,
    Index,
    Middle,
    Ring,
    Pinky,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FingerCurlConfig {
    pub max_curl_angle: f32,
    pub splay_range: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FingerCurlState {
    pub curls: [f32; 5],
    pub splays: [f32; 5],
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FingerCurlWeights {
    pub curl_weights: [f32; 5],
    pub splay_weights: [f32; 5],
}

#[allow(dead_code)]
pub fn default_finger_curl_config() -> FingerCurlConfig {
    FingerCurlConfig {
        max_curl_angle: PI * 0.75,
        splay_range: 0.3,
    }
}

#[allow(dead_code)]
pub fn new_finger_curl_state() -> FingerCurlState {
    FingerCurlState {
        curls: [0.0; 5],
        splays: [0.0; 5],
    }
}

#[allow(dead_code)]
pub fn finger_index(finger: Finger) -> usize {
    match finger {
        Finger::Thumb => 0,
        Finger::Index => 1,
        Finger::Middle => 2,
        Finger::Ring => 3,
        Finger::Pinky => 4,
    }
}

#[allow(dead_code)]
pub fn set_finger_curl(state: &mut FingerCurlState, finger: Finger, value: f32) {
    let idx = finger_index(finger);
    state.curls[idx] = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_finger_splay(state: &mut FingerCurlState, finger: Finger, value: f32) {
    let idx = finger_index(finger);
    state.splays[idx] = value.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn set_all_curls(state: &mut FingerCurlState, value: f32) {
    let v = value.clamp(0.0, 1.0);
    for c in &mut state.curls {
        *c = v;
    }
}

#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn compute_finger_curl_weights(state: &FingerCurlState, cfg: &FingerCurlConfig) -> FingerCurlWeights {
    let mut curl_weights = [0.0f32; 5];
    let mut splay_weights = [0.0f32; 5];
    for i in 0..5 {
        curl_weights[i] = (state.curls[i] * cfg.max_curl_angle / PI).clamp(0.0, 1.0);
        splay_weights[i] = (state.splays[i] * cfg.splay_range).clamp(-1.0, 1.0);
    }
    FingerCurlWeights { curl_weights, splay_weights }
}

#[allow(dead_code)]
pub fn finger_curl_to_json(state: &FingerCurlState) -> String {
    format!(
        r#"{{"curls":[{},{},{},{},{}],"splays":[{},{},{},{},{}]}}"#,
        state.curls[0], state.curls[1], state.curls[2], state.curls[3], state.curls[4],
        state.splays[0], state.splays[1], state.splays[2], state.splays[3], state.splays[4]
    )
}

#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn blend_finger_curl_states(a: &FingerCurlState, b: &FingerCurlState, t: f32) -> FingerCurlState {
    let t = t.clamp(0.0, 1.0);
    let mut curls = [0.0f32; 5];
    let mut splays = [0.0f32; 5];
    for i in 0..5 {
        curls[i] = a.curls[i] + (b.curls[i] - a.curls[i]) * t;
        splays[i] = a.splays[i] + (b.splays[i] - a.splays[i]) * t;
    }
    FingerCurlState { curls, splays }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_finger_curl_config();
        assert!(c.max_curl_angle > 0.0);
    }

    #[test]
    fn test_new_state_zeroed() {
        let s = new_finger_curl_state();
        assert!(s.curls.iter().all(|&v| v.abs() < 1e-6));
    }

    #[test]
    fn test_set_curl() {
        let mut s = new_finger_curl_state();
        set_finger_curl(&mut s, Finger::Index, 0.7);
        assert!((s.curls[1] - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_set_curl_clamp() {
        let mut s = new_finger_curl_state();
        set_finger_curl(&mut s, Finger::Thumb, 2.0);
        assert!((s.curls[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_splay() {
        let mut s = new_finger_curl_state();
        set_finger_splay(&mut s, Finger::Pinky, -0.5);
        assert!((s.splays[4] - (-0.5)).abs() < 1e-6);
    }

    #[test]
    fn test_set_all_curls() {
        let mut s = new_finger_curl_state();
        set_all_curls(&mut s, 0.8);
        assert!(s.curls.iter().all(|&v| (v - 0.8).abs() < 1e-6));
    }

    #[test]
    fn test_compute_weights() {
        let mut s = new_finger_curl_state();
        s.curls = [0.5; 5];
        let cfg = default_finger_curl_config();
        let w = compute_finger_curl_weights(&s, &cfg);
        assert!(w.curl_weights.iter().all(|&v| (0.0..=1.0).contains(&v)));
    }

    #[test]
    fn test_to_json() {
        let s = new_finger_curl_state();
        let j = finger_curl_to_json(&s);
        assert!(j.contains("curls"));
    }

    #[test]
    fn test_blend() {
        let a = new_finger_curl_state();
        let mut b = new_finger_curl_state();
        b.curls = [1.0; 5];
        let mid = blend_finger_curl_states(&a, &b, 0.5);
        assert!((mid.curls[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_finger_index() {
        assert_eq!(finger_index(Finger::Ring), 3);
    }
}
