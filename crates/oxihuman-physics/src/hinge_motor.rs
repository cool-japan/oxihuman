// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A hinge motor that drives a rotational joint toward a target angle/velocity.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HingeMotor {
    pub target_angle: f32,
    pub current_angle: f32,
    pub angular_velocity: f32,
    pub max_torque: f32,
    pub stiffness: f32,
    pub damping: f32,
    pub min_angle: f32,
    pub max_angle: f32,
}

#[allow(dead_code)]
impl HingeMotor {
    pub fn new(max_torque: f32, stiffness: f32, damping: f32) -> Self {
        Self {
            target_angle: 0.0,
            current_angle: 0.0,
            angular_velocity: 0.0,
            max_torque,
            stiffness,
            damping,
            min_angle: -PI,
            max_angle: PI,
        }
    }

    pub fn set_limits(&mut self, min: f32, max: f32) {
        self.min_angle = min;
        self.max_angle = max;
    }

    pub fn set_target(&mut self, angle: f32) {
        self.target_angle = angle.clamp(self.min_angle, self.max_angle);
    }

    pub fn compute_torque(&self) -> f32 {
        let error = self.target_angle - self.current_angle;
        let torque = self.stiffness * error - self.damping * self.angular_velocity;
        torque.clamp(-self.max_torque, self.max_torque)
    }

    pub fn step(&mut self, inertia: f32, dt: f32) {
        let torque = self.compute_torque();
        if inertia > 1e-12 {
            let alpha = torque / inertia;
            self.angular_velocity += alpha * dt;
        }
        self.current_angle += self.angular_velocity * dt;
        self.current_angle = self.current_angle.clamp(self.min_angle, self.max_angle);
        // Clamp at limits
        if self.current_angle <= self.min_angle || self.current_angle >= self.max_angle {
            self.angular_velocity = 0.0;
        }
    }

    pub fn angle_error(&self) -> f32 {
        (self.target_angle - self.current_angle).abs()
    }

    pub fn is_at_target(&self, tolerance: f32) -> bool {
        self.angle_error() < tolerance
    }

    pub fn kinetic_energy(&self, inertia: f32) -> f32 {
        0.5 * inertia * self.angular_velocity * self.angular_velocity
    }

    pub fn normalize_angle(angle: f32) -> f32 {
        let mut a = angle % (2.0 * PI);
        if a > PI { a -= 2.0 * PI; }
        if a < -PI { a += 2.0 * PI; }
        a
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = HingeMotor::new(10.0, 100.0, 5.0);
        assert!((m.current_angle).abs() < 1e-5);
        assert!((m.max_torque - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_set_target() {
        let mut m = HingeMotor::new(10.0, 100.0, 5.0);
        m.set_target(1.0);
        assert!((m.target_angle - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_compute_torque() {
        let mut m = HingeMotor::new(100.0, 10.0, 1.0);
        m.set_target(1.0);
        let t = m.compute_torque();
        assert!((t - 10.0).abs() < 1e-3);
    }

    #[test]
    fn test_torque_clamped() {
        let mut m = HingeMotor::new(5.0, 1000.0, 0.0);
        m.set_target(PI);
        let t = m.compute_torque();
        assert!((t - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_step_moves_toward_target() {
        let mut m = HingeMotor::new(100.0, 50.0, 10.0);
        m.set_target(1.0);
        for _ in 0..100 {
            m.step(1.0, 0.01);
        }
        assert!(m.angle_error() < 0.1);
    }

    #[test]
    fn test_is_at_target() {
        let m = HingeMotor::new(10.0, 100.0, 5.0);
        assert!(m.is_at_target(0.01));
    }

    #[test]
    fn test_limits() {
        let mut m = HingeMotor::new(10.0, 100.0, 5.0);
        m.set_limits(-0.5, 0.5);
        m.set_target(2.0);
        assert!((m.target_angle - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_kinetic_energy() {
        let mut m = HingeMotor::new(10.0, 100.0, 5.0);
        m.angular_velocity = 2.0;
        let e = m.kinetic_energy(3.0);
        assert!((e - 6.0).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_angle() {
        let a = HingeMotor::normalize_angle(3.0 * PI);
        assert!((a - PI).abs() < 0.01 || (a + PI).abs() < 0.01);
    }

    #[test]
    fn test_angle_error() {
        let mut m = HingeMotor::new(10.0, 100.0, 5.0);
        m.set_target(0.5);
        assert!((m.angle_error() - 0.5).abs() < 1e-5);
    }
}
