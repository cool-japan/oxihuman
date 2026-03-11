// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! SPH (Smoothed Particle Hydrodynamics) fluid simulation.

use std::f32::consts::PI;

/// A single SPH fluid particle.
#[allow(dead_code)]
pub struct FluidSphParticle {
    pub id: u32,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub density: f32,
    pub pressure: f32,
    pub mass: f32,
}

/// Configuration parameters for an SPH simulation.
#[allow(dead_code)]
pub struct FluidSphConfig {
    pub kernel_radius: f32,
    pub rest_density: f32,
    pub pressure_constant: f32,
    pub viscosity: f32,
    pub gravity: [f32; 3],
}

/// World containing all SPH particles and configuration.
#[allow(dead_code)]
pub struct FluidSphWorld {
    pub particles: Vec<FluidSphParticle>,
    pub config: FluidSphConfig,
    pub next_id: u32,
    /// AABB bounds (min, max).
    pub bounds: ([f32; 3], [f32; 3]),
}

/// Return a sensible default SPH configuration.
#[allow(dead_code)]
pub fn default_sph_config() -> FluidSphConfig {
    FluidSphConfig {
        kernel_radius: 0.1,
        rest_density: 1000.0,
        pressure_constant: 200.0,
        viscosity: 0.01,
        gravity: [0.0, -9.8, 0.0],
    }
}

/// Create a new SPH world with the given AABB bounds.
#[allow(dead_code)]
pub fn new_sph_world(bounds_min: [f32; 3], bounds_max: [f32; 3]) -> FluidSphWorld {
    FluidSphWorld {
        particles: Vec::new(),
        config: default_sph_config(),
        next_id: 0,
        bounds: (bounds_min, bounds_max),
    }
}

/// Add a particle at `pos` with the given `mass`.  Returns its id.
#[allow(dead_code)]
pub fn add_sph_particle(world: &mut FluidSphWorld, pos: [f32; 3], mass: f32) -> u32 {
    let id = world.next_id;
    world.next_id += 1;
    world.particles.push(FluidSphParticle {
        id,
        position: pos,
        velocity: [0.0; 3],
        density: world.config.rest_density,
        pressure: 0.0,
        mass,
    });
    id
}

// ── SPH kernels ──────────────────────────────────────────────────────────────

/// Müller poly6 kernel W(r, h).
#[allow(dead_code)]
pub fn sph_kernel_poly6(r: f32, h: f32) -> f32 {
    if r >= h || r < 0.0 {
        return 0.0;
    }
    let h2 = h * h;
    let r2 = r * r;
    let coeff = 315.0 / (64.0 * PI * h.powi(9));
    coeff * (h2 - r2).powi(3)
}

/// Gradient of the spiky kernel ∇W(r_vec, r, h).
#[allow(dead_code)]
pub fn sph_kernel_spiky_grad(r_vec: [f32; 3], r: f32, h: f32) -> [f32; 3] {
    if r >= h || r < 1e-12 {
        return [0.0; 3];
    }
    let coeff = -45.0 / (PI * h.powi(6));
    let scale = coeff * (h - r).powi(2) / r;
    [scale * r_vec[0], scale * r_vec[1], scale * r_vec[2]]
}

/// Viscosity kernel Laplacian ∇²W(r, h).
#[allow(dead_code)]
pub fn sph_kernel_viscosity_lap(r: f32, h: f32) -> f32 {
    if r >= h || r < 0.0 {
        return 0.0;
    }
    45.0 / (PI * h.powi(6)) * (h - r)
}

// ── simulation steps ─────────────────────────────────────────────────────────

/// Compute density for each particle using the poly6 kernel.
#[allow(dead_code)]
pub fn compute_densities(world: &mut FluidSphWorld) {
    let h = world.config.kernel_radius;
    let n = world.particles.len();
    let mut densities = vec![0.0f32; n];
    for (i, pi) in world.particles.iter().enumerate() {
        let mut rho = 0.0f32;
        for pj in &world.particles {
            let dx = pi.position[0] - pj.position[0];
            let dy = pi.position[1] - pj.position[1];
            let dz = pi.position[2] - pj.position[2];
            let r = (dx * dx + dy * dy + dz * dz).sqrt();
            rho += pj.mass * sph_kernel_poly6(r, h);
        }
        densities[i] = rho.max(1e-6);
    }
    for (p, rho) in world.particles.iter_mut().zip(densities.iter()) {
        p.density = *rho;
    }
}

