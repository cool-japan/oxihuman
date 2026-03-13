// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Smoothed Particle Hydrodynamics (SPH) fluid simulation.

use std::f32::consts::PI;

// ── Types ─────────────────────────────────────────────────────────────────────

/// Global parameters for the SPH simulation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SphConfig {
    /// Rest (reference) density ρ₀ [kg/m³].
    pub rest_density: f32,
    /// Dynamic viscosity μ [Pa·s].
    pub viscosity: f32,
    /// Pressure stiffness constant k (Tait equation).
    pub pressure_stiffness: f32,
    /// Smoothing radius h [m].
    pub particle_radius: f32,
    /// Gravity acceleration vector [m/s²].
    pub gravity: [f32; 3],
}

/// A single SPH particle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SphParticle {
    /// World-space position [m].
    pub position: [f32; 3],
    /// Velocity [m/s].
    pub velocity: [f32; 3],
    /// Mass [kg].
    pub mass: f32,
    /// Estimated density [kg/m³] (updated each step).
    pub density: f32,
    /// Pressure [Pa] (updated each step).
    pub pressure: f32,
}

/// The full SPH simulation state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SphSystem {
    /// All particles in the simulation.
    pub particles: Vec<SphParticle>,
    /// Simulation parameters.
    pub config: SphConfig,
    /// Accumulated simulation time [s].
    pub time: f32,
}

/// Per-step diagnostic output.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SphResult {
    /// Mean density over all particles.
    pub avg_density: f32,
    /// Maximum pressure among all particles.
    pub max_pressure: f32,
    /// Total kinetic energy ½Σmv².
    pub kinetic_energy: f32,
}

// ── Internal helpers ──────────────────────────────────────────────────────────

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Return default [`SphConfig`].
#[allow(dead_code)]
pub fn default_sph_config() -> SphConfig {
    SphConfig {
        rest_density: 1000.0,
        viscosity: 0.01,
        pressure_stiffness: 200.0,
        particle_radius: 0.05,
        gravity: [0.0, -9.81, 0.0],
    }
}

/// Create a new particle at `pos` with the given `mass`.
#[allow(dead_code)]
pub fn new_sph_particle(pos: [f32; 3], mass: f32) -> SphParticle {
    SphParticle {
        position: pos,
        velocity: [0.0; 3],
        mass,
        density: 1000.0,
        pressure: 0.0,
    }
}

/// Create an empty SPH system with the given configuration.
#[allow(dead_code)]
pub fn new_sph_system(cfg: SphConfig) -> SphSystem {
    SphSystem {
        particles: Vec::new(),
        config: cfg,
        time: 0.0,
    }
}

/// Add a particle to the system.
#[allow(dead_code)]
pub fn add_sph_particle(sys: &mut SphSystem, p: SphParticle) {
    sys.particles.push(p);
}

/// Poly6 smoothing kernel W_poly6(r, h).
///
/// Normalised for 3D: 315 / (64 π h⁹).
#[allow(dead_code)]
pub fn sph_kernel_poly6(r: f32, h: f32) -> f32 {
    if r < 0.0 || r > h {
        return 0.0;
    }
    let h2 = h * h;
    let r2 = r * r;
    let diff = h2 - r2;
    (315.0 / (64.0 * PI * h.powi(9))) * diff * diff * diff
}

/// Gradient magnitude of the Spiky kernel ∂W_spiky/∂r.
///
/// Normalised for 3D: –45 / (π h⁶).
#[allow(dead_code)]
pub fn sph_kernel_spiky_grad(r: f32, h: f32) -> f32 {
    if r <= 0.0 || r > h {
        return 0.0;
    }
    let diff = h - r;
    -45.0 / (PI * h.powi(6)) * diff * diff
}

/// Compute density for `particle` given a slice of neighbor particles.
#[allow(dead_code)]
pub fn compute_density(particle: &SphParticle, neighbors: &[SphParticle], h: f32) -> f32 {
    let mut rho = 0.0_f32;
    for nbr in neighbors {
        let r = dist3(particle.position, nbr.position);
        rho += nbr.mass * sph_kernel_poly6(r, h);
    }
    rho
}

