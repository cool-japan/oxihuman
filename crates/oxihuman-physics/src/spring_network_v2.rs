// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Mass-spring network simulation.

/// A particle in the spring network.
#[derive(Debug, Clone)]
pub struct SnParticle {
    pub pos: [f64; 3],
    pub vel: [f64; 3],
    pub mass: f64,
    pub pinned: bool,
}

impl SnParticle {
    pub fn new(pos: [f64; 3], mass: f64) -> Self {
        SnParticle { pos, vel: [0.0; 3], mass, pinned: false }
    }
}

/// A spring connecting two particles.
#[derive(Debug, Clone)]
pub struct SnSpring {
    pub a: usize,
    pub b: usize,
    pub rest_len: f64,
    pub stiffness: f64,
    pub damping: f64,
}

/// A mass-spring network.
#[derive(Debug, Default, Clone)]
pub struct SpringNetworkV2 {
    pub particles: Vec<SnParticle>,
    pub springs: Vec<SnSpring>,
    pub gravity: [f64; 3],
}

impl SpringNetworkV2 {
    /// Create a new empty spring network.
    pub fn new() -> Self {
        SpringNetworkV2 { particles: Vec::new(), springs: Vec::new(), gravity: [0.0, -9.81, 0.0] }
    }

    /// Add a particle; returns its index.
    pub fn add_particle(&mut self, pos: [f64; 3], mass: f64) -> usize {
        self.particles.push(SnParticle::new(pos, mass));
        self.particles.len() - 1
    }

    /// Add a spring between particles `a` and `b`.
    pub fn add_spring(&mut self, a: usize, b: usize, stiffness: f64, damping: f64) {
        let rest_len = dist3(self.particles[a].pos, self.particles[b].pos);
        self.springs.push(SnSpring { a, b, rest_len, stiffness, damping });
    }

    /// Step the simulation by `dt` seconds.
    #[allow(clippy::needless_range_loop)]
    pub fn step(&mut self, dt: f64) {
        let n = self.particles.len();
        let mut forces = vec![[0.0f64; 3]; n];

        /* gravity */
        for i in 0..n {
            if !self.particles[i].pinned {
                let m = self.particles[i].mass;
                for k in 0..3 {
                    forces[i][k] += m * self.gravity[k];
                }
            }
        }

        /* spring forces */
        for s in &self.springs {
            let pa = self.particles[s.a].pos;
            let pb = self.particles[s.b].pos;
            let va = self.particles[s.a].vel;
            let vb = self.particles[s.b].vel;
            let d = dist3(pa, pb);
            if d < 1e-10 { continue; }
            let ext = d - s.rest_len;
            let mut dir = [0.0f64; 3];
            for k in 0..3 { dir[k] = (pb[k] - pa[k]) / d; }
            let rel_vel: f64 = (0..3).map(|k| (vb[k] - va[k]) * dir[k]).sum();
            let force_mag = s.stiffness * ext + s.damping * rel_vel;
            for k in 0..3 {
                forces[s.a][k] += force_mag * dir[k];
                forces[s.b][k] -= force_mag * dir[k];
            }
        }

        /* integrate */
        for i in 0..n {
            if self.particles[i].pinned { continue; }
            let m = self.particles[i].mass;
            for k in 0..3 {
                self.particles[i].vel[k] += forces[i][k] / m * dt;
                self.particles[i].pos[k] += self.particles[i].vel[k] * dt;
            }
        }
    }

    /// Total kinetic energy.
    pub fn kinetic_energy(&self) -> f64 {
        self.particles.iter().map(|p| {
            let v2: f64 = p.vel.iter().map(|&v| v * v).sum();
            0.5 * p.mass * v2
        }).sum()
    }

    /// Number of particles.
    pub fn particle_count(&self) -> usize { self.particles.len() }

    /// Number of springs.
    pub fn spring_count(&self) -> usize { self.springs.len() }
}

