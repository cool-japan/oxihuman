// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Cam-follower mechanism with a circular cam profile.

#![allow(dead_code)]

/// A circular cam-follower mechanism.
/// The cam is an eccentric circle: the follower displacement is
/// r_cam * (1 - cos(theta)) where theta is the cam angle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CamFollower {
    /// Base circle radius in meters.
    pub base_radius: f32,
    /// Eccentricity (lift amount) in meters.
    pub eccentricity: f32,
    /// Current cam angle in radians.
    pub angle: f32,
    /// Angular velocity of cam in rad/s.
    pub omega: f32,
    /// Follower displacement from base position (meters).
    pub follower_pos: f32,
    /// Follower velocity (m/s).
    pub follower_vel: f32,
    /// Preload spring constant (N/m) for follower return.
    pub spring_k: f32,
}

/// Create a new cam-follower.
#[allow(dead_code)]
pub fn new_cam_follower(
    base_radius: f32,
    eccentricity: f32,
    omega: f32,
    spring_k: f32,
) -> CamFollower {
    CamFollower {
        base_radius,
        eccentricity,
        angle: 0.0,
        omega,
        follower_pos: 0.0,
        follower_vel: 0.0,
        spring_k,
    }
}

/// Compute follower displacement for a given cam angle.
/// Lift profile: h(theta) = eccentricity * (1 - cos(theta)).
#[allow(dead_code)]
pub fn cam_follower_lift(cam: &CamFollower, angle: f32) -> f32 {
    cam.eccentricity * (1.0 - angle.cos())
}

/// Compute follower velocity from the cam profile derivative.
/// dh/dt = eccentricity * sin(theta) * omega.
#[allow(dead_code)]
pub fn cam_follower_velocity_from_profile(cam: &CamFollower) -> f32 {
    cam.eccentricity * cam.angle.sin() * cam.omega
}

/// Step the cam-follower mechanism.
#[allow(dead_code)]
pub fn cam_follower_step(cam: &mut CamFollower, dt: f32) {
    cam.angle += cam.omega * dt;
    let target_pos = cam_follower_lift(cam, cam.angle);
    cam.follower_vel = (target_pos - cam.follower_pos) / dt.max(1e-10);
    cam.follower_pos = target_pos;
}

/// Contact force between cam and follower (spring preload).
#[allow(dead_code)]
pub fn cam_contact_force(cam: &CamFollower) -> f32 {
    cam.spring_k * cam.follower_pos
}

/// Maximum lift (at theta = pi).
#[allow(dead_code)]
pub fn cam_max_lift(cam: &CamFollower) -> f32 {
    2.0 * cam.eccentricity
}

/// Cam surface radius at angle theta (eccentric circle).
#[allow(dead_code)]
pub fn cam_radius_at(cam: &CamFollower, angle: f32) -> f32 {
    cam.base_radius + cam.eccentricity * (1.0 - angle.cos())
}

/// Set a new angular velocity.
#[allow(dead_code)]
pub fn cam_set_omega(cam: &mut CamFollower, omega: f32) {
    cam.omega = omega;
}

/// Reset the cam to initial position.
#[allow(dead_code)]
pub fn cam_reset(cam: &mut CamFollower) {
    cam.angle = 0.0;
    cam.follower_pos = 0.0;
    cam.follower_vel = 0.0;
}

/// RPM of the cam.
#[allow(dead_code)]
pub fn cam_rpm(cam: &CamFollower) -> f32 {
    cam.omega * 60.0 / (2.0 * std::f32::consts::PI)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn make_cam() -> CamFollower {
        new_cam_follower(0.05, 0.01, 10.0, 1000.0)
    }

    #[test]
    fn test_initial_lift_zero() {
        let cam = make_cam();
        assert_eq!(cam_follower_lift(&cam, 0.0), 0.0);
    }

    #[test]
    fn test_max_lift_at_pi() {
        let cam = make_cam();
        let lift = cam_follower_lift(&cam, PI);
        assert!((lift - cam_max_lift(&cam)).abs() < 1e-5);
    }

    #[test]
    fn test_step_advances_angle() {
        let mut cam = make_cam();
        cam_follower_step(&mut cam, 0.1);
        assert!(cam.angle > 0.0);
    }

    #[test]
    fn test_follower_pos_in_range() {
        let mut cam = make_cam();
        for _ in 0..100 {
            cam_follower_step(&mut cam, 0.01);
        }
        assert!(cam.follower_pos >= 0.0);
        assert!(cam.follower_pos <= cam_max_lift(&cam) + 1e-5);
    }

    #[test]
    fn test_max_lift() {
        let cam = make_cam();
        assert!((cam_max_lift(&cam) - 0.02).abs() < 1e-5);
    }

    #[test]
    fn test_cam_radius_at_zero() {
        let cam = make_cam();
        assert!((cam_radius_at(&cam, 0.0) - cam.base_radius).abs() < 1e-5);
    }

    #[test]
    fn test_contact_force_positive() {
        let mut cam = make_cam();
        cam_follower_step(&mut cam, 0.5);
        assert!(cam_contact_force(&cam) >= 0.0);
    }

    #[test]
    fn test_reset() {
        let mut cam = make_cam();
        cam_follower_step(&mut cam, 1.0);
        cam_reset(&mut cam);
        assert_eq!(cam.angle, 0.0);
        assert_eq!(cam.follower_pos, 0.0);
    }

    #[test]
    fn test_rpm() {
        let cam = make_cam();
        let rpm = cam_rpm(&cam);
        assert!(rpm > 0.0);
    }

    #[test]
    fn test_set_omega() {
        let mut cam = make_cam();
        cam_set_omega(&mut cam, 20.0);
        assert_eq!(cam.omega, 20.0);
    }
}
