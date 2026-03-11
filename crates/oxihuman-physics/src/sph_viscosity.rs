// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! SPH viscosity diffusion — computes viscous forces between SPH particles.

use std::f64::consts::PI;

/// SPH particle with position, velocity, and viscous force accumulator.
#[derive(Debug, Clone)]
pub struct SphViscParticle {
    pub pos: [f64; 3],
    pub vel: [f64; 3],
    pub mass: f64,
    pub density: f64,
    pub visc_force: [f64; 3],
}

impl SphViscParticle {
    pub fn new(pos: [f64; 3], vel: [f64; 3], mass: f64, density: f64) -> Self {
        SphViscParticle {
            pos,
            vel,
            mass,
            density,
            visc_force: [0.0; 3],
        }
    }
}

/// Viscosity kernel Laplacian W_visc(r, h).
pub fn viscosity_kernel_laplacian(r: f64, h: f64) -> f64 {
    if r > h || h <= 0.0 {
        return 0.0;
    }
    let sigma = 45.0 / (PI * h.powi(6));
    sigma * (h - r)
}

/// Compute viscous forces for all particles.
pub fn compute_viscosity_forces(particles: &mut [SphViscParticle], h: f64, viscosity: f64) {
    let n = particles.len();
    let snapshot: Vec<_> = particles
        .iter()
        .map(|p| (p.pos, p.vel, p.mass, p.density))
        .collect();
    for i in 0..n {
        let mut fv = [0.0f64; 3];
        let (pi_pos, pi_vel, pi_mass, _) = snapshot[i];
        for (j, &(pj_pos, pj_vel, pj_mass, pj_density)) in snapshot.iter().enumerate() {
            if i == j {
                continue;
            }
            let dx = pi_pos[0] - pj_pos[0];
            let dy = pi_pos[1] - pj_pos[1];
            let dz = pi_pos[2] - pj_pos[2];
            let r = (dx * dx + dy * dy + dz * dz).sqrt();
            let lap = viscosity_kernel_laplacian(r, h);
            let coeff = viscosity * pi_mass * pj_mass * lap / pj_density.max(1e-12);
            let dvx = pj_vel[0] - pi_vel[0];
            let dvy = pj_vel[1] - pi_vel[1];
            let dvz = pj_vel[2] - pi_vel[2];
            fv[0] += coeff * dvx;
            fv[1] += coeff * dvy;
            fv[2] += coeff * dvz;
        }
        let _ = pi_mass;
        particles[i].visc_force = fv;
    }
}

/// Apply viscous forces: update velocity.
pub fn apply_viscosity(particles: &mut [SphViscParticle], dt: f64) {
    for p in particles.iter_mut() {
        let inv_m = if p.mass > 1e-12 { 1.0 / p.mass } else { 0.0 };
        for k in 0..3 {
            p.vel[k] += p.visc_force[k] * inv_m * dt;
        }
    }
}

/// Clear viscous forces.
pub fn clear_visc_forces(particles: &mut [SphViscParticle]) {
    for p in particles.iter_mut() {
        p.visc_force = [0.0; 3];
    }
}

/// Total viscous energy dissipation estimate.
pub fn visc_dissipation(particles: &[SphViscParticle]) -> f64 {
    particles
        .iter()
        .map(|p| {
            let v2 = p.vel[0] * p.vel[0] + p.vel[1] * p.vel[1] + p.vel[2] * p.vel[2];
            v2 * p.mass
        })
        .sum::<f64>()
        * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_laplacian_at_zero() {
        let lap = viscosity_kernel_laplacian(0.0, 1.0);
        assert!(lap > 0.0 /* kernel laplacian positive at zero */);
    }

    #[test]
    fn test_kernel_laplacian_beyond_h() {
        let lap = viscosity_kernel_laplacian(1.5, 1.0);
        assert_eq!(lap, 0.0 /* zero beyond support */);
    }

    #[test]
    fn test_kernel_invalid_h() {
        let lap = viscosity_kernel_laplacian(0.5, 0.0);
        assert_eq!(lap, 0.0 /* h=0 returns 0 */);
    }

    #[test]
    fn test_compute_viscosity_forces_two_particles() {
        let mut particles = vec![
            SphViscParticle::new([0.0; 3], [1.0, 0.0, 0.0], 1.0, 1000.0),
            SphViscParticle::new([0.3, 0.0, 0.0], [0.0, 0.0, 0.0], 1.0, 1000.0),
        ];
        compute_viscosity_forces(&mut particles, 1.0, 0.01);
        /* viscosity should reduce velocity difference */
        assert!(particles[0].visc_force[0] < 0.0 /* decelerating force on fast particle */);
    }

    #[test]
    fn test_apply_viscosity() {
        let mut particles = vec![SphViscParticle::new([0.0; 3], [0.0; 3], 1.0, 1000.0)];
        particles[0].visc_force = [1.0, 0.0, 0.0];
        apply_viscosity(&mut particles, 0.1);
        assert!(particles[0].vel[0] > 0.0 /* velocity updated by force */);
    }

    #[test]
    fn test_clear_visc_forces() {
        let mut particles = vec![SphViscParticle::new([0.0; 3], [0.0; 3], 1.0, 1000.0)];
        particles[0].visc_force = [5.0, 5.0, 5.0];
        clear_visc_forces(&mut particles);
        assert_eq!(particles[0].visc_force, [0.0; 3] /* cleared */);
    }

    #[test]
    fn test_visc_dissipation_zero_vel() {
        let particles = vec![SphViscParticle::new([0.0; 3], [0.0; 3], 1.0, 1000.0)];
        assert_eq!(
            visc_dissipation(&particles),
            0.0 /* no velocity = no KE */
        );
    }

    #[test]
    fn test_visc_dissipation_nonzero() {
        let mut p = SphViscParticle::new([0.0; 3], [0.0; 3], 2.0, 1000.0);
        p.vel = [1.0, 0.0, 0.0];
        let particles = vec![p];
        assert!((visc_dissipation(&particles) - 1.0).abs() < 1e-10 /* KE = 0.5 * 2 * 1^2 = 1 */);
    }

    #[test]
    fn test_kernel_laplacian_decreasing() {
        let h = 1.0;
        let lap0 = viscosity_kernel_laplacian(0.0, h);
        let lap05 = viscosity_kernel_laplacian(0.5, h);
        assert!(lap0 > lap05 /* kernel laplacian decreases with distance */);
    }
}
