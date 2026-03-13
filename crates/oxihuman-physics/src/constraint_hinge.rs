// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Hinge (revolute) constraint: allows rotation around a single axis.

use std::f32::consts::PI;

/// A hinge constraint between two bodies with angular limits.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConstraintHingeDef {
    pub body_a: u32,
    pub body_b: u32,
    pub anchor_a: [f32; 3],
    pub anchor_b: [f32; 3],
    pub axis: [f32; 3],
    pub min_angle: f32,
    pub max_angle: f32,
    pub stiffness: f32,
    pub damping: f32,
}

#[allow(dead_code)]
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0]*v[0] + v[1]*v[1] + v[2]*v[2]).sqrt();
    if l < 1e-10 { return [0.0, 1.0, 0.0]; }
    [v[0]/l, v[1]/l, v[2]/l]
}

#[allow(dead_code)]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0]*b[0] + a[1]*b[1] + a[2]*b[2]
}

#[allow(dead_code)]
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]]
}

#[allow(dead_code)]
impl ConstraintHingeDef {
    pub fn new(body_a: u32, body_b: u32, anchor: [f32; 3], axis: [f32; 3]) -> Self {
        Self {
            body_a, body_b,
            anchor_a: anchor, anchor_b: anchor,
            axis: normalize3(axis),
            min_angle: -PI, max_angle: PI,
            stiffness: 1.0, damping: 0.0,
        }
    }

    pub fn with_limits(mut self, min: f32, max: f32) -> Self {
        self.min_angle = min;
        self.max_angle = max;
        self
    }

    pub fn with_stiffness(mut self, s: f32) -> Self {
        self.stiffness = s;
        self
    }

    pub fn with_damping(mut self, d: f32) -> Self {
        self.damping = d;
        self
    }

    pub fn angle_range(&self) -> f32 {
        self.max_angle - self.min_angle
    }

    /// Check if an angle is within limits.
    pub fn angle_in_limits(&self, angle: f32) -> bool {
        angle >= self.min_angle && angle <= self.max_angle
    }

    /// Clamp angle to limits.
    pub fn clamp_angle(&self, angle: f32) -> f32 {
        angle.clamp(self.min_angle, self.max_angle)
    }

    /// Compute angular error given current angle.
    pub fn angular_error(&self, current_angle: f32) -> f32 {
        if current_angle < self.min_angle {
            current_angle - self.min_angle
        } else if current_angle > self.max_angle {
            current_angle - self.max_angle
        } else {
            0.0
        }
    }

    /// Torque to correct angular error.
    pub fn correction_torque(&self, current_angle: f32) -> f32 {
        let err = self.angular_error(current_angle);
        -self.stiffness * err
    }

    /// Damping torque given angular velocity.
    pub fn damping_torque(&self, angular_vel: f32) -> f32 {
        -self.damping * angular_vel
    }

    /// Total torque = correction + damping.
    pub fn total_torque(&self, angle: f32, ang_vel: f32) -> f32 {
        self.correction_torque(angle) + self.damping_torque(ang_vel)
    }

    /// Positional error between anchors.
    pub fn positional_error(&self, pos_a: [f32; 3], pos_b: [f32; 3]) -> f32 {
        let d = [pos_a[0]-pos_b[0], pos_a[1]-pos_b[1], pos_a[2]-pos_b[2]];
        (d[0]*d[0] + d[1]*d[1] + d[2]*d[2]).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_angle_in_limits() {
        let h = ConstraintHingeDef::new(0, 1, [0.0;3], [0.0,1.0,0.0])
            .with_limits(-1.0, 1.0);
        assert!(h.angle_in_limits(0.0));
        assert!(h.angle_in_limits(1.0));
        assert!(!h.angle_in_limits(2.0));
    }

    #[test]
    fn test_clamp_angle() {
        let h = ConstraintHingeDef::new(0, 1, [0.0;3], [0.0,1.0,0.0])
            .with_limits(-0.5, 0.5);
        assert!((h.clamp_angle(1.0) - 0.5).abs() < 0.01);
        assert!((h.clamp_angle(-1.0) - (-0.5)).abs() < 0.01);
    }

    #[test]
    fn test_angular_error_in_range() {
        let h = ConstraintHingeDef::new(0, 1, [0.0;3], [0.0,1.0,0.0])
            .with_limits(-1.0, 1.0);
        assert!((h.angular_error(0.5)).abs() < 0.001);
    }

    #[test]
    fn test_angular_error_exceeded() {
        let h = ConstraintHingeDef::new(0, 1, [0.0;3], [0.0,1.0,0.0])
            .with_limits(-1.0, 1.0);
        assert!((h.angular_error(1.5) - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_correction_torque() {
        let h = ConstraintHingeDef::new(0, 1, [0.0;3], [0.0,1.0,0.0])
            .with_limits(-1.0, 1.0).with_stiffness(10.0);
        let t = h.correction_torque(1.5);
        assert!(t < 0.0); // corrects back
    }

    #[test]
    fn test_damping_torque() {
        let h = ConstraintHingeDef::new(0, 1, [0.0;3], [0.0,1.0,0.0])
            .with_damping(5.0);
        assert!((h.damping_torque(2.0) - (-10.0)).abs() < 0.01);
    }

    #[test]
    fn test_total_torque() {
        let h = ConstraintHingeDef::new(0, 1, [0.0;3], [0.0,1.0,0.0])
            .with_limits(-1.0, 1.0).with_stiffness(10.0).with_damping(1.0);
        let t = h.total_torque(0.5, 0.0);
        assert!(t.abs() < 0.001); // in range, no velocity
    }

    #[test]
    fn test_angle_range() {
        let h = ConstraintHingeDef::new(0, 1, [0.0;3], [0.0,1.0,0.0])
            .with_limits(-PI/4.0, PI/4.0);
        assert!((h.angle_range() - PI/2.0).abs() < 0.01);
    }

    #[test]
    fn test_positional_error() {
        let h = ConstraintHingeDef::new(0, 1, [0.0;3], [0.0,1.0,0.0]);
        let err = h.positional_error([0.0,0.0,0.0], [3.0,4.0,0.0]);
        assert!((err - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_default_limits() {
        let h = ConstraintHingeDef::new(0, 1, [0.0;3], [0.0,1.0,0.0]);
        assert!((h.min_angle - (-PI)).abs() < 0.01);
        assert!((h.max_angle - PI).abs() < 0.01);
    }
}