/// Compute pressure using the Tait equation: P = k * (rho - rho0).
#[allow(dead_code)]
pub fn compute_pressures(world: &mut FluidSphWorld) {
    let k = world.config.pressure_constant;
    let rho0 = world.config.rest_density;
    for p in &mut world.particles {
        p.pressure = k * (p.density - rho0).max(0.0);
    }
}

/// Compute pressure + viscosity + gravity forces for all particles.
#[allow(dead_code)]
pub fn compute_sph_forces(world: &FluidSphWorld) -> Vec<[f32; 3]> {
    let h = world.config.kernel_radius;
    let mu = world.config.viscosity;
    let g = world.config.gravity;
    let n = world.particles.len();
    let mut forces = vec![[0.0f32; 3]; n];

    for (i, pi) in world.particles.iter().enumerate() {
        let mut f = [pi.mass * g[0], pi.mass * g[1], pi.mass * g[2]];
        for (j, pj) in world.particles.iter().enumerate() {
            if i == j {
                continue;
            }
            let dx = pi.position[0] - pj.position[0];
            let dy = pi.position[1] - pj.position[1];
            let dz = pi.position[2] - pj.position[2];
            let r = (dx * dx + dy * dy + dz * dz).sqrt();
            if r >= h {
                continue;
            }
            // Pressure force
            let p_avg = (pi.pressure + pj.pressure) / 2.0;
            let rho_j = pj.density.max(1e-6);
            let grad = sph_kernel_spiky_grad([dx, dy, dz], r, h);
            let pscale = -pj.mass * p_avg / rho_j;
            f[0] += pscale * grad[0];
            f[1] += pscale * grad[1];
            f[2] += pscale * grad[2];

            // Viscosity force
            let lap = sph_kernel_viscosity_lap(r, h);
            let vscale = mu * pj.mass / rho_j * lap;
            f[0] += vscale * (pj.velocity[0] - pi.velocity[0]);
            f[1] += vscale * (pj.velocity[1] - pi.velocity[1]);
            f[2] += vscale * (pj.velocity[2] - pi.velocity[2]);
        }
        forces[i] = f;
    }
    forces
}

/// Integrate positions and velocities using semi-implicit Euler.
#[allow(dead_code)]
pub fn integrate_sph(world: &mut FluidSphWorld, forces: &[[f32; 3]], dt: f32) {
    for (p, f) in world.particles.iter_mut().zip(forces.iter()) {
        let inv_mass = if p.mass > 1e-12 { 1.0 / p.mass } else { 0.0 };
        p.velocity[0] += f[0] * inv_mass * dt;
        p.velocity[1] += f[1] * inv_mass * dt;
        p.velocity[2] += f[2] * inv_mass * dt;
        p.position[0] += p.velocity[0] * dt;
        p.position[1] += p.velocity[1] * dt;
        p.position[2] += p.velocity[2] * dt;
    }
}

/// Reflect particles off the AABB bounds (velocity restitution = 0.5).
#[allow(dead_code)]
pub fn enforce_sph_bounds(world: &mut FluidSphWorld) {
    let (bmin, bmax) = world.bounds;
    for p in &mut world.particles {
        for k in 0..3 {
            if p.position[k] < bmin[k] {
                p.position[k] = bmin[k];
                if p.velocity[k] < 0.0 {
                    p.velocity[k] = -p.velocity[k] * 0.5;
                }
            }
            if p.position[k] > bmax[k] {
                p.position[k] = bmax[k];
                if p.velocity[k] > 0.0 {
                    p.velocity[k] = -p.velocity[k] * 0.5;
                }
            }
        }
    }
}

/// Return the number of particles in the world.
#[allow(dead_code)]
pub fn sph_particle_count(world: &FluidSphWorld) -> usize {
    world.particles.len()
}

/// Perform one full SPH simulation step.
#[allow(dead_code)]
pub fn sph_step(world: &mut FluidSphWorld, dt: f32) {
    compute_densities(world);
    compute_pressures(world);
    let forces = compute_sph_forces(world);
    integrate_sph(world, &forces, dt);
    enforce_sph_bounds(world);
}

// ── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_world_with_particle() -> FluidSphWorld {
        let mut w = new_sph_world([-1.0; 3], [1.0; 3]);
        add_sph_particle(&mut w, [0.0; 3], 0.01);
        w
    }

    #[test]
    fn test_default_sph_config() {
        let cfg = default_sph_config();
        assert!(cfg.kernel_radius > 0.0);
        assert!(cfg.rest_density > 0.0);
        assert!(cfg.pressure_constant > 0.0);
    }

    #[test]
    fn test_new_sph_world() {
        let w = new_sph_world([-1.0; 3], [1.0; 3]);
        assert_eq!(sph_particle_count(&w), 0);
    }

    #[test]
    fn test_add_sph_particle() {
        let mut w = new_sph_world([-1.0; 3], [1.0; 3]);
        let id = add_sph_particle(&mut w, [0.0; 3], 0.01);
        assert_eq!(id, 0);
        assert_eq!(sph_particle_count(&w), 1);
    }

    #[test]
    fn test_sph_particle_count_multiple() {
        let mut w = new_sph_world([-1.0; 3], [1.0; 3]);
        for _ in 0..5 {
            add_sph_particle(&mut w, [0.0; 3], 0.01);
        }
        assert_eq!(sph_particle_count(&w), 5);
    }

    #[test]
    fn test_sph_kernel_poly6_positive_for_r_lt_h() {
        let h = 0.1;
        let r = 0.05;
        let w = sph_kernel_poly6(r, h);
        assert!(w > 0.0, "poly6 should be positive for r < h");
    }

    #[test]
    fn test_sph_kernel_poly6_zero_for_r_ge_h() {
        let h = 0.1;
        assert_eq!(sph_kernel_poly6(h, h), 0.0);
        assert_eq!(sph_kernel_poly6(h + 0.01, h), 0.0);
    }

    #[test]
    fn test_sph_kernel_poly6_larger_at_center() {
        let h = 0.1;
        let w0 = sph_kernel_poly6(0.0, h);
        let w1 = sph_kernel_poly6(0.05, h);
        assert!(w0 > w1, "kernel should be larger at r=0 than r=h/2");
    }

    #[test]
    fn test_sph_kernel_spiky_grad_zero_for_r_ge_h() {
        let h = 0.1;
        let g = sph_kernel_spiky_grad([h, 0.0, 0.0], h, h);
        assert_eq!(g, [0.0; 3]);
    }

    #[test]
    fn test_sph_kernel_viscosity_lap_positive() {
        let h = 0.1;
        let lap = sph_kernel_viscosity_lap(0.05, h);
        assert!(lap > 0.0);
    }

    #[test]
    fn test_compute_densities_runs() {
        let mut w = make_world_with_particle();
        compute_densities(&mut w);
        assert!(w.particles[0].density > 0.0);
    }

    #[test]
    fn test_compute_pressures_non_negative() {
        let mut w = make_world_with_particle();
        compute_densities(&mut w);
        compute_pressures(&mut w);
        for p in &w.particles {
            assert!(p.pressure >= 0.0);
        }
    }

    #[test]
    fn test_sph_step_no_nan() {
        let mut w = new_sph_world([-1.0; 3], [1.0; 3]);
        add_sph_particle(&mut w, [0.0, 0.5, 0.0], 0.01);
        add_sph_particle(&mut w, [0.0, 0.3, 0.0], 0.01);
        for _ in 0..10 {
            sph_step(&mut w, 0.001);
        }
        for p in &w.particles {
            assert!(!p.position[0].is_nan());
            assert!(!p.position[1].is_nan());
            assert!(!p.position[2].is_nan());
        }
    }

    #[test]
    fn test_enforce_sph_bounds_keeps_inside() {
        let mut w = new_sph_world([-1.0; 3], [1.0; 3]);
        add_sph_particle(&mut w, [2.0, 2.0, 2.0], 0.01);
        enforce_sph_bounds(&mut w);
        let p = &w.particles[0];
        assert!(p.position[0] <= 1.0);
        assert!(p.position[1] <= 1.0);
        assert!(p.position[2] <= 1.0);
    }

    #[test]
    fn test_enforce_sph_bounds_lower() {
        let mut w = new_sph_world([-1.0; 3], [1.0; 3]);
        add_sph_particle(&mut w, [-2.0, -2.0, -2.0], 0.01);
        enforce_sph_bounds(&mut w);
        let p = &w.particles[0];
        assert!(p.position[0] >= -1.0);
        assert!(p.position[1] >= -1.0);
        assert!(p.position[2] >= -1.0);
    }

    #[test]
    fn test_add_particle_id_increments() {
        let mut w = new_sph_world([-1.0; 3], [1.0; 3]);
        let id0 = add_sph_particle(&mut w, [0.0; 3], 0.01);
        let id1 = add_sph_particle(&mut w, [0.1, 0.0, 0.0], 0.01);
        assert_eq!(id0, 0);
        assert_eq!(id1, 1);
    }
}
