// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! 2D lattice Boltzmann fluid (D2Q9 scheme).

#![allow(dead_code)]

// D2Q9 velocities: 0=center, 1-8=directions
const E: [[i32; 2]; 9] = [
    [0, 0],
    [1, 0],
    [0, 1],
    [-1, 0],
    [0, -1],
    [1, 1],
    [-1, 1],
    [-1, -1],
    [1, -1],
];
const W: [f32; 9] = [
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

/// Equilibrium distribution function.
fn feq(w: f32, rho: f32, ux: f32, uy: f32, ex: f32, ey: f32) -> f32 {
    let eu = ex * ux + ey * uy;
    let u2 = ux * ux + uy * uy;
    w * rho * (1.0 + 3.0 * eu + 4.5 * eu * eu - 1.5 * u2)
}

/// D2Q9 lattice Boltzmann grid.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LatticeBoltzmann {
    pub nx: usize,
    pub ny: usize,
    /// Distribution functions `f[x][y][q]`, stored flat.
    pub f: Vec<f32>,
    /// Solid obstacle mask.
    pub solid: Vec<bool>,
    /// Relaxation time (related to viscosity).
    pub tau: f32,
}

#[allow(dead_code)]
impl LatticeBoltzmann {
    pub fn new(nx: usize, ny: usize, tau: f32) -> Self {
        let n = nx * ny * 9;
        // Initialize with equilibrium (rho=1, u=0)
        let mut f = Vec::with_capacity(n);
        for _ in 0..nx {
            for _ in 0..ny {
                for &w in W.iter() {
                    f.push(w);
                }
            }
        }
        Self {
            nx,
            ny,
            f,
            solid: vec![false; nx * ny],
            tau,
        }
    }

    fn idx(&self, x: usize, y: usize, q: usize) -> usize {
        (x * self.ny + y) * 9 + q
    }

    /// Get distribution value.
    pub fn get_f(&self, x: usize, y: usize, q: usize) -> f32 {
        self.f[self.idx(x, y, q)]
    }

    /// Set distribution value.
    pub fn set_f(&mut self, x: usize, y: usize, q: usize, val: f32) {
        let i = self.idx(x, y, q);
        self.f[i] = val;
    }

    /// Compute macroscopic density and velocity at (x, y).
    pub fn macroscopic(&self, x: usize, y: usize) -> (f32, f32, f32) {
        let mut rho = 0.0;
        let mut ux = 0.0;
        let mut uy = 0.0;
        for (q, e) in E.iter().enumerate() {
            let fq = self.get_f(x, y, q);
            rho += fq;
            ux += e[0] as f32 * fq;
            uy += e[1] as f32 * fq;
        }
        let inv_rho = if rho > 1e-10 { 1.0 / rho } else { 0.0 };
        (rho, ux * inv_rho, uy * inv_rho)
    }

    /// Collision step (BGK).
    pub fn collision(&mut self) {
        for x in 0..self.nx {
            for y in 0..self.ny {
                if self.solid[x * self.ny + y] {
                    continue;
                }
                let (rho, ux, uy) = self.macroscopic(x, y);
                for q in 0..9 {
                    let eq = feq(W[q], rho, ux, uy, E[q][0] as f32, E[q][1] as f32);
                    let i = self.idx(x, y, q);
                    self.f[i] += (eq - self.f[i]) / self.tau;
                }
            }
        }
    }

    /// Streaming step (with periodic boundaries).
    pub fn streaming(&mut self) {
        let f_old = self.f.clone();
        for x in 0..self.nx {
            for y in 0..self.ny {
                for (q, e) in E.iter().enumerate() {
                    let src_x = (x as i32 - e[0]).rem_euclid(self.nx as i32) as usize;
                    let src_y = (y as i32 - e[1]).rem_euclid(self.ny as i32) as usize;
                    let dst = self.idx(x, y, q);
                    let src = self.idx(src_x, src_y, q);
                    self.f[dst] = f_old[src];
                }
            }
        }
    }

    /// Full LBM step.
    pub fn step(&mut self) {
        self.collision();
        self.streaming();
    }

    /// Set obstacle at (x, y).
    pub fn set_obstacle(&mut self, x: usize, y: usize) {
        self.solid[x * self.ny + y] = true;
    }

    /// Total density sum.
    pub fn total_density(&self) -> f32 {
        let mut sum = 0.0;
        for x in 0..self.nx {
            for y in 0..self.ny {
                let (rho, _, _) = self.macroscopic(x, y);
                sum += rho;
            }
        }
        sum
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_uniform_density() {
        let lb = LatticeBoltzmann::new(10, 10, 1.0);
        let (rho, ux, uy) = lb.macroscopic(5, 5);
        assert!((rho - 1.0).abs() < 1e-4, "rho={rho}");
        assert!(ux.abs() < 1e-4);
        assert!(uy.abs() < 1e-4);
    }

    #[test]
    fn total_density_conserved() {
        let mut lb = LatticeBoltzmann::new(10, 10, 1.0);
        let d0 = lb.total_density();
        lb.step();
        let d1 = lb.total_density();
        assert!(
            (d1 - d0).abs() < 0.01 * d0,
            "density not conserved: {d0} vs {d1}"
        );
    }

    #[test]
    fn collision_step_runs() {
        let mut lb = LatticeBoltzmann::new(5, 5, 1.0);
        lb.collision();
        // Just ensure no panic
    }

    #[test]
    fn streaming_step_runs() {
        let mut lb = LatticeBoltzmann::new(5, 5, 1.0);
        lb.streaming();
    }

    #[test]
    fn set_f_and_get() {
        let mut lb = LatticeBoltzmann::new(5, 5, 1.0);
        lb.set_f(2, 3, 0, 0.7);
        assert!((lb.get_f(2, 3, 0) - 0.7).abs() < 1e-5);
    }

    #[test]
    fn obstacle_cell_not_updated() {
        let mut lb = LatticeBoltzmann::new(5, 5, 1.0);
        lb.set_obstacle(2, 2);
        let f0 = lb.get_f(2, 2, 0);
        lb.collision();
        let f1 = lb.get_f(2, 2, 0);
        // Collision skips solid cells, f should be unchanged after collision
        assert!((f1 - f0).abs() < 1e-5);
    }

    #[test]
    fn feq_zero_velocity() {
        let v = feq(4.0 / 9.0, 1.0, 0.0, 0.0, 0.0, 0.0);
        assert!((v - 4.0 / 9.0).abs() < 1e-5);
    }

    #[test]
    fn many_steps_stable() {
        let mut lb = LatticeBoltzmann::new(8, 8, 1.0);
        for _ in 0..20 {
            lb.step();
        }
        let d = lb.total_density();
        assert!(d.is_finite() && d > 0.0);
    }

    #[test]
    fn size_correct() {
        let lb = LatticeBoltzmann::new(4, 6, 1.0);
        assert_eq!(lb.nx, 4);
        assert_eq!(lb.ny, 6);
        assert_eq!(lb.f.len(), 4 * 6 * 9);
    }

    #[test]
    fn w_sums_to_one() {
        let sum: f32 = W.iter().sum();
        assert!((sum - 1.0).abs() < 1e-5);
    }
}
