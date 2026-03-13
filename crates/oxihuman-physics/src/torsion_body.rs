// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Body with torsional spring-damper.

use std::f32::consts::PI;

/// A body with a torsional spring about a single axis.
#[derive(Debug, Clone)]
pub struct TorsionBody {
    pub angle: f32,
    pub angular_velocity: f32,
    pub moment_of_inertia: f32,
    pub torsion_stiffness: f32,
    pub torsion_damping: f32,
    pub rest_angle: f32,
    pub accumulated_torque: f32,
    pub enabled: bool,
}

#[allow(dead_code)]
impl TorsionBody {
    pub fn new(moment_of_inertia: f32, torsion_stiffness: f32, torsion_damping: f32) -> Self {
        TorsionBody {
            angle: 0.0,
            angular_velocity: 0.0,
            moment_of_inertia: moment_of_inertia.max(1e-9),
            torsion_stiffness,
            torsion_damping,
            rest_angle: 0.0,
            accumulated_torque: 0.0,
            enabled: true,
        }
    }

    pub fn apply_torque(&mut self, t: f32) {
        self.accumulated_torque += t;
    }

    pub fn step(&mut self, dt: f32) {
        if !self.enabled {
            self.accumulated_torque = 0.0;
            return;
        }
        let spring_torque = -self.torsion_stiffness * (self.angle - self.rest_angle);
        let damp_torque = -self.torsion_damping * self.angular_velocity;
        let alpha =
            (self.accumulated_torque + spring_torque + damp_torque) / self.moment_of_inertia;
        self.angular_velocity += alpha * dt;
        self.angle += self.angular_velocity * dt;
        self.accumulated_torque = 0.0;
    }

    pub fn torsional_potential_energy(&self) -> f32 {
        0.5 * self.torsion_stiffness * (self.angle - self.rest_angle).powi(2)
    }

    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.moment_of_inertia * self.angular_velocity.powi(2)
    }

    pub fn total_energy(&self) -> f32 {
        self.torsional_potential_energy() + self.kinetic_energy()
    }

    pub fn is_at_rest(&self) -> bool {
        self.angular_velocity.abs() < 1e-6 && (self.angle - self.rest_angle).abs() < 1e-6
    }

    pub fn natural_frequency_hz(&self) -> f32 {
        (self.torsion_stiffness / self.moment_of_inertia).sqrt() / (2.0 * PI)
    }

    pub fn critical_damping(&self) -> f32 {
        2.0 * (self.torsion_stiffness * self.moment_of_inertia).sqrt()
    }

    pub fn angle_deg(&self) -> f32 {
        self.angle.to_degrees()
    }

    pub fn set_rest_angle(&mut self, a: f32) {
        self.rest_angle = a;
    }
}

pub fn new_torsion_body(moi: f32, k: f32, d: f32) -> TorsionBody {
    TorsionBody::new(moi, k, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spring_returns_to_rest() {
        let mut b = new_torsion_body(1.0, 100.0, 20.0);
        b.angle = 0.1;
        for _ in 0..1000 {
            b.step(0.01);
        }
        assert!((b.angle - b.rest_angle).abs() < 0.01);
    }

    #[test]
    fn potential_energy_at_displaced() {
        let mut b = new_torsion_body(1.0, 100.0, 0.0);
        b.angle = 1.0;
        assert!((b.torsional_potential_energy() - 50.0).abs() < 1e-5);
    }

    #[test]
    fn kinetic_energy() {
        let mut b = new_torsion_body(2.0, 10.0, 0.0);
        b.angular_velocity = 3.0;
        assert!((b.kinetic_energy() - 9.0).abs() < 1e-5);
    }

    #[test]
    fn natural_frequency() {
        let b = new_torsion_body(1.0, (2.0 * PI).powi(2), 0.0);
        assert!((b.natural_frequency_hz() - 1.0).abs() < 1e-4);
    }

    #[test]
    fn critical_damping() {
        let b = new_torsion_body(1.0, 100.0, 0.0);
        assert!((b.critical_damping() - 20.0).abs() < 1e-4);
    }

    #[test]
    fn disabled_no_step() {
        let mut b = new_torsion_body(1.0, 100.0, 0.0);
        b.angle = 1.0;
        b.enabled = false;
        b.step(1.0);
        assert_eq!(b.angle, 1.0);
    }

    #[test]
    fn apply_torque_affects_motion() {
        let mut b = new_torsion_body(1.0, 0.0, 0.0);
        b.apply_torque(10.0);
        b.step(1.0);
        assert!(b.angular_velocity > 0.0);
    }

    #[test]
    fn angle_deg_conversion() {
        let mut b = new_torsion_body(1.0, 0.0, 0.0);
        b.angle = PI;
        assert!((b.angle_deg() - 180.0).abs() < 1e-4);
    }

    #[test]
    fn at_rest_initially() {
        let b = new_torsion_body(1.0, 100.0, 0.0);
        assert!(b.is_at_rest());
    }
}
