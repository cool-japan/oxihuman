// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// Simple planar pendulum (theta measured from downward vertical).
#[allow(dead_code)]
pub struct PendulumBody {
    pub length: f32,
    pub mass: f32,
    pub theta: f32,
    pub omega: f32,
    pub damping: f32,
}

#[allow(dead_code)]
impl PendulumBody {
    pub fn new(length: f32, mass: f32, theta0: f32, damping: f32) -> Self {
        Self {
            length,
            mass,
            theta: theta0,
            omega: 0.0,
            damping,
        }
    }
    pub fn step(&mut self, dt: f32, g: f32) {
        let alpha = -(g / self.length.max(1e-8)) * self.theta.sin() - self.damping * self.omega;
        self.omega += alpha * dt;
        self.theta += self.omega * dt;
    }
    pub fn bob_position(&self) -> [f32; 2] {
        [
            self.length * self.theta.sin(),
            -self.length * self.theta.cos(),
        ]
    }
    pub fn kinetic_energy(&self) -> f32 {
        let v = self.length * self.omega;
        0.5 * self.mass * v * v
    }
    pub fn potential_energy(&self, g: f32) -> f32 {
        self.mass * g * self.length * (1.0 - self.theta.cos())
    }
    pub fn total_energy(&self, g: f32) -> f32 {
        self.kinetic_energy() + self.potential_energy(g)
    }
    pub fn period_approx(&self, g: f32) -> f32 {
        2.0 * PI * (self.length / g.max(1e-8)).sqrt()
    }
    pub fn is_at_rest(&self, tol: f32) -> bool {
        self.omega.abs() < tol && self.theta.abs() < tol
    }
    pub fn frequency_hz(&self, g: f32) -> f32 {
        1.0 / self.period_approx(g).max(1e-10)
    }
}

#[allow(dead_code)]
pub fn new_pendulum_body(length: f32, mass: f32, theta0: f32, damping: f32) -> PendulumBody {
    PendulumBody::new(length, mass, theta0, damping)
}
#[allow(dead_code)]
pub fn pb_step(p: &mut PendulumBody, dt: f32, g: f32) {
    p.step(dt, g);
}
#[allow(dead_code)]
pub fn pb_bob_pos(p: &PendulumBody) -> [f32; 2] {
    p.bob_position()
}
#[allow(dead_code)]
pub fn pb_kinetic_energy(p: &PendulumBody) -> f32 {
    p.kinetic_energy()
}
#[allow(dead_code)]
pub fn pb_potential_energy(p: &PendulumBody, g: f32) -> f32 {
    p.potential_energy(g)
}
#[allow(dead_code)]
pub fn pb_total_energy(p: &PendulumBody, g: f32) -> f32 {
    p.total_energy(g)
}
#[allow(dead_code)]
pub fn pb_period(p: &PendulumBody, g: f32) -> f32 {
    p.period_approx(g)
}
#[allow(dead_code)]
pub fn pb_is_at_rest(p: &PendulumBody, tol: f32) -> bool {
    p.is_at_rest(tol)
}
#[allow(dead_code)]
pub fn pb_frequency(p: &PendulumBody, g: f32) -> f32 {
    p.frequency_hz(g)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_rest_position() {
        let p = new_pendulum_body(1.0, 1.0, 0.0, 0.1);
        let pos = pb_bob_pos(&p);
        assert!((pos[0]).abs() < 1e-5);
        assert!((pos[1] + 1.0).abs() < 1e-5);
    }
    #[test]
    fn test_kinetic_energy_zero_at_rest() {
        let p = new_pendulum_body(1.0, 1.0, 0.0, 0.0);
        assert_eq!(pb_kinetic_energy(&p), 0.0);
    }
    #[test]
    fn test_potential_energy_zero_at_bottom() {
        let p = new_pendulum_body(1.0, 1.0, 0.0, 0.0);
        assert!((pb_potential_energy(&p, 9.81)).abs() < 1e-5);
    }
    #[test]
    fn test_step_oscillates() {
        let mut p = new_pendulum_body(1.0, 1.0, 0.3, 0.0);
        let theta0 = p.theta;
        for _ in 0..10 {
            pb_step(&mut p, 0.01, 9.81);
        }
        assert_ne!(p.theta, theta0);
    }
    #[test]
    fn test_period_positive() {
        let p = new_pendulum_body(1.0, 1.0, 0.1, 0.0);
        let period = pb_period(&p, 9.81);
        assert!(period > 0.0);
        assert!((period - 2.006).abs() < 0.01);
    }
    #[test]
    fn test_damping_reduces_energy() {
        let mut p = new_pendulum_body(1.0, 1.0, 0.5, 1.0);
        let e0 = pb_total_energy(&p, 9.81);
        for _ in 0..100 {
            pb_step(&mut p, 0.01, 9.81);
        }
        let e1 = pb_total_energy(&p, 9.81);
        assert!(e1 < e0);
    }
    #[test]
    fn test_is_at_rest() {
        let p = new_pendulum_body(1.0, 1.0, 0.0, 0.0);
        assert!(pb_is_at_rest(&p, 1e-5));
    }
    #[test]
    fn test_frequency_positive() {
        let p = new_pendulum_body(0.5, 1.0, 0.0, 0.0);
        let f = pb_frequency(&p, 9.81);
        assert!(f > 0.0);
    }
    #[test]
    fn test_longer_pendulum_slower() {
        let p1 = new_pendulum_body(0.5, 1.0, 0.0, 0.0);
        let p2 = new_pendulum_body(2.0, 1.0, 0.0, 0.0);
        assert!(pb_period(&p1, 9.81) < pb_period(&p2, 9.81));
    }
    #[test]
    fn test_total_energy_conserved_no_damping() {
        let mut p = new_pendulum_body(1.0, 1.0, 0.5, 0.0);
        let e0 = pb_total_energy(&p, 9.81);
        for _ in 0..100 {
            pb_step(&mut p, 0.001, 9.81);
        }
        let e1 = pb_total_energy(&p, 9.81);
        assert!((e1 - e0).abs() / e0.abs().max(1e-5) < 0.05);
    }
}
