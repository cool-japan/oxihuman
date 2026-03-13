// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Body with rotational-only degrees of freedom.

use std::f32::consts::PI;

/// A purely rotational rigid body.
#[derive(Debug, Clone)]
pub struct RotationalBody {
    pub angle: f32,
    pub angular_velocity: f32,
    pub moment_of_inertia: f32,
    pub drag: f32,
    pub accumulated_torque: f32,
    pub enabled: bool,
}

#[allow(dead_code)]
impl RotationalBody {
    pub fn new(moment_of_inertia: f32) -> Self {
        RotationalBody {
            angle: 0.0,
            angular_velocity: 0.0,
            moment_of_inertia: moment_of_inertia.max(1e-9),
            drag: 0.0,
            accumulated_torque: 0.0,
            enabled: true,
        }
    }

    pub fn apply_torque(&mut self, torque: f32) {
        self.accumulated_torque += torque;
    }

    pub fn step(&mut self, dt: f32) {
        if !self.enabled {
            self.accumulated_torque = 0.0;
            return;
        }
        let drag_torque = -self.angular_velocity * self.drag;
        let alpha = (self.accumulated_torque + drag_torque) / self.moment_of_inertia;
        self.angular_velocity += alpha * dt;
        self.angle += self.angular_velocity * dt;
        self.accumulated_torque = 0.0;
    }

    pub fn rotational_kinetic_energy(&self) -> f32 {
        0.5 * self.moment_of_inertia * self.angular_velocity.powi(2)
    }

    pub fn angular_momentum(&self) -> f32 {
        self.moment_of_inertia * self.angular_velocity
    }

    pub fn angle_deg(&self) -> f32 {
        self.angle.to_degrees()
    }

    pub fn revolutions(&self) -> f32 {
        self.angle / (2.0 * PI)
    }

    pub fn period(&self) -> Option<f32> {
        if self.angular_velocity.abs() < 1e-9 {
            None
        } else {
            Some(2.0 * PI / self.angular_velocity.abs())
        }
    }

    pub fn set_drag(&mut self, drag: f32) {
        self.drag = drag.max(0.0);
    }

    pub fn stop(&mut self) {
        self.angular_velocity = 0.0;
        self.accumulated_torque = 0.0;
    }

    pub fn normalize_angle(&mut self) {
        self.angle = self.angle.rem_euclid(2.0 * PI);
    }
}

pub fn new_rotational_body(moment_of_inertia: f32) -> RotationalBody {
    RotationalBody::new(moment_of_inertia)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn torque_produces_acceleration() {
        let mut b = new_rotational_body(1.0);
        b.apply_torque(10.0);
        b.step(1.0);
        assert!(b.angular_velocity > 0.0);
    }

    #[test]
    fn no_torque_constant_velocity() {
        let mut b = new_rotational_body(1.0);
        b.angular_velocity = 5.0;
        b.step(1.0);
        assert!((b.angular_velocity - 5.0).abs() < 1e-6);
    }

    #[test]
    fn drag_reduces_speed() {
        let mut b = new_rotational_body(1.0);
        b.angular_velocity = 10.0;
        b.set_drag(2.0);
        b.step(0.1);
        assert!(b.angular_velocity < 10.0);
    }

    #[test]
    fn kinetic_energy() {
        let mut b = new_rotational_body(2.0);
        b.angular_velocity = 3.0;
        assert!((b.rotational_kinetic_energy() - 9.0).abs() < 1e-5);
    }

    #[test]
    fn angular_momentum() {
        let mut b = new_rotational_body(4.0);
        b.angular_velocity = 5.0;
        assert!((b.angular_momentum() - 20.0).abs() < 1e-6);
    }

    #[test]
    fn revolutions() {
        let mut b = new_rotational_body(1.0);
        b.angle = 2.0 * PI;
        assert!((b.revolutions() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn period_when_spinning() {
        let mut b = new_rotational_body(1.0);
        b.angular_velocity = 2.0 * PI;
        let p = b.period().expect("should succeed");
        assert!((p - 1.0).abs() < 1e-5);
    }

    #[test]
    fn disabled_no_step() {
        let mut b = new_rotational_body(1.0);
        b.angular_velocity = 5.0;
        b.enabled = false;
        b.apply_torque(100.0);
        b.step(1.0);
        assert!((b.angular_velocity - 5.0).abs() < 1e-6);
    }

    #[test]
    fn normalize_angle() {
        let mut b = new_rotational_body(1.0);
        b.angle = 3.0 * PI;
        b.normalize_angle();
        assert!((0.0..=2.0 * PI).contains(&b.angle));
    }
}
