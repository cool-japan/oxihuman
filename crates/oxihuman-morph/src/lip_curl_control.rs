// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Lip curl morph (inward/outward roll of lip edges).

#![allow(dead_code)]

/// Configuration for lip curl morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LipCurlConfig {
    pub max_curl: f32,
}

/// Runtime state for lip curl morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LipCurlState {
    pub upper_curl: f32,
    pub lower_curl: f32,
    pub corner_l_curl: f32,
    pub corner_r_curl: f32,
}

#[allow(dead_code)]
pub fn default_lip_curl_config() -> LipCurlConfig {
    LipCurlConfig { max_curl: 1.0 }
}

#[allow(dead_code)]
pub fn new_lip_curl_state() -> LipCurlState {
    LipCurlState {
        upper_curl: 0.0,
        lower_curl: 0.0,
        corner_l_curl: 0.0,
        corner_r_curl: 0.0,
    }
}

#[allow(dead_code)]
pub fn lc_set_upper_curl(state: &mut LipCurlState, cfg: &LipCurlConfig, v: f32) {
    state.upper_curl = v.clamp(-cfg.max_curl, cfg.max_curl);
}

#[allow(dead_code)]
pub fn lc_set_lower_curl(state: &mut LipCurlState, cfg: &LipCurlConfig, v: f32) {
    state.lower_curl = v.clamp(-cfg.max_curl, cfg.max_curl);
}

#[allow(dead_code)]
pub fn lc_set_corner_curl(
    state: &mut LipCurlState,
    cfg: &LipCurlConfig,
    left: f32,
    right: f32,
) {
    state.corner_l_curl = left.clamp(-cfg.max_curl, cfg.max_curl);
    state.corner_r_curl = right.clamp(-cfg.max_curl, cfg.max_curl);
}

#[allow(dead_code)]
pub fn lc_reset(state: &mut LipCurlState) {
    *state = new_lip_curl_state();
}

#[allow(dead_code)]
pub fn lc_to_weights(state: &LipCurlState) -> Vec<(String, f32)> {
    vec![
        ("lip_upper_curl".to_string(), state.upper_curl),
        ("lip_lower_curl".to_string(), state.lower_curl),
        ("lip_corner_l_curl".to_string(), state.corner_l_curl),
        ("lip_corner_r_curl".to_string(), state.corner_r_curl),
    ]
}

#[allow(dead_code)]
pub fn lc_to_json(state: &LipCurlState) -> String {
    format!(
        r#"{{"upper_curl":{:.4},"lower_curl":{:.4},"corner_l_curl":{:.4},"corner_r_curl":{:.4}}}"#,
        state.upper_curl, state.lower_curl, state.corner_l_curl, state.corner_r_curl
    )
}

#[allow(dead_code)]
pub fn lc_clamp(state: &mut LipCurlState, cfg: &LipCurlConfig) {
    state.upper_curl = state.upper_curl.clamp(-cfg.max_curl, cfg.max_curl);
    state.lower_curl = state.lower_curl.clamp(-cfg.max_curl, cfg.max_curl);
    state.corner_l_curl = state.corner_l_curl.clamp(-cfg.max_curl, cfg.max_curl);
    state.corner_r_curl = state.corner_r_curl.clamp(-cfg.max_curl, cfg.max_curl);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_lip_curl_config();
        assert!((cfg.max_curl - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state_zeros() {
        let s = new_lip_curl_state();
        assert_eq!(s.upper_curl, 0.0);
        assert_eq!(s.corner_r_curl, 0.0);
    }

    #[test]
    fn test_set_upper_curl_clamps() {
        let cfg = default_lip_curl_config();
        let mut s = new_lip_curl_state();
        lc_set_upper_curl(&mut s, &cfg, 3.0);
        assert!((s.upper_curl - 1.0).abs() < 1e-6);
        lc_set_upper_curl(&mut s, &cfg, -3.0);
        assert!((s.upper_curl + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_lower_curl() {
        let cfg = default_lip_curl_config();
        let mut s = new_lip_curl_state();
        lc_set_lower_curl(&mut s, &cfg, 0.5);
        assert!((s.lower_curl - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_corner_curl() {
        let cfg = default_lip_curl_config();
        let mut s = new_lip_curl_state();
        lc_set_corner_curl(&mut s, &cfg, 0.3, -0.4);
        assert!((s.corner_l_curl - 0.3).abs() < 1e-6);
        assert!((s.corner_r_curl + 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_lip_curl_config();
        let mut s = new_lip_curl_state();
        lc_set_upper_curl(&mut s, &cfg, 0.5);
        lc_reset(&mut s);
        assert_eq!(s.upper_curl, 0.0);
    }

    #[test]
    fn test_to_weights_count() {
        let s = new_lip_curl_state();
        assert_eq!(lc_to_weights(&s).len(), 4);
    }

    #[test]
    fn test_to_json_contains_fields() {
        let s = new_lip_curl_state();
        let j = lc_to_json(&s);
        assert!(j.contains("upper_curl"));
        assert!(j.contains("corner_r_curl"));
    }
}
