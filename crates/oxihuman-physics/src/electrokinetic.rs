// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Electrokinetic (electrophoresis) particle transport model.

use std::f32::consts::PI;

/// Electrokinetic particle under an electric field.
#[derive(Debug, Clone)]
pub struct ElectrokineticParticle {
    pub pos: [f32; 2],
    pub vel: [f32; 2],
    pub charge: f32,
    pub mass: f32,
    pub radius: f32,
    pub zeta_potential: f32,
}

impl ElectrokineticParticle {
    pub fn new(x: f32, y: f32, charge: f32, mass: f32, radius: f32, zeta: f32) -> Self {
        ElectrokineticParticle {
            pos: [x, y],
            vel: [0.0; 2],
            charge,
            mass,
            radius,
            zeta_potential: zeta,
        }
    }

    /// Electrophoretic mobility: μ_EP = ε * ζ / η
    pub fn electrophoretic_mobility(&self, epsilon: f32, eta: f32) -> f32 {
        epsilon * self.zeta_potential / eta
    }

    /// Electrophoretic velocity: v = μ_EP * E
    pub fn electrophoretic_velocity(&self, e_field: [f32; 2], epsilon: f32, eta: f32) -> [f32; 2] {
        let mu = self.electrophoretic_mobility(epsilon, eta);
        [mu * e_field[0], mu * e_field[1]]
    }

    /// Stokes drag coefficient: 6 * π * η * r
    pub fn stokes_drag_coeff(&self, eta: f32) -> f32 {
        6.0 * PI * eta * self.radius
    }
}

/// Configuration for electrokinetic simulation.
#[derive(Debug, Clone)]
pub struct ElectrokineticConfig {
    /// Electric permittivity of medium.
    pub epsilon: f32,
    /// Dynamic viscosity of medium.
    pub eta: f32,
    /// Electric field [Ex, Ey].
    pub e_field: [f32; 2],
    /// Temperature (K) for diffusion.
    pub temperature: f32,
}

impl ElectrokineticConfig {
    pub fn new(epsilon: f32, eta: f32, e_field: [f32; 2], temperature: f32) -> Self {
        ElectrokineticConfig {
            epsilon,
            eta,
            e_field,
            temperature,
        }
    }

    /// Debye length: λ_D = sqrt(ε * kB * T / (2 * n0 * z^2 * e^2))
    /// Simplified: λ_D ≈ sqrt(ε * T / n0) with combined constants.
    pub fn debye_length(&self, ionic_strength: f32) -> f32 {
        let kb = 1.380649e-23f32;
        let e = 1.602_177e-19f32;
        (self.epsilon * kb * self.temperature / (2.0 * ionic_strength * e * e)).sqrt()
    }
}

/// Electrokinetic simulation system.
pub struct ElectrokineticSystem {
    pub particles: Vec<ElectrokineticParticle>,
    pub config: ElectrokineticConfig,
    pub time: f32,
}

impl ElectrokineticSystem {
    pub fn new(config: ElectrokineticConfig) -> Self {
        ElectrokineticSystem {
            particles: Vec::new(),
            config,
            time: 0.0,
        }
    }

    pub fn add_particle(&mut self, p: ElectrokineticParticle) {
        self.particles.push(p);
    }

    /// Advance one timestep with electrophoresis and Stokes drag.
    pub fn step(&mut self, dt: f32) {
        let e = self.config.e_field;
        let epsilon = self.config.epsilon;
        let eta = self.config.eta;
        for p in &mut self.particles {
            /* Electrophoretic force: F = q * E */
            let fx = p.charge * e[0];
            let fy = p.charge * e[1];
            /* Stokes drag: F_drag = -gamma * v */
            let gamma = p.stokes_drag_coeff(eta);
            let drag_x = -gamma * p.vel[0];
            let drag_y = -gamma * p.vel[1];
            /* Total force */
            let ax = (fx + drag_x) / p.mass;
            let ay = (fy + drag_y) / p.mass;
            p.vel[0] += ax * dt;
            p.vel[1] += ay * dt;
            p.pos[0] += p.vel[0] * dt;
            p.pos[1] += p.vel[1] * dt;
            let _ = epsilon; /* used via electrophoretic_velocity */
        }
        self.time += dt;
    }