/// Step the SPH simulation by `dt` seconds.
///
/// Implements a basic Weakly Compressible SPH (WCSPH) loop:
/// 1. Compute densities via Poly6 kernel.
/// 2. Compute pressures via Tait equation.
/// 3. Integrate velocities with gravity + pressure gradient + viscosity.
/// 4. Integrate positions.
#[allow(dead_code)]
pub fn step_sph(sys: &mut SphSystem, dt: f32) -> SphResult {
    let n = sys.particles.len();
    if n == 0 {
        sys.time += dt;
        return SphResult {
            avg_density: 0.0,
            max_pressure: 0.0,
            kinetic_energy: 0.0,
        };
    }

    let h = sys.config.particle_radius;
    let rho0 = sys.config.rest_density;
    let k = sys.config.pressure_stiffness;
    let mu = sys.config.viscosity;
    let g = sys.config.gravity;

    // ── 1. Density & pressure ─────────────────────────────────────────────────
    // Snapshot positions/masses for neighbor queries.
    let snapshot: Vec<SphParticle> = sys.particles.clone();

    for i in 0..n {
        let rho = compute_density(&snapshot[i], &snapshot, h).max(1e-6);
        sys.particles[i].density = rho;
        // Tait equation: p = k (ρ/ρ₀ – 1)
        sys.particles[i].pressure = k * (rho / rho0 - 1.0).max(0.0);
    }

    // ── 2. Accelerations ──────────────────────────────────────────────────────
    let pressures: Vec<f32> = sys.particles.iter().map(|p| p.pressure).collect();
    let densities: Vec<f32> = sys.particles.iter().map(|p| p.density).collect();

    let mut accel: Vec<[f32; 3]> = vec![[0.0; 3]; n];

    for i in 0..n {
        let pi = snapshot[i].position;
        let vi = snapshot[i].velocity;
        let rhoi = densities[i];
        let pri = pressures[i];

        let mut ax = g[0];
        let mut ay = g[1];
        let mut az = g[2];

        for j in 0..n {
            if i == j {
                continue;
            }
            let pj = snapshot[j].position;
            let r = dist3(pi, pj);
            if r < 1e-10 || r > h {
                continue;
            }

            let rhoj = densities[j];
            let prj = pressures[j];
            let mj = snapshot[j].mass;

            // Pressure force: –mj · (pi/ρi² + pj/ρj²) · ∇W_spiky
            let grad_w = sph_kernel_spiky_grad(r, h);
            let pressure_coeff = -mj * (pri / (rhoi * rhoi) + prj / (rhoj * rhoj)) * grad_w;

            // Direction from j to i (normalised)
            let inv_r = 1.0 / r;
            let dx = (pi[0] - pj[0]) * inv_r;
            let dy = (pi[1] - pj[1]) * inv_r;
            let dz = (pi[2] - pj[2]) * inv_r;

            ax += pressure_coeff * dx;
            ay += pressure_coeff * dy;
            az += pressure_coeff * dz;

            // Viscosity force: μ mj (vj–vi) / ρj · ∇²W_visc ≈ using poly6 kernel
            let w = sph_kernel_poly6(r, h);
            let vj = snapshot[j].velocity;
            let visc_coeff = mu * mj / rhoj * w;
            ax += visc_coeff * (vj[0] - vi[0]);
            ay += visc_coeff * (vj[1] - vi[1]);
            az += visc_coeff * (vj[2] - vi[2]);
        }

        accel[i] = [ax, ay, az];
    }

    // ── 3 & 4. Semi-implicit Euler integration ────────────────────────────────
    for (i, &acc) in accel.iter().enumerate().take(n) {
        sys.particles[i].velocity[0] += acc[0] * dt;
        sys.particles[i].velocity[1] += acc[1] * dt;
        sys.particles[i].velocity[2] += acc[2] * dt;
        sys.particles[i].position[0] += sys.particles[i].velocity[0] * dt;
        sys.particles[i].position[1] += sys.particles[i].velocity[1] * dt;
        sys.particles[i].position[2] += sys.particles[i].velocity[2] * dt;
    }

    sys.time += dt;

    // ── Diagnostics ───────────────────────────────────────────────────────────
    let avg_density = sys.particles.iter().map(|p| p.density).sum::<f32>() / n as f32;
    let max_pressure = sys
        .particles
        .iter()
        .map(|p| p.pressure)
        .fold(0.0_f32, f32::max);
    let kinetic_energy = sys
        .particles
        .iter()
        .map(|p| {
            let v2 = dot3(p.velocity, p.velocity);
            0.5 * p.mass * v2
        })
        .sum();

    SphResult {
        avg_density,
        max_pressure,
        kinetic_energy,
    }
}

