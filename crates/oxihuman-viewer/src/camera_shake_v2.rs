// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Camera shake v2 — frequency-band procedural camera trauma system.

use std::f32::consts::TAU;

/// Configuration for camera shake v2.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraShakeV2Config {
    pub max_trauma: f32,
    pub trauma_decay: f32,
    pub position_scale: f32,
    pub rotation_scale: f32,
}

/// Runtime state for camera shake v2.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraShakeV2State {
    pub trauma: f32,
    pub time: f32,
    pub seed: u32,
}

#[allow(dead_code)]
pub fn default_camera_shake_v2_config() -> CameraShakeV2Config {
    CameraShakeV2Config {
        max_trauma: 1.0,
        trauma_decay: 0.8,
        position_scale: 0.05,
        rotation_scale: 0.02,
    }
}

#[allow(dead_code)]
pub fn new_camera_shake_v2_state() -> CameraShakeV2State {
    CameraShakeV2State {
        trauma: 0.0,
        time: 0.0,
        seed: 42,
    }
}

#[allow(dead_code)]
pub fn cs2_add_trauma(state: &mut CameraShakeV2State, cfg: &CameraShakeV2Config, amount: f32) {
    state.trauma = (state.trauma + amount).clamp(0.0, cfg.max_trauma);
}

#[allow(dead_code)]
pub fn cs2_update(state: &mut CameraShakeV2State, cfg: &CameraShakeV2Config, dt: f32) {
    if state.trauma > 0.0 {
        state.trauma = (state.trauma - cfg.trauma_decay * dt).max(0.0);
    }
    state.time += dt;
}

#[allow(dead_code)]
pub fn cs2_position_offset(state: &CameraShakeV2State, cfg: &CameraShakeV2Config) -> [f32; 3] {
    let shake = state.trauma * state.trauma;
    let t = state.time;
    [
        shake * cfg.position_scale * (t * 12.9 * TAU / 10.0).sin(),
        shake * cfg.position_scale * (t * 17.3 * TAU / 10.0).sin(),
        shake * cfg.position_scale * (t * 8.7 * TAU / 10.0).sin(),
    ]
}

#[allow(dead_code)]
pub fn cs2_rotation_offset(state: &CameraShakeV2State, cfg: &CameraShakeV2Config) -> [f32; 3] {
    let shake = state.trauma * state.trauma;
    let t = state.time;
    [
        shake * cfg.rotation_scale * (t * 11.2 * TAU / 10.0).sin(),
        shake * cfg.rotation_scale * (t * 7.8 * TAU / 10.0).sin(),
        shake * cfg.rotation_scale * (t * 15.4 * TAU / 10.0).sin(),
    ]
}

#[allow(dead_code)]
pub fn cs2_is_active(state: &CameraShakeV2State) -> bool {
    state.trauma > 1e-6
}

#[allow(dead_code)]
pub fn cs2_reset(state: &mut CameraShakeV2State) {
    *state = new_camera_shake_v2_state();
}

#[allow(dead_code)]
pub fn cs2_to_json(state: &CameraShakeV2State) -> String {
    format!(
        r#"{{"trauma":{:.4},"time":{:.4}}}"#,
        state.trauma, state.time
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_camera_shake_v2_config();
        assert!((cfg.max_trauma - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_state_inactive() {
        let s = new_camera_shake_v2_state();
        assert!(!cs2_is_active(&s));
    }

    #[test]
    fn add_trauma() {
        let cfg = default_camera_shake_v2_config();
        let mut s = new_camera_shake_v2_state();
        cs2_add_trauma(&mut s, &cfg, 0.5);
        assert!(cs2_is_active(&s));
    }

    #[test]
    fn trauma_clamps_to_max() {
        let cfg = default_camera_shake_v2_config();
        let mut s = new_camera_shake_v2_state();
        cs2_add_trauma(&mut s, &cfg, 5.0);
        assert!((s.trauma - 1.0).abs() < 1e-6);
    }

    #[test]
    fn update_decays_trauma() {
        let cfg = default_camera_shake_v2_config();
        let mut s = new_camera_shake_v2_state();
        cs2_add_trauma(&mut s, &cfg, 1.0);
        let before = s.trauma;
        cs2_update(&mut s, &cfg, 0.1);
        assert!(s.trauma < before);
    }

    #[test]
    fn position_offset_zero_when_no_trauma() {
        let cfg = default_camera_shake_v2_config();
        let s = new_camera_shake_v2_state();
        let off = cs2_position_offset(&s, &cfg);
        assert!(off.iter().all(|v| v.abs() < 1e-6));
    }

    #[test]
    fn rotation_offset_nonzero_with_trauma() {
        let cfg = default_camera_shake_v2_config();
        let mut s = new_camera_shake_v2_state();
        cs2_add_trauma(&mut s, &cfg, 1.0);
        s.time = 0.5;
        let rot = cs2_rotation_offset(&s, &cfg);
        assert!(rot.iter().any(|v| v.abs() > 1e-6));
    }

    #[test]
    fn reset_clears_state() {
        let cfg = default_camera_shake_v2_config();
        let mut s = new_camera_shake_v2_state();
        cs2_add_trauma(&mut s, &cfg, 0.8);
        cs2_reset(&mut s);
        assert!(!cs2_is_active(&s));
    }

    #[test]
    fn to_json_fields() {
        let s = new_camera_shake_v2_state();
        let j = cs2_to_json(&s);
        assert!(j.contains("trauma"));
    }
}
