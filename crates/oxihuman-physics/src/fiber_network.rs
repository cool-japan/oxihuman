// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Random fiber network stub — models a 2-D network of elastic fibers for
//! soft-matter and biopolymer mechanics simulations.

use std::f64::consts::PI;

/// A fiber segment between two 2-D points.
#[derive(Debug, Clone)]
pub struct Fiber {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
    pub stiffness: f64,
}

impl Fiber {
    pub fn new(x0: f64, y0: f64, x1: f64, y1: f64, stiffness: f64) -> Self {
        Self {
            x0,
            y0,
            x1,
            y1,
            stiffness,
        }
    }

    /// Rest length of this fiber.
    pub fn length(&self) -> f64 {
        let dx = self.x1 - self.x0;
        let dy = self.y1 - self.y0;
        (dx * dx + dy * dy).sqrt()
    }

    /// Orientation angle in radians.
    pub fn angle(&self) -> f64 {
        let dx = self.x1 - self.x0;
        let dy = self.y1 - self.y0;
        dy.atan2(dx)
    }

    /// Axial strain energy for a given end separation.
    pub fn strain_energy(&self, current_length: f64) -> f64 {
        let dl = current_length - self.length();
        0.5 * self.stiffness * dl * dl
    }
}

/// Random fiber network.
pub struct FiberNetwork {
    pub fibers: Vec<Fiber>,
    pub domain_size: f64,
}

impl FiberNetwork {
    /// Create an empty fiber network.
    pub fn new(domain_size: f64) -> Self {
        Self {
            fibers: Vec::new(),
            domain_size,
        }
    }

    /// Add a fiber.
    pub fn add_fiber(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, stiffness: f64) {
        self.fibers.push(Fiber::new(x0, y0, x1, y1, stiffness));
    }

    /// Generate `n` random isotropic fibers with given length and stiffness.
    pub fn generate_isotropic(&mut self, n: usize, fiber_len: f64, stiffness: f64, seed: u64) {
        /* simple LCG pseudo-random for reproducibility */
        let mut rng = seed;
        let lcg = |r: &mut u64| -> f64 {
            *r = r
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (*r >> 33) as f64 / (u32::MAX as f64)
        };
        let ds = self.domain_size;
        for _ in 0..n {
            let cx = lcg(&mut rng) * ds;
            let cy = lcg(&mut rng) * ds;
            let theta = lcg(&mut rng) * 2.0 * PI;
            let hx = fiber_len * 0.5 * theta.cos();
            let hy = fiber_len * 0.5 * theta.sin();
            self.add_fiber(cx - hx, cy - hy, cx + hx, cy + hy, stiffness);
        }
    }

    /// Mean fiber length.
    pub fn mean_length(&self) -> f64 {
        if self.fibers.is_empty() {
            return 0.0;
        }
        self.fibers.iter().map(|f| f.length()).sum::<f64>() / self.fibers.len() as f64
    }

    /// Total number of fibers.
    pub fn fiber_count(&self) -> usize {
        self.fibers.len()
    }

    /// Estimate fiber density (fibers per unit area).
    pub fn density(&self) -> f64 {
        let area = self.domain_size * self.domain_size;
        if area > 0.0 {
            self.fibers.len() as f64 / area
        } else {
            0.0
        }
    }

    /// Total strain energy for all fibers given a uniform stretch factor.
    pub fn total_strain_energy(&self, stretch: f64) -> f64 {
        self.fibers
            .iter()
            .map(|f| f.strain_energy(f.length() * stretch))
            .sum()
    }
}

/// Create a new fiber network.
pub fn new_fiber_network(domain_size: f64) -> FiberNetwork {
    FiberNetwork::new(domain_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_fiber() {
        let mut net = FiberNetwork::new(10.0);
        net.add_fiber(0.0, 0.0, 1.0, 0.0, 1.0);
        assert_eq!(net.fiber_count(), 1); /* one fiber added */
    }

    #[test]
    fn test_fiber_length() {
        let f = Fiber::new(0.0, 0.0, 3.0, 4.0, 1.0);
        assert!((f.length() - 5.0).abs() < 1e-10); /* 3-4-5 triangle */
    }

    #[test]
    fn test_fiber_angle() {
        let f = Fiber::new(0.0, 0.0, 1.0, 0.0, 1.0);
        assert!(f.angle().abs() < 1e-10); /* horizontal fiber, angle=0 */
    }

    #[test]
    fn test_strain_energy_at_rest() {
        let f = Fiber::new(0.0, 0.0, 1.0, 0.0, 100.0);
        assert!(f.strain_energy(1.0).abs() < 1e-10); /* no strain at rest length */
    }

    #[test]
    fn test_generate_isotropic() {
        let mut net = FiberNetwork::new(10.0);
        net.generate_isotropic(20, 1.0, 1.0, 42);
        assert_eq!(net.fiber_count(), 20); /* 20 fibers generated */
    }

    #[test]
    fn test_mean_length_isotropic() {
        let mut net = FiberNetwork::new(10.0);
        net.generate_isotropic(50, 2.0, 1.0, 7);
        let ml = net.mean_length();
        assert!((ml - 2.0).abs() < 1e-10); /* all fibers same length */
    }

    #[test]
    fn test_density() {
        let mut net = FiberNetwork::new(10.0);
        net.add_fiber(0.0, 0.0, 1.0, 0.0, 1.0);
        assert!((net.density() - 0.01).abs() < 1e-10); /* 1 / 100 */
    }

    #[test]
    fn test_total_strain_energy_no_stretch() {
        let mut net = FiberNetwork::new(10.0);
        net.add_fiber(0.0, 0.0, 1.0, 0.0, 100.0);
        assert!(net.total_strain_energy(1.0).abs() < 1e-10); /* no strain at stretch=1 */
    }

    #[test]
    fn test_total_strain_energy_stretched() {
        let mut net = FiberNetwork::new(10.0);
        net.add_fiber(0.0, 0.0, 1.0, 0.0, 100.0);
        assert!(net.total_strain_energy(1.1) > 0.0); /* positive energy when stretched */
    }

    #[test]
    fn test_new_helper() {
        let net = new_fiber_network(5.0);
        assert_eq!(net.fiber_count(), 0); /* helper creates empty network */
    }
}
