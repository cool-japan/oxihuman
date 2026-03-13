// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! PBD v2: distance + bend + volume constraints.

/// A PBD v2 particle.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PbdV2Particle {
    pub pos: [f32; 3],
    pub prev_pos: [f32; 3],
    pub vel: [f32; 3],
    pub inv_mass: f32,
    pub pinned: bool,
}

impl PbdV2Particle {
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

/// A distance constraint between two particles.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DistConstraintV2 {
    pub a: usize,
    pub b: usize,
    pub rest_len: f32,
    pub stiffness: f32,
}

/// A bend constraint (dihedral) for cloth.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BendConstraintV2 {
    pub a: usize,
    pub b: usize,
    pub c: usize,
    pub rest_angle: f32,
    pub stiffness: f32,
}

/// A volume constraint for a tetrahedral cluster.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VolumeConstraintV2 {
    pub indices: [usize; 4],
    pub rest_volume: f32,
    pub stiffness: f32,
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
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

fn norm3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn tet_volume(p: [[f32; 3]; 4]) -> f32 {
    let v1 = sub3(p[1], p[0]);
    let v2 = sub3(p[2], p[0]);
    let v3 = sub3(p[3], p[0]);
    let cross = [
        v2[1] * v3[2] - v2[2] * v3[1],
        v2[2] * v3[0] - v2[0] * v3[2],
        v2[0] * v3[1] - v2[1] * v3[0],
    ];
    (v1[0] * cross[0] + v1[1] * cross[1] + v1[2] * cross[2]).abs() / 6.0
}

/// Project a distance constraint.
#[allow(dead_code)]
pub fn project_dist_v2(particles: &mut [PbdV2Particle], c: &DistConstraintV2) {
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
    let diff = (d - c.rest_len) / d;
    let ia = particles[a].inv_mass;
    let ib = particles[b].inv_mass;
    let total = ia + ib;
    if total < 1e-10 {
        return;
    }
    let dir = sub3(pb, pa);
    let wa = ia / total * c.stiffness * diff;
    let wb = ib / total * c.stiffness * diff;
    if !particles[a].pinned {
        particles[a].pos = add3(pa, scale3(dir, wa));
    }
    if !particles[b].pinned {
        particles[b].pos = add3(pb, scale3(dir, -wb));
    }
}

/// Project a volume constraint.
#[allow(dead_code)]
pub fn project_volume_v2(particles: &mut [PbdV2Particle], c: &VolumeConstraintV2) {
    let idx = c.indices;
    if idx.iter().any(|&i| i >= particles.len()) {
        return;
    }
    let ps = [
        particles[idx[0]].pos,
        particles[idx[1]].pos,
        particles[idx[2]].pos,
        particles[idx[3]].pos,
    ];
    let vol = tet_volume(ps);
    let diff = vol - c.rest_volume;
    if diff.abs() < 1e-10 {
        return;
    }
    let correction = scale3([1.0, 1.0, 1.0], -diff * c.stiffness * 0.1);
    for &i in &idx {
        if !particles[i].pinned {
            particles[i].pos = add3(particles[i].pos, correction);
        }
    }
}

/// Integrate particle positions (one step).
#[allow(dead_code)]
pub fn pbd_v2_integrate(particles: &mut [PbdV2Particle], gravity: [f32; 3], dt: f32) {
    for p in particles.iter_mut() {
        if p.pinned {
            continue;
        }
        p.vel = add3(p.vel, scale3(gravity, dt));
        p.prev_pos = p.pos;
        p.pos = add3(p.pos, scale3(p.vel, dt));
    }
}

/// Update velocities from position change.
#[allow(dead_code)]
pub fn pbd_v2_update_vel(particles: &mut [PbdV2Particle], dt: f32) {
    let inv_dt = if dt > 1e-10 { 1.0 / dt } else { 0.0 };
    for p in particles.iter_mut() {
        p.vel = scale3(sub3(p.pos, p.prev_pos), inv_dt);
    }
}

