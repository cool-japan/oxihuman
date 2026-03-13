// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Mass-spring soft body simulation.

/// A particle with mass, position, velocity.
#[derive(Debug, Clone)]
pub struct MsParticle {
    pub mass: f32,
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub force: [f32; 3],
    pub pinned: bool,
}

impl MsParticle {
    pub fn new(mass: f32, pos: [f32; 3]) -> Self {
        MsParticle {
            mass,
            pos,
            vel: [0.0; 3],
            force: [0.0; 3],
            pinned: false,
        }
    }
}

/// A spring connecting two particles.
#[derive(Debug, Clone, Copy)]
pub struct MsSpring {
    pub a: usize,
    pub b: usize,
    pub rest_len: f32,
    pub stiffness: f32,
    pub damping: f32,
}

/// A mass-spring soft body.
pub struct MsSoftBody {
    pub particles: Vec<MsParticle>,
    pub springs: Vec<MsSpring>,
}

impl MsSoftBody {
    pub fn new() -> Self {
        MsSoftBody {
            particles: Vec::new(),
            springs: Vec::new(),
        }
    }
}

impl Default for MsSoftBody {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new mass-spring soft body.
pub fn new_ms_body() -> MsSoftBody {
    MsSoftBody::new()
}

/// Add a particle; returns its index.
pub fn ms_add_particle(body: &mut MsSoftBody, mass: f32, pos: [f32; 3]) -> usize {
    let id = body.particles.len();
    body.particles.push(MsParticle::new(mass, pos));
    id
}

/// Add a spring between particles `a` and `b`.
pub fn ms_add_spring(body: &mut MsSoftBody, a: usize, b: usize, stiffness: f32, damping: f32) {
    if a < body.particles.len() && b < body.particles.len() {
        let dx = body.particles[b].pos[0] - body.particles[a].pos[0];
        let dy = body.particles[b].pos[1] - body.particles[a].pos[1];
        let dz = body.particles[b].pos[2] - body.particles[a].pos[2];
        let rest_len = (dx * dx + dy * dy + dz * dz).sqrt();
        body.springs.push(MsSpring {
            a,
            b,
            rest_len,
            stiffness,
            damping,
        });
    }
}

/// Step the simulation by `dt` seconds.
#[allow(clippy::needless_range_loop)]
pub fn ms_step(body: &mut MsSoftBody, dt: f32, gravity: [f32; 3]) {
    /* accumulate spring forces */
    let n = body.particles.len();
    let mut forces = vec![[0.0f32; 3]; n];

    for s in &body.springs {
        if s.a >= n || s.b >= n {
            continue;
        }
        let pa = body.particles[s.a].pos;
        let pb = body.particles[s.b].pos;
        let va = body.particles[s.a].vel;
        let vb = body.particles[s.b].vel;
        let mut d = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-10);
        for x in &mut d {
            *x /= len;
        }
        let stretch = len - s.rest_len;
        /* relative velocity along spring axis */
        let rv = (vb[0] - va[0]) * d[0] + (vb[1] - va[1]) * d[1] + (vb[2] - va[2]) * d[2];
        let f_mag = s.stiffness * stretch + s.damping * rv;
        for k in 0..3 {
            forces[s.a][k] += f_mag * d[k];
            forces[s.b][k] -= f_mag * d[k];
        }
    }

    /* integrate */
    for i in 0..n {
        if body.particles[i].pinned {
            continue;
        }
        let inv_m = 1.0 / body.particles[i].mass.max(1e-10);
        for k in 0..3 {
            let acc = (forces[i][k] + gravity[k] * body.particles[i].mass) * inv_m;
            body.particles[i].vel[k] += acc * dt;
            body.particles[i].pos[k] += body.particles[i].vel[k] * dt;
        }
    }
}

/// Return total kinetic energy.
pub fn ms_kinetic_energy(body: &MsSoftBody) -> f32 {
    body.particles
        .iter()
        .filter(|p| !p.pinned)
        .map(|p| {
            let v2 = p.vel[0] * p.vel[0] + p.vel[1] * p.vel[1] + p.vel[2] * p.vel[2];
            0.5 * p.mass * v2
        })
        .sum()
}

/// Return total spring potential energy.
pub fn ms_potential_energy(body: &MsSoftBody) -> f32 {
    body.springs
        .iter()
        .map(|s| {
            if s.a >= body.particles.len() || s.b >= body.particles.len() {
                return 0.0;
            }
            let pa = body.particles[s.a].pos;
            let pb = body.particles[s.b].pos;
            let d2 = (pb[0] - pa[0]).powi(2) + (pb[1] - pa[1]).powi(2) + (pb[2] - pa[2]).powi(2);
            let len = d2.sqrt();
            let stretch = len - s.rest_len;
            0.5 * s.stiffness * stretch * stretch
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_particle_body() -> MsSoftBody {
        let mut b = new_ms_body();
        ms_add_particle(&mut b, 1.0, [0.0, 0.0, 0.0]);
        ms_add_particle(&mut b, 1.0, [1.0, 0.0, 0.0]);
        ms_add_spring(&mut b, 0, 1, 100.0, 1.0);
        b
    }

    #[test]
    fn test_particle_count() {
        let b = two_particle_body();
        assert_eq!(b.particles.len(), 2);
    }

    #[test]
    fn test_spring_count() {
        let b = two_particle_body();
        assert_eq!(b.springs.len(), 1);
    }

    #[test]
    fn test_rest_length_computed() {
        let b = two_particle_body();
        assert!((b.springs[0].rest_len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_step_moves_particles() {
        let mut b = two_particle_body();
        /* pin particle 0, stretch spring, let 1 move */
        b.particles[0].pinned = true;
        b.particles[1].pos = [2.0, 0.0, 0.0]; /* stretched */
        ms_step(&mut b, 0.01, [0.0, 0.0, 0.0]);
        /* particle 1 should have moved toward 0 */
        assert!(b.particles[1].pos[0] < 2.0);
    }

    #[test]
    fn test_kinetic_energy_zero_at_rest() {
        let b = two_particle_body();
        assert!((ms_kinetic_energy(&b) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_potential_energy_at_rest() {
        let b = two_particle_body();
        /* particles at rest length: PE = 0 */
        assert!(ms_potential_energy(&b) < 1e-5);
    }

    #[test]
    fn test_pinned_particle_does_not_move() {
        let mut b = new_ms_body();
        let a = ms_add_particle(&mut b, 1.0, [0.0, 0.0, 0.0]);
        b.particles[a].pinned = true;
        ms_step(&mut b, 0.1, [0.0, -9.8, 0.0]);
        assert_eq!(b.particles[a].pos, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_gravity_applies_to_free_particle() {
        let mut b = new_ms_body();
        ms_add_particle(&mut b, 1.0, [0.0, 0.0, 0.0]);
        ms_step(&mut b, 0.1, [0.0, -9.8, 0.0]);
        assert!(b.particles[0].vel[1] < 0.0);
    }

    #[test]
    fn test_kinetic_energy_positive_after_stretch() {
        let mut b = two_particle_body();
        b.particles[0].pinned = true;
        b.particles[1].pos = [3.0, 0.0, 0.0];
        ms_step(&mut b, 0.01, [0.0, 0.0, 0.0]);
        assert!(ms_kinetic_energy(&b) > 0.0);
    }
}
