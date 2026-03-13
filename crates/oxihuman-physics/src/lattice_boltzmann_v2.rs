// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Lattice Boltzmann Method v2 — D2Q9 LBM fluid simulation stub.

/// D2Q9 velocity weights.
const WEIGHTS: [f64; 9] = [
    4.0 / 9.0,
    1.0 / 9.0,
    1.0 / 9.0,
    1.0 / 9.0,
    1.0 / 9.0,
    1.0 / 36.0,
    1.0 / 36.0,
    1.0 / 36.0,
    1.0 / 36.0,
];

/// D2Q9 velocity vectors (ex, ey).
const EX: [i32; 9] = [0, 1, 0, -1, 0, 1, -1, -1, 1];
const EY: [i32; 9] = [0, 0, 1, 0, -1, 1, 1, -1, -1];

/// Opposite direction indices for bounce-back.
const OPP: [usize; 9] = [0, 3, 4, 1, 2, 7, 8, 5, 6];

/// LBM D2Q9 grid.
pub struct LatticeBoltzmannV2 {
    pub nx: usize,
    pub ny: usize,
    pub f: Vec<f64>, /* distribution functions [ny * nx * 9] */
    pub solid: Vec<bool>,
    pub omega: f64, /* relaxation parameter */
}

impl LatticeBoltzmannV2 {
    /// Create a new LBM grid with relaxation parameter `omega`.
    pub fn new(nx: usize, ny: usize, omega: f64) -> Self {
        let n = nx * ny * 9;
        let mut f = vec![0.0f64; n];
        for i in 0..(nx * ny) {
            f[i * 9] = WEIGHTS[0]; /* initialize to equilibrium at rest */
            for q in 1..9 {
                f[i * 9 + q] = WEIGHTS[q];
            }
        }
        LatticeBoltzmannV2 {
            nx,
            ny,
            f,
            solid: vec![false; nx * ny],
            omega: omega.clamp(0.5, 1.9),
        }
    }

    fn idx(&self, x: usize, y: usize) -> usize {
        (y * self.nx + x) * 9
    }

    /// Compute macroscopic density at cell (x, y).
    pub fn density(&self, x: usize, y: usize) -> f64 {
        let base = self.idx(x, y);
        self.f[base..base + 9].iter().sum()
    }

    /// Compute macroscopic velocity at cell (x, y).
    pub fn velocity(&self, x: usize, y: usize) -> (f64, f64) {
        let base = self.idx(x, y);
        let rho = self.density(x, y);
        if rho < 1e-12 {
            return (0.0, 0.0);
        }
        let ux: f64 = (0..9).map(|q| EX[q] as f64 * self.f[base + q]).sum::<f64>() / rho;
        let uy: f64 = (0..9).map(|q| EY[q] as f64 * self.f[base + q]).sum::<f64>() / rho;
        (ux, uy)
    }

    /// Equilibrium distribution function.
    pub fn feq(rho: f64, ux: f64, uy: f64, q: usize) -> f64 {
        let eu = EX[q] as f64 * ux + EY[q] as f64 * uy;
        let u2 = ux * ux + uy * uy;
        WEIGHTS[q] * rho * (1.0 + 3.0 * eu + 4.5 * eu * eu - 1.5 * u2)
    }

    /// Collision step (BGK relaxation).
    pub fn collide(&mut self) {
        for y in 0..self.ny {
            for x in 0..self.nx {
                if self.solid[y * self.nx + x] {
                    continue;
                }
                let (ux, uy) = self.velocity(x, y);
                let rho = self.density(x, y);
                let base = self.idx(x, y);
                for q in 0..9 {
                    let feq = Self::feq(rho, ux, uy, q);
                    self.f[base + q] += self.omega * (feq - self.f[base + q]);
                }
            }
        }
    }

