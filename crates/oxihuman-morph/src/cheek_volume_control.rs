// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cheek volume morph (puffiness/hollowness).

#![allow(dead_code)]

/// Configuration for cheek volume morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheekVolumeConfig {
    pub max_puff: f32,
    pub max_hollow: f32,
}

/// Runtime state for cheek volume morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CheekVolumeState {
    pub puff_l: f32,
    pub puff_r: f32,
    pub hollow_l: f32,
    pub hollow_r: f32,
}

#[allow(dead_code)]
pub fn default_cheek_volume_config() -> CheekVolumeConfig {
    CheekVolumeConfig {
        max_puff: 1.0,
        max_hollow: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_cheek_volume_state() -> CheekVolumeState {
    CheekVolumeState {
        puff_l: 0.0,
        puff_r: 0.0,
        hollow_l: 0.0,
        hollow_r: 0.0,
    }
}

#[allow(dead_code)]
pub fn cv_set_puff(state: &mut CheekVolumeState, cfg: &CheekVolumeConfig, left: f32, right: f32) {
    state.puff_l = left.clamp(0.0, cfg.max_puff);
    state.puff_r = right.clamp(0.0, cfg.max_puff);
}

#[allow(dead_code)]
pub fn cv_set_hollow(
    state: &mut CheekVolumeState,
    cfg: &CheekVolumeConfig,
    left: f32,
    right: f32,
) {
    state.hollow_l = left.clamp(0.0, cfg.max_hollow);
    state.hollow_r = right.clamp(0.0, cfg.max_hollow);
}

#[allow(dead_code)]
pub fn cv_mirror(state: &mut CheekVolumeState) {
    let avg_p = (state.puff_l + state.puff_r) * 0.5;
    let avg_h = (state.hollow_l + state.hollow_r) * 0.5;
    state.puff_l = avg_p;
    state.puff_r = avg_p;
    state.hollow_l = avg_h;
    state.hollow_r = avg_h;
}

#[allow(dead_code)]
pub fn cv_reset(state: &mut CheekVolumeState) {
    *state = new_cheek_volume_state();
}

#[allow(dead_code)]
pub fn cv_to_weights(state: &CheekVolumeState) -> Vec<(String, f32)> {
    vec![
        ("cheek_puff_l".to_string(), state.puff_l),
        ("cheek_puff_r".to_string(), state.puff_r),
        ("cheek_hollow_l".to_string(), state.hollow_l),
        ("cheek_hollow_r".to_string(), state.hollow_r),
    ]
}

#[allow(dead_code)]
pub fn cv_to_json(state: &CheekVolumeState) -> String {
    format!(
        r#"{{"puff_l":{:.4},"puff_r":{:.4},"hollow_l":{:.4},"hollow_r":{:.4}}}"#,
        state.puff_l, state.puff_r, state.hollow_l, state.hollow_r
    )
}

#[allow(dead_code)]
pub fn cv_clamp(state: &mut CheekVolumeState, cfg: &CheekVolumeConfig) {
    state.puff_l = state.puff_l.clamp(0.0, cfg.max_puff);
    state.puff_r = state.puff_r.clamp(0.0, cfg.max_puff);
    state.hollow_l = state.hollow_l.clamp(0.0, cfg.max_hollow);
    state.hollow_r = state.hollow_r.clamp(0.0, cfg.max_hollow);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_cheek_volume_config();
        assert!((cfg.max_puff - 1.0).abs() < 1e-6);
        assert!((cfg.max_hollow - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state_zeros() {
        let s = new_cheek_volume_state();
        assert_eq!(s.puff_l, 0.0);
        assert_eq!(s.hollow_r, 0.0);
    }

    #[test]
    fn test_set_puff_clamps() {
        let cfg = default_cheek_volume_config();
        let mut s = new_cheek_volume_state();
        cv_set_puff(&mut s, &cfg, 3.0, -1.0);
        assert!((s.puff_l - 1.0).abs() < 1e-6);
        assert_eq!(s.puff_r, 0.0);
    }

    #[test]
    fn test_set_hollow_valid() {
        let cfg = default_cheek_volume_config();
        let mut s = new_cheek_volume_state();
        cv_set_hollow(&mut s, &cfg, 0.3, 0.7);
        assert!((s.hollow_l - 0.3).abs() < 1e-6);
        assert!((s.hollow_r - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_mirror() {
        let cfg = default_cheek_volume_config();
        let mut s = new_cheek_volume_state();
        cv_set_puff(&mut s, &cfg, 0.2, 0.8);
        cv_mirror(&mut s);
        assert!((s.puff_l - 0.5).abs() < 1e-6);
        assert!((s.puff_r - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_cheek_volume_config();
        let mut s = new_cheek_volume_state();
        cv_set_puff(&mut s, &cfg, 0.5, 0.5);
        cv_reset(&mut s);
        assert_eq!(s.puff_l, 0.0);
    }

    #[test]
    fn test_to_weights_count() {
        let s = new_cheek_volume_state();
        assert_eq!(cv_to_weights(&s).len(), 4);
    }

    #[test]
    fn test_to_json_contains_fields() {
        let s = new_cheek_volume_state();
        let j = cv_to_json(&s);
        assert!(j.contains("puff_l"));
        assert!(j.contains("hollow_r"));
    }

    #[test]
    fn test_clamp() {
        let cfg = default_cheek_volume_config();
        let mut s = CheekVolumeState {
            puff_l: 2.0,
            puff_r: -0.5,
            hollow_l: 3.0,
            hollow_r: -1.0,
        };
        cv_clamp(&mut s, &cfg);
        assert!((s.puff_l - 1.0).abs() < 1e-6);
        assert_eq!(s.puff_r, 0.0);
        assert!((s.hollow_l - 1.0).abs() < 1e-6);
        assert_eq!(s.hollow_r, 0.0);
    }
}
