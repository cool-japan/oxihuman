// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct DemParticle {
    pub pos: [f32; 2],
    pub vel: [f32; 2],
    pub radius: f32,
    pub mass: f32,
}

pub fn new_dem_particle(pos: [f32; 2], radius: f32, mass: f32) -> DemParticle {
    DemParticle {
        pos,
        vel: [0.0, 0.0],
        radius,
        mass,
    }
}

pub fn dem_overlap(a: &DemParticle, b: &DemParticle) -> f32 {
    let dx = a.pos[0] - b.pos[0];
    let dy = a.pos[1] - b.pos[1];
    let dist = (dx * dx + dy * dy).sqrt();
    a.radius + b.radius - dist
}

pub fn dem_contact_force(a: &DemParticle, b: &DemParticle, stiffness: f32) -> [f32; 2] {
    let overlap = dem_overlap(a, b);
    if overlap <= 0.0 {
        return [0.0, 0.0];
    }
    let dx = a.pos[0] - b.pos[0];
    let dy = a.pos[1] - b.pos[1];
    let dist = (dx * dx + dy * dy).sqrt().max(1e-10);
    let nx = dx / dist;
    let ny = dy / dist;
    let f = stiffness * overlap;
    [f * nx, f * ny]
}

pub fn dem_step(p: &mut DemParticle, force: [f32; 2], dt: f32) {
    let inv_mass = if p.mass > 0.0 { 1.0 / p.mass } else { 0.0 };
    p.vel[0] += force[0] * inv_mass * dt;
    p.vel[1] += force[1] * inv_mass * dt;
    p.pos[0] += p.vel[0] * dt;
    p.pos[1] += p.vel[1] * dt;
}

pub fn dem_kinetic_energy(p: &DemParticle) -> f32 {
    0.5 * p.mass * (p.vel[0] * p.vel[0] + p.vel[1] * p.vel[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_dem_particle() {
        /* create DEM particle with pos, radius, mass */
        let p = new_dem_particle([0.0, 0.0], 0.5, 1.0);
        assert_eq!(p.radius, 0.5);
        assert_eq!(p.mass, 1.0);
    }

    #[test]
    fn test_dem_overlap_contact() {
        /* overlapping particles have positive overlap */
        let a = new_dem_particle([0.0, 0.0], 0.5, 1.0);
        let b = new_dem_particle([0.8, 0.0], 0.5, 1.0);
        let ov = dem_overlap(&a, &b);
        assert!(ov > 0.0);
    }

    #[test]
    fn test_dem_overlap_no_contact() {
        /* separated particles have negative overlap */
        let a = new_dem_particle([0.0, 0.0], 0.3, 1.0);
        let b = new_dem_particle([2.0, 0.0], 0.3, 1.0);
        let ov = dem_overlap(&a, &b);
        assert!(ov < 0.0);
    }

    #[test]
    fn test_dem_contact_force_direction() {
        /* contact force pushes particles apart */
        let a = new_dem_particle([0.0, 0.0], 0.5, 1.0);
        let b = new_dem_particle([0.8, 0.0], 0.5, 1.0);
        let f = dem_contact_force(&a, &b, 100.0);
        /* a is to the left, force on a should push left (negative x) */
        assert!(f[0] < 0.0);
    }

    #[test]
    fn test_dem_step_moves_particle() {
        /* step integrates velocity and position */
        let mut p = new_dem_particle([0.0, 0.0], 0.5, 1.0);
        dem_step(&mut p, [1.0, 0.0], 0.1);
        assert!(p.vel[0] > 0.0);
        assert!(p.pos[0] > 0.0);
    }

    #[test]
    fn test_dem_kinetic_energy() {
        /* kinetic energy is 0.5 * m * v^2 */
        let mut p = new_dem_particle([0.0, 0.0], 0.5, 2.0);
        dem_step(&mut p, [2.0, 0.0], 1.0);
        let ke = dem_kinetic_energy(&p);
        assert!(ke > 0.0);
    }
}
