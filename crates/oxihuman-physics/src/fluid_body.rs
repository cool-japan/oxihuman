// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Fluid body: a simple SPH (Smoothed Particle Hydrodynamics) fluid simulation in 2-D.

use std::f32::consts::PI;

/// A single SPH fluid particle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FluidParticle {
    pub pos: [f32; 2],
    pub vel: [f32; 2],
    pub density: f32,
    pub pressure: f32,
}

#[allow(dead_code)]
impl FluidParticle {
    pub fn new(pos: [f32; 2]) -> Self {
        Self {
            pos,
            vel: [0.0; 2],
            density: 1.0,
            pressure: 0.0,
        }
    }
}

/// SPH kernel: poly6 smoothing function.
pub fn poly6_kernel(r_sq: f32, h: f32) -> f32 {
    let h2 = h * h;
    if r_sq >= h2 {
        return 0.0;
    }
    let diff = h2 - r_sq;
    (315.0 / (64.0 * PI * h.powi(9))) * diff * diff * diff
}

/// Spiky kernel gradient magnitude.
pub fn spiky_kernel_grad(r: f32, h: f32) -> f32 {
    if r >= h || r < 1e-9 {
        return 0.0;
    }
    let diff = h - r;
    -(45.0 / (PI * h.powi(6))) * diff * diff
}

/// A simple 2-D SPH fluid.
#[allow(dead_code)]
pub struct FluidBody {
    pub particles: Vec<FluidParticle>,
    pub smoothing_radius: f32,
    pub rest_density: f32,
    pub pressure_k: f32,
    pub viscosity: f32,
}

#[allow(dead_code)]
impl FluidBody {
    pub fn new(smoothing_radius: f32, rest_density: f32, pressure_k: f32, viscosity: f32) -> Self {
        Self {
            particles: Vec::new(),
            smoothing_radius,
            rest_density,
            pressure_k,
            viscosity,
        }
    }

    pub fn add_particle(&mut self, pos: [f32; 2]) {
        self.particles.push(FluidParticle::new(pos));
    }

    /// Compute density for each particle.
    pub fn compute_densities(&mut self) {
        let h = self.smoothing_radius;
        let n = self.particles.len();
        let mut densities = vec![0.0_f32; n];
        #[allow(clippy::needless_range_loop)]
        for i in 0..n {
            for j in 0..n {
                let dx = self.particles[j].pos[0] - self.particles[i].pos[0];
                let dy = self.particles[j].pos[1] - self.particles[i].pos[1];
                let r_sq = dx * dx + dy * dy;
                densities[i] += poly6_kernel(r_sq, h);
            }
            densities[i] = densities[i].max(1e-6);
        }
        for (i, p) in self.particles.iter_mut().enumerate() {
            p.density = densities[i];
            p.pressure = self.pressure_k * (p.density / self.rest_density - 1.0);
        }
    }

    /// Integrate one timestep under gravity.
    pub fn step(&mut self, dt: f32, gravity: f32) {
        self.compute_densities();
        let h = self.smoothing_radius;
        let n = self.particles.len();
        let mut forces = vec![[0.0_f32; 2]; n];
        #[allow(clippy::needless_range_loop)]
        for i in 0..n {
            for j in 0..n {
                if i == j {
                    continue;
                }
                let dx = self.particles[j].pos[0] - self.particles[i].pos[0];
                let dy = self.particles[j].pos[1] - self.particles[i].pos[1];
                let r = (dx * dx + dy * dy).sqrt();
                if r < 1e-9 || r >= h {
                    continue;
                }
                let nx = dx / r;
                let ny = dy / r;
                let grad = spiky_kernel_grad(r, h);
                let pressure_f = -(self.particles[i].pressure + self.particles[j].pressure)
                    / (2.0 * self.particles[j].density)
                    * grad;
                forces[i][0] += pressure_f * nx;
                forces[i][1] += pressure_f * ny;
            }
        }
        for (i, p) in self.particles.iter_mut().enumerate() {
            p.vel[0] += (forces[i][0] / p.density) * dt;
            p.vel[1] += (forces[i][1] / p.density - gravity) * dt;
            p.pos[0] += p.vel[0] * dt;
            p.pos[1] += p.vel[1] * dt;
        }
    }

    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    pub fn average_density(&self) -> f32 {
        if self.particles.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.particles.iter().map(|p| p.density).sum();
        sum / self.particles.len() as f32
    }

