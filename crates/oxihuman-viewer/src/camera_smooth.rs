// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Camera smoothing / damping controller.

use std::f32::consts::FRAC_PI_4;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraSmoothConfig {
    pub position_lag: f32,
    pub rotation_lag: f32,
}

impl Default for CameraSmoothConfig {
    fn default() -> Self {
        Self {
            position_lag: 0.1,
            rotation_lag: 0.08,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraSmoothState {
    pub position: [f32; 3],
    pub target: [f32; 3],
    pub config: CameraSmoothConfig,
}

#[allow(dead_code)]
pub fn default_camera_smooth_config() -> CameraSmoothConfig {
    CameraSmoothConfig::default()
}

#[allow(dead_code)]
pub fn new_camera_smooth_state(config: CameraSmoothConfig) -> CameraSmoothState {
    CameraSmoothState {
        position: [0.0; 3],
        target: [0.0; 3],
        config,
    }
}

#[allow(dead_code)]
pub fn csm_step(
    state: &mut CameraSmoothState,
    desired_pos: [f32; 3],
    desired_tgt: [f32; 3],
    dt: f32,
) {
    let lag_p = state.config.position_lag.clamp(1e-6, 1.0);
    let lag_r = state.config.rotation_lag.clamp(1e-6, 1.0);
    let alpha_p = (dt / lag_p).min(1.0);
    let alpha_r = (dt / lag_r).min(1.0);
    for i in 0..3 {
        state.position[i] += (desired_pos[i] - state.position[i]) * alpha_p;
        state.target[i] += (desired_tgt[i] - state.target[i]) * alpha_r;
    }
}

#[allow(dead_code)]
pub fn csm_set_instant(state: &mut CameraSmoothState, pos: [f32; 3], tgt: [f32; 3]) {
    state.position = pos;
    state.target = tgt;
}

#[allow(dead_code)]
pub fn csm_reset(state: &mut CameraSmoothState) {
    state.position = [0.0; 3];
    state.target = [0.0; 3];
}

#[allow(dead_code)]
pub fn csm_is_at_origin(state: &CameraSmoothState) -> bool {
    state.position.iter().all(|v| v.abs() < 1e-6)
}

#[allow(dead_code)]
pub fn csm_distance(state: &CameraSmoothState) -> f32 {
    let d: f32 = (0..3)
        .map(|i| (state.position[i] - state.target[i]).powi(2))
        .sum();
    d.sqrt()
}

#[allow(dead_code)]
pub fn csm_view_angle_rad(state: &CameraSmoothState) -> f32 {
    let d = csm_distance(state);
    if d > 1e-9 {
        (1.0 / d).atan().min(FRAC_PI_4)
    } else {
        0.0
    }
}

#[allow(dead_code)]
pub fn csm_to_json(state: &CameraSmoothState) -> String {
    format!(
        "{{\"dist\":{:.4},\"pos\":[{:.4},{:.4},{:.4}]}}",
        csm_distance(state),
        state.position[0],
        state.position[1],
        state.position[2]
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_at_origin() {
        assert!(csm_is_at_origin(&new_camera_smooth_state(
            default_camera_smooth_config()
        )));
    }
    #[test]
    fn set_instant_applies() {
        let mut s = new_camera_smooth_state(default_camera_smooth_config());
        csm_set_instant(&mut s, [1.0, 2.0, 3.0], [0.0; 3]);
        assert!((s.position[0] - 1.0).abs() < 1e-5);
    }
    #[test]
    fn reset_zeroes() {
        let mut s = new_camera_smooth_state(default_camera_smooth_config());
        csm_set_instant(&mut s, [1.0; 3], [0.0; 3]);
        csm_reset(&mut s);
        assert!(csm_is_at_origin(&s));
    }
    #[test]
    fn step_moves_toward_target() {
        let mut s = new_camera_smooth_state(default_camera_smooth_config());
        csm_step(&mut s, [10.0, 0.0, 0.0], [0.0; 3], 1.0);
        assert!(s.position[0] > 0.0);
    }
    #[test]
    fn step_does_not_overshoot() {
        let mut s = new_camera_smooth_state(default_camera_smooth_config());
        csm_step(&mut s, [1.0, 0.0, 0.0], [0.0; 3], 1.0);
        assert!(s.position[0] <= 1.0);
    }
    #[test]
    fn distance_nonzero_after_set() {
        let mut s = new_camera_smooth_state(default_camera_smooth_config());
        csm_set_instant(&mut s, [3.0, 4.0, 0.0], [0.0; 3]);
        assert!((csm_distance(&s) - 5.0).abs() < 1e-3);
    }
    #[test]
    fn view_angle_nonneg() {
        let s = new_camera_smooth_state(default_camera_smooth_config());
        assert!(csm_view_angle_rad(&s) >= 0.0);
    }
    #[test]
    fn to_json_has_dist() {
        assert!(
            csm_to_json(&new_camera_smooth_state(default_camera_smooth_config()))
                .contains("\"dist\"")
        );
    }
    #[test]
    fn lag_clamps_in_step() {
        let cfg = CameraSmoothConfig {
            position_lag: 0.0,
            rotation_lag: 0.0,
        };
        let mut s = new_camera_smooth_state(cfg);
        csm_step(&mut s, [1.0, 0.0, 0.0], [0.0; 3], 0.016);
        assert!(s.position[0].is_finite());
    }
    #[test]
    fn distance_zero_at_origin() {
        let s = new_camera_smooth_state(default_camera_smooth_config());
        assert!(csm_distance(&s).abs() < 1e-6);
    }
}
