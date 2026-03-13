// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! SPH v2 with WCSPH pressure and XSPH viscosity.

/// An SPH v2 particle.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SphV2Particle {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub density: f32,
    pub pressure: f32,
    pub mass: f32,
}

impl SphV2Particle {
    #[allow(dead_code)]
    pub fn new(pos: [f32; 3], mass: f32) -> Self {
        Self {
            pos,
            vel: [0.0; 3],
            density: 0.0,
            pressure: 0.0,
            mass,
        }
    }
}

/// WCSPH configuration.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WcsphConfig {
    pub smoothing_radius: f32,
    pub rest_density: f32,
    pub stiffness: f32,
    pub gamma: f32,
    pub viscosity: f32,
    pub xsph_coeff: f32,
    pub gravity: [f32; 3],
}

impl Default for WcsphConfig {
    fn default() -> Self {
        Self {
            smoothing_radius: 0.1,
            rest_density: 1000.0,
            stiffness: 1.0,
            gamma: 7.0,
            viscosity: 0.01,
            xsph_coeff: 0.1,
            gravity: [0.0, -9.8, 0.0],
        }
    }
}

/// Poly6 kernel for density computation.
#[allow(dead_code)]
pub fn sph2_poly6(r_sq: f32, h: f32) -> f32 {
    let h_sq = h * h;
    if r_sq > h_sq {
        return 0.0;
    }
    let diff = h_sq - r_sq;
    315.0 / (64.0 * std::f32::consts::PI * h.powi(9)) * diff.powi(3)
}

/// Spiky kernel gradient magnitude.
#[allow(dead_code)]
pub fn sph2_spiky_grad_mag(r: f32, h: f32) -> f32 {
    if r > h || r < 1e-8 {
        return 0.0;
    }
    -45.0 / (std::f32::consts::PI * h.powi(6)) * (h - r).powi(2)
}

/// Viscosity kernel Laplacian.
#[allow(dead_code)]
pub fn sph2_viscosity_lap(r: f32, h: f32) -> f32 {
    if r > h {
        return 0.0;
    }
    45.0 / (std::f32::consts::PI * h.powi(6)) * (h - r)
}

fn dist3(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    (0..3).map(|i| (a[i] - b[i]).powi(2)).sum::<f32>().sqrt()
}

/// Compute densities for all particles.
#[allow(dead_code)]
pub fn sph2_compute_densities(particles: &mut [SphV2Particle], config: &WcsphConfig) {
    let n = particles.len();
    let h = config.smoothing_radius;
    let mut densities = vec![0.0f32; n];
    for i in 0..n {
        let pi = particles[i].pos;
        let mi = particles[i].mass;
        for p in particles.iter() {
            let pj = p.pos;
            let mj = p.mass;
            let r_sq = (0..3).map(|k| (pi[k] - pj[k]).powi(2)).sum();
            densities[i] += mj * sph2_poly6(r_sq, h);
            let _ = mi;
        }
    }
    for (p, d) in particles.iter_mut().zip(densities.iter()) {
        p.density = *d;
    }
}

/// Compute pressures using Tait equation (WCSPH).
#[allow(dead_code)]
pub fn sph2_compute_pressures(particles: &mut [SphV2Particle], config: &WcsphConfig) {
    let rho0 = config.rest_density;
    let k = config.stiffness;
    let g = config.gamma;
    for p in particles.iter_mut() {
        let ratio = (p.density / rho0).max(1.0);
        p.pressure = k * (ratio.powf(g) - 1.0);
    }
}

/// Integrate particles with pressure + viscosity forces.
#[allow(dead_code)]
pub fn sph2_integrate(particles: &mut [SphV2Particle], config: &WcsphConfig, dt: f32) {
    let n = particles.len();
    let h = config.smoothing_radius;
    let mut forces = vec![[0.0f32; 3]; n];

    for i in 0..n {
        for j in 0..n {
            if i == j {
                continue;
            }
            let pi = particles[i].pos;
            let pj = particles[j].pos;
            let r = dist3(&pi, &pj);
            if r > h || r < 1e-8 {
                continue;
            }
            let mj = particles[j].mass;
            let rho_j = particles[j].density.max(1e-5);
            let rho_i = particles[i].density.max(1e-5);
            let dir = [
                (pi[0] - pj[0]) / r,
                (pi[1] - pj[1]) / r,
                (pi[2] - pj[2]) / r,
            ];

            let pres_grad = sph2_spiky_grad_mag(r, h);
            let pres_term = mj
                * (particles[i].pressure / (rho_i * rho_i)
                    + particles[j].pressure / (rho_j * rho_j))
                * pres_grad;
            for (fk, dk) in forces[i].iter_mut().zip(dir.iter()) {
                *fk += pres_term * dk;
            }

            let visc_lap = sph2_viscosity_lap(r, h);
            let visc_term = config.viscosity * mj * visc_lap / rho_j;
            let vel_j = particles[j].vel;
            let vel_i = particles[i].vel;
            for (fk, (vj, vi)) in forces[i].iter_mut().zip(vel_j.iter().zip(vel_i.iter())) {
                *fk += visc_term * (vj - vi);
            }
        }
        let gravity = config.gravity;
        let density_i = particles[i].density;
        for (fk, gk) in forces[i].iter_mut().zip(gravity.iter()) {
            *fk += gk * density_i;
        }
    }

    for (p, force) in particles.iter_mut().zip(forces.iter()) {
        let inv_rho = 1.0 / p.density.max(1e-5);
        p.vel[0] += force[0] * inv_rho * dt;
        p.vel[1] += force[1] * inv_rho * dt;
        p.vel[2] += force[2] * inv_rho * dt;
        p.pos[0] += p.vel[0] * dt;
        p.pos[1] += p.vel[1] * dt;
        p.pos[2] += p.vel[2] * dt;
    }
}

