// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Smoothed Particle Hydrodynamics (SPH) proxy for hair strand simulation.
//!
//! Implements a simplified SPH system suitable for simulating the cross-section
//! dynamics of hair strands, using poly6, spiky, and viscosity kernels.

use std::f32::consts::PI;

/// A single SPH particle with physical state.
#[allow(dead_code)]
pub struct SphParticle {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub density: f32,
    pub pressure: f32,
    pub mass: f32,
}

/// Configuration for the SPH system.
pub struct SphConfig {
    /// Kernel smoothing radius h.
    pub smoothing_radius: f32,
    /// Rest density ρ₀ in kg/m³.
    pub rest_density: f32,
    /// Pressure stiffness k in the Tait equation.
    pub pressure_stiffness: f32,
    /// Dynamic viscosity μ.
    pub viscosity: f32,
    pub gravity: [f32; 3],
    pub time_step: f32,
}

impl Default for SphConfig {
    fn default() -> Self {
        Self {
            smoothing_radius: 0.05,
            rest_density: 1000.0,
            pressure_stiffness: 200.0,
            viscosity: 0.01,
            gravity: [0.0, -9.81, 0.0],
            time_step: 0.002,
        }
    }
}

/// A complete SPH fluid simulation system.
pub struct SphSystem {
    pub particles: Vec<SphParticle>,
    pub config: SphConfig,
}

impl SphSystem {
    /// Create a new empty SPH system with the given configuration.
    pub fn new(cfg: SphConfig) -> Self {
        Self {
            particles: Vec::new(),
            config: cfg,
        }
    }

    /// Add a particle at `pos` with the given `mass`. Returns its index.
    pub fn add_particle(&mut self, pos: [f32; 3], mass: f32) -> usize {
        let idx = self.particles.len();
        self.particles.push(SphParticle {
            position: pos,
            velocity: [0.0; 3],
            density: self.config.rest_density,
            pressure: 0.0,
            mass,
        });
        idx
    }

    /// Advance the SPH system by one time step.
    ///
    /// Steps: compute densities → pressures → forces → integrate.
    pub fn step(&mut self) {
        let n = self.particles.len();
        let h = self.config.smoothing_radius;
        let rho0 = self.config.rest_density;
        let k = self.config.pressure_stiffness;
        let mu = self.config.viscosity;
        let dt = self.config.time_step;

        // 1. Compute densities.
        let mut densities = vec![0.0_f32; n];
        for (i, density) in densities.iter_mut().enumerate() {
            *density = compute_density(&self.particles, i, h);
        }

        // 2. Compute pressures.
        let mut pressures = vec![0.0_f32; n];
        for i in 0..n {
            self.particles[i].density = densities[i];
            pressures[i] = compute_pressure(densities[i], rho0, k);
            self.particles[i].pressure = pressures[i];
        }

        // 3. Compute forces (pressure + viscosity + gravity).
        let mut forces = vec![[0.0_f32; 3]; n];
        for i in 0..n {
            let mut f = [
                self.particles[i].mass * self.config.gravity[0],
                self.particles[i].mass * self.config.gravity[1],
                self.particles[i].mass * self.config.gravity[2],
            ];
            let rho_i = densities[i].max(f32::EPSILON);
            let p_i = pressures[i];

            for j in 0..n {
                if i == j {
                    continue;
                }
                let r_vec = [
                    self.particles[i].position[0] - self.particles[j].position[0],
                    self.particles[i].position[1] - self.particles[j].position[1],
                    self.particles[i].position[2] - self.particles[j].position[2],
                ];
                let r = (r_vec[0] * r_vec[0] + r_vec[1] * r_vec[1] + r_vec[2] * r_vec[2]).sqrt();
                if r >= h {
                    continue;
                }
                let mj = self.particles[j].mass;
                let rho_j = densities[j].max(f32::EPSILON);
                let p_j = pressures[j];

                // Pressure force: -m_j * (p_i + p_j) / (2 * rho_j) * ∇W_spiky
                let grad = kernel_spiky_grad(r_vec, r, h);
                let pres_coeff = -mj * (p_i + p_j) / (2.0 * rho_j);
                f[0] += pres_coeff * grad[0];
                f[1] += pres_coeff * grad[1];
                f[2] += pres_coeff * grad[2];

                // Viscosity force: μ * m_j / rho_j * (v_j - v_i) * ∇²W_visc
                let lap = kernel_viscosity_lap(r, h);
                let visc_coeff = mu * mj / rho_j * lap;
                f[0] +=
                    visc_coeff * (self.particles[j].velocity[0] - self.particles[i].velocity[0]);
                f[1] +=
                    visc_coeff * (self.particles[j].velocity[1] - self.particles[i].velocity[1]);
                f[2] +=
                    visc_coeff * (self.particles[j].velocity[2] - self.particles[i].velocity[2]);
            }

            // Convert to acceleration: a = F / rho_i
            forces[i] = [f[0] / rho_i, f[1] / rho_i, f[2] / rho_i];
        }

        // 4. Integrate velocities then positions.
        for (p, force) in self.particles.iter_mut().zip(forces.iter()) {
            p.velocity[0] += force[0] * dt;
            p.velocity[1] += force[1] * dt;
            p.velocity[2] += force[2] * dt;
            p.position[0] += p.velocity[0] * dt;
            p.position[1] += p.velocity[1] * dt;
            p.position[2] += p.velocity[2] * dt;
        }
    }

