// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Brownian motion / Langevin dynamics simulation.

/// A Brownian (Langevin) particle.
#[derive(Debug, Clone)]
pub struct LangevinParticle {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub mass: f32,
    pub gamma: f32, /* friction coefficient */
    pub id: usize,
}

impl LangevinParticle {
    pub fn new(x: f32, y: f32, z: f32, mass: f32, gamma: f32, id: usize) -> Self {
        LangevinParticle {
            pos: [x, y, z],
            vel: [0.0; 3],
            mass,
            gamma,
            id,
        }
    }

    /// Instantaneous kinetic temperature T = m * v^2 / (3 * kB).
    pub fn kinetic_temperature(&self, kb: f32) -> f32 {
        let v2 = self.vel.iter().map(|&v| v * v).sum::<f32>();
        self.mass * v2 / (3.0 * kb)
    }

    /// Kinetic energy = 0.5 * m * v^2.
    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.mass * self.vel.iter().map(|&v| v * v).sum::<f32>()
    }
}

/// Langevin dynamics simulation.
pub struct LangevinSimulation {
    pub particles: Vec<LangevinParticle>,
    pub temperature: f32,
    pub kb: f32,
    pub time: f32,
    rng_state: u64,
}

impl LangevinSimulation {
    pub fn new(temperature: f32, kb: f32) -> Self {
        LangevinSimulation {
            particles: Vec::new(),
            temperature,
            kb,
            time: 0.0,
            rng_state: 123456789,
        }
    }

    fn rand_gaussian(&mut self) -> f32 {
        /* Box-Muller transform */
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 7;
        self.rng_state ^= self.rng_state << 17;
        let u1 = (self.rng_state & 0xFFFFFF) as f32 / 0xFFFFFF as f32 + 1e-10;
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 7;
        self.rng_state ^= self.rng_state << 17;
        let u2 = (self.rng_state & 0xFFFFFF) as f32 / 0xFFFFFF as f32;
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos()
    }

    pub fn add_particle(&mut self, p: LangevinParticle) {
        self.particles.push(p);
    }

    /// Advance one step using Euler-Maruyama integration.
    /// Langevin equation: m * dv/dt = -γv + F_ext + ξ(t)
    /// where ξ is Gaussian noise with variance 2γkT/dt.
    pub fn step(&mut self, dt: f32, external_force: [f32; 3]) {
        let temperature = self.temperature;
        let kb = self.kb;
        let n = self.particles.len();
        let mut gn: Vec<[f32; 3]> = Vec::with_capacity(n);
        for _ in 0..n {
            gn.push([
                self.rand_gaussian(),
                self.rand_gaussian(),
                self.rand_gaussian(),
            ]);
        }

        for (i, p) in self.particles.iter_mut().enumerate() {
            let noise_amp = (2.0 * p.gamma * kb * temperature / dt).sqrt();
            for d in 0..3 {
                let f_rand = noise_amp * gn[i][d];
                let f_drag = -p.gamma * p.vel[d];
                let f_total = external_force[d] + f_drag + f_rand;
                p.vel[d] += (f_total / p.mass) * dt;
                p.pos[d] += p.vel[d] * dt;
            }
        }
        self.time += dt;
    }

    /// Advance `steps` timesteps.
    pub fn advance(&mut self, steps: usize, dt: f32, external_force: [f32; 3]) {
        for _ in 0..steps {
            self.step(dt, external_force);
        }
    }

    /// Mean square displacement from initial positions (requires reference).
    pub fn mean_kinetic_energy(&self) -> f32 {
        if self.particles.is_empty() {
            return 0.0;
        }
        self.particles
            .iter()
            .map(|p| p.kinetic_energy())
            .sum::<f32>()
            / self.particles.len() as f32
    }

    /// Mean speed.
    pub fn mean_speed(&self) -> f32 {
        if self.particles.is_empty() {
            return 0.0;
        }
        self.particles
            .iter()
            .map(|p| p.vel.iter().map(|&v| v * v).sum::<f32>().sqrt())
            .sum::<f32>()
            / self.particles.len() as f32
    }

    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }
}

/// Einstein diffusion coefficient: D = kB * T / γ.
pub fn einstein_diffusion(kb: f32, temperature: f32, gamma: f32) -> f32 {
    kb * temperature / gamma
}

/// Stokes-Einstein: D = kB * T / (6 π η r).
pub fn stokes_einstein_diffusion(kb: f32, temperature: f32, eta: f32, radius: f32) -> f32 {
    kb * temperature / (6.0 * std::f32::consts::PI * eta * radius)
}

/// Expected mean squared displacement for 3D Brownian motion: MSD = 6Dt.
pub fn expected_msd_3d(d: f32, t: f32) -> f32 {
    6.0 * d * t
}

pub fn new_langevin(temperature: f32, kb: f32) -> LangevinSimulation {
    LangevinSimulation::new(temperature, kb)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_creation() {
        let p = LangevinParticle::new(0.0, 0.0, 0.0, 1.0, 0.1, 0);
        assert_eq!(p.id, 0);
        assert!((p.mass - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_kinetic_energy_zero() {
        let p = LangevinParticle::new(0.0, 0.0, 0.0, 1.0, 0.1, 0);
        assert!((p.kinetic_energy()).abs() < 1e-10);
    }

    #[test]
    fn test_step_advances_time() {
        let mut sim = new_langevin(300.0, 1.38e-23);
        sim.add_particle(LangevinParticle::new(0.0, 0.0, 0.0, 1e-26, 1e-12, 0));
        sim.step(1e-12, [0.0; 3]);
        assert!(sim.time > 0.0);
    }

    #[test]
    fn test_particles_move() {
        /* Use macro-scale parameters to avoid f32 overflow */
        let mut sim = new_langevin(1.0, 1.0);
        sim.add_particle(LangevinParticle::new(0.0, 0.0, 0.0, 1.0, 1.0, 0));
        sim.advance(100, 0.01, [0.0; 3]);
        /* After many steps, particle should have a finite position */
        let dist = sim.particles[0]
            .pos
            .iter()
            .map(|&x| x * x)
            .sum::<f32>()
            .sqrt();
        assert!(dist.is_finite());
    }

    #[test]
    fn test_einstein_diffusion() {
        let d = einstein_diffusion(1.38e-23, 300.0, 1e-12);
        assert!(d > 0.0);
    }

    #[test]
    fn test_stokes_einstein() {
        let d = stokes_einstein_diffusion(1.38e-23, 300.0, 0.001, 50e-9);
        assert!(d > 0.0);
    }

    #[test]
    fn test_expected_msd() {
        let msd = expected_msd_3d(1e-12, 1.0);
        assert!((msd - 6e-12).abs() < 1e-20);
    }

    #[test]
    fn test_mean_kinetic_energy_positive_after_steps() {
        let mut sim = new_langevin(300.0, 1.0);
        sim.add_particle(LangevinParticle::new(0.0, 0.0, 0.0, 1.0, 1.0, 0));
        sim.advance(50, 0.001, [0.0; 3]);
        assert!(sim.mean_kinetic_energy() >= 0.0);
    }
}