/// XSPH velocity correction.
#[allow(dead_code)]
pub fn sph2_xsph_correction(particles: &mut [SphV2Particle], config: &WcsphConfig) {
    let n = particles.len();
    let h = config.smoothing_radius;
    let eps = config.xsph_coeff;
    let mut corr = vec![[0.0f32; 3]; n];
    for i in 0..n {
        for j in 0..n {
            if i == j {
                continue;
            }
            let pi = particles[i].pos;
            let pj = particles[j].pos;
            let r_sq = (0..3).map(|k| (pi[k] - pj[k]).powi(2)).sum();
            let w = sph2_poly6(r_sq, h);
            let mj = particles[j].mass;
            let rho_j = particles[j].density.max(1e-5);
            let vel_j = particles[j].vel;
            let vel_i = particles[i].vel;
            for (ck, (vj, vi)) in corr[i].iter_mut().zip(vel_j.iter().zip(vel_i.iter())) {
                *ck += eps * mj / rho_j * (vj - vi) * w;
            }
        }
    }
    for (p, c) in particles.iter_mut().zip(corr.iter()) {
        for (vel_k, ck) in p.vel.iter_mut().zip(c.iter()) {
            *vel_k += ck;
        }
    }
}

/// Total kinetic energy.
#[allow(dead_code)]
pub fn sph2_kinetic_energy(particles: &[SphV2Particle]) -> f32 {
    particles
        .iter()
        .map(|p| {
            let v2: f32 = p.vel.iter().map(|x| x * x).sum();
            0.5 * p.mass * v2
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_particles() -> Vec<SphV2Particle> {
        vec![
            SphV2Particle::new([0.0, 0.0, 0.0], 1.0),
            SphV2Particle::new([0.05, 0.0, 0.0], 1.0),
        ]
    }

    #[test]
    fn poly6_zero_outside_h() {
        assert_eq!(sph2_poly6(1.0, 0.5), 0.0);
    }

    #[test]
    fn poly6_positive_inside_h() {
        assert!(sph2_poly6(0.0, 1.0) > 0.0);
    }

    #[test]
    fn spiky_grad_mag_zero_outside_h() {
        assert_eq!(sph2_spiky_grad_mag(2.0, 1.0), 0.0);
    }

    #[test]
    fn viscosity_lap_zero_outside_h() {
        assert_eq!(sph2_viscosity_lap(2.0, 1.0), 0.0);
    }

    #[test]
    fn compute_densities_nonzero() {
        let mut ps = sample_particles();
        let config = WcsphConfig::default();
        sph2_compute_densities(&mut ps, &config);
        assert!(ps[0].density > 0.0);
    }

    #[test]
    fn compute_pressures_nonnegative() {
        let mut ps = sample_particles();
        let config = WcsphConfig::default();
        sph2_compute_densities(&mut ps, &config);
        sph2_compute_pressures(&mut ps, &config);
        assert!(ps[0].pressure >= 0.0);
    }

    #[test]
    fn integrate_moves_particles() {
        let mut ps = sample_particles();
        let config = WcsphConfig {
            rest_density: 1.0,
            ..WcsphConfig::default()
        };
        sph2_compute_densities(&mut ps, &config);
        sph2_compute_pressures(&mut ps, &config);
        let before = ps[0].pos[1];
        sph2_integrate(&mut ps, &config, 0.01);
        assert!(ps[0].pos[1] != before);
    }

    #[test]
    fn kinetic_energy_zero_at_rest() {
        let ps = sample_particles();
        assert!(sph2_kinetic_energy(&ps) < 1e-8);
    }

    #[test]
    fn xsph_correction_runs_without_panic() {
        let mut ps = sample_particles();
        let config = WcsphConfig::default();
        sph2_compute_densities(&mut ps, &config);
        sph2_xsph_correction(&mut ps, &config);
    }

    #[test]
    fn new_particle_zero_vel() {
        let p = SphV2Particle::new([0.0, 0.0, 0.0], 1.0);
        assert_eq!(p.vel, [0.0; 3]);
    }
}
