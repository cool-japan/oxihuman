// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Meshfree SPH solver stub — a kernel-based smoothed-particle hydrodynamics
//! solver that needs no background mesh.

use std::f64::consts::PI;

/// SPH particle.
#[derive(Debug, Clone)]
pub struct SphParticle {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub mass: f64,
    pub density: f64,
    pub pressure: f64,
}

impl SphParticle {
    pub fn new(x: f64, y: f64, mass: f64) -> Self {
        Self {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            mass,
            density: 0.0,
            pressure: 0.0,
        }
    }
}

/// Cubic spline kernel W(r, h).
pub fn cubic_kernel(r: f64, h: f64) -> f64 {
    let q = r / h;
    let sigma = 10.0 / (7.0 * PI * h * h);
    if q <= 1.0 {
        sigma * (1.0 - 1.5 * q * q * (1.0 - q / 2.0))
    } else if q <= 2.0 {
        sigma * 0.25 * (2.0 - q).powi(3)
    } else {
        0.0
    }
}

/// Meshfree SPH solver.
pub struct MeshfreeSph {
    pub particles: Vec<SphParticle>,
    pub h: f64,       /* smoothing length */
    pub rho0: f64,    /* reference density */
    pub k_stiff: f64, /* stiffness constant */
}

impl MeshfreeSph {
    /// Create a new SPH solver.
    pub fn new(h: f64, rho0: f64, k_stiff: f64) -> Self {
        Self {
            particles: Vec::new(),
            h,
            rho0,
            k_stiff,
        }
    }

    /// Add a particle.
    pub fn add_particle(&mut self, x: f64, y: f64, mass: f64) {
        self.particles.push(SphParticle::new(x, y, mass));
    }

    /// Compute densities for all particles.
    #[allow(clippy::needless_range_loop)]
    pub fn compute_densities(&mut self) {
        let n = self.particles.len();
        let mut densities = vec![0.0f64; n];
        for i in 0..n {
            for j in 0..n {
                let dx = self.particles[i].x - self.particles[j].x;
                let dy = self.particles[i].y - self.particles[j].y;
                let r = (dx * dx + dy * dy).sqrt();
                densities[i] += self.particles[j].mass * cubic_kernel(r, self.h);
            }
        }
        for (i, p) in self.particles.iter_mut().enumerate() {
            p.density = densities[i];
        }
    }

    /// Compute pressures using equation of state p = k*(rho - rho0).
    pub fn compute_pressures(&mut self) {
        let rho0 = self.rho0;
        let k = self.k_stiff;
        for p in &mut self.particles {
            p.pressure = k * (p.density - rho0).max(0.0);
        }
    }

    /// Number of particles.
    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    /// Average density.
    pub fn avg_density(&self) -> f64 {
        if self.particles.is_empty() {
            return 0.0;
        }
        self.particles.iter().map(|p| p.density).sum::<f64>() / self.particles.len() as f64
    }
}

/// Create a new SPH solver.
pub fn new_meshfree_sph(h: f64, rho0: f64, k_stiff: f64) -> MeshfreeSph {
    MeshfreeSph::new(h, rho0, k_stiff)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_zero_distance() {
        let w = cubic_kernel(0.0, 1.0);
        assert!(w > 0.0); /* kernel is positive at origin */
    }

    #[test]
    fn test_kernel_beyond_support() {
        let w = cubic_kernel(3.0, 1.0);
        assert_eq!(w, 0.0); /* outside support radius */
    }

    #[test]
    fn test_add_particle() {
        let mut sph = MeshfreeSph::new(1.0, 1000.0, 10.0);
        sph.add_particle(0.0, 0.0, 1.0);
        assert_eq!(sph.particle_count(), 1); /* one particle added */
    }

    #[test]
    fn test_compute_densities_single() {
        let mut sph = MeshfreeSph::new(1.0, 1000.0, 10.0);
        sph.add_particle(0.0, 0.0, 1.0);
        sph.compute_densities();
        assert!(sph.particles[0].density > 0.0); /* self-density > 0 */
    }

    #[test]
    fn test_compute_pressures() {
        let mut sph = MeshfreeSph::new(1.0, 0.0, 1.0);
        sph.add_particle(0.0, 0.0, 1.0);
        sph.compute_densities();
        sph.compute_pressures();
        assert!(sph.particles[0].pressure >= 0.0); /* non-negative pressure */
    }

    #[test]
    fn test_avg_density_empty() {
        let sph = MeshfreeSph::new(1.0, 1000.0, 10.0);
        assert_eq!(sph.avg_density(), 0.0); /* empty system */
    }

    #[test]
    fn test_new_helper() {
        let sph = new_meshfree_sph(0.5, 1000.0, 5.0);
        assert_eq!(sph.particle_count(), 0); /* helper creates empty solver */
    }

    #[test]
    fn test_kernel_monotone() {
        /* kernel should decrease with distance */
        let w0 = cubic_kernel(0.0, 1.0);
        let w1 = cubic_kernel(0.5, 1.0);
        assert!(w0 > w1); /* decreasing */
    }

    #[test]
    fn test_two_close_particles() {
        let mut sph = MeshfreeSph::new(2.0, 0.0, 1.0);
        sph.add_particle(0.0, 0.0, 1.0);
        sph.add_particle(0.1, 0.0, 1.0);
        sph.compute_densities();
        /* both particles should have higher density than isolated */
        assert!(sph.particles[0].density > 0.0);
        assert!(sph.particles[1].density > 0.0);
    }
}
