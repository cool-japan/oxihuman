// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
#![allow(clippy::needless_range_loop)]

//! Linear elastic deformable body (spring-mass lattice).

/// A particle in the deformable body lattice.
#[derive(Debug, Clone)]
pub struct DeformableParticle {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub mass: f32,
    pub pinned: bool,
}

/// A spring connecting two particles.
#[derive(Debug, Clone)]
pub struct DeformableSpringEdge {
    pub i: usize,
    pub j: usize,
    pub rest_length: f32,
    pub stiffness: f32,
}

/// A linear elastic deformable body.
pub struct DeformableBody {
    pub particles: Vec<DeformableParticle>,
    pub springs: Vec<DeformableSpringEdge>,
    pub damping: f32,
}

/// Construct a new DeformableBody.
pub fn new_deformable_body(damping: f32) -> DeformableBody {
    DeformableBody {
        particles: Vec::new(),
        springs: Vec::new(),
        damping,
    }
}

impl DeformableBody {
    /// Add a particle.
    pub fn add_particle(&mut self, pos: [f32; 3], mass: f32, pinned: bool) -> usize {
        let idx = self.particles.len();
        self.particles.push(DeformableParticle {
            position: pos,
            velocity: [0.0; 3],
            mass: mass.max(1e-6),
            pinned,
        });
        idx
    }

    /// Add a spring between particles i and j.
    pub fn add_spring(&mut self, i: usize, j: usize, stiffness: f32) {
        if i < self.particles.len() && j < self.particles.len() {
            let dx = self.particles[j].position[0] - self.particles[i].position[0];
            let dy = self.particles[j].position[1] - self.particles[i].position[1];
            let dz = self.particles[j].position[2] - self.particles[i].position[2];
            let rest_length = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-6);
            self.springs.push(DeformableSpringEdge {
                i,
                j,
                rest_length,
                stiffness,
            });
        }
    }

    /// Simulate one timestep.
    pub fn step(&mut self, dt: f32, gravity: [f32; 3]) {
        let n = self.particles.len();
        let mut forces = vec![[0.0f32; 3]; n];

        /* gravity */
        for (i, p) in self.particles.iter().enumerate() {
            if !p.pinned {
                for k in 0..3 {
                    forces[i][k] += p.mass * gravity[k];
                }
            }
        }

        /* spring forces */
        for s in &self.springs {
            let pi = self.particles[s.i].position;
            let pj = self.particles[s.j].position;
            let dx = pj[0] - pi[0];
            let dy = pj[1] - pi[1];
            let dz = pj[2] - pi[2];
            let len = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-9);
            let stretch = len - s.rest_length;
            let force_mag = s.stiffness * stretch / len;
            let f = [dx * force_mag, dy * force_mag, dz * force_mag];
            for k in 0..3 {
                forces[s.i][k] += f[k];
                forces[s.j][k] -= f[k];
            }
        }

        /* integrate */
        for (i, p) in self.particles.iter_mut().enumerate() {
            if p.pinned {
                continue;
            }
            for k in 0..3 {
                let accel = forces[i][k] / p.mass;
                p.velocity[k] = p.velocity[k] * (1.0 - self.damping * dt) + accel * dt;
                p.position[k] += p.velocity[k] * dt;
            }
        }
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

    /// Particle count.
    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    /// Spring count.
    pub fn spring_count(&self) -> usize {
        self.springs.len()
    }
}

/// Build a simple 1D chain of particles.
pub fn build_chain_body(n: usize, spacing: f32, mass: f32, stiffness: f32) -> DeformableBody {
    let mut body = new_deformable_body(0.1);
    for i in 0..n {
        let pinned = i == 0;
        body.add_particle([i as f32 * spacing, 0.0, 0.0], mass, pinned);
    }
    for i in 0..(n.saturating_sub(1)) {
        body.add_spring(i, i + 1, stiffness);
    }
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_body() {
        /* new deformable body has zero particles and springs */
        let b = new_deformable_body(0.1);
        assert_eq!(b.particle_count(), 0);
        assert_eq!(b.spring_count(), 0);
    }

    #[test]
    fn test_add_particle() {
        /* add_particle increments count */
        let mut b = new_deformable_body(0.0);
        b.add_particle([0.0, 0.0, 0.0], 1.0, false);
        assert_eq!(b.particle_count(), 1);
    }

    #[test]
    fn test_add_spring() {
        /* add_spring increments spring count */
        let mut b = new_deformable_body(0.0);
        b.add_particle([0.0, 0.0, 0.0], 1.0, false);
        b.add_particle([1.0, 0.0, 0.0], 1.0, false);
        b.add_spring(0, 1, 100.0);
        assert_eq!(b.spring_count(), 1);
    }

    #[test]
    fn test_step_pinned_particle_does_not_move() {
        /* pinned particle stays at its initial position */
        let mut b = new_deformable_body(0.0);
        let i = b.add_particle([0.0, 0.0, 0.0], 1.0, true);
        b.step(0.01, [0.0, -9.8, 0.0]);
        assert!((b.particles[i].position[1]).abs() < 1e-6);
    }

    #[test]
    fn test_step_gravity_moves_free_particle() {
        /* free particle falls under gravity */
        let mut b = new_deformable_body(0.0);
        b.add_particle([0.0, 0.0, 0.0], 1.0, false);
        let y0 = b.particles[0].position[1];
        b.step(0.1, [0.0, -9.8, 0.0]);
        assert!(b.particles[0].position[1] < y0);
    }

    #[test]
    fn test_kinetic_energy_initially_zero() {
        /* kinetic energy of static body is zero */
        let mut b = new_deformable_body(0.0);
        b.add_particle([0.0, 0.0, 0.0], 1.0, false);
        assert!(b.kinetic_energy() < 1e-10);
    }

    #[test]
    fn test_kinetic_energy_after_step() {
        /* after step under gravity, kinetic energy increases */
        let mut b = new_deformable_body(0.0);
        b.add_particle([0.0, 0.0, 0.0], 1.0, false);
        b.step(0.1, [0.0, -9.8, 0.0]);
        assert!(b.kinetic_energy() > 0.0);
    }

    #[test]
    fn test_build_chain() {
        /* build_chain_body creates correct number of particles and springs */
        let b = build_chain_body(5, 1.0, 1.0, 100.0);
        assert_eq!(b.particle_count(), 5);
        assert_eq!(b.spring_count(), 4);
    }

    #[test]
    fn test_chain_step_runs() {
        /* stepping chain body does not panic */
        let mut b = build_chain_body(4, 0.5, 1.0, 50.0);
        for _ in 0..10 {
            b.step(0.001, [0.0, -9.8, 0.0]);
        }
        assert!(b.kinetic_energy().is_finite());
    }
}
