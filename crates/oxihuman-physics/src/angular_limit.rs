// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// An angular limit constraint that clamps rotation within a range.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AngularLimit {
    pub min_angle: f32,
    pub max_angle: f32,
    pub stiffness: f32,
    pub damping: f32,
}

#[allow(dead_code)]
impl AngularLimit {
    pub fn new(min_angle: f32, max_angle: f32) -> Self {
        Self {
            min_angle,
            max_angle,
            stiffness: 1.0,
            damping: 0.1,
        }
    }

    pub fn symmetric(half_angle: f32) -> Self {
        Self::new(-half_angle, half_angle)
    }

    pub fn full_range() -> Self {
        Self::new(-PI, PI)
    }

    pub fn clamp(&self, angle: f32) -> f32 {
        angle.clamp(self.min_angle, self.max_angle)
    }

    pub fn is_within(&self, angle: f32) -> bool {
        (self.min_angle..=self.max_angle).contains(&angle)
    }

    pub fn violation(&self, angle: f32) -> f32 {
        if angle < self.min_angle {
            self.min_angle - angle
        } else if angle > self.max_angle {
            angle - self.max_angle
        } else {
            0.0
        }
    }

    pub fn correction_torque(&self, angle: f32, angular_velocity: f32) -> f32 {
        let v = self.violation(angle);
        if v == 0.0 {
            return 0.0;
        }
        let sign = if angle < self.min_angle { 1.0 } else { -1.0 };
        sign * v * self.stiffness - angular_velocity * self.damping
    }

    pub fn range(&self) -> f32 {
        self.max_angle - self.min_angle
    }

    pub fn midpoint(&self) -> f32 {
        (self.min_angle + self.max_angle) * 0.5
    }

    pub fn normalized_position(&self, angle: f32) -> f32 {
        let r = self.range();
        if r <= 0.0 {
            return 0.5;
        }
        (angle - self.min_angle) / r
    }

    pub fn with_stiffness(mut self, s: f32) -> Self {
        self.stiffness = s;
        self
    }

    pub fn with_damping(mut self, d: f32) -> Self {
        self.damping = d;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clamp_within() {
        let limit = AngularLimit::new(-1.0, 1.0);
        assert!((limit.clamp(0.5) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_clamp_below() {
        let limit = AngularLimit::new(-1.0, 1.0);
        assert!((limit.clamp(-2.0) - (-1.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_clamp_above() {
        let limit = AngularLimit::new(-1.0, 1.0);
        assert!((limit.clamp(2.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_is_within() {
        let limit = AngularLimit::new(-1.0, 1.0);
        assert!(limit.is_within(0.0));
        assert!(!limit.is_within(1.5));
    }

    #[test]
    fn test_violation_none() {
        let limit = AngularLimit::new(-1.0, 1.0);
        assert!((limit.violation(0.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_violation_below() {
        let limit = AngularLimit::new(-1.0, 1.0);
        assert!((limit.violation(-1.5) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_symmetric() {
        let limit = AngularLimit::symmetric(PI / 4.0);
        assert!((limit.range() - PI / 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_range() {
        let limit = AngularLimit::new(-0.5, 1.5);
        assert!((limit.range() - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_midpoint() {
        let limit = AngularLimit::new(-1.0, 1.0);
        assert!((limit.midpoint()).abs() < f32::EPSILON);
    }

    #[test]
    fn test_correction_torque_no_violation() {
        let limit = AngularLimit::new(-1.0, 1.0);
        assert!((limit.correction_torque(0.0, 0.0)).abs() < f32::EPSILON);
    }
}