/// Compute total kinetic energy.
#[allow(dead_code)]
pub fn pbd_v2_kinetic_energy(particles: &[PbdV2Particle]) -> f32 {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn two_particles() -> Vec<PbdV2Particle> {
        vec![
            PbdV2Particle::new([0.0, 0.0, 0.0], 1.0),
            PbdV2Particle::new([2.0, 0.0, 0.0], 1.0),
        ]
    }

    #[test]
    fn project_dist_moves_particles_toward_rest() {
        let mut ps = two_particles();
        let c = DistConstraintV2 {
            a: 0,
            b: 1,
            rest_len: 1.0,
            stiffness: 1.0,
        };
        project_dist_v2(&mut ps, &c);
        let d = dist(ps[0].pos, ps[1].pos);
        assert!(d < 2.0);
    }

    #[test]
    fn pinned_particle_doesnt_move() {
        let mut ps = two_particles();
        ps[0].pinned = true;
        let c = DistConstraintV2 {
            a: 0,
            b: 1,
            rest_len: 1.0,
            stiffness: 1.0,
        };
        project_dist_v2(&mut ps, &c);
        assert_eq!(ps[0].pos, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn integrate_applies_gravity() {
        let mut ps = vec![PbdV2Particle::new([0.0, 0.0, 0.0], 1.0)];
        pbd_v2_integrate(&mut ps, [0.0, -9.8, 0.0], 0.01);
        assert!(ps[0].pos[1] < 0.0);
    }

    #[test]
    fn integrate_skips_pinned() {
        let mut ps = vec![PbdV2Particle::new([0.0, 0.0, 0.0], 1.0)];
        ps[0].pinned = true;
        pbd_v2_integrate(&mut ps, [0.0, -9.8, 0.0], 0.01);
        assert_eq!(ps[0].pos, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn update_vel_from_position_change() {
        let mut ps = vec![PbdV2Particle::new([0.0, 0.0, 0.0], 1.0)];
        ps[0].prev_pos = [0.0, 0.0, 0.0];
        ps[0].pos = [0.0, 1.0, 0.0];
        pbd_v2_update_vel(&mut ps, 0.1);
        assert!((ps[0].vel[1] - 10.0).abs() < 1e-3);
    }

    #[test]
    fn kinetic_energy_positive_for_moving_particle() {
        let mut p = PbdV2Particle::new([0.0, 0.0, 0.0], 1.0);
        p.vel = [1.0, 0.0, 0.0];
        let ke = pbd_v2_kinetic_energy(&[p]);
        assert!(ke > 0.0);
    }

    #[test]
    fn kinetic_energy_zero_at_rest() {
        let ps = vec![PbdV2Particle::new([0.0, 0.0, 0.0], 1.0)];
        assert!((pbd_v2_kinetic_energy(&ps)).abs() < 1e-8);
    }

    #[test]
    fn volume_constraint_runs_without_panic() {
        let mut ps: Vec<PbdV2Particle> = (0..4)
            .map(|i| PbdV2Particle::new([i as f32, 0.0, 0.0], 1.0))
            .collect();
        let c = VolumeConstraintV2 {
            indices: [0, 1, 2, 3],
            rest_volume: 1.0,
            stiffness: 0.1,
        };
        project_volume_v2(&mut ps, &c);
    }

    #[test]
    fn dist_constraint_at_rest_no_change() {
        let mut ps = two_particles();
        let c = DistConstraintV2 {
            a: 0,
            b: 1,
            rest_len: 2.0,
            stiffness: 1.0,
        };
        let before = ps[0].pos;
        project_dist_v2(&mut ps, &c);
        let after = ps[0].pos;
        assert!((before[0] - after[0]).abs() < 1e-5);
    }

    #[test]
    fn particle_new_sets_zero_vel() {
        let p = PbdV2Particle::new([1.0, 2.0, 3.0], 1.0);
        assert_eq!(p.vel, [0.0, 0.0, 0.0]);
    }
}
