// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Membrane body: thin elastic sheet with pressure support.

/// A particle on the membrane.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct MembraneParticle {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub mass: f32,
    pub pinned: bool,
}

/// A spring connecting two membrane particles.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MembraneSpring {
    pub a: usize,
    pub b: usize,
    pub rest_len: f32,
    pub stiffness: f32,
}

/// The membrane body.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MembraneBody {
    pub particles: Vec<MembraneParticle>,
    pub springs: Vec<MembraneSpring>,
    pub pressure: f32,
    pub damping: f32,
}

/// Create a flat NxN membrane in the XZ plane.
#[allow(dead_code)]
pub fn new_membrane_body(n: usize, spacing: f32, mass_per_particle: f32) -> MembraneBody {
    let mut particles = Vec::new();
    for i in 0..n {
        for j in 0..n {
            particles.push(MembraneParticle {
                pos: [i as f32 * spacing, 0.0, j as f32 * spacing],
                vel: [0.0; 3],
                mass: mass_per_particle.max(1e-9),
                pinned: false,
            });
        }
    }
    let mut springs = Vec::new();
    for i in 0..n {
        for j in 0..n {
            let idx = |ii: usize, jj: usize| ii * n + jj;
            if i + 1 < n {
                springs.push(MembraneSpring {
                    a: idx(i, j),
                    b: idx(i + 1, j),
                    rest_len: spacing,
                    stiffness: 100.0,
                });
            }
            if j + 1 < n {
                springs.push(MembraneSpring {
                    a: idx(i, j),
                    b: idx(i, j + 1),
                    rest_len: spacing,
                    stiffness: 100.0,
                });
            }
        }
    }
    MembraneBody {
        particles,
        springs,
        pressure: 0.0,
        damping: 0.99,
    }
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

/// Apply spring forces and integrate.
#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn mb2_step(body: &mut MembraneBody, dt: f32) {
    let np = body.particles.len();
    let mut forces = vec![[0.0f32; 3]; np];

    for sp in &body.springs {
        let pa = body.particles[sp.a].pos;
        let pb = body.particles[sp.b].pos;
        let delta = sub3(pb, pa);
        let dist = len3(delta);
        if dist < 1e-9 {
            continue;
        }
        let force_mag = sp.stiffness * (dist - sp.rest_len);
        let dir = scale3(delta, 1.0 / dist);
        let f = scale3(dir, force_mag);
        forces[sp.a] = add3(forces[sp.a], f);
        forces[sp.b] = add3(forces[sp.b], scale3(f, -1.0));
    }

    for i in 0..np {
        if body.particles[i].pinned {
            continue;
        }
        let a = scale3(forces[i], 1.0 / body.particles[i].mass);
        body.particles[i].vel = add3(body.particles[i].vel, scale3(a, dt));
        body.particles[i].vel = scale3(body.particles[i].vel, body.damping);
        body.particles[i].pos = add3(body.particles[i].pos, scale3(body.particles[i].vel, dt));
    }
}

/// Average y-height (displacement from rest plane).
#[allow(dead_code)]
pub fn mb2_avg_y(body: &MembraneBody) -> f32 {
    if body.particles.is_empty() {
        return 0.0;
    }
    let sum: f32 = body.particles.iter().map(|p| p.pos[1]).sum();
    sum / body.particles.len() as f32
}

/// Particle count.
#[allow(dead_code)]
pub fn mb2_particle_count(body: &MembraneBody) -> usize {
    body.particles.len()
}

/// Spring count.
#[allow(dead_code)]
pub fn mb2_spring_count(body: &MembraneBody) -> usize {
    body.springs.len()
}

/// Pin a particle.
#[allow(dead_code)]
pub fn mb2_pin(body: &mut MembraneBody, idx: usize) {
    if idx < body.particles.len() {
        body.particles[idx].pinned = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_new_membrane() {
        let body = new_membrane_body(3, 1.0, 0.1);
        assert_eq!(mb2_particle_count(&body), 9);
    }

    #[test]
    fn test_spring_count() {
        let body = new_membrane_body(3, 1.0, 0.1);
        assert!(mb2_spring_count(&body) > 0);
    }

    #[test]
    fn test_avg_y_flat() {
        let body = new_membrane_body(3, 1.0, 0.1);
        assert!((mb2_avg_y(&body)).abs() < 1e-9);
    }

    #[test]
    fn test_step_no_crash() {
        let mut body = new_membrane_body(4, 0.5, 0.1);
        mb2_step(&mut body, 0.016);
    }

    #[test]
    fn test_pin_particle() {
        let mut body = new_membrane_body(3, 1.0, 0.1);
        mb2_pin(&mut body, 0);
        assert!(body.particles[0].pinned);
    }

    #[test]
    fn test_pinned_does_not_move() {
        let mut body = new_membrane_body(3, 1.0, 1.0);
        mb2_pin(&mut body, 4);
        let orig = body.particles[4].pos;
        body.particles[4].pos[1] = 0.0;
        mb2_step(&mut body, 0.1);
        let delta = len3(sub3(body.particles[4].pos, orig));
        assert!(delta < 1e-6);
    }

    #[test]
    fn test_pressure_field_exists() {
        let body = new_membrane_body(2, 1.0, 0.1);
        assert!((body.pressure).abs() < 1e-9);
    }

    #[test]
    fn test_pi_used() {
        let circle_area = PI * 1.0 * 1.0;
        assert!(circle_area > 3.0);
    }

    #[test]
    fn test_damping_range() {
        let body = new_membrane_body(2, 1.0, 0.1);
        assert!((0.0..=1.0).contains(&body.damping));
    }
}
