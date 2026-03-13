// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! XPBD v2: compliance-based distance + dihedral constraints.

/// An XPBD v2 particle.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct XpbdV2Particle {
    pub pos: [f32; 3],
    pub prev_pos: [f32; 3],
    pub vel: [f32; 3],
    pub inv_mass: f32,
    pub pinned: bool,
}

impl XpbdV2Particle {
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

/// An XPBD distance constraint with compliance.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct XpbdDistV2 {
    pub a: usize,
    pub b: usize,
    pub rest_len: f32,
    pub compliance: f32,
    pub lambda: f32,
}

/// An XPBD dihedral bend constraint.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct XpbdDihedralV2 {
    pub p0: usize,
    pub p1: usize,
    pub p2: usize,
    pub p3: usize,
    pub rest_angle: f32,
    pub compliance: f32,
    pub lambda: f32,
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
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

/// Project an XPBD distance constraint.
#[allow(dead_code)]
pub fn xpbd_v2_project_dist(particles: &mut [XpbdV2Particle], c: &mut XpbdDistV2, dt: f32) {
    let (a, b) = (c.a, c.b);
    if a >= particles.len() || b >= particles.len() {
        return;
    }
    let pa = particles[a].pos;
    let pb = particles[b].pos;
    let d = dist(pa, pb);
    if d < 1e-8 {
        return;
    }
    let constraint = d - c.rest_len;
    let alpha = c.compliance / (dt * dt);
    let ia = particles[a].inv_mass;
    let ib = particles[b].inv_mass;
    let w_sum = ia + ib;
    if w_sum < 1e-10 {
        return;
    }
    let d_lambda = (-constraint - alpha * c.lambda) / (w_sum + alpha);
    c.lambda += d_lambda;
    let dir = sub3(pb, pa);
    let s = if d > 1e-8 { 1.0 / d } else { 0.0 };
    let n = scale3(dir, s);
    if !particles[a].pinned {
        particles[a].pos = add3(pa, scale3(n, -ia * d_lambda));
    }
    if !particles[b].pinned {
        particles[b].pos = add3(pb, scale3(n, ib * d_lambda));
    }
}

/// Reset all lambdas to zero at start of substep.
#[allow(dead_code)]
pub fn xpbd_v2_reset_lambdas(dists: &mut [XpbdDistV2], dihedrals: &mut [XpbdDihedralV2]) {
    for c in dists.iter_mut() {
        c.lambda = 0.0;
    }
    for c in dihedrals.iter_mut() {
        c.lambda = 0.0;
    }
}

/// Predict positions with external forces.
#[allow(dead_code)]
pub fn xpbd_v2_predict(particles: &mut [XpbdV2Particle], gravity: [f32; 3], dt: f32) {
    for p in particles.iter_mut() {
        if p.pinned {
            continue;
        }
        p.vel = add3(p.vel, scale3(gravity, dt));
        p.prev_pos = p.pos;
        p.pos = add3(p.pos, scale3(p.vel, dt));
    }
}

/// Update velocities from corrected positions.
#[allow(dead_code)]
pub fn xpbd_v2_update_vel(particles: &mut [XpbdV2Particle], dt: f32) {
    let inv_dt = if dt > 1e-10 { 1.0 / dt } else { 0.0 };
    for p in particles.iter_mut() {
        p.vel = scale3(sub3(p.pos, p.prev_pos), inv_dt);
    }
}

/// Kinetic energy of all particles.
#[allow(dead_code)]
pub fn xpbd_v2_kinetic_energy(particles: &[XpbdV2Particle]) -> f32 {
    particles
        .iter()
        .map(|p| {
            if p.inv_mass < 1e-10 {
                return 0.0;
            }
            let m = 1.0 / p.inv_mass;
            let v2 = p.vel[0] * p.vel[0] + p.vel[1] * p.vel[1] + p.vel[2] * p.vel[2];
            0.5 * m * v2
        })
        .sum()
}

