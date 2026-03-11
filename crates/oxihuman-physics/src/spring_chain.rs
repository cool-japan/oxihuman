// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Linear chain of spring-mass units.

use std::f32::consts::PI;

/// A mass in the spring chain.
#[derive(Debug, Clone)]
pub struct ChainMass {
    pub position: f32,
    pub velocity: f32,
    pub mass: f32,
    pub pinned: bool,
}

/// A spring connecting two adjacent masses.
#[derive(Debug, Clone)]
pub struct ChainSpring {
    pub stiffness: f32,
    pub rest_length: f32,
    pub damping: f32,
}

/// Linear spring-mass chain.
pub struct SpringChain {
    pub masses: Vec<ChainMass>,
    pub springs: Vec<ChainSpring>,
}

#[allow(dead_code)]
impl SpringChain {
    pub fn new() -> Self {
        SpringChain {
            masses: Vec::new(),
            springs: Vec::new(),
        }
    }

    pub fn add_mass(&mut self, position: f32, mass: f32, pinned: bool) {
        self.masses.push(ChainMass {
            position,
            velocity: 0.0,
            mass: mass.max(1e-9),
            pinned,
        });
    }

    pub fn add_spring(&mut self, stiffness: f32, rest_length: f32, damping: f32) {
        self.springs.push(ChainSpring {
            stiffness,
            rest_length,
            damping,
        });
    }

    pub fn step(&mut self, dt: f32) {
        let n = self.masses.len();
        let s = self.springs.len();
        let links = n.min(s + 1) - 1;
        let mut forces = vec![0.0f32; n];

        for i in 0..links {
            if i >= s {
                break;
            }
            let dx = self.masses[i + 1].position - self.masses[i].position;
            let dv = self.masses[i + 1].velocity - self.masses[i].velocity;
            let spring = &self.springs[i];
            let stretch = dx - spring.rest_length;
            let f = spring.stiffness * stretch + spring.damping * dv;
            forces[i] += f;
            forces[i + 1] -= f;
        }

        #[allow(clippy::needless_range_loop)]
        for i in 0..n {
            if self.masses[i].pinned {
                continue;
            }
            self.masses[i].velocity += forces[i] / self.masses[i].mass * dt;
            self.masses[i].position += self.masses[i].velocity * dt;
        }
    }

    pub fn total_mass(&self) -> f32 {
        self.masses.iter().map(|m| m.mass).sum()
    }

    pub fn potential_energy(&self) -> f32 {
        let s = self.springs.len();
        let n = self.masses.len();
        let links = n.min(s + 1) - 1;
        let mut e = 0.0f32;
        for i in 0..links {
            if i >= s {
                break;
            }
            let dx = self.masses[i + 1].position - self.masses[i].position;
            let stretch = dx - self.springs[i].rest_length;
            e += 0.5 * self.springs[i].stiffness * stretch.powi(2);
        }
        e
    }

    pub fn kinetic_energy(&self) -> f32 {
        self.masses
            .iter()
            .map(|m| 0.5 * m.mass * m.velocity.powi(2))
            .sum()
    }

    pub fn total_energy(&self) -> f32 {
        self.potential_energy() + self.kinetic_energy()
    }

    pub fn mass_count(&self) -> usize {
        self.masses.len()
    }

    pub fn spring_count(&self) -> usize {
        self.springs.len()
    }

    pub fn natural_frequency(&self) -> Option<f32> {
        if self.masses.is_empty() || self.springs.is_empty() {
            return None;
        }
        let k = self.springs[0].stiffness;
        let m = self
            .masses
            .iter()
            .filter(|m| !m.pinned)
            .map(|m| m.mass)
            .next()?;
        Some((k / m).sqrt() / (2.0 * PI))
    }
}

impl Default for SpringChain {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_spring_chain() -> SpringChain {
    SpringChain::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_chain() -> SpringChain {
        let mut c = new_spring_chain();
        c.add_mass(0.0, 1.0, true);
        c.add_mass(1.0, 1.0, false);
        c.add_spring(100.0, 1.0, 0.0);
        c
    }

    #[test]
    fn mass_and_spring_count() {
        let c = simple_chain();
        assert_eq!(c.mass_count(), 2);
        assert_eq!(c.spring_count(), 1);
    }

    #[test]
    fn at_rest_no_force() {
        let mut c = simple_chain();
        let p0 = c.masses[1].position;
        c.step(0.01);
        assert!((c.masses[1].position - p0).abs() < 1e-10);
    }

    #[test]
    fn displaced_mass_oscillates() {
        let mut c = simple_chain();
        c.masses[1].position = 1.1;
        c.step(0.01);
        assert!(c.masses[1].velocity.abs() > 0.0);
    }

    #[test]
    fn pinned_doesnt_move() {
        let mut c = simple_chain();
        c.masses[1].position = 1.5;
        c.step(0.01);
        assert_eq!(c.masses[0].position, 0.0);
    }

    #[test]
    fn total_mass() {
        let c = simple_chain();
        assert!((c.total_mass() - 2.0).abs() < 1e-6);
    }

    #[test]
    fn natural_frequency_positive() {
        let c = simple_chain();
        assert!(c.natural_frequency().unwrap() > 0.0);
    }

    #[test]
    fn energy_at_rest() {
        let c = simple_chain();
        assert!(c.total_energy() < 1e-10);
    }

    #[test]
    fn energy_increases_when_displaced() {
        let mut c = simple_chain();
        c.masses[1].position = 1.2;
        assert!(c.potential_energy() > 0.0);
    }

    #[test]
    fn pi_used() {
        let _ = PI;
        let c = simple_chain();
        assert!(c.natural_frequency().is_some());
    }
}
