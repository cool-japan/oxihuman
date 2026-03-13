// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Fly-through camera controller for the 3D viewer.

use std::f32::consts::PI;

/// Fly camera state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FlyCameraState {
    pub position: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub speed: f32,
    pub sensitivity: f32,
}

/// Fly camera input.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct FlyCameraInput {
    pub forward: f32,
    pub right: f32,
    pub up: f32,
    pub yaw_delta: f32,
    pub pitch_delta: f32,
    pub dt: f32,
}

/// Default fly camera state.
#[allow(dead_code)]
pub fn default_fly_camera() -> FlyCameraState {
    FlyCameraState {
        position: [0.0, 1.0, -3.0],
        yaw: 0.0,
        pitch: 0.0,
        speed: 5.0,
        sensitivity: 0.5,
    }
}

/// Update fly camera with input.
#[allow(dead_code)]
pub fn update_fly_camera(state: &mut FlyCameraState, input: &FlyCameraInput) {
    state.yaw += input.yaw_delta * state.sensitivity;
    state.pitch = (state.pitch + input.pitch_delta * state.sensitivity)
        .clamp(-PI * 0.49, PI * 0.49);

    let (sy, cy) = state.yaw.sin_cos();
    let (sp, cp) = state.pitch.sin_cos();

    let forward = [cy * cp, sp, sy * cp];
    let right = [sy, 0.0, -cy];
    let up = [0.0, 1.0, 0.0];

    let move_speed = state.speed * input.dt;
    for i in 0..3 {
        state.position[i] += forward[i] * input.forward * move_speed;
        state.position[i] += right[i] * input.right * move_speed;
        state.position[i] += up[i] * input.up * move_speed;
    }
}

/// Get the forward direction.
#[allow(dead_code)]
pub fn fly_camera_forward(state: &FlyCameraState) -> [f32; 3] {
    let (sy, cy) = state.yaw.sin_cos();
    let (sp, cp) = state.pitch.sin_cos();
    [cy * cp, sp, sy * cp]
}

/// Set camera speed.
#[allow(dead_code)]
pub fn set_fly_camera_speed(state: &mut FlyCameraState, speed: f32) {
    state.speed = speed.clamp(0.1, 100.0);
}

/// Reset camera.
#[allow(dead_code)]
pub fn reset_fly_camera(state: &mut FlyCameraState) {
    *state = default_fly_camera();
}

/// Get look-at target from current state.
#[allow(dead_code)]
pub fn fly_camera_target(state: &FlyCameraState) -> [f32; 3] {
    let fwd = fly_camera_forward(state);
    [
        state.position[0] + fwd[0],
        state.position[1] + fwd[1],
        state.position[2] + fwd[2],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let s = default_fly_camera();
        assert!((s.speed - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_update_no_input() {
        let mut s = default_fly_camera();
        let input = FlyCameraInput { forward: 0.0, right: 0.0, up: 0.0, yaw_delta: 0.0, pitch_delta: 0.0, dt: 0.016 };
        let pos_before = s.position;
        update_fly_camera(&mut s, &input);
        assert!((s.position[0] - pos_before[0]).abs() < 1e-6);
    }

    #[test]
    fn test_update_forward() {
        let mut s = default_fly_camera();
        let input = FlyCameraInput { forward: 1.0, right: 0.0, up: 0.0, yaw_delta: 0.0, pitch_delta: 0.0, dt: 1.0 };
        update_fly_camera(&mut s, &input);
        // Position should have changed
        let dist = (s.position[0].powi(2) + (s.position[1] - 1.0).powi(2) + (s.position[2] + 3.0).powi(2)).sqrt();
        assert!(dist > 0.1);
    }

    #[test]
    fn test_forward_direction() {
        let s = default_fly_camera();
        let fwd = fly_camera_forward(&s);
        // At yaw=0, pitch=0, forward should be [1,0,0]
        assert!((fwd[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_speed() {
        let mut s = default_fly_camera();
        set_fly_camera_speed(&mut s, 10.0);
        assert!((s.speed - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_speed() {
        let mut s = default_fly_camera();
        set_fly_camera_speed(&mut s, 200.0);
        assert!((s.speed - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let mut s = default_fly_camera();
        s.speed = 50.0;
        reset_fly_camera(&mut s);
        assert!((s.speed - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_target() {
        let s = default_fly_camera();
        let t = fly_camera_target(&s);
        let fwd = fly_camera_forward(&s);
        assert!((t[0] - s.position[0] - fwd[0]).abs() < 1e-6);
    }

    #[test]
    fn test_pitch_clamp() {
        let mut s = default_fly_camera();
        let input = FlyCameraInput { forward: 0.0, right: 0.0, up: 0.0, yaw_delta: 0.0, pitch_delta: 100.0, dt: 1.0 };
        update_fly_camera(&mut s, &input);
        assert!(s.pitch < PI * 0.5);
    }

    #[test]
    fn test_yaw_rotation() {
        let mut s = default_fly_camera();
        let input = FlyCameraInput { forward: 0.0, right: 0.0, up: 0.0, yaw_delta: 1.0, pitch_delta: 0.0, dt: 1.0 };
        update_fly_camera(&mut s, &input);
        assert!(s.yaw.abs() > 0.0);
    }
}