    pub fn kinetic_energy(&self) -> f32 {
        self.particles
            .iter()
            .map(|p| 0.5 * (p.vel[0] * p.vel[0] + p.vel[1] * p.vel[1]))
            .sum()
    }
}

pub fn new_fluid_body(h: f32, rest_density: f32, k: f32, viscosity: f32) -> FluidBody {
    FluidBody::new(h, rest_density, k, viscosity)
}

pub fn fb2_add(body: &mut FluidBody, pos: [f32; 2]) {
    body.add_particle(pos);
}

pub fn fb2_step(body: &mut FluidBody, dt: f32, gravity: f32) {
    body.step(dt, gravity);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_fluid_empty() {
        let f = new_fluid_body(0.1, 1000.0, 200.0, 0.1);
        assert_eq!(f.particle_count(), 0);
    }

    #[test]
    fn add_particle() {
        let mut f = new_fluid_body(0.1, 1000.0, 200.0, 0.1);
        fb2_add(&mut f, [0.0, 0.0]);
        assert_eq!(f.particle_count(), 1);
    }

    #[test]
    fn poly6_kernel_zero_at_boundary() {
        assert_eq!(poly6_kernel(1.0, 1.0), 0.0);
    }

    #[test]
    fn poly6_kernel_positive_inside() {
        assert!(poly6_kernel(0.0, 1.0) > 0.0);
    }

    #[test]
    fn spiky_grad_zero_outside() {
        assert_eq!(spiky_kernel_grad(2.0, 1.0), 0.0);
    }

    #[test]
    fn compute_densities_sets_positive() {
        let mut f = new_fluid_body(1.0, 1.0, 1.0, 0.0);
        fb2_add(&mut f, [0.0, 0.0]);
        fb2_add(&mut f, [0.1, 0.0]);
        f.compute_densities();
        for p in &f.particles {
            assert!(p.density > 0.0);
        }
    }

    #[test]
    fn step_changes_position() {
        let mut f = new_fluid_body(1.0, 1.0, 1.0, 0.0);
        fb2_add(&mut f, [0.0, 1.0]);
        let old_y = f.particles[0].pos[1];
        fb2_step(&mut f, 0.01, 9.81);
        assert!(f.particles[0].pos[1] != old_y);
    }

    #[test]
    fn average_density_positive_after_step() {
        let mut f = new_fluid_body(1.0, 1.0, 1.0, 0.0);
        fb2_add(&mut f, [0.0, 0.0]);
        fb2_step(&mut f, 0.01, 0.0);
        assert!(f.average_density() > 0.0);
    }

    #[test]
    fn kinetic_energy_after_gravity() {
        let mut f = new_fluid_body(1.0, 1.0, 1.0, 0.0);
        fb2_add(&mut f, [0.0, 0.5]);
        fb2_step(&mut f, 0.1, 9.81);
        assert!(f.kinetic_energy() > 0.0);
    }

    #[test]
    fn empty_average_density_zero() {
        let f = new_fluid_body(0.1, 1.0, 1.0, 0.0);
        assert_eq!(f.average_density(), 0.0);
    }

    #[test]
    fn no_nan_after_multiple_steps() {
        let mut f = new_fluid_body(0.5, 1.0, 10.0, 0.1);
        for i in 0..4 {
            fb2_add(&mut f, [i as f32 * 0.1, 0.0]);
        }
        for _ in 0..10 {
            fb2_step(&mut f, 0.001, 0.0);
        }
        for p in &f.particles {
            assert!(!p.pos[0].is_nan());
        }
    }
}