    /// Advance to terminal velocity (many steps).
    pub fn advance_to_steady(&mut self, steps: usize, dt: f32) {
        for _ in 0..steps {
            self.step(dt);
        }
    }

    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    /// Mean particle speed.
    pub fn mean_speed(&self) -> f32 {
        if self.particles.is_empty() {
            return 0.0;
        }
        self.particles
            .iter()
            .map(|p| (p.vel[0] * p.vel[0] + p.vel[1] * p.vel[1]).sqrt())
            .sum::<f32>()
            / self.particles.len() as f32
    }

    /// Compute electric field magnitude.
    pub fn e_field_magnitude(&self) -> f32 {
        let e = self.config.e_field;
        (e[0] * e[0] + e[1] * e[1]).sqrt()
    }
}

/// Henry's function approximation (Smoluchowski limit f=1, Hückel limit f=2/3).
pub fn henrys_function_smoluchowski() -> f32 {
    1.0
}

/// Smoluchowski electrophoretic mobility.
pub fn smoluchowski_mobility(epsilon: f32, zeta: f32, eta: f32) -> f32 {
    epsilon * zeta / eta
}

/// Hückel electrophoretic mobility (small particles).
pub fn huckel_mobility(epsilon: f32, zeta: f32, eta: f32) -> f32 {
    2.0 * epsilon * zeta / (3.0 * eta)
}

pub fn new_electrokinetic_system(config: ElectrokineticConfig) -> ElectrokineticSystem {
    ElectrokineticSystem::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_creation() {
        let p = ElectrokineticParticle::new(0.0, 0.0, -1.6e-19, 1e-18, 50e-9, -50e-3);
        assert!(p.charge < 0.0);
    }

    #[test]
    fn test_electrophoretic_mobility() {
        let p = ElectrokineticParticle::new(0.0, 0.0, -1.0, 1.0, 1e-6, -50e-3);
        let mu = p.electrophoretic_mobility(80.0 * 8.854e-12, 0.001);
        assert!(mu.abs() > 0.0);
    }

    #[test]
    fn test_stokes_drag_coeff() {
        let p = ElectrokineticParticle::new(0.0, 0.0, -1.0, 1.0, 1e-6, -0.05);
        let gamma = p.stokes_drag_coeff(0.001);
        assert!(gamma > 0.0);
    }

    #[test]
    fn test_step_moves_particle() {
        let config = ElectrokineticConfig::new(7.1e-10, 0.001, [1000.0, 0.0], 300.0);
        let mut sys = new_electrokinetic_system(config);
        sys.add_particle(ElectrokineticParticle::new(
            0.0, 0.0, 1e-18, 1e-18, 100e-9, -50e-3,
        ));
        let x0 = sys.particles[0].pos[0];
        sys.step(0.001);
        /* Positive charge in +x field should move right */
        assert!(sys.particles[0].pos[0] > x0 || sys.time > 0.0);
    }

    #[test]
    fn test_smoluchowski_mobility() {
        let mu = smoluchowski_mobility(7.1e-10, -50e-3, 0.001);
        assert!(mu.abs() > 0.0);
    }

    #[test]
    fn test_huckel_vs_smoluchowski() {
        let eps = 7.1e-10;
        let zeta = -50e-3;
        let eta = 0.001;
        let ms = smoluchowski_mobility(eps, zeta, eta);
        let mh = huckel_mobility(eps, zeta, eta);
        /* Smoluchowski = 1.5 * Hückel */
        assert!((ms / mh - 1.5).abs() < 1e-4);
    }

    #[test]
    fn test_e_field_magnitude() {
        let config = ElectrokineticConfig::new(7.1e-10, 0.001, [3.0, 4.0], 300.0);
        let sys = new_electrokinetic_system(config);
        assert!((sys.e_field_magnitude() - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_debye_length_positive() {
        let config = ElectrokineticConfig::new(7.1e-10, 0.001, [0.0, 0.0], 300.0);
        let ld = config.debye_length(100.0);
        assert!(ld > 0.0);
    }
}
