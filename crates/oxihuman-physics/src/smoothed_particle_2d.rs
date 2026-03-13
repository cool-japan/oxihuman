// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! 2D Smoothed Particle Hydrodynamics (SPH) with Wendland kernel.

use std::f32::consts::PI;

/// Wendland C2 kernel for 2D SPH.
pub fn wendland_w(r: f32, h: f32) -> f32 {
    let q = r / h;
    if q >= 2.0 {
        return 0.0;
    }
    let t = 1.0 - 0.5 * q;
    (7.0 / (4.0 * PI * h * h)) * t.powi(4) * (1.0 + 2.0 * q)
}

/// Gradient of Wendland C2 kernel.
pub fn wendland_grad_w(r: [f32; 2], h: f32) -> [f32; 2] {
    let dist = (r[0] * r[0] + r[1] * r[1]).sqrt();
    if dist < 1e-12 || dist / h >= 2.0 {
        return [0.0, 0.0];
    }
    let q = dist / h;
    let t = 1.0 - 0.5 * q;
    let dw_dr =
        (7.0 / (4.0 * PI * h * h)) * (-2.0 * t.powi(3) * (1.0 + 2.0 * q) + t.powi(4) * 2.0) / h;
    [dw_dr * r[0] / dist, dw_dr * r[1] / dist]
}

/// An SPH particle in 2D.
#[derive(Debug, Clone)]
pub struct SphParticle2D {
    pub pos: [f32; 2],
    pub vel: [f32; 2],
    pub mass: f32,
    pub density: f32,
    pub pressure: f32,
    pub acc: [f32; 2],
}

impl SphParticle2D {
    pub fn new(x: f32, y: f32, mass: f32) -> Self {
        SphParticle2D {
            pos: [x, y],
            vel: [0.0; 2],
            mass,
            density: 0.0,
            pressure: 0.0,
            acc: [0.0; 2],
        }
    }
}

/// 2D SPH fluid simulation.
pub struct Sph2D {
    pub particles: Vec<SphParticle2D>,
    pub h: f32,
    pub rho0: f32,
    pub k: f32,
    pub mu: f32,
    pub time: f32,
}

impl Sph2D {
    pub fn new(h: f32, rho0: f32, k: f32, mu: f32) -> Self {
        Sph2D {
            particles: Vec::new(),
            h,
            rho0,
            k,
            mu,
            time: 0.0,
        }
    }

    pub fn add_particle(&mut self, x: f32, y: f32, mass: f32) {
        self.particles.push(SphParticle2D::new(x, y, mass));
    }

    /// Compute densities for all particles.
    pub fn compute_densities(&mut self) {
        let n = self.particles.len();
        let h = self.h;
        let positions: Vec<[f32; 2]> = self.particles.iter().map(|p| p.pos).collect();
        let masses: Vec<f32> = self.particles.iter().map(|p| p.mass).collect();
        for i in 0..n {
            let mut rho = 0.0f32;
            for j in 0..n {
                let rx = positions[i][0] - positions[j][0];
                let ry = positions[i][1] - positions[j][1];
                let r = (rx * rx + ry * ry).sqrt();
                rho += masses[j] * wendland_w(r, h);
            }
            self.particles[i].density = rho.max(1e-6);
        }
    }

    /// Compute pressures using equation of state: p = k * (rho - rho0).
    pub fn compute_pressures(&mut self) {
        let k = self.k;
        let rho0 = self.rho0;
        for p in &mut self.particles {
            p.pressure = k * (p.density - rho0).max(0.0);
        }
    }

