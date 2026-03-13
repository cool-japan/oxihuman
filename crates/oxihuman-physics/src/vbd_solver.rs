// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Vertex block descent (VBD) solver for elastic bodies.

/// A VBD particle.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VbdParticle {
    pub pos: [f32; 3],
    pub prev_pos: [f32; 3],
    pub vel: [f32; 3],
    pub inv_mass: f32,
    pub pinned: bool,
}

impl VbdParticle {
    #[allow(dead_code)]
    pub fn new(pos: [f32; 3], inv_mass: f32) -> Self {
        Self {
            pos,
            prev_pos: pos,
            vel: [0.0; 3],
            inv_mass,
            pinned: false,
        }
    }
}

/// A spring (edge) constraint in VBD.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VbdSpring {
    pub a: usize,
    pub b: usize,
    pub rest_len: f32,
    pub stiffness: f32,
}

fn vbd_sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn vbd_add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn vbd_scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn vbd_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// VBD local solve for vertex `vi`: minimizes elastic energy of all springs incident to `vi`.
#[allow(dead_code)]
pub fn vbd_local_solve(particles: &mut [VbdParticle], springs: &[VbdSpring], vi: usize, dt: f32) {
    if vi >= particles.len() || particles[vi].pinned {
        return;
    }
    let inv_m = particles[vi].inv_mass;
    if inv_m < 1e-10 {
        return;
    }
    let alpha = inv_m / (dt * dt);
    let y_i = vbd_add3(particles[vi].prev_pos, vbd_scale3(particles[vi].vel, dt));

    let mut numerator = vbd_scale3(y_i, alpha);
    let mut denominator = alpha;

    let pos_vi = particles[vi].pos;
    for s in springs {
        let other = if s.a == vi {
            s.b
        } else if s.b == vi {
            s.a
        } else {
            continue;
        };
        if other >= particles.len() {
            continue;
        }
        let pos_j = particles[other].pos;
        let d = vbd_sub3(pos_vi, pos_j);
        let len = vbd_len(d);
        if len < 1e-8 {
            continue;
        }
        let n = vbd_scale3(d, 1.0 / len);
        let proj = vbd_add3(pos_j, vbd_scale3(n, s.rest_len));
        for k in 0..3 {
            numerator[k] += s.stiffness * proj[k];
        }
        denominator += s.stiffness;
    }

    if denominator > 1e-10 {
        let new_pos = vbd_scale3(numerator, 1.0 / denominator);
        particles[vi].pos = new_pos;
    }
}

/// Run one full VBD iteration over all non-pinned vertices.
#[allow(dead_code)]
pub fn vbd_step(particles: &mut [VbdParticle], springs: &[VbdSpring], dt: f32) {
    let n = particles.len();
    for vi in 0..n {
        vbd_local_solve(particles, springs, vi, dt);
    }
}

/// Predict step: apply external forces and store inertia target.
#[allow(dead_code)]
pub fn vbd_predict(particles: &mut [VbdParticle], gravity: [f32; 3], dt: f32) {
    for p in particles.iter_mut() {
        if p.pinned {
            continue;
        }
        p.vel = vbd_add3(p.vel, vbd_scale3(gravity, dt));
        p.prev_pos = p.pos;
        p.pos = vbd_add3(p.pos, vbd_scale3(p.vel, dt));
    }
}

/// Update velocities from corrected positions.
#[allow(dead_code)]
pub fn vbd_update_vel(particles: &mut [VbdParticle], dt: f32) {
    let inv_dt = if dt > 1e-10 { 1.0 / dt } else { 0.0 };
    for p in particles.iter_mut() {
        p.vel = vbd_scale3(vbd_sub3(p.pos, p.prev_pos), inv_dt);
    }
}

