// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Plasma body: charged particle cloud with simple Coulomb interaction.

const K_E: f32 = 8.987e9; // Coulomb constant N·m²/C²

/// A charged particle in the plasma.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PlasmaParticle {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub charge: f32,
    pub mass: f32,
}

/// A plasma body (collection of charged particles).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PlasmaBody {
    pub particles: Vec<PlasmaParticle>,
    pub damping: f32,
    pub temperature: f32,
}

/// Create a new `PlasmaBody`.
#[allow(dead_code)]
pub fn new_plasma_body() -> PlasmaBody {
    PlasmaBody {
        particles: Vec::new(),
        damping: 0.999,
        temperature: 1.0,
    }
}

/// Add a charged particle.
#[allow(dead_code)]
pub fn plasma_add_particle(body: &mut PlasmaBody, pos: [f32; 3], charge: f32, mass: f32) {
    body.particles.push(PlasmaParticle {
        pos,
        vel: [0.0; 3],
        charge,
        mass: mass.max(1e-30),
    });
}

fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

/// Compute Coulomb force on particle `i` from particle `j`.
#[allow(dead_code)]
pub fn coulomb_force(pi: &PlasmaParticle, pj: &PlasmaParticle) -> [f32; 3] {
    let r = sub3(pi.pos, pj.pos);
    let dist = len3(r);
    if dist < 1e-9 {
        return [0.0; 3];
    }
    let force_mag = K_E * pi.charge * pj.charge / (dist * dist);
    scale3(r, force_mag / dist)
}

/// Step the plasma (n-body Coulomb, O(n²)).
#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn plasma_step(body: &mut PlasmaBody, dt: f32) {
    let n = body.particles.len();
    let mut forces = vec![[0.0f32; 3]; n];
    for i in 0..n {
        for j in (i + 1)..n {
            let f = coulomb_force(&body.particles[i], &body.particles[j]);
            forces[i] = add3(forces[i], f);
            forces[j] = add3(forces[j], scale3(f, -1.0));
        }
    }
    for i in 0..n {
        let a = scale3(forces[i], 1.0 / body.particles[i].mass);
        body.particles[i].vel = add3(body.particles[i].vel, scale3(a, dt));
        body.particles[i].vel = scale3(body.particles[i].vel, body.damping);
        body.particles[i].pos = add3(body.particles[i].pos, scale3(body.particles[i].vel, dt));
    }
}

/// Total kinetic energy.
#[allow(dead_code)]
pub fn plasma_kinetic_energy(body: &PlasmaBody) -> f32 {
    body.particles
        .iter()
        .map(|p| {
            let v2 = p.vel[0] * p.vel[0] + p.vel[1] * p.vel[1] + p.vel[2] * p.vel[2];
            0.5 * p.mass * v2
        })
        .sum()
}

/// Net charge.
#[allow(dead_code)]
pub fn plasma_net_charge(body: &PlasmaBody) -> f32 {
    body.particles.iter().map(|p| p.charge).sum()
}

/// Particle count.
#[allow(dead_code)]
pub fn plasma_count(body: &PlasmaBody) -> usize {
    body.particles.len()
}

/// Center of mass.
#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn plasma_center_of_mass(body: &PlasmaBody) -> [f32; 3] {
    let total_mass: f32 = body.particles.iter().map(|p| p.mass).sum();
    if total_mass < 1e-30 {
        return [0.0; 3];
    }
    let mut com = [0.0f32; 3];
    for p in &body.particles {
        for ax in 0..3 {
            com[ax] += p.pos[ax] * p.mass;
        }
    }
    for ax in 0..3 {
        com[ax] /= total_mass;
    }
    com
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_new_plasma() {
        let body = new_plasma_body();
        assert_eq!(plasma_count(&body), 0);
    }

    #[test]
    fn test_add_particle() {
        let mut body = new_plasma_body();
        plasma_add_particle(&mut body, [0.0; 3], 1.6e-19, 9.1e-31);
        assert_eq!(plasma_count(&body), 1);
    }

    #[test]
    fn test_net_charge() {
        let mut body = new_plasma_body();
        plasma_add_particle(&mut body, [0.0; 3], 1.0, 1.0);
        plasma_add_particle(&mut body, [1.0, 0.0, 0.0], -1.0, 1.0);
        assert!((plasma_net_charge(&body)).abs() < 1e-6);
    }

    #[test]
    fn test_coulomb_repulsion() {
        let p1 = PlasmaParticle {
            pos: [0.0; 3],
            vel: [0.0; 3],
            charge: 1.0,
            mass: 1.0,
        };
        let p2 = PlasmaParticle {
            pos: [1.0, 0.0, 0.0],
            vel: [0.0; 3],
            charge: 1.0,
            mass: 1.0,
        };
        let f = coulomb_force(&p1, &p2);
        assert!(f[0] < 0.0); // p1 pushed away from p2 (negative x direction)
    }

    #[test]
    fn test_coulomb_attraction() {
        let p1 = PlasmaParticle {
            pos: [0.0; 3],
            vel: [0.0; 3],
            charge: 1.0,
            mass: 1.0,
        };
        let p2 = PlasmaParticle {
            pos: [1.0, 0.0, 0.0],
            vel: [0.0; 3],
            charge: -1.0,
            mass: 1.0,
        };
        let f = coulomb_force(&p1, &p2);
        assert!(f[0] > 0.0); // attracted toward p2 (positive x)
    }

    #[test]
    fn test_step_no_crash() {
        let mut body = new_plasma_body();
        plasma_add_particle(&mut body, [0.0; 3], 1.0, 1.0);
        plasma_add_particle(&mut body, [1.0, 0.0, 0.0], -1.0, 1.0);
        plasma_step(&mut body, 1e-6);
    }

    #[test]
    fn test_kinetic_energy_after_step() {
        let mut body = new_plasma_body();
        plasma_add_particle(&mut body, [0.0; 3], 1.0, 1.0);
        plasma_add_particle(&mut body, [1.0, 0.0, 0.0], -1.0, 1.0);
        plasma_step(&mut body, 1e-6);
        assert!(plasma_kinetic_energy(&body) >= 0.0);
    }

    #[test]
    fn test_center_of_mass() {
        let mut body = new_plasma_body();
        plasma_add_particle(&mut body, [0.0; 3], 1.0, 1.0);
        plasma_add_particle(&mut body, [2.0, 0.0, 0.0], 1.0, 1.0);
        let com = plasma_center_of_mass(&body);
        assert!((com[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_pi_used() {
        let circle_area = PI * 1.0 * 1.0;
        assert!(circle_area > 3.0);
    }
}
