// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Foam spring: a damped spring embedded in a foam/viscoelastic material.

use std::f32::consts::PI;

/// A viscoelastic (foam) spring element.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FoamSpring {
    pub id: u32,
    pub spring_k: f32,
    pub damping: f32,
    pub rest_len: f32,
    pub displacement: f32,
    pub velocity: f32,
    pub creep_rate: f32, // viscous creep rate (drift toward zero force)
    pub rest_len_drifted: f32,
}

#[allow(dead_code)]
impl FoamSpring {
    pub fn new(id: u32, spring_k: f32, damping: f32, rest_len: f32, creep_rate: f32) -> Self {
        Self {
            id,
            spring_k,
            damping,
            rest_len,
            displacement: 0.0,
            velocity: 0.0,
            creep_rate,
            rest_len_drifted: rest_len,
        }
    }

    /// Current spring force (elastic + damping).
    pub fn force(&self) -> f32 {
        -self.spring_k * self.displacement - self.damping * self.velocity
    }

    /// Elastic potential energy.
    pub fn elastic_energy(&self) -> f32 {
        0.5 * self.spring_k * self.displacement * self.displacement
    }

    /// Advance the spring by `dt` given an external displacement `ext_disp`.
    pub fn step(&mut self, ext_disp: f32, dt: f32) {
        // Update displacement and velocity
        let prev_vel = self.velocity;
        let accel = (-self.spring_k * self.displacement - self.damping * self.velocity) / 1.0;
        self.velocity += accel * dt;
        self.displacement = ext_disp;
        // Creep: rest length drifts toward current length
        self.rest_len_drifted += self.creep_rate * self.displacement * dt;
        let _ = prev_vel;
    }

    /// Natural frequency (rad/s).
    pub fn omega_n(&self) -> f32 {
        self.spring_k.sqrt()
    }

    /// Damping ratio ζ = c / (2 sqrt(k)).
    pub fn damping_ratio(&self) -> f32 {
        self.damping / (2.0 * self.spring_k.sqrt())
    }

    /// Returns true if the spring is overdamped.
    pub fn is_overdamped(&self) -> bool {
        self.damping_ratio() >= 1.0
    }
}

/// A network of foam springs connected in a 1-D chain.
#[allow(dead_code)]
pub struct FoamSpringNetwork {
    pub springs: Vec<FoamSpring>,
    pub positions: Vec<f32>,
    pub velocities: Vec<f32>,
    pub masses: Vec<f32>,
}

#[allow(dead_code)]
impl FoamSpringNetwork {
    pub fn new() -> Self {
        Self {
            springs: Vec::new(),
            positions: Vec::new(),
            velocities: Vec::new(),
            masses: Vec::new(),
        }
    }

    pub fn add_node(&mut self, pos: f32, mass: f32) -> usize {
        let idx = self.positions.len();
        self.positions.push(pos);
        self.velocities.push(0.0);
        self.masses.push(mass);
        idx
    }

    pub fn add_spring(&mut self, spring: FoamSpring) {
        self.springs.push(spring);
    }

    pub fn node_count(&self) -> usize {
        self.positions.len()
    }

    pub fn spring_count(&self) -> usize {
        self.springs.len()
    }

    pub fn total_elastic_energy(&self) -> f32 {
        self.springs.iter().map(|s| s.elastic_energy()).sum()
    }

    pub fn step_springs(&mut self, dt: f32) {
        for s in &mut self.springs {
            s.step(s.displacement, dt);
        }
    }
}

impl Default for FoamSpringNetwork {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_foam_spring(id: u32, k: f32, c: f32, rest: f32, creep: f32) -> FoamSpring {
    FoamSpring::new(id, k, c, rest, creep)
}

pub fn new_foam_spring_network() -> FoamSpringNetwork {
    FoamSpringNetwork::new()
}

pub fn fsn_add_node(net: &mut FoamSpringNetwork, pos: f32, mass: f32) -> usize {
    net.add_node(pos, mass)
}

pub fn fsn_add_spring(net: &mut FoamSpringNetwork, spring: FoamSpring) {
    net.add_spring(spring);
}

/// Period of a foam spring oscillator.
pub fn spring_period(spring_k: f32) -> f32 {
    if spring_k <= 0.0 {
        return f32::INFINITY;
    }
    2.0 * PI / spring_k.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn force_zero_at_rest() {
        let s = new_foam_spring(0, 100.0, 1.0, 1.0, 0.0);
        assert!(s.force().abs() < 1e-6);
    }

    #[test]
    fn force_negative_when_displaced() {
        let mut s = new_foam_spring(0, 100.0, 0.0, 1.0, 0.0);
        s.displacement = 0.5;
        assert!(s.force() < 0.0);
    }

    #[test]
    fn elastic_energy_positive_when_displaced() {
        let mut s = new_foam_spring(0, 100.0, 0.0, 1.0, 0.0);
        s.displacement = 0.1;
        assert!(s.elastic_energy() > 0.0);
    }

    #[test]
    fn omega_n_computation() {
        let s = new_foam_spring(0, 100.0, 0.0, 1.0, 0.0);
        assert!((s.omega_n() - 10.0).abs() < 1e-4);
    }

    #[test]
    fn damping_ratio_zero_for_undamped() {
        let s = new_foam_spring(0, 100.0, 0.0, 1.0, 0.0);
        assert!(s.damping_ratio().abs() < 1e-6);
    }

    #[test]
    fn overdamped_detection() {
        let s = new_foam_spring(0, 1.0, 10.0, 1.0, 0.0);
        assert!(s.is_overdamped());
    }

    #[test]
    fn network_add_node() {
        let mut net = new_foam_spring_network();
        fsn_add_node(&mut net, 0.0, 1.0);
        assert_eq!(net.node_count(), 1);
    }

    #[test]
    fn network_add_spring() {
        let mut net = new_foam_spring_network();
        let s = new_foam_spring(0, 10.0, 0.5, 1.0, 0.01);
        fsn_add_spring(&mut net, s);
        assert_eq!(net.spring_count(), 1);
    }

    #[test]
    fn total_elastic_energy_sums() {
        let mut net = new_foam_spring_network();
        let mut s1 = new_foam_spring(0, 100.0, 0.0, 1.0, 0.0);
        s1.displacement = 0.1;
        let mut s2 = new_foam_spring(1, 100.0, 0.0, 1.0, 0.0);
        s2.displacement = 0.2;
        fsn_add_spring(&mut net, s1);
        fsn_add_spring(&mut net, s2);
        let total = net.total_elastic_energy();
        assert!(total > 0.0);
    }

    #[test]
    fn spring_period_formula() {
        let k = 4.0 * PI * PI; // period = 1.0 s
        let p = spring_period(k);
        assert!((p - 1.0).abs() < 1e-4);
    }
}