/// Number of distance constraints.
#[allow(dead_code)]
pub fn xpbd_v2_dist_count(dists: &[XpbdDistV2]) -> usize {
    dists.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_particles() -> Vec<XpbdV2Particle> {
        vec![
            XpbdV2Particle::new([0.0, 0.0, 0.0], 1.0),
            XpbdV2Particle::new([2.0, 0.0, 0.0], 1.0),
        ]
    }

    #[test]
    fn project_dist_reduces_stretch() {
        let mut ps = two_particles();
        let mut c = XpbdDistV2 {
            a: 0,
            b: 1,
            rest_len: 1.0,
            compliance: 0.0,
            lambda: 0.0,
        };
        xpbd_v2_project_dist(&mut ps, &mut c, 0.01);
        let d = dist(ps[0].pos, ps[1].pos);
        assert!(d < 2.0);
    }

    #[test]
    fn lambda_updates_after_project() {
        let mut ps = two_particles();
        let mut c = XpbdDistV2 {
            a: 0,
            b: 1,
            rest_len: 1.0,
            compliance: 0.0,
            lambda: 0.0,
        };
        xpbd_v2_project_dist(&mut ps, &mut c, 0.01);
        assert!(c.lambda.abs() > 1e-8);
    }

    #[test]
    fn reset_lambdas_clears() {
        let mut dists = vec![XpbdDistV2 {
            a: 0,
            b: 1,
            rest_len: 1.0,
            compliance: 0.0,
            lambda: 5.0,
        }];
        let mut dihedrals: Vec<XpbdDihedralV2> = vec![];
        xpbd_v2_reset_lambdas(&mut dists, &mut dihedrals);
        assert_eq!(dists[0].lambda, 0.0);
    }

    #[test]
    fn predict_moves_position() {
        let mut ps = vec![XpbdV2Particle::new([0.0, 0.0, 0.0], 1.0)];
        xpbd_v2_predict(&mut ps, [0.0, -9.8, 0.0], 0.01);
        assert!(ps[0].pos[1] < 0.0);
    }

    #[test]
    fn predict_skips_pinned() {
        let mut ps = vec![XpbdV2Particle::new([0.0, 0.0, 0.0], 1.0)];
        ps[0].pinned = true;
        xpbd_v2_predict(&mut ps, [0.0, -9.8, 0.0], 0.01);
        assert_eq!(ps[0].pos, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn update_vel_from_displacement() {
        let mut ps = vec![XpbdV2Particle::new([0.0, 0.0, 0.0], 1.0)];
        ps[0].prev_pos = [0.0, 0.0, 0.0];
        ps[0].pos = [0.0, 0.5, 0.0];
        xpbd_v2_update_vel(&mut ps, 0.1);
        assert!((ps[0].vel[1] - 5.0).abs() < 1e-3);
    }

    #[test]
    fn kinetic_energy_zero_at_rest() {
        let ps = vec![XpbdV2Particle::new([0.0, 0.0, 0.0], 1.0)];
        assert!((xpbd_v2_kinetic_energy(&ps)).abs() < 1e-8);
    }

    #[test]
    fn dist_count_correct() {
        let dists = vec![
            XpbdDistV2 {
                a: 0,
                b: 1,
                rest_len: 1.0,
                compliance: 0.0,
                lambda: 0.0,
            },
            XpbdDistV2 {
                a: 1,
                b: 2,
                rest_len: 1.0,
                compliance: 0.0,
                lambda: 0.0,
            },
        ];
        assert_eq!(xpbd_v2_dist_count(&dists), 2);
    }

    #[test]
    fn pinned_particle_not_moved() {
        let mut ps = two_particles();
        ps[0].pinned = true;
        let mut c = XpbdDistV2 {
            a: 0,
            b: 1,
            rest_len: 1.0,
            compliance: 0.0,
            lambda: 0.0,
        };
        xpbd_v2_project_dist(&mut ps, &mut c, 0.01);
        assert_eq!(ps[0].pos, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn compliance_reduces_correction() {
        let mut ps_stiff = two_particles();
        let mut c_stiff = XpbdDistV2 {
            a: 0,
            b: 1,
            rest_len: 1.0,
            compliance: 0.0,
            lambda: 0.0,
        };
        xpbd_v2_project_dist(&mut ps_stiff, &mut c_stiff, 0.01);

        let mut ps_soft = two_particles();
        let mut c_soft = XpbdDistV2 {
            a: 0,
            b: 1,
            rest_len: 1.0,
            compliance: 1e3,
            lambda: 0.0,
        };
        xpbd_v2_project_dist(&mut ps_soft, &mut c_soft, 0.01);

        let d_stiff = dist(ps_stiff[0].pos, ps_stiff[1].pos);
        let d_soft = dist(ps_soft[0].pos, ps_soft[1].pos);
        assert!(d_soft > d_stiff);
    }
}
