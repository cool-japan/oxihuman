// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[allow(dead_code)]
pub struct SphBoundaryParticle {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub psi: f32,
}

#[allow(dead_code)]
pub struct SphBoundary {
    pub particles: Vec<SphBoundaryParticle>,
    pub stiffness: f32,
}

#[allow(dead_code)]
pub fn new_sph_boundary(stiffness: f32) -> SphBoundary {
    SphBoundary { particles: Vec::new(), stiffness }
}

#[allow(dead_code)]
pub fn sb_add_particle(b: &mut SphBoundary, pos: [f32; 3], normal: [f32; 3]) {
    b.particles.push(SphBoundaryParticle { pos, normal, psi: 1.0 });
}

#[allow(dead_code)]
pub fn sb_particle_count(b: &SphBoundary) -> usize {
    b.particles.len()
}

#[allow(dead_code)]
pub fn sb_boundary_force(b: &SphBoundary, fluid_pos: [f32; 3], kernel_radius: f32) -> [f32; 3] {
    let mut force = [0.0f32; 3];
    for p in &b.particles {
        let dx = fluid_pos[0] - p.pos[0];
        let dy = fluid_pos[1] - p.pos[1];
        let dz = fluid_pos[2] - p.pos[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        if dist < kernel_radius && dist > 1e-7 {
            let repulsion = b.stiffness * (1.0 - dist / kernel_radius).powi(2) / dist;
            force[0] += repulsion * dx;
            force[1] += repulsion * dy;
            force[2] += repulsion * dz;
        }
    }
    force
}

#[allow(dead_code)]
pub fn sb_stiffness(b: &SphBoundary) -> f32 {
    b.stiffness
}

#[allow(dead_code)]
pub fn sb_clear(b: &mut SphBoundary) {
    b.particles.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let b = new_sph_boundary(1.0);
        assert_eq!(sb_particle_count(&b), 0);
    }

    #[test]
    fn test_add_particle() {
        let mut b = new_sph_boundary(1.0);
        sb_add_particle(&mut b, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(sb_particle_count(&b), 1);
    }

    #[test]
    fn test_count() {
        let mut b = new_sph_boundary(2.0);
        sb_add_particle(&mut b, [0.0; 3], [0.0, 1.0, 0.0]);
        sb_add_particle(&mut b, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(sb_particle_count(&b), 2);
    }

    #[test]
    fn test_boundary_force_close() {
        let mut b = new_sph_boundary(10.0);
        sb_add_particle(&mut b, [0.0; 3], [0.0, 1.0, 0.0]);
        let force = sb_boundary_force(&b, [0.1, 0.0, 0.0], 1.0);
        assert!(force[0].abs() > 0.0 || force[1].abs() > 0.0 || force[2].abs() > 0.0);
    }

    #[test]
    fn test_boundary_force_far() {
        let mut b = new_sph_boundary(10.0);
        sb_add_particle(&mut b, [0.0; 3], [0.0, 1.0, 0.0]);
        let force = sb_boundary_force(&b, [100.0, 0.0, 0.0], 1.0);
        assert!((force[0]).abs() < 1e-6);
    }

    #[test]
    fn test_stiffness_getter() {
        let b = new_sph_boundary(5.5);
        assert!((sb_stiffness(&b) - 5.5).abs() < 1e-6);
    }

    #[test]
    fn test_clear() {
        let mut b = new_sph_boundary(1.0);
        sb_add_particle(&mut b, [0.0; 3], [0.0, 1.0, 0.0]);
        sb_clear(&mut b);
        assert_eq!(sb_particle_count(&b), 0);
    }

    #[test]
    fn test_psi_initialized() {
        let mut b = new_sph_boundary(1.0);
        sb_add_particle(&mut b, [1.0, 2.0, 3.0], [0.0, 1.0, 0.0]);
        assert!((b.particles[0].psi - 1.0).abs() < 1e-6);
    }
}
