// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! SPH pressure gradient force — computes inter-particle pressure forces.

use std::f64::consts::PI;

/// SPH particle with position, velocity, density, pressure, and accumulated force.
#[derive(Debug, Clone)]
pub struct SphPressureParticle {
    pub pos: [f64; 3],
    pub vel: [f64; 3],
    pub mass: f64,
    pub density: f64,
    pub pressure: f64,
    pub force: [f64; 3],
}

impl SphPressureParticle {
    pub fn new(pos: [f64; 3], mass: f64, density: f64, pressure: f64) -> Self {
        SphPressureParticle {
            pos,
            vel: [0.0; 3],
            mass,
            density,
            pressure,
            force: [0.0; 3],
        }
    }
}

/// Gradient of the cubic spline kernel W'(r, h) in direction (dx, dy, dz).
pub fn kernel_gradient(dx: f64, dy: f64, dz: f64, h: f64) -> [f64; 3] {
    let r = (dx * dx + dy * dy + dz * dz).sqrt();
    if r < 1e-12 || h <= 0.0 {
        return [0.0; 3];
    }
    let q = r / h;
    let sigma = 1.0 / (PI * h * h * h * h);
    let dw_dq = if (0.0..=1.0).contains(&q) {
        sigma * (-3.0 * q + 2.25 * q * q)
    } else if (1.0..=2.0).contains(&q) {
        sigma * (-0.75 * (2.0 - q).powi(2))
    } else {
        0.0
    };
    let factor = dw_dq / (r * h);
    [dx * factor, dy * factor, dz * factor]
}

/// Compute pressure from density using a simple equation of state.
pub fn pressure_eos(density: f64, rest_density: f64, stiffness: f64) -> f64 {
    stiffness * ((density / rest_density).powi(7) - 1.0)
}

/// Compute pressure gradient forces for all particles.
pub fn compute_pressure_forces(particles: &mut [SphPressureParticle], h: f64) {
    let n = particles.len();
    let snapshot: Vec<_> = particles
        .iter()
        .map(|p| (p.pos, p.mass, p.density, p.pressure))
        .collect();
    for i in 0..n {
        let mut f = [0.0f64; 3];
        let (pi_pos, pi_mass, pi_density, pi_pressure) = snapshot[i];
        let pi_term = if pi_density > 1e-12 {
            pi_pressure / (pi_density * pi_density)
        } else {
            0.0
        };
        for (j, &(pj_pos, pj_mass, pj_density, pj_pressure)) in snapshot.iter().enumerate() {
            if i == j {
                continue;
            }
            let pj_term = if pj_density > 1e-12 {
                pj_pressure / (pj_density * pj_density)
            } else {
                0.0
            };
            let dx = pi_pos[0] - pj_pos[0];
            let dy = pi_pos[1] - pj_pos[1];
            let dz = pi_pos[2] - pj_pos[2];
            let grad = kernel_gradient(dx, dy, dz, h);
            let coeff = -pi_mass * pj_mass * (pi_term + pj_term);
            f[0] += coeff * grad[0];
            f[1] += coeff * grad[1];
            f[2] += coeff * grad[2];
        }
        let _ = pi_mass; /* suppress unused warning */
        particles[i].force = f;
    }
}

/// Integrate velocities and positions from pressure forces.
pub fn integrate_pressure(particles: &mut [SphPressureParticle], dt: f64) {
    for p in particles.iter_mut() {
        let inv_m = if p.mass > 1e-12 { 1.0 / p.mass } else { 0.0 };
        for k in 0..3 {
            p.vel[k] += p.force[k] * inv_m * dt;
            p.pos[k] += p.vel[k] * dt;
        }
    }
}

/// Clear all forces.
pub fn clear_forces(particles: &mut [SphPressureParticle]) {
    for p in particles.iter_mut() {
        p.force = [0.0; 3];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_gradient_at_zero() {
        let g = kernel_gradient(0.0, 0.0, 0.0, 1.0);
        assert_eq!(g, [0.0; 3] /* gradient at zero is zero */);
    }

    #[test]
    fn test_kernel_gradient_beyond_support() {
        let g = kernel_gradient(3.0, 0.0, 0.0, 1.0);
        assert_eq!(g, [0.0; 3] /* gradient beyond 2h is zero */);
    }

    #[test]
    fn test_pressure_eos_at_rest_density() {
        let p = pressure_eos(1000.0, 1000.0, 1.0);
        assert!(p.abs() < 1e-10 /* pressure zero at rest density */);
    }

    #[test]
    fn test_pressure_eos_compressed() {
        let p = pressure_eos(1100.0, 1000.0, 1.0);
        assert!(p > 0.0 /* compressed fluid has positive pressure */);
    }

    #[test]
    fn test_compute_pressure_forces_two_particles() {
        let mut particles = vec![
            SphPressureParticle::new([0.0, 0.0, 0.0], 1.0, 1000.0, 10.0),
            SphPressureParticle::new([0.5, 0.0, 0.0], 1.0, 1000.0, 10.0),
        ];
        compute_pressure_forces(&mut particles, 1.0);
        /* forces should be non-zero */
        let f0 = particles[0].force;
        let f1 = particles[1].force;
        /* by Newton's 3rd law, forces should be approximately opposite */
        assert!((f0[0] + f1[0]).abs() < 1e-6 /* action-reaction */);
    }

    #[test]
    fn test_clear_forces() {
        let mut particles = vec![SphPressureParticle::new([0.0; 3], 1.0, 1000.0, 10.0)];
        particles[0].force = [1.0, 2.0, 3.0];
        clear_forces(&mut particles);
        assert_eq!(particles[0].force, [0.0; 3] /* forces cleared */);
    }

    #[test]
    fn test_integrate_pressure_moves_particle() {
        let mut particles = vec![SphPressureParticle::new([0.0; 3], 1.0, 1000.0, 0.0)];
        particles[0].force = [1.0, 0.0, 0.0];
        integrate_pressure(&mut particles, 0.1);
        assert!(particles[0].pos[0] > 0.0 /* particle moved in x */);
    }

    #[test]
    fn test_integrate_zero_force() {
        let mut particles = vec![SphPressureParticle::new([0.0; 3], 1.0, 1000.0, 0.0)];
        integrate_pressure(&mut particles, 0.1);
        assert_eq!(
            particles[0].pos,
            [0.0; 3] /* no movement without force */
        );
    }

    #[test]
    fn test_kernel_gradient_antisymmetric() {
        let g1 = kernel_gradient(0.5, 0.0, 0.0, 1.0);
        let g2 = kernel_gradient(-0.5, 0.0, 0.0, 1.0);
        assert!((g1[0] + g2[0]).abs() < 1e-10 /* gradient is antisymmetric */);
    }
}
