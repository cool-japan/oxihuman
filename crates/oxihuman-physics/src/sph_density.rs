// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! SPH density estimator — computes particle density via kernel summation.

use std::f64::consts::PI;

/// SPH particle with position, mass, and density.
#[derive(Debug, Clone)]
pub struct SphParticleDensity {
    pub pos: [f64; 3],
    pub mass: f64,
    pub density: f64,
}

impl SphParticleDensity {
    pub fn new(pos: [f64; 3], mass: f64) -> Self {
        SphParticleDensity {
            pos,
            mass,
            density: 0.0,
        }
    }
}

/// Cubic spline kernel W(r, h).
pub fn cubic_spline_kernel(r: f64, h: f64) -> f64 {
    if h <= 0.0 {
        return 0.0;
    }
    let q = r / h;
    let sigma = 1.0 / (PI * h * h * h);
    if (0.0..=1.0).contains(&q) {
        sigma * (1.0 - 1.5 * q * q + 0.75 * q * q * q)
    } else if (1.0..=2.0).contains(&q) {
        sigma * 0.25 * (2.0 - q).powi(3)
    } else {
        0.0
    }
}

/// Distance between two 3D points.
pub fn dist3(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Compute density for all particles using SPH kernel summation.
pub fn compute_density(particles: &mut [SphParticleDensity], h: f64) {
    let n = particles.len();
    let mut densities = vec![0.0f64; n];
    for i in 0..n {
        for j in 0..n {
            let r = dist3(particles[i].pos, particles[j].pos);
            densities[i] += particles[j].mass * cubic_spline_kernel(r, h);
        }
    }
    for i in 0..n {
        particles[i].density = densities[i];
    }
}

/// Return density estimate for a single query point given a slice of particles.
pub fn estimate_density_at(pos: [f64; 3], particles: &[SphParticleDensity], h: f64) -> f64 {
    particles
        .iter()
        .map(|p| p.mass * cubic_spline_kernel(dist3(pos, p.pos), h))
        .sum()
}

/// Average density across all particles.
pub fn average_density(particles: &[SphParticleDensity]) -> f64 {
    if particles.is_empty() {
        return 0.0;
    }
    particles.iter().map(|p| p.density).sum::<f64>() / particles.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_at_zero() {
        let w = cubic_spline_kernel(0.0, 1.0);
        assert!(w > 0.0 /* kernel positive at origin */);
    }

    #[test]
    fn test_kernel_zero_beyond_support() {
        let w = cubic_spline_kernel(3.0, 1.0);
        assert_eq!(w, 0.0 /* zero beyond 2h */);
    }

    #[test]
    fn test_kernel_decreasing() {
        let h = 1.0;
        let w0 = cubic_spline_kernel(0.0, h);
        let w1 = cubic_spline_kernel(0.5, h);
        let w2 = cubic_spline_kernel(1.5, h);
        assert!(w0 > w1 && w1 > w2 /* kernel decreases with distance */);
    }

    #[test]
    fn test_dist3() {
        let a = [1.0, 0.0, 0.0];
        let b = [4.0, 0.0, 0.0];
        assert!((dist3(a, b) - 3.0).abs() < 1e-10 /* dist3 correct */);
    }

    #[test]
    fn test_compute_density_single_particle() {
        let mut p = vec![SphParticleDensity::new([0.0, 0.0, 0.0], 1.0)];
        compute_density(&mut p, 1.0);
        assert!(p[0].density > 0.0 /* self-contribution gives positive density */);
    }

    #[test]
    fn test_estimate_density_at() {
        let particles = vec![SphParticleDensity {
            pos: [0.0, 0.0, 0.0],
            mass: 1.0,
            density: 0.0,
        }];
        let rho = estimate_density_at([0.0, 0.0, 0.0], &particles, 1.0);
        assert!(rho > 0.0 /* density at particle position is positive */);
    }

    #[test]
    fn test_average_density_empty() {
        let particles: Vec<SphParticleDensity> = vec![];
        assert_eq!(average_density(&particles), 0.0 /* empty slice -> 0 */);
    }

    #[test]
    fn test_average_density() {
        let mut particles = vec![
            SphParticleDensity {
                pos: [0.0; 3],
                mass: 1.0,
                density: 2.0,
            },
            SphParticleDensity {
                pos: [0.0; 3],
                mass: 1.0,
                density: 4.0,
            },
        ];
        particles[0].density = 2.0;
        particles[1].density = 4.0;
        assert!((average_density(&particles) - 3.0).abs() < 1e-10 /* average is 3 */);
    }

    #[test]
    fn test_kernel_invalid_h() {
        let w = cubic_spline_kernel(0.5, 0.0);
        assert_eq!(w, 0.0 /* h=0 returns 0 */);
    }
}
