// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// Constrains rotation to a cone defined by a half-angle around an axis.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct ConeConstraint {
    axis: [f32; 3],
    half_angle: f32,
    stiffness: f32,
}

#[allow(dead_code)]
impl ConeConstraint {
    pub fn new(axis: [f32; 3], half_angle_degrees: f32) -> Self {
        let len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
        let normalized = if len > 1e-9 {
            [axis[0] / len, axis[1] / len, axis[2] / len]
        } else {
            [0.0, 1.0, 0.0]
        };
        Self {
            axis: normalized,
            half_angle: half_angle_degrees.clamp(0.0, 180.0) * PI / 180.0,
            stiffness: 1.0,
        }
    }

    pub fn with_stiffness(mut self, stiffness: f32) -> Self {
        self.stiffness = stiffness;
        self
    }

    pub fn axis(&self) -> [f32; 3] {
        self.axis
    }

    pub fn half_angle_radians(&self) -> f32 {
        self.half_angle
    }

    pub fn half_angle_degrees(&self) -> f32 {
        self.half_angle * 180.0 / PI
    }

    pub fn stiffness(&self) -> f32 {
        self.stiffness
    }

    pub fn is_within_cone(&self, direction: [f32; 3]) -> bool {
        let len = (direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2]).sqrt();
        if len < 1e-9 {
            return true;
        }
        let dot: f32 = (0..3).map(|i| self.axis[i] * direction[i] / len).sum();
        let angle = dot.clamp(-1.0, 1.0).acos();
        angle <= self.half_angle
    }

    pub fn clamp_direction(&self, direction: [f32; 3]) -> [f32; 3] {
        let len = (direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2]).sqrt();
        if len < 1e-9 {
            return self.axis;
        }
        let norm = [direction[0] / len, direction[1] / len, direction[2] / len];
        let dot: f32 = (0..3).map(|i| self.axis[i] * norm[i]).sum();
        let angle = dot.clamp(-1.0, 1.0).acos();

        if angle <= self.half_angle {
            return norm;
        }

        // Project onto cone surface
        let t = self.half_angle / angle;
        let mut result = [0.0f32; 3];
        for i in 0..3 {
            result[i] = self.axis[i] * (1.0 - t) + norm[i] * t;
        }
        let rlen = (result[0] * result[0] + result[1] * result[1] + result[2] * result[2]).sqrt();
        if rlen > 1e-9 {
            for r in &mut result {
                *r /= rlen;
            }
        }
        result
    }

    pub fn violation_angle(&self, direction: [f32; 3]) -> f32 {
        let len = (direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2]).sqrt();
        if len < 1e-9 {
            return 0.0;
        }
        let dot: f32 = (0..3).map(|i| self.axis[i] * direction[i] / len).sum();
        let angle = dot.clamp(-1.0, 1.0).acos();
        (angle - self.half_angle).max(0.0)
    }

    pub fn solid_angle(&self) -> f32 {
        2.0 * PI * (1.0 - self.half_angle.cos())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let c = ConeConstraint::new([0.0, 1.0, 0.0], 45.0);
        assert!((c.half_angle_degrees() - 45.0).abs() < 1e-4);
    }

    #[test]
    fn test_axis_normalized() {
        let c = ConeConstraint::new([0.0, 2.0, 0.0], 30.0);
        let a = c.axis();
        let len = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_within_cone() {
        let c = ConeConstraint::new([0.0, 1.0, 0.0], 90.0);
        assert!(c.is_within_cone([0.0, 1.0, 0.0]));
        assert!(c.is_within_cone([1.0, 1.0, 0.0]));
    }

    #[test]
    fn test_outside_cone() {
        let c = ConeConstraint::new([0.0, 1.0, 0.0], 10.0);
        assert!(!c.is_within_cone([1.0, 0.0, 0.0]));
    }

    #[test]
    fn test_clamp_within_cone() {
        let c = ConeConstraint::new([0.0, 1.0, 0.0], 45.0);
        let clamped = c.clamp_direction([0.0, 1.0, 0.0]);
        assert!((clamped[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_clamp_outside_cone() {
        let c = ConeConstraint::new([0.0, 1.0, 0.0], 10.0);
        let clamped = c.clamp_direction([1.0, 0.0, 0.0]);
        assert!(c.is_within_cone(clamped) || c.violation_angle(clamped) < 0.1);
    }

    #[test]
    fn test_violation_angle_within() {
        let c = ConeConstraint::new([0.0, 1.0, 0.0], 45.0);
        assert!(c.violation_angle([0.0, 1.0, 0.0]).abs() < 1e-6);
    }

    #[test]
    fn test_violation_angle_outside() {
        let c = ConeConstraint::new([0.0, 1.0, 0.0], 10.0);
        assert!(c.violation_angle([1.0, 0.0, 0.0]) > 0.0);
    }

    #[test]
    fn test_solid_angle() {
        let c = ConeConstraint::new([0.0, 1.0, 0.0], 90.0);
        assert!((c.solid_angle() - 2.0 * PI).abs() < 1e-4);
    }

    #[test]
    fn test_stiffness() {
        let c = ConeConstraint::new([0.0, 1.0, 0.0], 30.0).with_stiffness(0.5);
        assert!((c.stiffness() - 0.5).abs() < 1e-9);
    }
}
