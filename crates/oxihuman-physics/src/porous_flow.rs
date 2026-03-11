// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Darcy flow in porous media.
//!
//! q = -K/μ * ∇p   (Darcy's law)

/// 1D Darcy flow solver.
pub struct DarcyFlow1D {
    pub pressure: Vec<f32>,
    pub permeability: f32,
    pub viscosity: f32,
    pub porosity: f32,
    pub dx: f32,
    pub length: f32,
}

impl DarcyFlow1D {
    /// Create a 1D Darcy solver with n nodes.
    pub fn new(n: usize, length: f32, k: f32, mu: f32, phi: f32) -> Self {
        let n = n.max(3);
        DarcyFlow1D {
            pressure: vec![0.0f32; n],
            permeability: k,
            viscosity: mu,
            porosity: phi,
            dx: length / (n - 1) as f32,
            length,
        }
    }

    /// Compute Darcy flux at each cell face: q = -K/mu * dp/dx.
    #[allow(clippy::needless_range_loop)]
    pub fn darcy_flux(&self) -> Vec<f32> {
        let n = self.pressure.len();
        let mut flux = vec![0.0f32; n - 1];
        for i in 0..n - 1 {
            let dp_dx = (self.pressure[i + 1] - self.pressure[i]) / self.dx;
            flux[i] = -(self.permeability / self.viscosity) * dp_dx;
        }
        flux
    }

    /// Set linear pressure gradient (p0 at x=0, p1 at x=L).
    pub fn set_pressure_gradient(&mut self, p0: f32, p1: f32) {
        let n = self.pressure.len();
        for i in 0..n {
            self.pressure[i] = p0 + (p1 - p0) * i as f32 / (n - 1).max(1) as f32;
        }
    }

    /// Solve steady-state Darcy pressure field with BC `p[0]`=p_inlet, `p[n-1]`=p_outlet
    /// (trivially linear for homogeneous K).
    pub fn solve_steady(&mut self, p_inlet: f32, p_outlet: f32) {
        self.set_pressure_gradient(p_inlet, p_outlet);
    }

    /// Mean flux magnitude.
    pub fn mean_flux(&self) -> f32 {
        let flux = self.darcy_flux();
        flux.iter().map(|&q| q.abs()).sum::<f32>() / flux.len().max(1) as f32
    }

    /// Reynolds number: Re = rho * v * L / mu (approximation).
    pub fn reynolds_number(&self, rho: f32, v_mean: f32, char_length: f32) -> f32 {
        rho * v_mean * char_length / self.viscosity
    }

    pub fn grid_size(&self) -> usize {
        self.pressure.len()
    }
}

/// 2D Darcy flow on a uniform grid.
pub struct DarcyFlow2D {
    pub pressure: Vec<f32>,
    pub nx: usize,
    pub ny: usize,
    pub dx: f32,
    pub dy: f32,
    pub k: f32,
    pub mu: f32,
}

impl DarcyFlow2D {
    pub fn new(nx: usize, ny: usize, lx: f32, ly: f32, k: f32, mu: f32) -> Self {
        let nx = nx.max(3);
        let ny = ny.max(3);
        DarcyFlow2D {
            pressure: vec![0.0f32; nx * ny],
            nx,
            ny,
            dx: lx / (nx - 1) as f32,
            dy: ly / (ny - 1) as f32,
            k,
            mu,
        }
    }

    pub fn idx(&self, i: usize, j: usize) -> usize {
        i * self.ny + j
    }

    pub fn get_p(&self, i: usize, j: usize) -> f32 {
        self.pressure[self.idx(i, j)]
    }

    pub fn set_p(&mut self, i: usize, j: usize, val: f32) {
        let idx = self.idx(i, j);
        self.pressure[idx] = val;
    }

    /// Jacobi iteration to solve ∇²p = 0 (Laplace = steady Darcy with div-free flow).
    pub fn jacobi_step(&mut self) {
        let nx = self.nx;
        let ny = self.ny;
        let dx2 = self.dx * self.dx;
        let dy2 = self.dy * self.dy;
        let mut p_new = self.pressure.clone();
        for i in 1..nx - 1 {
            for j in 1..ny - 1 {
                let ci = self.idx(i, j);
                p_new[ci] = (self.pressure[self.idx(i - 1, j)] / dx2
                    + self.pressure[self.idx(i + 1, j)] / dx2
                    + self.pressure[self.idx(i, j - 1)] / dy2
                    + self.pressure[self.idx(i, j + 1)] / dy2)
                    / (2.0 / dx2 + 2.0 / dy2);
            }
        }
        self.pressure = p_new;
    }