    /// Compute accelerations (pressure gradient + viscosity + gravity).
    pub fn compute_accelerations(&mut self, gravity: [f32; 2]) {
        let n = self.particles.len();
        let h = self.h;
        let mu = self.mu;
        let positions: Vec<[f32; 2]> = self.particles.iter().map(|p| p.pos).collect();
        let velocities: Vec<[f32; 2]> = self.particles.iter().map(|p| p.vel).collect();
        let densities: Vec<f32> = self.particles.iter().map(|p| p.density).collect();
        let pressures: Vec<f32> = self.particles.iter().map(|p| p.pressure).collect();
        let masses: Vec<f32> = self.particles.iter().map(|p| p.mass).collect();
        for i in 0..n {
            let mut ax = gravity[0];
            let mut ay = gravity[1];
            for j in 0..n {
                if i == j {
                    continue;
                }
                let r = [
                    positions[i][0] - positions[j][0],
                    positions[i][1] - positions[j][1],
                ];
                let grad = wendland_grad_w(r, h);
                /* Pressure gradient */
                let pij = -(pressures[i] / (densities[i] * densities[i])
                    + pressures[j] / (densities[j] * densities[j]));
                ax += masses[j] * pij * grad[0];
                ay += masses[j] * pij * grad[1];
                /* Viscosity */
                let dvx = velocities[j][0] - velocities[i][0];
                let dvy = velocities[j][1] - velocities[i][1];
                let dist = (r[0] * r[0] + r[1] * r[1]).sqrt().max(1e-12);
                let visc = mu * masses[j] * (dvx * r[0] + dvy * r[1])
                    / (densities[j] * (dist * dist + 0.01 * h * h));
                ax += visc * grad[0];
                ay += visc * grad[1];
            }
            self.particles[i].acc = [ax, ay];
        }
    }

    /// Advance one timestep using symplectic Euler integration.
    pub fn step(&mut self, dt: f32, gravity: [f32; 2]) {
        self.compute_densities();
        self.compute_pressures();
        self.compute_accelerations(gravity);
        for p in &mut self.particles {
            p.vel[0] += p.acc[0] * dt;
            p.vel[1] += p.acc[1] * dt;
            p.pos[0] += p.vel[0] * dt;
            p.pos[1] += p.vel[1] * dt;
        }
        self.time += dt;
    }

    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    pub fn total_mass(&self) -> f32 {
        self.particles.iter().map(|p| p.mass).sum()
    }

    pub fn avg_density(&self) -> f32 {
        if self.particles.is_empty() {
            return 0.0;
        }
        self.particles.iter().map(|p| p.density).sum::<f32>() / self.particles.len() as f32
    }
}

pub fn new_sph_2d(h: f32, rho0: f32, k: f32, mu: f32) -> Sph2D {
    Sph2D::new(h, rho0, k, mu)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wendland_kernel_at_zero() {
        let w = wendland_w(0.0, 1.0);
        assert!(w > 0.0);
    }

    #[test]
    fn test_wendland_kernel_compact() {
        let w = wendland_w(2.0, 1.0);
        assert!((w).abs() < 1e-10);
    }

    #[test]
    fn test_sph_particle_creation() {
        let mut sph = new_sph_2d(0.1, 1000.0, 1.0, 0.001);
        sph.add_particle(0.0, 0.0, 1.0);
        sph.add_particle(0.05, 0.0, 1.0);
        assert_eq!(sph.particle_count(), 2);
    }

    #[test]
    fn test_density_computation() {
        let mut sph = new_sph_2d(0.1, 1000.0, 1.0, 0.001);
        sph.add_particle(0.0, 0.0, 1.0);
        sph.add_particle(0.05, 0.0, 1.0);
        sph.compute_densities();
        assert!(sph.particles[0].density > 0.0);
    }

    #[test]
    fn test_total_mass() {
        let mut sph = new_sph_2d(0.1, 1000.0, 1.0, 0.001);
        sph.add_particle(0.0, 0.0, 2.0);
        sph.add_particle(0.1, 0.0, 3.0);
        assert!((sph.total_mass() - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_step_advances_time() {
        let mut sph = new_sph_2d(0.1, 1000.0, 1.0, 0.001);
        sph.add_particle(0.0, 0.0, 1.0);
        sph.step(0.001, [0.0, -9.81]);
        assert!(sph.time > 0.0);
    }

    #[test]
    fn test_gravity_accelerates_particles() {
        let mut sph = new_sph_2d(0.2, 1000.0, 0.1, 0.0001);
        sph.add_particle(0.0, 1.0, 1.0);
        let y0 = sph.particles[0].pos[1];
        for _ in 0..10 {
            sph.step(0.01, [0.0, -9.81]);
        }
        /* Under gravity, y should decrease */
        assert!(sph.particles[0].pos[1] < y0);
    }

    #[test]
    fn test_wendland_gradient_nonzero() {
        let g = wendland_grad_w([0.5, 0.0], 1.0);
        assert!(g[0].abs() > 0.0 || g[1].abs() > 0.0);
    }
}