    /// Streaming step.
    pub fn stream(&mut self) {
        let old_f = self.f.clone();
        for y in 0..self.ny {
            for x in 0..self.nx {
                for q in 0..9 {
                    let nx_i = (x as i32 + EX[q]).rem_euclid(self.nx as i32) as usize;
                    let ny_i = (y as i32 + EY[q]).rem_euclid(self.ny as i32) as usize;
                    let src = (y * self.nx + x) * 9 + q;
                    let dst = (ny_i * self.nx + nx_i) * 9 + q;
                    self.f[dst] = old_f[src];
                }
            }
        }
    }

    /// Apply bounce-back on solid cells.
    pub fn bounce_back(&mut self) {
        let old_f = self.f.clone();
        for y in 0..self.ny {
            for x in 0..self.nx {
                if self.solid[y * self.nx + x] {
                    let base = self.idx(x, y);
                    for q in 0..9 {
                        self.f[base + OPP[q]] = old_f[base + q];
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_density_initial() {
        let lbm = LatticeBoltzmannV2::new(4, 4, 1.0);
        let rho = lbm.density(0, 0);
        assert!((rho - 1.0).abs() < 1e-9 /* initial density should be 1 */);
    }

    #[test]
    fn test_velocity_initial_zero() {
        let lbm = LatticeBoltzmannV2::new(4, 4, 1.0);
        let (ux, uy) = lbm.velocity(0, 0);
        assert!(ux.abs() < 1e-9 /* initial ux zero */);
        assert!(uy.abs() < 1e-9 /* initial uy zero */);
    }

    #[test]
    fn test_feq_at_rest() {
        let feq = LatticeBoltzmannV2::feq(1.0, 0.0, 0.0, 0);
        assert!((feq - WEIGHTS[0]).abs() < 1e-9 /* feq at rest equals weight */);
    }

    #[test]
    fn test_collide_does_not_change_mass() {
        let mut lbm = LatticeBoltzmannV2::new(4, 4, 1.0);
        let before: f64 = (0..4)
            .flat_map(|y| (0..4).map(move |x| (x, y)))
            .map(|(x, y)| lbm.density(x, y))
            .sum();
        lbm.collide();
        let after: f64 = (0..4)
            .flat_map(|y| (0..4).map(move |x| (x, y)))
            .map(|(x, y)| lbm.density(x, y))
            .sum();
        assert!((before - after).abs() < 1e-6 /* mass conserved after collide */);
    }

    #[test]
    fn test_stream_does_not_change_mass() {
        let mut lbm = LatticeBoltzmannV2::new(4, 4, 1.0);
        let before: f64 = lbm.f.iter().sum();
        lbm.stream();
        let after: f64 = lbm.f.iter().sum();
        assert!((before - after).abs() < 1e-9 /* mass conserved after stream */);
    }

    #[test]
    fn test_omega_clamp() {
        let lbm = LatticeBoltzmannV2::new(2, 2, 3.0);
        assert!(lbm.omega <= 1.9 /* omega clamped at 1.9 */);
    }

    #[test]
    fn test_solid_flag() {
        let mut lbm = LatticeBoltzmannV2::new(4, 4, 1.0);
        lbm.solid[0] = true;
        lbm.collide(); /* should not panic on solid cell */
    }

    #[test]
    fn test_grid_size() {
        let lbm = LatticeBoltzmannV2::new(8, 6, 1.0);
        assert_eq!(
            lbm.f.len(),
            8 * 6 * 9 /* total distribution functions */
        );
    }

    #[test]
    fn test_one_step() {
        let mut lbm = LatticeBoltzmannV2::new(4, 4, 1.0);
        lbm.collide();
        lbm.stream();
        /* after one step, total density should still be 16 */
        let total: f64 = (0..4)
            .flat_map(|y| (0..4).map(move |x| (x, y)))
            .map(|(x, y)| lbm.density(x, y))
            .sum();
        assert!((total - 16.0).abs() < 1e-6);
    }
}
