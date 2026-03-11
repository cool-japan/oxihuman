// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Foot toe spread morph — controls how spread out the toes are.

/// Number of toes on each foot.
pub const TOE_COUNT: usize = 5;

/// Configuration for foot toe spread.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FootToeSpreadConfig {
    pub max_spread: f32,
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FootToeSpreadState {
    pub left_spread: [f32; TOE_COUNT],
    pub right_spread: [f32; TOE_COUNT],
    pub left_curl: f32,
    pub right_curl: f32,
}

#[allow(dead_code)]
pub fn default_foot_toe_spread_config() -> FootToeSpreadConfig {
    FootToeSpreadConfig { max_spread: 1.0 }
}

#[allow(dead_code)]
pub fn new_foot_toe_spread_state() -> FootToeSpreadState {
    FootToeSpreadState {
        left_spread: [0.0; TOE_COUNT],
        right_spread: [0.0; TOE_COUNT],
        left_curl: 0.0,
        right_curl: 0.0,
    }
}

#[allow(dead_code)]
pub fn fts_set_left_all(state: &mut FootToeSpreadState, cfg: &FootToeSpreadConfig, v: f32) {
    let clamped = v.clamp(0.0, cfg.max_spread);
    #[allow(clippy::needless_range_loop)]
    for i in 0..TOE_COUNT {
        state.left_spread[i] = clamped;
    }
}

#[allow(dead_code)]
pub fn fts_set_right_all(state: &mut FootToeSpreadState, cfg: &FootToeSpreadConfig, v: f32) {
    let clamped = v.clamp(0.0, cfg.max_spread);
    #[allow(clippy::needless_range_loop)]
    for i in 0..TOE_COUNT {
        state.right_spread[i] = clamped;
    }
}

#[allow(dead_code)]
pub fn fts_set_toe(
    state: &mut FootToeSpreadState,
    cfg: &FootToeSpreadConfig,
    left: bool,
    toe: usize,
    v: f32,
) {
    if toe >= TOE_COUNT {
        return;
    }
    let clamped = v.clamp(0.0, cfg.max_spread);
    if left {
        state.left_spread[toe] = clamped;
    } else {
        state.right_spread[toe] = clamped;
    }
}

