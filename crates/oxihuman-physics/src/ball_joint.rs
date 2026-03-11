// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// A ball-and-socket joint allowing rotation within a cone limit.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct BallJoint {
    anchor_a: [f32; 3],
    anchor_b: [f32; 3],
    cone_angle: f32,
    stiffness: f32,
    damping: f32,
    broken: bool,
    breaking_torque: f32,
}

#[allow(dead_code)]
impl BallJoint {
    pub fn new(anchor_a: [f32; 3], anchor_b: [f32; 3]) -> Self {
        Self {
            anchor_a,
            anchor_b,
            cone_angle: PI,
            stiffness: 1000.0,
            damping: 50.0,
            broken: false,
            breaking_torque: f32::MAX,
        }
    }

    pub fn with_cone_angle(mut self, angle: f32) -> Self {
        self.cone_angle = angle.clamp(0.0, PI);
        self
    }

    pub fn with_stiffness(mut self, stiffness: f32) -> Self {
        self.stiffness = stiffness;
        self
    }

    pub fn with_damping(mut self, damping: f32) -> Self {
        self.damping = damping;
        self
    }

    pub fn with_breaking_torque(mut self, torque: f32) -> Self {
        self.breaking_torque = torque;
        self
    }

    pub fn anchor_a(&self) -> [f32; 3] {
        self.anchor_a
    }

    pub fn anchor_b(&self) -> [f32; 3] {
        self.anchor_b
    }

    pub fn cone_angle(&self) -> f32 {
        self.cone_angle
    }

    pub fn is_broken(&self) -> bool {
        self.broken
    }

    pub fn reset(&mut self) {
        self.broken = false;
    }

    pub fn compute_force(
        &mut self,
        world_a: [f32; 3],
        world_b: [f32; 3],
        vel_a: [f32; 3],
        vel_b: [f32; 3],
    ) -> [f32; 3] {
        if self.broken {
            return [0.0; 3];
        }
        let mut force = [0.0f32; 3];
        let mut mag_sq = 0.0f32;
        for i in 0..3 {
            let diff = world_b[i] - world_a[i];
            let rel_vel = vel_b[i] - vel_a[i];
            let f = diff * self.stiffness + rel_vel * self.damping;
            force[i] = f;
            mag_sq += f * f;
        }
        if mag_sq > self.breaking_torque * self.breaking_torque {
            self.broken = true;
            return [0.0; 3];
        }
        force
    }

    pub fn error(&self, world_a: [f32; 3], world_b: [f32; 3]) -> f32 {
        let dx = world_b[0] - world_a[0];
        let dy = world_b[1] - world_a[1];
        let dz = world_b[2] - world_a[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    pub fn is_within_cone(&self, axis: [f32; 3], direction: [f32; 3]) -> bool {
        let dot = axis[0] * direction[0] + axis[1] * direction[1] + axis[2] * direction[2];
        let a_len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
        let d_len = (direction[0] * direction[0]
            + direction[1] * direction[1]
            + direction[2] * direction[2])
            .sqrt();
        if a_len < 1e-9 || d_len < 1e-9 {
            return true;
        }
        let cos_angle = dot / (a_len * d_len);
        cos_angle >= self.cone_angle.cos()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let j = BallJoint::new([0.0; 3], [1.0, 0.0, 0.0]);
        assert!(!j.is_broken());
        assert!((j.cone_angle() - PI).abs() < 1e-6);
    }

    #[test]
    fn test_with_cone_angle() {
        let j = BallJoint::new([0.0; 3], [0.0; 3]).with_cone_angle(1.0);
        assert!((j.cone_angle() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cone_angle_clamped() {
        let j = BallJoint::new([0.0; 3], [0.0; 3]).with_cone_angle(10.0);
        assert!((j.cone_angle() - PI).abs() < 1e-6);
    }

    #[test]
    fn test_compute_force_zero_error() {
        let mut j = BallJoint::new([0.0; 3], [0.0; 3]);
        let f = j.compute_force([1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0; 3], [0.0; 3]);
        for v in &f {
            assert!(v.abs() < 1e-6);
        }
    }

    #[test]
    fn test_compute_force_nonzero() {
        let mut j = BallJoint::new([0.0; 3], [0.0; 3]);
        let f = j.compute_force([0.0; 3], [1.0, 0.0, 0.0], [0.0; 3], [0.0; 3]);
        assert!(f[0] > 0.0);
    }

    #[test]
    fn test_breaking() {
        let mut j = BallJoint::new([0.0; 3], [0.0; 3]).with_breaking_torque(1.0);
        let _ = j.compute_force([0.0; 3], [100.0, 0.0, 0.0], [0.0; 3], [0.0; 3]);
        assert!(j.is_broken());
    }

    #[test]
    fn test_broken_returns_zero() {
        let mut j = BallJoint::new([0.0; 3], [0.0; 3]).with_breaking_torque(1.0);
        let _ = j.compute_force([0.0; 3], [100.0, 0.0, 0.0], [0.0; 3], [0.0; 3]);
        let f = j.compute_force([0.0; 3], [100.0, 0.0, 0.0], [0.0; 3], [0.0; 3]);
        assert_eq!(f, [0.0; 3]);
    }

    #[test]
    fn test_error() {
        let j = BallJoint::new([0.0; 3], [0.0; 3]);
        let e = j.error([0.0; 3], [3.0, 4.0, 0.0]);
        assert!((e - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_reset() {
        let mut j = BallJoint::new([0.0; 3], [0.0; 3]).with_breaking_torque(1.0);
        let _ = j.compute_force([0.0; 3], [100.0, 0.0, 0.0], [0.0; 3], [0.0; 3]);
        j.reset();
        assert!(!j.is_broken());
    }

    #[test]
    fn test_is_within_cone() {
        let j = BallJoint::new([0.0; 3], [0.0; 3]).with_cone_angle(PI / 4.0);
        assert!(j.is_within_cone([0.0, 1.0, 0.0], [0.0, 1.0, 0.0]));
        assert!(!j.is_within_cone([0.0, 1.0, 0.0], [0.0, -1.0, 0.0]));
    }
}
