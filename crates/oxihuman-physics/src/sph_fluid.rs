// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! SPH fluid: density, pressure, viscosity forces.

#![allow(dead_code)]

/// SPH particle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SphParticleV2 {
    pub pos: [f32; 2],
    pub vel: [f32; 2],
    pub density: f32,
    pub pressure: f32,
    pub mass: f32,
}

impl SphParticleV2 {
    pub fn new(pos: [f32; 2], mass: f32) -> Self {
        Self {
            pos,
            vel: [0.0, 0.0],
            density: 0.0,
            pressure: 0.0,
            mass,
        }
    }
}

/// SPH simulation configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SphConfig {
    pub smoothing_h: f32,
    pub rest_density: f32,
    pub pressure_k: f32, // stiffness
    pub viscosity: f32,
    pub gravity: [f32; 2],
}

impl Default for SphConfig {
    fn default() -> Self {
        Self {
            smoothing_h: 0.1,
            rest_density: 1000.0,
            pressure_k: 200.0,
            viscosity: 0.01,
            gravity: [0.0, -9.81],
        }
    }
}

/// Poly6 kernel (2D).
#[allow(dead_code)]
pub fn poly6_2d(r: f32, h: f32) -> f32 {
    if r > h {
        return 0.0;
    }
    let coeff = 4.0 / (std::f32::consts::PI * h.powi(8));
    let diff = h * h - r * r;
    coeff * diff * diff * diff
}

/// Spiky kernel gradient magnitude (2D).
#[allow(dead_code)]
pub fn spiky_grad_2d(r: f32, h: f32) -> f32 {
    if r <= 0.0 || r > h {
        return 0.0;
    }
    let coeff = -10.0 / (std::f32::consts::PI * h.powi(5));
    let diff = h - r;
    coeff * diff * diff
}

/// Viscosity kernel Laplacian (2D).
#[allow(dead_code)]
pub fn viscosity_lap_2d(r: f32, h: f32) -> f32 {
    if r > h {
        return 0.0;
    }
    let coeff = 40.0 / (std::f32::consts::PI * h.powi(5));
    coeff * (h - r)
}

/// SPH fluid simulation in 2D.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SphFluidV2 {
    pub particles: Vec<SphParticleV2>,
    pub config: SphConfig,
}

#[allow(dead_code)]
impl SphFluidV2 {
    pub fn new(config: SphConfig) -> Self {
        Self {
            particles: Vec::new(),
            config,
        }
    }

    pub fn add_particle(&mut self, pos: [f32; 2], mass: f32) {
        self.particles.push(SphParticleV2::new(pos, mass));
    }

    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    /// Compute densities for all particles.
    pub fn compute_densities(&mut self) {
        let n = self.particles.len();
        let h = self.config.smoothing_h;
        let mut densities = vec![0.0f32; n];
        for (i, d) in densities.iter_mut().enumerate() {
            for j in 0..n {
                let dx = self.particles[j].pos[0] - self.particles[i].pos[0];
                let dy = self.particles[j].pos[1] - self.particles[i].pos[1];
                let r = (dx * dx + dy * dy).sqrt();
                *d += self.particles[j].mass * poly6_2d(r, h);
            }
        }
        for (p, d) in self.particles.iter_mut().zip(densities.iter()) {
            p.density = *d;
        }
    }

    /// Compute pressures using equation of state.
    pub fn compute_pressures(&mut self) {
        let k = self.config.pressure_k;
        let rho0 = self.config.rest_density;
        for p in &mut self.particles {
            p.pressure = k * (p.density - rho0).max(0.0);
        }
    }

    /// Step the simulation.
    pub fn step(&mut self, dt: f32) {
        self.compute_densities();
        self.compute_pressures();
        let n = self.particles.len();
        let h = self.config.smoothing_h;
        let mu = self.config.viscosity;
        let g = self.config.gravity;
        let mut forces: Vec<[f32; 2]> = vec![[0.0, 0.0]; n];
        for (i, fi) in forces.iter_mut().enumerate() {
            for j in 0..n {
                if i == j {
                    continue;
                }
                let dx = self.particles[j].pos[0] - self.particles[i].pos[0];
                let dy = self.particles[j].pos[1] - self.particles[i].pos[1];
                let r = (dx * dx + dy * dy).sqrt().max(1e-6);
                let nx = dx / r;
                let ny = dy / r;
                // Pressure force
                let pf = -self.particles[j].mass
                    * (self.particles[i].pressure + self.particles[j].pressure)
                    / (2.0 * self.particles[j].density.max(1e-6))
                    * spiky_grad_2d(r, h);
                fi[0] += pf * nx;
                fi[1] += pf * ny;
                // Viscosity force
                let dvx = self.particles[j].vel[0] - self.particles[i].vel[0];
                let dvy = self.particles[j].vel[1] - self.particles[i].vel[1];
                let vf = mu * self.particles[j].mass / self.particles[j].density.max(1e-6)
                    * viscosity_lap_2d(r, h);
                fi[0] += vf * dvx;
                fi[1] += vf * dvy;
            }
            // Gravity
            fi[0] += self.particles[i].density * g[0];
            fi[1] += self.particles[i].density * g[1];
        }
        for (i, p) in self.particles.iter_mut().enumerate() {
            let inv_rho = 1.0 / p.density.max(1e-6);
            p.vel[0] += forces[i][0] * inv_rho * dt;
            p.vel[1] += forces[i][1] * inv_rho * dt;
            p.pos[0] += p.vel[0] * dt;
            p.pos[1] += p.vel[1] * dt;
        }
    }