    pub fn solve(&mut self, iters: usize) {
        for _ in 0..iters {
            self.jacobi_step();
        }
    }

    /// Velocity components at interior node (i,j).
    pub fn velocity(&self, i: usize, j: usize) -> [f32; 2] {
        if i == 0 || i >= self.nx - 1 || j == 0 || j >= self.ny - 1 {
            return [0.0; 2];
        }
        let dp_dx = (self.pressure[self.idx(i + 1, j)] - self.pressure[self.idx(i - 1, j)])
            / (2.0 * self.dx);
        let dp_dy = (self.pressure[self.idx(i, j + 1)] - self.pressure[self.idx(i, j - 1)])
            / (2.0 * self.dy);
        [-(self.k / self.mu) * dp_dx, -(self.k / self.mu) * dp_dy]
    }
}

/// Darcy velocity: q = -K/mu * dp/dx.
pub fn darcy_velocity(k: f32, mu: f32, dp_dx: f32) -> f32 {
    -k / mu * dp_dx
}

/// Forchheimer correction for high-Reynolds porous flow.
pub fn forchheimer_velocity(k: f32, mu: f32, beta: f32, rho: f32, dp_dx: f32) -> f32 {
    let dp = dp_dx.abs();
    /* Solves: dp = mu/k * q + beta * rho * q^2 for q */
    if beta.abs() < 1e-15 {
        return darcy_velocity(k, mu, dp_dx);
    }
    let a = beta * rho;
    let b = mu / k;
    let c = -dp;
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        return 0.0;
    }
    let q = (-b + disc.sqrt()) / (2.0 * a);
    if dp_dx < 0.0 {
        q
    } else {
        -q
    }
}

pub fn new_darcy_1d(n: usize, length: f32, k: f32, mu: f32, phi: f32) -> DarcyFlow1D {
    DarcyFlow1D::new(n, length, k, mu, phi)
}

pub fn new_darcy_2d(nx: usize, ny: usize, lx: f32, ly: f32, k: f32, mu: f32) -> DarcyFlow2D {
    DarcyFlow2D::new(nx, ny, lx, ly, k, mu)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1d_construction() {
        let d = new_darcy_1d(20, 1.0, 1e-12, 0.001, 0.3);
        assert_eq!(d.grid_size(), 20);
    }

    #[test]
    fn test_darcy_flux_linear() {
        let mut d = DarcyFlow1D::new(11, 1.0, 1.0, 1.0, 0.3);
        d.set_pressure_gradient(10.0, 0.0);
        let flux = d.darcy_flux();
        /* Flux should be positive (flow in +x direction, high p on left) */
        assert!(flux.iter().all(|&q| q > 0.0));
    }

    #[test]
    fn test_solve_steady() {
        let mut d = DarcyFlow1D::new(11, 1.0, 1e-12, 0.001, 0.3);
        d.solve_steady(100.0, 0.0);
        assert!((d.pressure[0] - 100.0).abs() < 1e-4);
        assert!((d.pressure[10]).abs() < 1e-4);
    }

    #[test]
    fn test_darcy_velocity_fn() {
        let q = darcy_velocity(1.0, 1.0, -1.0);
        assert!((q - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_2d_construction() {
        let d = new_darcy_2d(10, 10, 1.0, 1.0, 1e-12, 0.001);
        assert_eq!(d.nx, 10);
        assert_eq!(d.ny, 10);
    }

    #[test]
    fn test_2d_solve() {
        let mut d = DarcyFlow2D::new(10, 10, 1.0, 1.0, 1.0, 1.0);
        /* Set left boundary to 1, right to 0 */
        for j in 0..10 {
            d.set_p(0, j, 1.0);
        }
        d.solve(50);
        /* Interior pressure should be between 0 and 1 */
        let p_mid = d.get_p(5, 5);
        assert!((0.0..=(1.0 + 1e-3)).contains(&p_mid));
    }

    #[test]
    fn test_reynolds_number() {
        let d = DarcyFlow1D::new(10, 1.0, 1.0, 0.001, 0.3);
        let re = d.reynolds_number(1000.0, 0.01, 0.001);
        assert!(re > 0.0);
    }

    #[test]
    fn test_mean_flux() {
        let mut d = DarcyFlow1D::new(11, 1.0, 1.0, 1.0, 0.3);
        d.set_pressure_gradient(10.0, 0.0);
        assert!(d.mean_flux() > 0.0);
    }
}
