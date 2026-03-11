// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// A hinge joint with angular limits, motor, and spring.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct HingeLimit {
    anchor: [f32; 3],
    axis: [f32; 3],
    angle: f32,
    angular_velocity: f32,
    min_angle: f32,
    max_angle: f32,
    motor_speed: f32,
    motor_max_torque: f32,
    motor_enabled: bool,
    spring_stiffness: f32,
    spring_damping: f32,
}

#[allow(dead_code)]
impl HingeLimit {
    pub fn new(anchor: [f32; 3], axis: [f32; 3]) -> Self {
        Self {
            anchor,
            axis,
            angle: 0.0,
            angular_velocity: 0.0,
            min_angle: -PI,
            max_angle: PI,
            motor_speed: 0.0,
            motor_max_torque: 100.0,
            motor_enabled: false,
            spring_stiffness: 0.0,
            spring_damping: 0.0,
        }
    }

    pub fn with_limits(mut self, min: f32, max: f32) -> Self {
        self.min_angle = min.max(-PI);
        self.max_angle = max.min(PI);
        self
    }

    pub fn with_motor(mut self, speed: f32, max_torque: f32) -> Self {
        self.motor_speed = speed;
        self.motor_max_torque = max_torque.max(0.0);
        self.motor_enabled = true;
        self
    }

    pub fn with_spring(mut self, stiffness: f32, damping: f32) -> Self {
        self.spring_stiffness = stiffness.max(0.0);
        self.spring_damping = damping.max(0.0);
        self
    }

    pub fn set_angle(&mut self, a: f32) {
        self.angle = a.clamp(self.min_angle, self.max_angle);
    }

    pub fn set_angular_velocity(&mut self, v: f32) {
        self.angular_velocity = v;
    }

    pub fn angle(&self) -> f32 {
        self.angle
    }

    pub fn angular_velocity(&self) -> f32 {
        self.angular_velocity
    }

    pub fn is_at_limit(&self) -> bool {
        (self.angle - self.min_angle).abs() < 1e-5 || (self.angle - self.max_angle).abs() < 1e-5
    }

    pub fn is_at_min(&self) -> bool {
        (self.angle - self.min_angle).abs() < 1e-5
    }

    pub fn is_at_max(&self) -> bool {
        (self.angle - self.max_angle).abs() < 1e-5
    }

    pub fn limit_torque(&self) -> f32 {
        if self.angle <= self.min_angle {
            (self.min_angle - self.angle).max(0.0)
        } else if self.angle >= self.max_angle {
            (self.max_angle - self.angle).min(0.0)
        } else {
            0.0
        }
    }

    pub fn motor_torque(&self) -> f32 {
        if !self.motor_enabled {
            return 0.0;
        }
        let error = self.motor_speed - self.angular_velocity;
        error.clamp(-self.motor_max_torque, self.motor_max_torque)
    }

    pub fn spring_torque(&self) -> f32 {
        -self.spring_stiffness * self.angle - self.spring_damping * self.angular_velocity
    }

    pub fn step(&mut self, dt: f32, inertia: f32) {
        if inertia <= 0.0 || dt <= 0.0 {
            return;
        }
        let total = self.spring_torque() + self.motor_torque();
        self.angular_velocity += (total / inertia) * dt;
        self.angle += self.angular_velocity * dt;
        self.angle = self.angle.clamp(self.min_angle, self.max_angle);
    }

    pub fn range(&self) -> f32 {
        self.max_angle - self.min_angle
    }

    pub fn normalized_position(&self) -> f32 {
        let r = self.range();
        if r.abs() < f32::EPSILON {
            return 0.0;
        }
        (self.angle - self.min_angle) / r
    }

    pub fn anchor(&self) -> [f32; 3] {
        self.anchor
    }

    pub fn axis(&self) -> [f32; 3] {
        self.axis
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let h = HingeLimit::new([0.0; 3], [0.0, 1.0, 0.0]);
        assert!((h.angle() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_limits() {
        let mut h = HingeLimit::new([0.0; 3], [0.0, 1.0, 0.0]).with_limits(-0.5, 0.5);
        h.set_angle(1.0);
        assert!((h.angle() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_is_at_limit() {
        let mut h = HingeLimit::new([0.0; 3], [0.0, 1.0, 0.0]).with_limits(-1.0, 1.0);
        h.set_angle(1.0);
        assert!(h.is_at_max());
    }

    #[test]
    fn test_motor_torque() {
        let h = HingeLimit::new([0.0; 3], [0.0, 1.0, 0.0]).with_motor(5.0, 100.0);
        assert!(h.motor_torque() > 0.0);
    }

    #[test]
    fn test_motor_disabled() {
        let h = HingeLimit::new([0.0; 3], [0.0, 1.0, 0.0]);
        assert!((h.motor_torque() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_spring_torque() {
        let mut h = HingeLimit::new([0.0; 3], [0.0, 1.0, 0.0]).with_spring(100.0, 0.0);
        h.set_angle(0.5);
        assert!(h.spring_torque() < 0.0);
    }

    #[test]
    fn test_step() {
        let mut h = HingeLimit::new([0.0; 3], [0.0, 1.0, 0.0])
            .with_spring(100.0, 10.0)
            .with_limits(-PI, PI);
        h.set_angle(1.0);
        for _ in 0..1000 {
            h.step(0.01, 1.0);
        }
        assert!(h.angle().abs() < 0.2);
    }

    #[test]
    fn test_range() {
        let h = HingeLimit::new([0.0; 3], [0.0, 1.0, 0.0]).with_limits(-1.0, 1.0);
        assert!((h.range() - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_normalized_position() {
        let mut h = HingeLimit::new([0.0; 3], [0.0, 1.0, 0.0]).with_limits(0.0, 1.0);
        h.set_angle(0.5);
        assert!((h.normalized_position() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_anchor_axis() {
        let h = HingeLimit::new([1.0, 2.0, 3.0], [0.0, 0.0, 1.0]);
        assert_eq!(h.anchor(), [1.0, 2.0, 3.0]);
        assert_eq!(h.axis(), [0.0, 0.0, 1.0]);
    }
}
