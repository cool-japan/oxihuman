#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::f32::consts::FRAC_PI_2;

/// Camera fly-through mode state.
#[derive(Debug, Clone)]
pub struct CameraFlyMode {
    pub position: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub speed: f32,
    pub boost: f32,
    pub active: bool,
}

#[allow(dead_code)]
pub fn new_camera_fly_mode() -> CameraFlyMode {
    CameraFlyMode {
        position: [0.0, 0.0, 0.0],
        yaw: 0.0,
        pitch: 0.0,
        speed: 1.0,
        boost: 3.0,
        active: false,
    }
}

#[allow(dead_code)]
pub fn fly_move(mode: &mut CameraFlyMode, forward: f32, right: f32, up: f32, dt: f32) {
    let dir = fly_direction(mode);
    // right vector = (sin(yaw + PI/2), 0, cos(yaw + PI/2))
    let right_angle = mode.yaw + FRAC_PI_2;
    let rv = [right_angle.sin(), 0.0f32, right_angle.cos()];
    let speed = mode.speed;
    mode.position[0] += (dir[0] * forward + rv[0] * right) * speed * dt;
    mode.position[1] += (dir[1] * forward + up) * speed * dt;
    mode.position[2] += (dir[2] * forward + rv[2] * right) * speed * dt;
}

#[allow(dead_code)]
pub fn fly_rotate(mode: &mut CameraFlyMode, dyaw: f32, dpitch: f32) {
    mode.yaw += dyaw;
    mode.pitch = (mode.pitch + dpitch).clamp(-FRAC_PI_2 + 0.01, FRAC_PI_2 - 0.01);
}

#[allow(dead_code)]
pub fn fly_direction(mode: &CameraFlyMode) -> [f32; 3] {
    let cp = mode.pitch.cos();
    [
        cp * mode.yaw.sin(),
        mode.pitch.sin(),
        cp * mode.yaw.cos(),
    ]
}

#[allow(dead_code)]
pub fn toggle_fly(mode: &mut CameraFlyMode) {
    mode.active = !mode.active;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        let m = new_camera_fly_mode();
        assert_eq!(m.position, [0.0, 0.0, 0.0]);
        assert!(!m.active);
    }

    #[test]
    fn test_toggle_fly_on() {
        let mut m = new_camera_fly_mode();
        toggle_fly(&mut m);
        assert!(m.active);
    }

    #[test]
    fn test_toggle_fly_off() {
        let mut m = new_camera_fly_mode();
        toggle_fly(&mut m);
        toggle_fly(&mut m);
        assert!(!m.active);
    }

    #[test]
    fn test_fly_rotate_yaw() {
        let mut m = new_camera_fly_mode();
        fly_rotate(&mut m, 0.1, 0.0);
        assert!((m.yaw - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_fly_rotate_pitch_clamped() {
        let mut m = new_camera_fly_mode();
        fly_rotate(&mut m, 0.0, 10.0);
        assert!(m.pitch < FRAC_PI_2);
    }

    #[test]
    fn test_fly_direction_forward() {
        let m = new_camera_fly_mode(); // yaw=0, pitch=0 → looking along +Z
        let d = fly_direction(&m);
        assert!(d[2] > 0.0);
    }

    #[test]
    fn test_fly_move_forward() {
        let mut m = new_camera_fly_mode();
        let prev_z = m.position[2];
        fly_move(&mut m, 1.0, 0.0, 0.0, 1.0);
        assert!(m.position[2] > prev_z);
    }

    #[test]
    fn test_fly_move_zero_dt() {
        let mut m = new_camera_fly_mode();
        let pos = m.position;
        fly_move(&mut m, 1.0, 1.0, 1.0, 0.0);
        assert_eq!(m.position, pos);
    }

    #[test]
    fn test_speed_default() {
        let m = new_camera_fly_mode();
        assert!((m.speed - 1.0).abs() < 1e-6);
    }
}
