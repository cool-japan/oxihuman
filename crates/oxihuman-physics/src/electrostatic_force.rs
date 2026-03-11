// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]
#![allow(clippy::needless_range_loop)]

//! Coulomb electrostatic force between charged particles.

/// Coulomb constant k = 1/(4πε₀) in N·m²/C².
pub const COULOMB_K: f32 = 8.987e9;

/// A charged particle.
#[derive(Debug, Clone)]
pub struct ChargedParticle {
    pub position: [f32; 3],
    pub charge: f32, /* Coulombs */
    pub mass: f32,
    pub velocity: [f32; 3],
}

/// Construct a new ChargedParticle.
pub fn new_charged_particle(pos: [f32; 3], charge: f32, mass: f32) -> ChargedParticle {
    ChargedParticle {
        position: pos,
        charge,
        mass,
        velocity: [0.0; 3],
    }
}

/// Coulomb force on `target` due to `source` (3D).
pub fn coulomb_force(source: &ChargedParticle, target: &ChargedParticle) -> [f32; 3] {
    let dr = [
        target.position[0] - source.position[0],
        target.position[1] - source.position[1],
        target.position[2] - source.position[2],
    ];
    let r2 = dr[0] * dr[0] + dr[1] * dr[1] + dr[2] * dr[2];
    if r2 < 1e-18 {
        return [0.0; 3];
    }
    let r = r2.sqrt();
    let mag = COULOMB_K * source.charge * target.charge / r2;
    [mag * dr[0] / r, mag * dr[1] / r, mag * dr[2] / r]
}

/// Coulomb potential energy between two charges.
pub fn coulomb_potential(source: &ChargedParticle, target: &ChargedParticle) -> f32 {
    let dr = [
        target.position[0] - source.position[0],
        target.position[1] - source.position[1],
        target.position[2] - source.position[2],
    ];
    let r = (dr[0] * dr[0] + dr[1] * dr[1] + dr[2] * dr[2])
        .sqrt()
        .max(1e-9);
    COULOMB_K * source.charge * target.charge / r
}

/// Electrostatic field at position `r` due to `source`.
pub fn electric_field(source: &ChargedParticle, r: [f32; 3]) -> [f32; 3] {
    let dr = [
        r[0] - source.position[0],
        r[1] - source.position[1],
        r[2] - source.position[2],
    ];
    let r2 = dr[0] * dr[0] + dr[1] * dr[1] + dr[2] * dr[2];
    if r2 < 1e-18 {
        return [0.0; 3];
    }
    let r_len = r2.sqrt();
    let mag = COULOMB_K * source.charge / r2;
    [
        mag * dr[0] / r_len,
        mag * dr[1] / r_len,
        mag * dr[2] / r_len,
    ]
}

/// Electrostatic particle system.
pub struct ElectrostaticSystem {
    pub particles: Vec<ChargedParticle>,
    pub damping: f32,
}

/// Construct a new ElectrostaticSystem.
pub fn new_electrostatic_system(damping: f32) -> ElectrostaticSystem {
    ElectrostaticSystem {
        particles: Vec::new(),
        damping,
    }
}

impl ElectrostaticSystem {
    /// Add a particle.
    pub fn add_particle(&mut self, p: ChargedParticle) {
        self.particles.push(p);
    }

    /// Particle count.
    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    /// Total system charge.
    pub fn total_charge(&self) -> f32 {
        self.particles.iter().map(|p| p.charge).sum()
    }

    /// Simulate one timestep.
    pub fn step(&mut self, dt: f32, gravity: [f32; 3]) {
        let n = self.particles.len();
        let mut forces = vec![[0.0f32; 3]; n];

        for i in 0..n {
            for k in 0..3 {
                forces[i][k] += self.particles[i].mass * gravity[k];
            }
        }

        for i in 0..n {
            for j in 0..n {
                if i == j {
                    continue;
                }
                let f = coulomb_force(&self.particles[j], &self.particles[i]);
                for k in 0..3 {
                    forces[i][k] += f[k];
                }
            }
        }

        for i in 0..n {
            for k in 0..3 {
                let a = forces[i][k] / self.particles[i].mass;
                self.particles[i].velocity[k] =
                    self.particles[i].velocity[k] * (1.0 - self.damping * dt) + a * dt;
                self.particles[i].position[k] += self.particles[i].velocity[k] * dt;
            }
        }
    }

