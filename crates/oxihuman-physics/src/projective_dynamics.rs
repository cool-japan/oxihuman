// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Projective dynamics (local/global) for cloth.

/// A particle for projective dynamics.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PdParticle {
    pub pos: [f32; 3],
    pub prev_pos: [f32; 3],
    pub vel: [f32; 3],
    pub inv_mass: f32,
    pub pinned: bool,
}

impl PdParticle {
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

/// A spring/edge constraint for projective dynamics.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PdSpringConstraint {
    pub a: usize,
    pub b: usize,
    pub rest_len: f32,
    pub stiffness: f32,
}

fn pd_sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn pd_add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn pd_scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn pd_norm3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Local step: compute projection for each constraint.
/// Returns the target position for each constraint endpoint.
#[allow(dead_code)]
pub fn pd_local_step(
    particles: &[PdParticle],
    constraints: &[PdSpringConstraint],
) -> Vec<([f32; 3], [f32; 3])> {
    constraints
        .iter()
        .map(|c| {
            if c.a >= particles.len() || c.b >= particles.len() {
                return ([0.0; 3], [0.0; 3]);
            }
            let pa = particles[c.a].pos;
            let pb = particles[c.b].pos;
            let d = pd_sub3(pb, pa);
            let len = pd_norm3(d);
            if len < 1e-8 {
                return (pa, pb);
            }
            let n = pd_scale3(d, 1.0 / len);
            let mid = pd_scale3(pd_add3(pa, pb), 0.5);
            let ha = pd_scale3(n, -c.rest_len * 0.5);
            let hb = pd_scale3(n, c.rest_len * 0.5);
            (pd_add3(mid, ha), pd_add3(mid, hb))
        })
        .collect()
}

/// Global step: weighted sum of projections.
#[allow(dead_code)]
pub fn pd_global_step(
    particles: &mut [PdParticle],
    constraints: &[PdSpringConstraint],
    projections: &[([f32; 3], [f32; 3])],
    dt: f32,
) {
    let n = particles.len();
    let mut acc = vec![[0.0f32; 3]; n];
    let mut weight = vec![0.0f32; n];

    for (c, (pa, pb)) in constraints.iter().zip(projections.iter()) {
        if c.a < n {
            for k in 0..3 {
                acc[c.a][k] += c.stiffness * pa[k];
            }
            weight[c.a] += c.stiffness;
        }
        if c.b < n {
            for k in 0..3 {
                acc[c.b][k] += c.stiffness * pb[k];
            }
            weight[c.b] += c.stiffness;
        }
    }

    for (i, p) in particles.iter_mut().enumerate() {
        if p.pinned || weight[i] < 1e-10 {
            continue;
        }
        let alpha = 1.0 / p.inv_mass.max(1e-10) / (dt * dt);
        let denom = weight[i] + alpha;
        let inertia = pd_add3(p.prev_pos, pd_scale3(p.vel, dt));
        for k in 0..3 {
            p.pos[k] = (acc[i][k] + alpha * inertia[k]) / denom;
        }
    }
}

/// Predict positions.
#[allow(dead_code)]
pub fn pd_predict(particles: &mut [PdParticle], gravity: [f32; 3], dt: f32) {
    for p in particles.iter_mut() {
        if p.pinned {
            continue;
        }
        p.vel = pd_add3(p.vel, pd_scale3(gravity, dt));
        p.prev_pos = p.pos;
        p.pos = pd_add3(p.pos, pd_scale3(p.vel, dt));
    }
}

/// Update velocities.
#[allow(dead_code)]
pub fn pd_update_vel(particles: &mut [PdParticle], dt: f32) {
    let inv_dt = if dt > 1e-10 { 1.0 / dt } else { 0.0 };
    for p in particles.iter_mut() {
        p.vel = pd_scale3(pd_sub3(p.pos, p.prev_pos), inv_dt);
    }
}

/// Total kinetic energy.
#[allow(dead_code)]
pub fn pd_kinetic_energy(particles: &[PdParticle]) -> f32 {
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

/// Number of constraints.
#[allow(dead_code)]
pub fn pd_constraint_count(constraints: &[PdSpringConstraint]) -> usize {
    constraints.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_particles() -> Vec<PdParticle> {
        vec![
            PdParticle::new([0.0, 0.0, 0.0], 1.0),
            PdParticle::new([2.0, 0.0, 0.0], 1.0),
        ]
    }

    fn make_constraint() -> PdSpringConstraint {
        PdSpringConstraint {
            a: 0,
            b: 1,
            rest_len: 1.0,
            stiffness: 1.0,
        }
    }

    #[test]
    fn local_step_returns_projections() {
        let ps = make_particles();
        let cs = vec![make_constraint()];
        let proj = pd_local_step(&ps, &cs);
        assert_eq!(proj.len(), 1);
    }

    #[test]
    fn local_step_target_distance_is_rest_len() {
        let ps = make_particles();
        let cs = vec![make_constraint()];
        let proj = pd_local_step(&ps, &cs);
        let (pa, pb) = proj[0];
        let d = pd_sub3(pb, pa);
        let len = pd_norm3(d);
        assert!((len - 1.0).abs() < 1e-4);
    }

    #[test]
    fn predict_moves_particle() {
        let mut ps = make_particles();
        pd_predict(&mut ps, [0.0, -9.8, 0.0], 0.01);
        assert!(ps[0].pos[1] < 0.0);
    }

    #[test]
    fn predict_skips_pinned() {
        let mut ps = make_particles();
        ps[0].pinned = true;
        pd_predict(&mut ps, [0.0, -9.8, 0.0], 0.01);
        assert_eq!(ps[0].pos, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn update_vel_correct() {
        let mut ps = make_particles();
        ps[0].prev_pos = [0.0, 0.0, 0.0];
        ps[0].pos = [1.0, 0.0, 0.0];
        pd_update_vel(&mut ps, 0.1);
        assert!((ps[0].vel[0] - 10.0).abs() < 1e-3);
    }

    #[test]
    fn kinetic_energy_zero_at_rest() {
        let ps = make_particles();
        assert!(pd_kinetic_energy(&ps) < 1e-8);
    }

    #[test]
    fn constraint_count_correct() {
        let cs = vec![make_constraint(), make_constraint()];
        assert_eq!(pd_constraint_count(&cs), 2);
    }

    #[test]
    fn global_step_runs_without_panic() {
        let mut ps = make_particles();
        let cs = vec![make_constraint()];
        let proj = pd_local_step(&ps, &cs);
        pd_global_step(&mut ps, &cs, &proj, 0.01);
    }

    #[test]
    fn new_particle_has_zero_vel() {
        let p = PdParticle::new([1.0, 2.0, 3.0], 1.0);
        assert_eq!(p.vel, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn pd_local_step_empty_constraints() {
        let ps = make_particles();
        let proj = pd_local_step(&ps, &[]);
        assert!(proj.is_empty());
    }
}
