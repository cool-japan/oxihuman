// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Chin recession control — posterior chin displacement and retrogenia shaping.

/// Configuration for chin recession.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChinRecessionConfig {
    pub max_recession: f32,
    pub max_protrusion: f32,
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChinRecessionState {
    pub recession: f32,
    pub vertical_shift: f32,
    pub lateral_tilt: f32,
}

#[allow(dead_code)]
pub fn default_chin_recession_config() -> ChinRecessionConfig {
    ChinRecessionConfig {
        max_recession: 1.0,
        max_protrusion: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_chin_recession_state() -> ChinRecessionState {
    ChinRecessionState {
        recession: 0.0,
        vertical_shift: 0.0,
        lateral_tilt: 0.0,
    }
}

#[allow(dead_code)]
pub fn chin_rec_set_recession(state: &mut ChinRecessionState, cfg: &ChinRecessionConfig, v: f32) {
    state.recession = v.clamp(-cfg.max_protrusion, cfg.max_recession);
}

#[allow(dead_code)]
pub fn chin_rec_set_vertical(state: &mut ChinRecessionState, v: f32) {
    state.vertical_shift = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn chin_rec_set_tilt(state: &mut ChinRecessionState, v: f32) {
    state.lateral_tilt = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn chin_rec_reset(state: &mut ChinRecessionState) {
    *state = new_chin_recession_state();
}

#[allow(dead_code)]
pub fn chin_rec_is_neutral(state: &ChinRecessionState) -> bool {
    state.recession.abs() < 1e-6
        && state.vertical_shift.abs() < 1e-6
        && state.lateral_tilt.abs() < 1e-6
}

#[allow(dead_code)]
pub fn chin_rec_net_offset(state: &ChinRecessionState) -> f32 {
    state.recession
}

#[allow(dead_code)]
pub fn chin_rec_blend(
    a: &ChinRecessionState,
    b: &ChinRecessionState,
    t: f32,
) -> ChinRecessionState {
    let t = t.clamp(0.0, 1.0);
    ChinRecessionState {
        recession: a.recession + (b.recession - a.recession) * t,
        vertical_shift: a.vertical_shift + (b.vertical_shift - a.vertical_shift) * t,
        lateral_tilt: a.lateral_tilt + (b.lateral_tilt - a.lateral_tilt) * t,
    }
}

#[allow(dead_code)]
pub fn chin_rec_to_weights(state: &ChinRecessionState) -> Vec<(String, f32)> {
    vec![
        ("chin_recession".to_string(), state.recession),
        ("chin_vertical_shift".to_string(), state.vertical_shift),
        ("chin_lateral_tilt".to_string(), state.lateral_tilt),
    ]
}

#[allow(dead_code)]
pub fn chin_rec_to_json(state: &ChinRecessionState) -> String {
    format!(
        r#"{{"recession":{:.4},"vertical_shift":{:.4},"lateral_tilt":{:.4}}}"#,
        state.recession, state.vertical_shift, state.lateral_tilt
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_chin_recession_config();
        assert!((cfg.max_recession - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_state_neutral() {
        let s = new_chin_recession_state();
        assert!(chin_rec_is_neutral(&s));
    }

    #[test]
    fn set_recession_clamps() {
        let cfg = default_chin_recession_config();
        let mut s = new_chin_recession_state();
        chin_rec_set_recession(&mut s, &cfg, 5.0);
        assert!((s.recession - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_recession_negative() {
        let cfg = default_chin_recession_config();
        let mut s = new_chin_recession_state();
        chin_rec_set_recession(&mut s, &cfg, -0.5);
        assert!((s.recession + 0.5).abs() < 1e-6);
    }

    #[test]
    fn set_vertical() {
        let mut s = new_chin_recession_state();
        chin_rec_set_vertical(&mut s, 0.3);
        assert!((s.vertical_shift - 0.3).abs() < 1e-6);
    }

    #[test]
    fn set_tilt_clamps() {
        let mut s = new_chin_recession_state();
        chin_rec_set_tilt(&mut s, 2.0);
        assert!((s.lateral_tilt - 1.0).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_chin_recession_config();
        let mut s = new_chin_recession_state();
        chin_rec_set_recession(&mut s, &cfg, 0.8);
        chin_rec_reset(&mut s);
        assert!(chin_rec_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let a = new_chin_recession_state();
        let cfg = default_chin_recession_config();
        let mut b = new_chin_recession_state();
        chin_rec_set_recession(&mut b, &cfg, 1.0);
        let m = chin_rec_blend(&a, &b, 0.5);
        assert!((m.recession - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_weights_count() {
        let s = new_chin_recession_state();
        assert_eq!(chin_rec_to_weights(&s).len(), 3);
    }

    #[test]
    fn to_json_fields() {
        let s = new_chin_recession_state();
        let j = chin_rec_to_json(&s);
        assert!(j.contains("recession"));
    }
}
