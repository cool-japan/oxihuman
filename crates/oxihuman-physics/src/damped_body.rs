// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Simple damped rigid body with linear and angular damping.

use std::f32::consts::E;

/// Damped rigid body with linear and angular velocity damping.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DampedBody {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub angular_velocity: [f32; 3],
    pub mass: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub force_accum: [f32; 3],
}

#[allow(dead_code)]
impl DampedBody {
    pub fn new(mass: f32, linear_damping: f32, angular_damping: f32) -> Self {
        Self {
            position: [0.0; 3],
            velocity: [0.0; 3],
            angular_velocity: [0.0; 3],
            mass: mass.max(1e-9),
            linear_damping: linear_damping.clamp(0.0, 1.0),
            angular_damping: angular_damping.clamp(0.0, 1.0),
            force_accum: [0.0; 3],
        }
    }

    pub fn apply_force(&mut self, force: [f32; 3]) {
        self.force_accum[0] += force[0];
        self.force_accum[1] += force[1];
        self.force_accum[2] += force[2];
    }

    pub fn integrate(&mut self, dt: f32) {
        let inv_mass = 1.0 / self.mass;

        // Linear integration
        self.velocity[0] += self.force_accum[0] * inv_mass * dt;
        self.velocity[1] += self.force_accum[1] * inv_mass * dt;
        self.velocity[2] += self.force_accum[2] * inv_mass * dt;

        // Damping (exponential decay approximation)
        let lin_damp = damping_factor(self.linear_damping, dt);
        let ang_damp = damping_factor(self.angular_damping, dt);
        self.velocity[0] *= lin_damp;
        self.velocity[1] *= lin_damp;
        self.velocity[2] *= lin_damp;
        self.angular_velocity[0] *= ang_damp;
        self.angular_velocity[1] *= ang_damp;
        self.angular_velocity[2] *= ang_damp;

        // Position update
        self.position[0] += self.velocity[0] * dt;
        self.position[1] += self.velocity[1] * dt;
        self.position[2] += self.velocity[2] * dt;

        self.force_accum = [0.0; 3];
    }

    pub fn speed(&self) -> f32 {
        let [vx, vy, vz] = self.velocity;
        (vx * vx + vy * vy + vz * vz).sqrt()
    }

    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.mass * self.speed().powi(2)
    }

    pub fn is_nearly_at_rest(&self, threshold: f32) -> bool {
        self.speed() < threshold
    }

    pub fn set_velocity(&mut self, v: [f32; 3]) {
        self.velocity = v;
    }

    pub fn set_position(&mut self, p: [f32; 3]) {
        self.position = p;
    }
}

/// Compute per-step damping factor using exponential model.
#[allow(dead_code)]
pub fn damping_factor(damping: f32, dt: f32) -> f32 {
    E.powf(-damping * dt)
}

/// Critical damping coefficient for given mass and stiffness.
#[allow(dead_code)]
pub fn critical_damping(mass: f32, stiffness: f32) -> f32 {
    2.0 * (mass * stiffness).sqrt()
}

/// Overdamped body: settling time (approx) in seconds.
#[allow(dead_code)]
pub fn overdamped_settling_time(damping: f32, mass: f32) -> Option<f32> {
    if damping < 1e-9 || mass < 1e-9 {
        return None;
    }
    Some(mass / damping * 5.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_body_at_rest() {
        let b = DampedBody::new(1.0, 0.1, 0.1);
        assert_eq!(b.speed(), 0.0);
    }

    #[test]
    fn apply_force_moves_body() {
        let mut b = DampedBody::new(1.0, 0.0, 0.0);
        b.apply_force([10.0, 0.0, 0.0]);
        b.integrate(0.1);
        assert!(b.velocity[0] > 0.0);
        assert!(b.position[0] > 0.0);
    }

    #[test]
    fn damping_reduces_speed() {
        let mut b = DampedBody::new(1.0, 0.5, 0.0);
        b.set_velocity([10.0, 0.0, 0.0]);
        let speed_before = b.speed();
        b.integrate(0.1);
        assert!(b.speed() < speed_before);
    }

    #[test]
    fn kinetic_energy_formula() {
        let mut b = DampedBody::new(2.0, 0.0, 0.0);
        b.set_velocity([3.0, 0.0, 0.0]);
        // KE = 0.5 * 2.0 * 9.0 = 9.0
        assert!((b.kinetic_energy() - 9.0).abs() < 1e-4);
    }

    #[test]
    fn damping_factor_full_damp_approaches_zero() {
        let f = damping_factor(10.0, 1.0);
        assert!(f < 0.01);
    }

    #[test]
    fn damping_factor_zero_damp_is_one() {
        let f = damping_factor(0.0, 0.1);
        assert!((f - 1.0).abs() < 1e-6);
    }

    #[test]
    fn critical_damping_formula() {
        // mass=1, stiffness=4 → c = 2*sqrt(4) = 4
        let c = critical_damping(1.0, 4.0);
        assert!((c - 4.0).abs() < 1e-5);
    }

    #[test]
    fn is_nearly_at_rest() {
        let mut b = DampedBody::new(1.0, 0.0, 0.0);
        b.set_velocity([0.001, 0.0, 0.0]);
        assert!(b.is_nearly_at_rest(0.01));
        b.set_velocity([1.0, 0.0, 0.0]);
        assert!(!b.is_nearly_at_rest(0.01));
    }

    #[test]
    fn force_accumulator_cleared_after_integrate() {
        let mut b = DampedBody::new(1.0, 0.0, 0.0);
        b.apply_force([5.0, 0.0, 0.0]);
        b.integrate(0.1);
        assert_eq!(b.force_accum, [0.0; 3]);
    }

    #[test]
    fn overdamped_settling_time_none_zero_damping() {
        assert!(overdamped_settling_time(0.0, 1.0).is_none());
    }
}
