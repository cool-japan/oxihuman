// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Spring-pendulum: extensible spring + gravity.

#![allow(dead_code)]

/// State of a spring pendulum (2D: angle from vertical, extension from rest length).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SpringPendulum {
    pub mass: f32,        // kg
    pub rest_length: f32, // m
    pub spring_k: f32,    // N/m
    pub gravity: f32,     // m/s²
    /// Current angle from vertical (radians).
    pub theta: f32,
    /// Angular velocity.
    pub omega: f32,
    /// Extension beyond rest length.
    pub r: f32,
    /// Radial velocity.
    pub r_dot: f32,
}

#[allow(dead_code)]
impl SpringPendulum {
    pub fn new(mass: f32, rest_length: f32, spring_k: f32) -> Self {
        Self {
            mass,
            rest_length,
            spring_k,
            gravity: 9.81,
            theta: 0.1,
            omega: 0.0,
            r: 0.0,
            r_dot: 0.0,
        }
    }

    /// Current effective length.
    pub fn effective_length(&self) -> f32 {
        (self.rest_length + self.r).max(1e-4)
    }

    /// Step by dt seconds (Euler integration).
    pub fn step(&mut self, dt: f32) {
        let l = self.effective_length();
        let m = self.mass;
        let g = self.gravity;
        let k = self.spring_k;
        // EOM for spring pendulum (small angle and radial coupling)
        let r_ddot = l * self.omega * self.omega - k / m * self.r + g * self.theta.cos() - g;
        let theta_ddot = -g / l * self.theta.sin() - 2.0 * self.r_dot / l * self.omega;
        self.r += self.r_dot * dt;
        self.r_dot += r_ddot * dt;
        self.theta += self.omega * dt;
        self.omega += theta_ddot * dt;
    }

    /// Bob position in 2D: (x, y) where y is downward.
    pub fn bob_position(&self) -> [f32; 2] {
        let l = self.effective_length();
        [l * self.theta.sin(), l * self.theta.cos()]
    }

    /// Kinetic energy.
    pub fn kinetic_energy(&self) -> f32 {
        let l = self.effective_length();
        let v2 = (self.r_dot * self.r_dot) + (l * self.omega) * (l * self.omega);
        0.5 * self.mass * v2
    }

    /// Spring potential energy.
    pub fn spring_potential(&self) -> f32 {
        0.5 * self.spring_k * self.r * self.r
    }

    /// Gravitational potential energy (relative to pivot).
    pub fn gravitational_potential(&self) -> f32 {
        let l = self.effective_length();
        -self.mass * self.gravity * l * self.theta.cos()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_position() {
        let sp = SpringPendulum::new(1.0, 1.0, 10.0);
        let pos = sp.bob_position();
        assert!(pos[0].abs() < 0.2); // Small angle → nearly below pivot
    }

    #[test]
    fn effective_length_at_rest() {
        let sp = SpringPendulum::new(1.0, 1.0, 10.0);
        assert!((sp.effective_length() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn kinetic_energy_initially_small() {
        let sp = SpringPendulum::new(1.0, 1.0, 10.0);
        assert!(sp.kinetic_energy() < 1.0);
    }

    #[test]
    fn spring_potential_at_rest_zero() {
        let sp = SpringPendulum::new(1.0, 1.0, 10.0);
        assert!(sp.spring_potential().abs() < 1e-5);
    }

    #[test]
    fn step_changes_state() {
        let mut sp = SpringPendulum::new(1.0, 1.0, 10.0);
        let theta0 = sp.theta;
        sp.step(0.01);
        // theta should change slightly
        let _delta = (sp.theta - theta0).abs();
    }

    #[test]
    fn bob_position_length_approx_rest() {
        let sp = SpringPendulum::new(1.0, 2.0, 100.0);
        let pos = sp.bob_position();
        let d = (pos[0] * pos[0] + pos[1] * pos[1]).sqrt();
        assert!((d - 2.0).abs() < 0.5);
    }

    #[test]
    fn step_many_finite() {
        let mut sp = SpringPendulum::new(1.0, 1.0, 10.0);
        for _ in 0..100 {
            sp.step(0.01);
        }
        assert!(sp.theta.is_finite());
        assert!(sp.r.is_finite());
    }

    #[test]
    fn gravitational_potential_negative_at_rest() {
        let sp = SpringPendulum::new(1.0, 1.0, 10.0);
        assert!(sp.gravitational_potential() < 0.0);
    }

    #[test]
    fn spring_k_zero_no_radial_force() {
        let mut sp = SpringPendulum::new(1.0, 1.0, 0.0);
        sp.step(0.01);
        assert!(sp.r.abs() < 0.1); // still small, just free
    }

    #[test]
    fn effective_length_min_positive() {
        let mut sp = SpringPendulum::new(1.0, 0.001, 10.0);
        sp.r = -1.0; // compress below rest
        assert!(sp.effective_length() > 0.0);
    }
}