    /// Total electrostatic potential energy.
    pub fn potential_energy(&self) -> f32 {
        let n = self.particles.len();
        let mut e = 0.0f32;
        for i in 0..n {
            for j in (i + 1)..n {
                e += coulomb_potential(&self.particles[i], &self.particles[j]);
            }
        }
        e
    }

    /// Total kinetic energy.
    pub fn kinetic_energy(&self) -> f32 {
        self.particles
            .iter()
            .map(|p| {
                let v2: f32 = p.velocity.iter().map(|&v| v * v).sum();
                0.5 * p.mass * v2
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_system() {
        /* new system is empty */
        let s = new_electrostatic_system(0.0);
        assert_eq!(s.particle_count(), 0);
    }

    #[test]
    fn test_add_particle() {
        /* add_particle increments count */
        let mut s = new_electrostatic_system(0.0);
        s.add_particle(new_charged_particle([0.0; 3], 1e-6, 1.0));
        assert_eq!(s.particle_count(), 1);
    }

    #[test]
    fn test_coulomb_force_repulsion() {
        /* same sign charges repel */
        let a = new_charged_particle([0.0, 0.0, 0.0], 1e-6, 1.0);
        let b = new_charged_particle([1.0, 0.0, 0.0], 1e-6, 1.0);
        let f = coulomb_force(&a, &b);
        assert!(f[0] > 0.0, "same-sign should repel, f[0]={}", f[0]);
    }

    #[test]
    fn test_coulomb_force_attraction() {
        /* opposite sign charges attract */
        let a = new_charged_particle([0.0, 0.0, 0.0], 1e-6, 1.0);
        let b = new_charged_particle([1.0, 0.0, 0.0], -1e-6, 1.0);
        let f = coulomb_force(&a, &b);
        assert!(f[0] < 0.0, "opposite signs should attract, f[0]={}", f[0]);
    }

    #[test]
    fn test_coulomb_potential_positive_same_sign() {
        /* same sign charges have positive potential energy */
        let a = new_charged_particle([0.0; 3], 1e-6, 1.0);
        let b = new_charged_particle([1.0, 0.0, 0.0], 1e-6, 1.0);
        assert!(coulomb_potential(&a, &b) > 0.0);
    }

    #[test]
    fn test_electric_field_nonzero() {
        /* electric field from non-zero charge is non-zero */
        let src = new_charged_particle([0.0; 3], 1.0, 1.0);
        let e = electric_field(&src, [1.0, 0.0, 0.0]);
        assert!(e[0].abs() > 0.0);
    }

    #[test]
    fn test_total_charge() {
        /* total charge is sum of individual charges */
        let mut s = new_electrostatic_system(0.0);
        s.add_particle(new_charged_particle([0.0; 3], 1e-6, 1.0));
        s.add_particle(new_charged_particle([1.0, 0.0, 0.0], -2e-6, 1.0));
        assert!((s.total_charge() - (-1e-6)).abs() < 1e-14);
    }

    #[test]
    fn test_kinetic_energy_zero_initially() {
        /* kinetic energy is zero before any steps */
        let mut s = new_electrostatic_system(0.0);
        s.add_particle(new_charged_particle([0.0; 3], 1e-6, 1.0));
        assert!(s.kinetic_energy() < 1e-20);
    }

    #[test]
    fn test_potential_energy_two_charges() {
        /* potential energy of two equal charges at 1m is positive */
        let mut s = new_electrostatic_system(0.0);
        s.add_particle(new_charged_particle([0.0; 3], 1e-6, 1.0));
        s.add_particle(new_charged_particle([1.0, 0.0, 0.0], 1e-6, 1.0));
        assert!(s.potential_energy() > 0.0);
    }
}
