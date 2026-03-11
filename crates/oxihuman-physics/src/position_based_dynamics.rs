// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[allow(dead_code)]
pub struct PBDParticle {
    pub pos: [f32; 3],
    pub prev_pos: [f32; 3],
    pub inv_mass: f32,
}

#[allow(dead_code)]
pub struct PBDSystem {
    pub particles: Vec<PBDParticle>,
    pub gravity: [f32; 3],
    pub substeps: usize,
}

#[allow(dead_code)]
pub fn new_pbd_system(substeps: usize) -> PBDSystem {
    PBDSystem {
        particles: Vec::new(),
        gravity: [0.0, -9.81, 0.0],
        substeps,
    }
}

#[allow(dead_code)]
pub fn pbd_add_particle(s: &mut PBDSystem, pos: [f32; 3], mass: f32) -> usize {
    let inv_mass = if mass > 1e-10 { 1.0 / mass } else { 0.0 };
    let idx = s.particles.len();
    s.particles.push(PBDParticle { pos, prev_pos: pos, inv_mass });
    idx
}

#[allow(dead_code)]
pub fn pbd_step(s: &mut PBDSystem, dt: f32) {
    let dt2 = dt * dt;
    for p in s.particles.iter_mut() {
        if p.inv_mass < 1e-10 {
            continue;
        }
        let new_pos = [
            p.pos[0] + s.gravity[0] * dt2,
            p.pos[1] + s.gravity[1] * dt2,
            p.pos[2] + s.gravity[2] * dt2,
        ];
        p.prev_pos = p.pos;
        p.pos = new_pos;
    }
}

#[allow(dead_code)]
pub fn pbd_particle_count(s: &PBDSystem) -> usize {
    s.particles.len()
}

#[allow(dead_code)]
pub fn pbd_pin(s: &mut PBDSystem, i: usize) {
    s.particles[i].inv_mass = 0.0;
}

#[allow(dead_code)]
pub fn pbd_kinetic_energy(s: &PBDSystem, dt: f32) -> f32 {
    if dt < 1e-10 {
        return 0.0;
    }
    s.particles.iter().map(|p| {
        if p.inv_mass < 1e-10 {
            return 0.0;
        }
        let mass = 1.0 / p.inv_mass;
        let vx = (p.pos[0] - p.prev_pos[0]) / dt;
        let vy = (p.pos[1] - p.prev_pos[1]) / dt;
        let vz = (p.pos[2] - p.prev_pos[2]) / dt;
        0.5 * mass * (vx * vx + vy * vy + vz * vz)
    }).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_particle() {
        let mut s = new_pbd_system(4);
        pbd_add_particle(&mut s, [0.0, 0.0, 0.0], 1.0);
        assert_eq!(pbd_particle_count(&s), 1);
    }

    #[test]
    fn test_particle_count() {
        let mut s = new_pbd_system(1);
        pbd_add_particle(&mut s, [0.0; 3], 1.0);
        pbd_add_particle(&mut s, [1.0, 0.0, 0.0], 1.0);
        assert_eq!(pbd_particle_count(&s), 2);
    }

    #[test]
    fn test_step_moves_particle() {
        let mut s = new_pbd_system(1);
        pbd_add_particle(&mut s, [0.0, 10.0, 0.0], 1.0);
        let old_y = s.particles[0].pos[1];
        pbd_step(&mut s, 0.1);
        let new_y = s.particles[0].pos[1];
        assert!(new_y < old_y);
    }

    #[test]
    fn test_pin_stops_movement() {
        let mut s = new_pbd_system(1);
        pbd_add_particle(&mut s, [0.0, 10.0, 0.0], 1.0);
        pbd_pin(&mut s, 0);
        pbd_step(&mut s, 0.1);
        assert!((s.particles[0].pos[1] - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_kinetic_energy_after_step() {
        let mut s = new_pbd_system(1);
        pbd_add_particle(&mut s, [0.0, 10.0, 0.0], 1.0);
        pbd_step(&mut s, 0.1);
        let ke = pbd_kinetic_energy(&s, 0.1);
        assert!(ke >= 0.0);
    }

    #[test]
    fn test_substeps_stored() {
        let s = new_pbd_system(8);
        assert_eq!(s.substeps, 8);
    }

    #[test]
    fn test_zero_mass_is_pinned() {
        let mut s = new_pbd_system(1);
        pbd_add_particle(&mut s, [0.0; 3], 0.0);
        pbd_step(&mut s, 0.1);
        assert!((s.particles[0].pos[1]).abs() < 1e-6);
    }

    #[test]
    fn test_gravity_direction() {
        let s = new_pbd_system(1);
        assert!(s.gravity[1] < 0.0);
    }
}