fn dist3(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Create a new spring network.
pub fn new_spring_network_v2() -> SpringNetworkV2 { SpringNetworkV2::new() }

/// Add particle.
pub fn sn2_add_particle(net: &mut SpringNetworkV2, pos: [f64; 3], mass: f64) -> usize {
    net.add_particle(pos, mass)
}

/// Add spring.
pub fn sn2_add_spring(net: &mut SpringNetworkV2, a: usize, b: usize, k: f64, d: f64) {
    net.add_spring(a, b, k, d);
}

/// Step.
pub fn sn2_step(net: &mut SpringNetworkV2, dt: f64) { net.step(dt); }

/// Kinetic energy.
pub fn sn2_kinetic_energy(net: &SpringNetworkV2) -> f64 { net.kinetic_energy() }

/// Particle count.
pub fn sn2_particle_count(net: &SpringNetworkV2) -> usize { net.particle_count() }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_particle() {
        let mut net = new_spring_network_v2();
        sn2_add_particle(&mut net, [0.0, 0.0, 0.0], 1.0);
        assert_eq!(sn2_particle_count(&net), 1 /* one particle */);
    }

    #[test]
    fn test_add_spring() {
        let mut net = new_spring_network_v2();
        sn2_add_particle(&mut net, [0.0, 0.0, 0.0], 1.0);
        sn2_add_particle(&mut net, [1.0, 0.0, 0.0], 1.0);
        sn2_add_spring(&mut net, 0, 1, 100.0, 0.1);
        assert_eq!(net.spring_count(), 1 /* one spring */);
    }

    #[test]
    fn test_rest_length_computed() {
        let mut net = new_spring_network_v2();
        sn2_add_particle(&mut net, [0.0, 0.0, 0.0], 1.0);
        sn2_add_particle(&mut net, [3.0, 4.0, 0.0], 1.0);
        sn2_add_spring(&mut net, 0, 1, 10.0, 0.0);
        assert!((net.springs[0].rest_len - 5.0).abs() < 1e-9 /* 3-4-5 triangle */);
    }

    #[test]
    fn test_pinned_particle_doesnt_move() {
        let mut net = new_spring_network_v2();
        let i = sn2_add_particle(&mut net, [0.0, 5.0, 0.0], 1.0);
        net.particles[i].pinned = true;
        sn2_step(&mut net, 0.1);
        assert!((net.particles[i].pos[1] - 5.0).abs() < 1e-12 /* pinned */);
    }

    #[test]
    fn test_gravity_pulls_down() {
        let mut net = new_spring_network_v2();
        sn2_add_particle(&mut net, [0.0, 0.0, 0.0], 1.0);
        sn2_step(&mut net, 0.1);
        assert!(net.particles[0].vel[1] < 0.0 /* falling */);
    }

    #[test]
    fn test_kinetic_energy_initially_zero() {
        let mut net = new_spring_network_v2();
        sn2_add_particle(&mut net, [0.0, 0.0, 0.0], 1.0);
        assert_eq!(sn2_kinetic_energy(&net), 0.0 /* no motion */);
    }

    #[test]
    fn test_kinetic_energy_increases() {
        let mut net = new_spring_network_v2();
        sn2_add_particle(&mut net, [0.0, 10.0, 0.0], 1.0);
        sn2_step(&mut net, 0.5);
        assert!(sn2_kinetic_energy(&net) > 0.0 /* energy gained */);
    }

    #[test]
    fn test_two_particle_spring() {
        let mut net = new_spring_network_v2();
        let a = sn2_add_particle(&mut net, [0.0, 0.0, 0.0], 1.0);
        let b = sn2_add_particle(&mut net, [2.0, 0.0, 0.0], 1.0);
        net.particles[a].pinned = true;
        net.particles[b].pinned = true;
        sn2_add_spring(&mut net, a, b, 100.0, 0.0);
        sn2_step(&mut net, 0.01);
        /* pinned particles should not move */
        assert!((net.particles[b].pos[0] - 2.0).abs() < 1e-12 /* still at 2 */);
    }

    #[test]
    fn test_no_nan_after_step() {
        let mut net = new_spring_network_v2();
        sn2_add_particle(&mut net, [0.0, 1.0, 0.0], 1.0);
        sn2_add_particle(&mut net, [0.0, 0.0, 0.0], 1.0);
        net.particles[1].pinned = true;
        sn2_add_spring(&mut net, 0, 1, 50.0, 1.0);
        for _ in 0..10 {
            sn2_step(&mut net, 0.01);
        }
        assert!(!net.particles[0].pos[1].is_nan() /* no NaN */);
    }

    #[test]
    fn test_spring_count_after_multiple() {
        let mut net = new_spring_network_v2();
        for i in 0..4 {
            sn2_add_particle(&mut net, [i as f64, 0.0, 0.0], 1.0);
        }
        sn2_add_spring(&mut net, 0, 1, 10.0, 0.5);
        sn2_add_spring(&mut net, 1, 2, 10.0, 0.5);
        sn2_add_spring(&mut net, 2, 3, 10.0, 0.5);
        assert_eq!(net.spring_count(), 3 /* three springs */);
    }
}