/// Number of particles in the system.
#[allow(dead_code)]
#[inline]
pub fn sph_particle_count(sys: &SphSystem) -> usize {
    sys.particles.len()
}

/// Serialize the SPH system summary to JSON.
#[allow(dead_code)]
pub fn sph_system_to_json(sys: &SphSystem) -> String {
    format!(
        "{{\"particle_count\":{},\"time\":{},\"rest_density\":{}}}",
        sys.particles.len(),
        sys.time,
        sys.config.rest_density
    )
}

/// Serialize an [`SphResult`] to JSON.
#[allow(dead_code)]
pub fn sph_result_to_json(r: &SphResult) -> String {
    format!(
        "{{\"avg_density\":{},\"max_pressure\":{},\"kinetic_energy\":{}}}",
        r.avg_density, r.max_pressure, r.kinetic_energy
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn two_particle_system() -> SphSystem {
        let cfg = default_sph_config();
        let mut sys = new_sph_system(cfg);
        add_sph_particle(&mut sys, new_sph_particle([0.0, 0.0, 0.0], 0.1));
        add_sph_particle(&mut sys, new_sph_particle([0.02, 0.0, 0.0], 0.1));
        sys
    }

    #[test]
    fn default_config_smoke() {
        let cfg = default_sph_config();
        assert!(cfg.rest_density > 0.0);
        assert!(cfg.particle_radius > 0.0);
    }

    #[test]
    fn particle_count_correct() {
        let sys = two_particle_system();
        assert_eq!(sph_particle_count(&sys), 2);
    }

    #[test]
    fn poly6_kernel_zero_outside_h() {
        let h = 0.05;
        assert_eq!(sph_kernel_poly6(h + 0.001, h), 0.0);
        assert_eq!(sph_kernel_poly6(-0.01, h), 0.0);
    }

    #[test]
    fn poly6_kernel_positive_inside_h() {
        let h = 0.05;
        assert!(sph_kernel_poly6(0.0, h) > 0.0);
        assert!(sph_kernel_poly6(h * 0.5, h) > 0.0);
    }

    #[test]
    fn spiky_grad_zero_outside_h() {
        let h = 0.05;
        assert_eq!(sph_kernel_spiky_grad(h + 0.001, h), 0.0);
        assert_eq!(sph_kernel_spiky_grad(-0.01, h), 0.0);
    }

    #[test]
    fn step_advances_time() {
        let mut sys = two_particle_system();
        let _result = step_sph(&mut sys, 0.001);
        assert!((sys.time - 0.001).abs() < 1e-8);
    }

    #[test]
    fn step_returns_non_negative_density() {
        let mut sys = two_particle_system();
        let result = step_sph(&mut sys, 0.001);
        assert!(result.avg_density >= 0.0);
        assert!(result.max_pressure >= 0.0);
    }

    #[test]
    fn json_output_contains_fields() {
        let sys = two_particle_system();
        let json = sph_system_to_json(&sys);
        assert!(json.contains("particle_count"));
        assert!(json.contains("time"));

        let r = SphResult {
            avg_density: 100.0,
            max_pressure: 5.0,
            kinetic_energy: 0.5,
        };
        let rjson = sph_result_to_json(&r);
        assert!(rjson.contains("avg_density"));
        assert!(rjson.contains("kinetic_energy"));
    }
}
