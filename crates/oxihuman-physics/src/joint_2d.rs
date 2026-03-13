// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! 2D revolute/prismatic joint stub.

#[derive(Debug, Clone, PartialEq)]
pub enum JointKind2d {
    Revolute,
    Prismatic,
    Fixed,
}

#[derive(Debug, Clone)]
pub struct Joint2d {
    pub kind: JointKind2d,
    pub body_a: usize,
    pub body_b: usize,
    /// Anchor in world space.
    pub anchor: [f32; 2],
    pub stiffness: f32,
    pub damping: f32,
    /// For prismatic: axis of allowed translation.
    pub axis: [f32; 2],
    /// For revolute: lower angle limit (radians).
    pub angle_min: f32,
    /// For revolute: upper angle limit (radians).
    pub angle_max: f32,
}

impl Joint2d {
    pub fn revolute(body_a: usize, body_b: usize, anchor: [f32; 2]) -> Self {
        Joint2d {
            kind: JointKind2d::Revolute,
            body_a,
            body_b,
            anchor,
            stiffness: 1000.0,
            damping: 10.0,
            axis: [1.0, 0.0],
            angle_min: -std::f32::consts::PI,
            angle_max: std::f32::consts::PI,
        }
    }

    pub fn prismatic(body_a: usize, body_b: usize, anchor: [f32; 2], axis: [f32; 2]) -> Self {
        Joint2d {
            kind: JointKind2d::Prismatic,
            body_a,
            body_b,
            anchor,
            stiffness: 1000.0,
            damping: 10.0,
            axis,
            angle_min: 0.0,
            angle_max: 0.0,
        }
    }

    pub fn fixed(body_a: usize, body_b: usize, anchor: [f32; 2]) -> Self {
        Joint2d {
            kind: JointKind2d::Fixed,
            body_a,
            body_b,
            anchor,
            stiffness: 1e6,
            damping: 100.0,
            axis: [1.0, 0.0],
            angle_min: 0.0,
            angle_max: 0.0,
        }
    }

    pub fn is_active(&self) -> bool {
        self.body_a != self.body_b
    }
}

/// Compute spring force pulling pos_b toward anchor.
pub fn joint_spring_force(anchor: [f32; 2], pos_b: [f32; 2], stiffness: f32) -> [f32; 2] {
    [
        (anchor[0] - pos_b[0]) * stiffness,
        (anchor[1] - pos_b[1]) * stiffness,
    ]
}

/// Compute damping force opposing relative velocity.
pub fn joint_damping_force(vel_a: [f32; 2], vel_b: [f32; 2], damping: f32) -> [f32; 2] {
    [
        (vel_a[0] - vel_b[0]) * damping,
        (vel_a[1] - vel_b[1]) * damping,
    ]
}

pub fn joint_error(anchor: [f32; 2], pos: [f32; 2]) -> f32 {
    let dx = anchor[0] - pos[0];
    let dy = anchor[1] - pos[1];
    (dx * dx + dy * dy).sqrt()
}

pub fn clamp_angle(angle: f32, min: f32, max: f32) -> f32 {
    angle.clamp(min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_revolute_creation() {
        let j = Joint2d::revolute(0, 1, [0.0, 0.0]);
        assert_eq!(j.kind, JointKind2d::Revolute);
    }

    #[test]
    fn test_prismatic_creation() {
        let j = Joint2d::prismatic(0, 1, [0.0, 0.0], [1.0, 0.0]);
        assert_eq!(j.kind, JointKind2d::Prismatic);
    }

    #[test]
    fn test_joint_is_active() {
        let j = Joint2d::revolute(0, 1, [0.0, 0.0]);
        assert!(j.is_active() /* different bodies */,);
    }

    #[test]
    fn test_joint_inactive() {
        let j = Joint2d::revolute(0, 0, [0.0, 0.0]);
        assert!(!j.is_active() /* same body */,);
    }

    #[test]
    fn test_spring_force_toward_anchor() {
        let f = joint_spring_force([1.0, 0.0], [0.0, 0.0], 100.0);
        assert!((f[0] - 100.0).abs() < 1e-5, /* force = stiffness * displacement */);
    }

    #[test]
    fn test_damping_force() {
        let f = joint_damping_force([1.0, 0.0], [0.0, 0.0], 10.0);
        assert!((f[0] - 10.0).abs() < 1e-5, /* damping * relative velocity */);
    }

    #[test]
    fn test_joint_error_zero() {
        let err = joint_error([1.0, 2.0], [1.0, 2.0]);
        assert!(err.abs() < 1e-6 /* anchor == pos, no error */,);
    }

    #[test]
    fn test_joint_error_nonzero() {
        let err = joint_error([0.0, 0.0], [3.0, 4.0]);
        assert!((err - 5.0).abs() < 1e-5 /* 3-4-5 triangle */,);
    }

    #[test]
    fn test_clamp_angle() {
        use std::f32::consts::PI;
        let clamped = clamp_angle(2.0 * PI, -PI, PI);
        assert!(clamped <= PI /* clamped at upper limit */,);
    }
}