/// Total kinetic energy.
#[allow(dead_code)]
pub fn vbd_kinetic_energy(particles: &[VbdParticle]) -> f32 {
    particles
        .iter()
        .map(|p| {
            if p.inv_mass < 1e-10 {
                return 0.0;
            }
            let m = 1.0 / p.inv_mass;
            let v2: f32 = p.vel.iter().map(|x| x * x).sum();
            0.5 * m * v2
        })
        .sum()
}

/// Spring potential energy.
#[allow(dead_code)]
pub fn vbd_spring_energy(particles: &[VbdParticle], springs: &[VbdSpring]) -> f32 {
    springs
        .iter()
        .map(|s| {
            if s.a >= particles.len() || s.b >= particles.len() {
                return 0.0;
            }
            let d = vbd_sub3(particles[s.a].pos, particles[s.b].pos);
            let stretch = vbd_len(d) - s.rest_len;
            0.5 * s.stiffness * stretch * stretch
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_particles() -> (Vec<VbdParticle>, Vec<VbdSpring>) {
        let ps = vec![
            VbdParticle::new([0.0, 0.0, 0.0], 1.0),
            VbdParticle::new([2.0, 0.0, 0.0], 1.0),
        ];
        let ss = vec![VbdSpring {
            a: 0,
            b: 1,
            rest_len: 1.0,
            stiffness: 10.0,
        }];
        (ps, ss)
    }

    #[test]
    fn predict_moves_particle() {
        let (mut ps, _) = two_particles();
        vbd_predict(&mut ps, [0.0, -9.8, 0.0], 0.01);
        assert!(ps[0].pos[1] < 0.0);
    }

    #[test]
    fn predict_skips_pinned() {
        let (mut ps, _) = two_particles();
        ps[0].pinned = true;
        vbd_predict(&mut ps, [0.0, -9.8, 0.0], 0.01);
        assert_eq!(ps[0].pos, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn vbd_step_runs_without_panic() {
        let (mut ps, ss) = two_particles();
        vbd_step(&mut ps, &ss, 0.01);
    }

    #[test]
    fn local_solve_moves_toward_rest() {
        let (mut ps, ss) = two_particles();
        let before = ps[0].pos[0];
        vbd_local_solve(&mut ps, &ss, 0, 0.01);
        let after = ps[0].pos[0];
        assert!(after > before);
    }

    #[test]
    fn kinetic_energy_zero_at_rest() {
        let (ps, _) = two_particles();
        assert!(vbd_kinetic_energy(&ps) < 1e-8);
    }

    #[test]
    fn spring_energy_positive_when_stretched() {
        let (ps, ss) = two_particles();
        let e = vbd_spring_energy(&ps, &ss);
        assert!(e > 0.0);
    }

    #[test]
    fn spring_energy_zero_at_rest_len() {
        let ps = vec![
            VbdParticle::new([0.0, 0.0, 0.0], 1.0),
            VbdParticle::new([1.0, 0.0, 0.0], 1.0),
        ];
        let ss = vec![VbdSpring {
            a: 0,
            b: 1,
            rest_len: 1.0,
            stiffness: 10.0,
        }];
        assert!(vbd_spring_energy(&ps, &ss) < 1e-6);
    }

    #[test]
    fn update_vel_from_displacement() {
        let (mut ps, _) = two_particles();
        ps[0].prev_pos = [0.0, 0.0, 0.0];
        ps[0].pos = [0.5, 0.0, 0.0];
        vbd_update_vel(&mut ps, 0.1);
        assert!((ps[0].vel[0] - 5.0).abs() < 1e-3);
    }

    #[test]
    fn pinned_particle_not_moved_by_local_solve() {
        let (mut ps, ss) = two_particles();
        ps[0].pinned = true;
        vbd_local_solve(&mut ps, &ss, 0, 0.01);
        assert_eq!(ps[0].pos, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn particle_new_zero_vel() {
        let p = VbdParticle::new([1.0, 0.0, 0.0], 1.0);
        assert_eq!(p.vel, [0.0, 0.0, 0.0]);
    }
}