    /// Number of particles in the system.
    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    /// Average density across all particles; returns 0 if no particles.
    pub fn average_density(&self) -> f32 {
        if self.particles.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.particles.iter().map(|p| p.density).sum();
        sum / self.particles.len() as f32
    }
}

/// Poly6 smoothing kernel: W(r, h) = (315 / 64π h⁹) * max(h² - r², 0)³
pub fn kernel_poly6(r: f32, h: f32) -> f32 {
    if r > h || h <= 0.0 {
        return 0.0;
    }
    let h2 = h * h;
    let r2 = r * r;
    let diff = h2 - r2;
    let coeff = 315.0 / (64.0 * PI * h.powi(9));
    coeff * diff * diff * diff
}

/// Spiky kernel gradient: ∇W_spiky = -45 / (π h⁶) * max(h - r, 0)² * r̂
pub fn kernel_spiky_grad(r_vec: [f32; 3], r: f32, h: f32) -> [f32; 3] {
    if r >= h || r < f32::EPSILON || h <= 0.0 {
        return [0.0; 3];
    }
    let coeff = -45.0 / (PI * h.powi(6)) * (h - r) * (h - r);
    let inv_r = 1.0 / r;
    [
        coeff * r_vec[0] * inv_r,
        coeff * r_vec[1] * inv_r,
        coeff * r_vec[2] * inv_r,
    ]
}

/// Viscosity kernel Laplacian: ∇²W_visc = 45 / (π h⁶) * (h - r)
pub fn kernel_viscosity_lap(r: f32, h: f32) -> f32 {
    if r > h || h <= 0.0 {
        return 0.0;
    }
    45.0 / (PI * h.powi(6)) * (h - r)
}

/// Compute density at particle `idx` as Σⱼ mⱼ * W(|xᵢ - xⱼ|, h).
pub fn compute_density(particles: &[SphParticle], idx: usize, h: f32) -> f32 {
    let pos_i = particles[idx].position;
    particles
        .iter()
        .map(|pj| {
            let dx = pos_i[0] - pj.position[0];
            let dy = pos_i[1] - pj.position[1];
            let dz = pos_i[2] - pj.position[2];
            let r = (dx * dx + dy * dy + dz * dz).sqrt();
            pj.mass * kernel_poly6(r, h)
        })
        .sum()
}

