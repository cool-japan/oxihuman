// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// A rotational spring that applies torque proportional to angular displacement.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct AngularSpring {
    rest_angle: f32,
    current_angle: f32,
    stiffness: f32,
    damping: f32,
    angular_velocity: f32,
    min_angle: f32,
    max_angle: f32,
}

#[allow(dead_code)]
impl AngularSpring {
    pub fn new(rest_angle: f32, stiffness: f32, damping: f32) -> Self {
        Self {
            rest_angle,
            current_angle: rest_angle,
            stiffness: stiffness.max(0.0),
            damping: damping.max(0.0),
            angular_velocity: 0.0,
            min_angle: -PI,
            max_angle: PI,
        }
    }

    pub fn with_limits(mut self, min: f32, max: f32) -> Self {
        self.min_angle = min;
        self.max_angle = max;
        self
    }

    pub fn torque(&self) -> f32 {
        let displacement = self.current_angle - self.rest_angle;
        -self.stiffness * displacement - self.damping * self.angular_velocity
    }

    pub fn step(&mut self, dt: f32, inertia: f32) {
        if inertia <= 0.0 || dt <= 0.0 {
            return;
        }
        let t = self.torque();
        let angular_accel = t / inertia;
        self.angular_velocity += angular_accel * dt;
        self.current_angle += self.angular_velocity * dt;
        self.current_angle = self.current_angle.clamp(self.min_angle, self.max_angle);
    }

    pub fn current_angle(&self) -> f32 {
        self.current_angle
    }

    pub fn set_angle(&mut self, angle: f32) {
        self.current_angle = angle.clamp(self.min_angle, self.max_angle);
    }

    pub fn angular_velocity(&self) -> f32 {
        self.angular_velocity
    }

    pub fn set_angular_velocity(&mut self, vel: f32) {
        self.angular_velocity = vel;
    }

    pub fn rest_angle(&self) -> f32 {
        self.rest_angle
    }

    pub fn stiffness(&self) -> f32 {
        self.stiffness
    }

    pub fn damping(&self) -> f32 {
        self.damping
    }

    pub fn displacement(&self) -> f32 {
        self.current_angle - self.rest_angle
    }

    pub fn energy(&self) -> f32 {
        let d = self.displacement();
        0.5 * self.stiffness * d * d
    }

    pub fn kinetic_energy(&self, inertia: f32) -> f32 {
        0.5 * inertia * self.angular_velocity * self.angular_velocity
    }

    pub fn is_at_rest(&self, threshold: f32) -> bool {
        self.displacement().abs() < threshold && self.angular_velocity.abs() < threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let s = AngularSpring::new(0.0, 100.0, 10.0);
        assert!((s.current_angle() - 0.0).abs() < f32::EPSILON);
        assert!((s.stiffness() - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_torque_at_rest() {
        let s = AngularSpring::new(0.0, 100.0, 10.0);
        assert!((s.torque() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_torque_displaced() {
        let mut s = AngularSpring::new(0.0, 100.0, 0.0);
        s.set_angle(0.5);
        assert!((s.torque() + 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_step_returns_to_rest() {
        let mut s = AngularSpring::new(0.0, 100.0, 20.0);
        s.set_angle(1.0);
        for _ in 0..1000 {
            s.step(0.01, 1.0);
        }
        assert!(s.displacement().abs() < 0.1);
    }

    #[test]
    fn test_limits() {
        let mut s = AngularSpring::new(0.0, 100.0, 10.0).with_limits(-0.5, 0.5);
        s.set_angle(2.0);
        assert!((s.current_angle() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_energy() {
        let mut s = AngularSpring::new(0.0, 200.0, 0.0);
        s.set_angle(1.0);
        assert!((s.energy() - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_kinetic_energy() {
        let mut s = AngularSpring::new(0.0, 100.0, 0.0);
        s.set_angular_velocity(2.0);
        assert!((s.kinetic_energy(1.0) - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_displacement() {
        let mut s = AngularSpring::new(0.5, 100.0, 10.0);
        s.set_angle(1.0);
        assert!((s.displacement() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_is_at_rest() {
        let s = AngularSpring::new(0.0, 100.0, 10.0);
        assert!(s.is_at_rest(0.01));
    }

    #[test]
    fn test_with_limits() {
        let s = AngularSpring::new(0.0, 100.0, 10.0).with_limits(-1.0, 1.0);
        assert!((s.min_angle + 1.0).abs() < f32::EPSILON);
        assert!((s.max_angle - 1.0).abs() < f32::EPSILON);
    }
}
