// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Camera rig v2 — orbit, dolly, and first-person modes with spring damping.

use std::f32::consts::{FRAC_PI_2, PI};

/// Camera rig mode.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigModeV2 {
    Orbit,
    Fly,
    Cinematic,
}

/// Spring-damped orbit state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraRigV2 {
    pub mode: RigModeV2,
    pub target: [f32; 3],
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub fov_deg: f32,
    /// Damping factor 0..=1 (0 = instant, 1 = frozen).
    pub damping: f32,
    /// Current angular velocities.
    yaw_vel: f32,
    pitch_vel: f32,
}

impl Default for CameraRigV2 {
    fn default() -> Self {
        Self {
            mode: RigModeV2::Orbit,
            target: [0.0, 0.9, 0.0],
            distance: 3.0,
            yaw: 0.0,
            pitch: 0.2,
            fov_deg: 60.0,
            damping: 0.85,
            yaw_vel: 0.0,
            pitch_vel: 0.0,
        }
    }
}

#[allow(dead_code)]
pub fn new_camera_rig_v2() -> CameraRigV2 {
    CameraRigV2::default()
}

/// Apply angular impulse (yaw/pitch in radians).
#[allow(dead_code)]
pub fn crv2_apply_impulse(rig: &mut CameraRigV2, dyaw: f32, dpitch: f32) {
    rig.yaw_vel += dyaw;
    rig.pitch_vel += dpitch;
}

/// Step the rig physics (call once per frame).
#[allow(dead_code)]
pub fn crv2_step(rig: &mut CameraRigV2, dt: f32) {
    rig.yaw += rig.yaw_vel * dt;
    rig.pitch = (rig.pitch + rig.pitch_vel * dt).clamp(-FRAC_PI_2 + 0.01, FRAC_PI_2 - 0.01);
    rig.yaw_vel *= rig.damping;
    rig.pitch_vel *= rig.damping;
    // Wrap yaw to −π..π
    if rig.yaw > PI {
        rig.yaw -= 2.0 * PI;
    } else if rig.yaw < -PI {
        rig.yaw += 2.0 * PI;
    }
}

/// Compute eye position from rig.
#[allow(dead_code)]
pub fn crv2_eye_position(rig: &CameraRigV2) -> [f32; 3] {
    let cos_p = rig.pitch.cos();
    let sin_p = rig.pitch.sin();
    let cos_y = rig.yaw.cos();
    let sin_y = rig.yaw.sin();
    [
        rig.target[0] + rig.distance * cos_p * sin_y,
        rig.target[1] + rig.distance * sin_p,
        rig.target[2] + rig.distance * cos_p * cos_y,
    ]
}

/// Dolly (adjust distance).
#[allow(dead_code)]
pub fn crv2_dolly(rig: &mut CameraRigV2, delta: f32) {
    rig.distance = (rig.distance - delta).max(0.05);
}

/// Set mode.
#[allow(dead_code)]
pub fn crv2_set_mode(rig: &mut CameraRigV2, mode: RigModeV2) {
    rig.mode = mode;
}

/// Reset to defaults.
#[allow(dead_code)]
pub fn crv2_reset(rig: &mut CameraRigV2) {
    *rig = CameraRigV2::default();
}

/// Is the rig approximately still (velocities near zero)?
#[allow(dead_code)]
pub fn crv2_is_still(rig: &CameraRigV2) -> bool {
    rig.yaw_vel.abs() < 1e-4 && rig.pitch_vel.abs() < 1e-4
}

/// JSON serialisation.
#[allow(dead_code)]
pub fn crv2_to_json(rig: &CameraRigV2) -> String {
    format!(
        "{{\"yaw\":{:.4},\"pitch\":{:.4},\"distance\":{:.4},\"fov_deg\":{:.1}}}",
        rig.yaw, rig.pitch, rig.distance, rig.fov_deg
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_orbit_mode() {
        assert_eq!(new_camera_rig_v2().mode, RigModeV2::Orbit);
    }

    #[test]
    fn eye_position_differs_from_target() {
        let rig = new_camera_rig_v2();
        let eye = crv2_eye_position(&rig);
        let t = rig.target;
        let diff = (eye[0] - t[0]).powi(2) + (eye[1] - t[1]).powi(2) + (eye[2] - t[2]).powi(2);
        assert!(diff.sqrt() > 0.1);
    }

    #[test]
    fn step_decays_velocity() {
        let mut rig = new_camera_rig_v2();
        crv2_apply_impulse(&mut rig, 1.0, 0.0);
        crv2_step(&mut rig, 0.016);
        assert!(rig.yaw_vel.abs() < 1.0);
    }

    #[test]
    fn pitch_clamps_at_extremes() {
        let mut rig = new_camera_rig_v2();
        crv2_apply_impulse(&mut rig, 0.0, 1000.0);
        crv2_step(&mut rig, 1.0);
        assert!(rig.pitch <= FRAC_PI_2);
    }

    #[test]
    fn dolly_reduces_distance() {
        let mut rig = new_camera_rig_v2();
        let before = rig.distance;
        crv2_dolly(&mut rig, 1.0);
        assert!(rig.distance < before);
    }

    #[test]
    fn dolly_clamps_minimum() {
        let mut rig = new_camera_rig_v2();
        crv2_dolly(&mut rig, 1000.0);
        assert!(rig.distance >= 0.04);
    }

    #[test]
    fn set_mode_works() {
        let mut rig = new_camera_rig_v2();
        crv2_set_mode(&mut rig, RigModeV2::Fly);
        assert_eq!(rig.mode, RigModeV2::Fly);
    }

    #[test]
    fn reset_returns_to_default() {
        let mut rig = new_camera_rig_v2();
        crv2_dolly(&mut rig, 2.0);
        crv2_reset(&mut rig);
        assert!((rig.distance - 3.0).abs() < 1e-5);
    }

    #[test]
    fn still_when_no_impulse() {
        assert!(crv2_is_still(&new_camera_rig_v2()));
    }

    #[test]
    fn json_has_yaw() {
        let j = crv2_to_json(&new_camera_rig_v2());
        assert!(j.contains("yaw") && j.contains("fov_deg"));
    }
}