#[allow(dead_code)]
pub fn fts_set_curl(state: &mut FootToeSpreadState, left_curl: f32, right_curl: f32) {
    state.left_curl = left_curl.clamp(0.0, 1.0);
    state.right_curl = right_curl.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn fts_reset(state: &mut FootToeSpreadState) {
    *state = new_foot_toe_spread_state();
}

#[allow(dead_code)]
pub fn fts_is_neutral(state: &FootToeSpreadState) -> bool {
    let left_zero =
        !state.left_spread.is_empty() && state.left_spread.iter().all(|v| v.abs() < 1e-6);
    let right_zero =
        !state.right_spread.is_empty() && state.right_spread.iter().all(|v| v.abs() < 1e-6);
    left_zero && right_zero && state.left_curl.abs() < 1e-6 && state.right_curl.abs() < 1e-6
}

#[allow(dead_code)]
pub fn fts_average_spread(state: &FootToeSpreadState) -> f32 {
    let total: f32 = state.left_spread.iter().sum::<f32>() + state.right_spread.iter().sum::<f32>();
    total / (2 * TOE_COUNT) as f32
}

#[allow(dead_code)]
pub fn fts_blend(a: &FootToeSpreadState, b: &FootToeSpreadState, t: f32) -> FootToeSpreadState {
    let t = t.clamp(0.0, 1.0);
    let mut ls = [0.0f32; TOE_COUNT];
    let mut rs = [0.0f32; TOE_COUNT];
    #[allow(clippy::needless_range_loop)]
    for i in 0..TOE_COUNT {
        ls[i] = a.left_spread[i] + (b.left_spread[i] - a.left_spread[i]) * t;
        rs[i] = a.right_spread[i] + (b.right_spread[i] - a.right_spread[i]) * t;
    }
    FootToeSpreadState {
        left_spread: ls,
        right_spread: rs,
        left_curl: a.left_curl + (b.left_curl - a.left_curl) * t,
        right_curl: a.right_curl + (b.right_curl - a.right_curl) * t,
    }
}

#[allow(dead_code)]
pub fn fts_to_weights(state: &FootToeSpreadState) -> Vec<(String, f32)> {
    let mut out = Vec::with_capacity(TOE_COUNT * 2 + 2);
    #[allow(clippy::needless_range_loop)]
    for i in 0..TOE_COUNT {
        out.push((format!("toe_spread_l_{i}"), state.left_spread[i]));
        out.push((format!("toe_spread_r_{i}"), state.right_spread[i]));
    }
    out.push(("toe_curl_l".to_string(), state.left_curl));
    out.push(("toe_curl_r".to_string(), state.right_curl));
    out
}

#[allow(dead_code)]
pub fn fts_to_json(state: &FootToeSpreadState) -> String {
    format!(
        r#"{{"left_avg":{:.4},"right_avg":{:.4},"left_curl":{:.4},"right_curl":{:.4}}}"#,
        state.left_spread.iter().sum::<f32>() / TOE_COUNT as f32,
        state.right_spread.iter().sum::<f32>() / TOE_COUNT as f32,
        state.left_curl,
        state.right_curl
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_foot_toe_spread_config();
        assert!((cfg.max_spread - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_state_neutral() {
        let s = new_foot_toe_spread_state();
        assert!(fts_is_neutral(&s));
    }

    #[test]
    fn set_left_all_clamps() {
        let cfg = default_foot_toe_spread_config();
        let mut s = new_foot_toe_spread_state();
        fts_set_left_all(&mut s, &cfg, 5.0);
        assert!(s.left_spread.iter().all(|&v| (v - 1.0).abs() < 1e-6));
    }

    #[test]
    fn set_right_all() {
        let cfg = default_foot_toe_spread_config();
        let mut s = new_foot_toe_spread_state();
        fts_set_right_all(&mut s, &cfg, 0.5);
        assert!(s.right_spread.iter().all(|&v| (v - 0.5).abs() < 1e-6));
    }

    #[test]
    fn set_single_toe() {
        let cfg = default_foot_toe_spread_config();
        let mut s = new_foot_toe_spread_state();
        fts_set_toe(&mut s, &cfg, true, 2, 0.7);
        assert!((s.left_spread[2] - 0.7).abs() < 1e-6);
    }

    #[test]
    fn set_curl() {
        let mut s = new_foot_toe_spread_state();
        fts_set_curl(&mut s, 0.3, 0.6);
        assert!((s.left_curl - 0.3).abs() < 1e-6);
        assert!((s.right_curl - 0.6).abs() < 1e-6);
    }

    #[test]
    fn average_spread() {
        let cfg = default_foot_toe_spread_config();
        let mut s = new_foot_toe_spread_state();
        fts_set_left_all(&mut s, &cfg, 1.0);
        fts_set_right_all(&mut s, &cfg, 1.0);
        assert!((fts_average_spread(&s) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_foot_toe_spread_config();
        let mut s = new_foot_toe_spread_state();
        fts_set_left_all(&mut s, &cfg, 0.8);
        fts_reset(&mut s);
        assert!(fts_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let a = new_foot_toe_spread_state();
        let cfg = default_foot_toe_spread_config();
        let mut b = new_foot_toe_spread_state();
        fts_set_left_all(&mut b, &cfg, 1.0);
        let mid = fts_blend(&a, &b, 0.5);
        assert!((mid.left_spread[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_weights_count() {
        let s = new_foot_toe_spread_state();
        assert_eq!(fts_to_weights(&s).len(), TOE_COUNT * 2 + 2);
    }

    #[test]
    fn to_json_fields() {
        let s = new_foot_toe_spread_state();
        let j = fts_to_json(&s);
        assert!(j.contains("left_avg"));
        assert!(j.contains("right_curl"));
    }
}