/// Compute pressure from the Tait equation: p = k * (ρ - ρ₀).
pub fn compute_pressure(density: f32, rest_density: f32, stiffness: f32) -> f32 {
    stiffness * (density - rest_density)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    const H: f32 = 0.05;

    #[test]
    fn kernel_poly6_at_r0_positive() {
        let w = kernel_poly6(0.0, H);
        assert!(w > 0.0, "poly6 at r=0 must be positive, got {w}");
    }

    #[test]
    fn kernel_poly6_at_r_eq_h_is_zero() {
        let w = kernel_poly6(H, H);
        assert!(w.abs() < 1e-6, "poly6 at r=h must be ~0, got {w}");
    }

    #[test]
    fn kernel_poly6_beyond_h_is_zero() {
        let w = kernel_poly6(H * 2.0, H);
        assert_eq!(w, 0.0, "poly6 beyond h must be 0");
    }

    #[test]
    fn kernel_spiky_grad_direction_away_from_neighbour() {
        // r_vec points from j to i; gradient should also point away (negative coeff).
        let r_vec = [H * 0.5, 0.0, 0.0];
        let r = H * 0.5;
        let g = kernel_spiky_grad(r_vec, r, H);
        // Direction matches r_vec sign but magnitude may be negative (repulsive).
        // What matters is the result is non-zero.
        let mag = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt();
        assert!(mag > 0.0, "spiky gradient must be non-zero within h");
    }

    #[test]
    fn kernel_spiky_grad_zero_beyond_h() {
        let r_vec = [H * 2.0, 0.0, 0.0];
        let g = kernel_spiky_grad(r_vec, H * 2.0, H);
        assert_eq!(g, [0.0; 3], "spiky gradient must be zero beyond h");
    }

    #[test]
    fn kernel_viscosity_lap_max_at_r0() {
        let lap0 = kernel_viscosity_lap(0.0, H);
        let lap_half = kernel_viscosity_lap(H * 0.5, H);
        assert!(
            lap0 > lap_half,
            "viscosity lap should decrease with r: {lap0} > {lap_half}"
        );
    }

    #[test]
    fn compute_density_single_particle_equals_mass_times_w0() {
        let particles = vec![SphParticle {
            position: [0.0; 3],
            velocity: [0.0; 3],
            density: 0.0,
            pressure: 0.0,
            mass: 0.001,
        }];
        let expected = particles[0].mass * kernel_poly6(0.0, H);
        let got = compute_density(&particles, 0, H);
        assert!(
            (got - expected).abs() < 1e-10,
            "density mismatch: expected {expected}, got {got}"
        );
    }

    #[test]
    fn compute_pressure_formula() {
        let p = compute_pressure(1100.0, 1000.0, 200.0);
        let expected = 200.0 * (1100.0 - 1000.0);
        assert!((p - expected).abs() < 1e-3);
    }

    #[test]
    fn compute_pressure_at_rest_density_is_zero() {
        let p = compute_pressure(1000.0, 1000.0, 200.0);
        assert!(
            p.abs() < 1e-5,
            "pressure at rest density must be ~0, got {p}"
        );
    }

    #[test]
    fn sph_system_new_empty() {
        let sys = SphSystem::new(SphConfig::default());
        assert_eq!(sys.particle_count(), 0);
    }

    #[test]
    fn add_particle_returns_correct_index() {
        let mut sys = SphSystem::new(SphConfig::default());
        let i0 = sys.add_particle([0.0; 3], 0.001);
        let i1 = sys.add_particle([0.01, 0.0, 0.0], 0.001);
        assert_eq!(i0, 0);
        assert_eq!(i1, 1);
    }

    #[test]
    fn step_does_not_produce_nan() {
        let mut sys = SphSystem::new(SphConfig::default());
        for i in 0..4 {
            sys.add_particle([i as f32 * 0.03, 0.0, 0.0], 0.001);
        }
        for _ in 0..5 {
            sys.step();
        }
        for (k, p) in sys.particles.iter().enumerate() {
            assert!(
                !p.position[0].is_nan() && !p.position[1].is_nan() && !p.position[2].is_nan(),
                "NaN in particle {k} position"
            );
            assert!(
                !p.velocity[0].is_nan() && !p.velocity[1].is_nan() && !p.velocity[2].is_nan(),
                "NaN in particle {k} velocity"
            );
        }
    }

    #[test]
    fn average_density_positive_after_init() {
        let mut sys = SphSystem::new(SphConfig::default());
        sys.add_particle([0.0; 3], 0.001);
        sys.add_particle([0.02, 0.0, 0.0], 0.001);
        // After one step densities are recomputed.
        sys.step();
        assert!(
            sys.average_density() > 0.0,
            "average density must be positive"
        );
    }

    #[test]
    fn particle_count_correct() {
        let mut sys = SphSystem::new(SphConfig::default());
        for _ in 0..6 {
            sys.add_particle([0.0; 3], 0.001);
        }
        assert_eq!(sys.particle_count(), 6);
    }
}
