// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Simple harmonic oscillator body for vibration simulation.

use std::f32::consts::PI;

/// A 1-D simple harmonic oscillator (mass-spring-damper).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VibrationBody {
    pub mass: f32,
    pub stiffness: f32,
    pub damping: f32,
    pub displacement: f32,
    pub velocity: f32,
    pub equilibrium: f32,
    pub time: f32,
    pub steps: u64,
}

#[allow(dead_code)]
impl VibrationBody {
    pub fn new(mass: f32, stiffness: f32, damping: f32) -> Self {
        Self {
            mass: mass.max(1e-6),
            stiffness: stiffness.max(0.0),
            damping: damping.max(0.0),
            displacement: 0.0,
            velocity: 0.0,
            equilibrium: 0.0,
            time: 0.0,
            steps: 0,
        }
    }

    /// Natural angular frequency ω₀ = √(k/m).
    pub fn omega0(&self) -> f32 {
        (self.stiffness / self.mass).sqrt()
    }

    /// Damping ratio ζ = c / (2√(km)).
    pub fn damping_ratio(&self) -> f32 {
        let critical = 2.0 * (self.stiffness * self.mass).sqrt();
        if critical > 0.0 {
            self.damping / critical
        } else {
            0.0
        }
    }

    /// Natural period T = 2π / ω₀.
    pub fn period(&self) -> f32 {
        let w = self.omega0();
        if w > 0.0 {
            2.0 * PI / w
        } else {
            f32::INFINITY
        }
    }

    /// Apply an external impulse (changes velocity).
    pub fn apply_impulse(&mut self, impulse: f32) {
        self.velocity += impulse / self.mass;
    }

    /// Apply an external force over dt using Euler integration.
    pub fn step(&mut self, dt: f32, external_force: f32) {
        let rel = self.displacement - self.equilibrium;
        let spring_force = -self.stiffness * rel;
        let damp_force = -self.damping * self.velocity;
        let acc = (spring_force + damp_force + external_force) / self.mass;
        self.velocity += acc * dt;
        self.displacement += self.velocity * dt;
        self.time += dt;
        self.steps += 1;
    }

    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.mass * self.velocity * self.velocity
    }

    pub fn potential_energy(&self) -> f32 {
        let rel = self.displacement - self.equilibrium;
        0.5 * self.stiffness * rel * rel
    }

    pub fn total_energy(&self) -> f32 {
        self.kinetic_energy() + self.potential_energy()
    }

    pub fn is_at_rest(&self, tol: f32) -> bool {
        self.velocity.abs() < tol && (self.displacement - self.equilibrium).abs() < tol
    }

    pub fn reset(&mut self) {
        self.displacement = self.equilibrium;
        self.velocity = 0.0;
        self.time = 0.0;
        self.steps = 0;
    }
}

impl Default for VibrationBody {
    fn default() -> Self {
        Self::new(1.0, 10.0, 0.5)
    }
}

pub fn new_vibration_body(mass: f32, stiffness: f32, damping: f32) -> VibrationBody {
    VibrationBody::new(mass, stiffness, damping)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn omega0_positive() {
        let b = new_vibration_body(1.0, 4.0, 0.0);
        assert!((b.omega0() - 2.0).abs() < 1e-5);
    }

    #[test]
    fn period_nonzero() {
        let b = new_vibration_body(1.0, 1.0, 0.0);
        assert!(b.period() > 0.0);
        assert!(b.period().is_finite());
    }

    #[test]
    fn impulse_sets_velocity() {
        let mut b = new_vibration_body(2.0, 10.0, 0.0);
        b.apply_impulse(4.0);
        assert!((b.velocity - 2.0).abs() < 1e-5);
    }

    #[test]
    fn spring_restores_to_equilibrium() {
        let mut b = new_vibration_body(1.0, 100.0, 10.0);
        b.displacement = 1.0;
        for _ in 0..1000 {
            b.step(0.01, 0.0);
        }
        assert!((b.displacement - b.equilibrium).abs() < 0.05);
    }

    #[test]
    fn potential_energy_nonzero_when_displaced() {
        let mut b = new_vibration_body(1.0, 10.0, 0.0);
        b.displacement = 1.0;
        assert!(b.potential_energy() > 0.0);
    }

    #[test]
    fn kinetic_energy_nonzero_when_moving() {
        let mut b = new_vibration_body(1.0, 0.0, 0.0);
        b.velocity = 2.0;
        assert!(b.kinetic_energy() > 0.0);
    }

    #[test]
    fn steps_increment() {
        let mut b = new_vibration_body(1.0, 1.0, 0.0);
        b.step(0.01, 0.0);
        b.step(0.01, 0.0);
        assert_eq!(b.steps, 2);
    }

    #[test]
    fn reset_to_rest() {
        let mut b = new_vibration_body(1.0, 10.0, 0.1);
        b.apply_impulse(5.0);
        b.step(0.1, 2.0);
        b.reset();
        assert!(b.is_at_rest(1e-5));
    }

    #[test]
    fn damping_ratio_underdamped() {
        let b = new_vibration_body(1.0, 10.0, 0.1);
        assert!(b.damping_ratio() < 1.0);
    }

    #[test]
    fn total_energy_non_negative() {
        let mut b = new_vibration_body(1.0, 5.0, 0.0);
        b.displacement = 0.5;
        b.velocity = 1.0;
        assert!(b.total_energy() >= 0.0);
    }
}