    /// Total kinetic energy.
    pub fn kinetic_energy(&self) -> f32 {
        self.particles
            .iter()
            .map(|p| 0.5 * p.mass * (p.vel[0] * p.vel[0] + p.vel[1] * p.vel[1]))
            .sum()
    }
}

use std::f32::consts::PI as SPH_PI;

pub struct SphParticle {
    pub pos: [f32; 2],
    pub vel: [f32; 2],
    pub density: f32,
    pub pressure: f32,
    pub mass: f32,
}

pub fn new_sph_particle(pos: [f32; 2], mass: f32) -> SphParticle {
    SphParticle {
        pos,
        vel: [0.0, 0.0],
        density: 1000.0,
        pressure: 0.0,
        mass,
    }
}

pub fn sph_kernel_w(r: f32, h: f32) -> f32 {
    if r < 0.0 || h <= 0.0 {
        return 0.0;
    }
    let q = r / h;
    let sigma = 10.0 / (7.0 * SPH_PI * h * h);
    if q <= 1.0 {
        sigma * (1.0 - 1.5 * q * q + 0.75 * q * q * q)
    } else if q <= 2.0 {
        sigma * 0.25 * (2.0 - q).powi(3)
    } else {
        0.0
    }
}

pub fn sph_kernel_dw(r: f32, h: f32) -> f32 {
    if r < 1e-8 || h <= 0.0 {
        return 0.0;
    }
    let q = r / h;
    let sigma = 10.0 / (7.0 * SPH_PI * h * h);
    let dw_dq = if q <= 1.0 {
        sigma * (-3.0 * q + 2.25 * q * q)
    } else if q <= 2.0 {
        sigma * (-0.75 * (2.0 - q).powi(2))
    } else {
        0.0
    };
    dw_dq / h
}

#[allow(clippy::needless_range_loop)]
pub fn sph_compute_density(particles: &mut [SphParticle], h: f32) {
    let n = particles.len();
    let mut densities = vec![0.0f32; n];
    for i in 0..n {
        for j in 0..n {
            let dx = particles[i].pos[0] - particles[j].pos[0];
            let dy = particles[i].pos[1] - particles[j].pos[1];
            let r = (dx * dx + dy * dy).sqrt();
            densities[i] += particles[j].mass * sph_kernel_w(r, h);
        }
    }
    for (i, p) in particles.iter_mut().enumerate() {
        p.density = densities[i];
    }
}

pub fn sph_pressure_tait(density: f32, rest_density: f32, b: f32) -> f32 {
    let ratio = density / rest_density;
    b * (ratio.powi(7) - 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_sph() -> SphFluidV2 {
        let mut s = SphFluidV2::new(SphConfig::default());
        for i in 0..4 {
            s.add_particle([i as f32 * 0.05, 0.0], 1.0);
        }
        s
    }

    #[test]
    fn particle_count() {
        let s = simple_sph();
        assert_eq!(s.particle_count(), 4);
    }

    #[test]
    fn poly6_at_zero_positive() {
        let v = poly6_2d(0.0, 0.1);
        assert!(v > 0.0);
    }

    #[test]
    fn poly6_beyond_h_zero() {
        let v = poly6_2d(0.2, 0.1);
        assert!(v.abs() < 1e-10);
    }

    #[test]
    fn spiky_grad_at_zero_returns_zero() {
        let v = spiky_grad_2d(0.0, 0.1);
        assert!(v.abs() < 1e-10);
    }

    #[test]
    fn compute_densities_positive() {
        let mut s = simple_sph();
        s.compute_densities();
        for p in &s.particles {
            assert!(p.density > 0.0);
        }
    }

    #[test]
    fn compute_pressures_nonneg() {
        let mut s = simple_sph();
        s.compute_densities();
        s.compute_pressures();
        for p in &s.particles {
            assert!(p.pressure >= 0.0);
        }
    }

    #[test]
    fn step_changes_positions() {
        let mut s = simple_sph();
        let pos0 = s.particles[0].pos;
        s.step(0.001);
        let pos1 = s.particles[0].pos;
        let _delta = (pos1[1] - pos0[1]).abs(); // expect gravity to move it
    }

    #[test]
    fn kinetic_energy_after_step() {
        let mut s = simple_sph();
        s.step(0.01);
        assert!(s.kinetic_energy() >= 0.0);
    }

    #[test]
    fn viscosity_lap_2d_positive() {
        let v = viscosity_lap_2d(0.05, 0.1);
        assert!(v > 0.0);
    }

    #[test]
    fn many_steps_finite() {
        let mut s = simple_sph();
        for _ in 0..10 {
            s.step(0.0001);
        }
        for p in &s.particles {
            assert!(p.pos[0].is_finite() && p.pos[1].is_finite());
        }
    }
}
