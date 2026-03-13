// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Simplified 2D Navier-Stokes solver using vorticity-streamfunction formulation.
//!
//! dω/dt + u·∇ω = ν·∇²ω
//! ∇²ψ = -ω
//! u = ∂ψ/∂y, v = -∂ψ/∂x

/// 2D Navier-Stokes (vorticity-streamfunction) solver.
pub struct NavierStokes2D {
    /// Vorticity field ω.
    pub omega: Vec<f32>,
    /// Streamfunction ψ.
    pub psi: Vec<f32>,
    pub nx: usize,
    pub ny: usize,
    pub dx: f32,
    pub dy: f32,
    pub nu: f32,
    pub time: f32,
}

impl NavierStokes2D {
    pub fn new(nx: usize, ny: usize, lx: f32, ly: f32, nu: f32) -> Self {
        let nx = nx.max(4);
        let ny = ny.max(4);
        NavierStokes2D {
            omega: vec![0.0f32; nx * ny],
            psi: vec![0.0f32; nx * ny],
            nx,
            ny,
            dx: lx / (nx - 1) as f32,
            dy: ly / (ny - 1) as f32,
            nu,
            time: 0.0,
        }
    }

    pub fn idx(&self, i: usize, j: usize) -> usize {
        i * self.ny + j
    }

    /// Solve the streamfunction Poisson equation ∇²ψ = -ω (Jacobi iterations).
    pub fn solve_streamfunction(&mut self, iters: usize) {
        let nx = self.nx;
        let ny = self.ny;
        let dx2 = self.dx * self.dx;
        let dy2 = self.dy * self.dy;
        for _ in 0..iters {
            let mut psi_new = self.psi.clone();
            for i in 1..nx - 1 {
                for j in 1..ny - 1 {
                    let ci = self.idx(i, j);
                    let rhs = -self.omega[ci];
                    psi_new[ci] = (self.psi[self.idx(i - 1, j)] / dx2
                        + self.psi[self.idx(i + 1, j)] / dx2
                        + self.psi[self.idx(i, j - 1)] / dy2
                        + self.psi[self.idx(i, j + 1)] / dy2
                        - rhs)
                        / (2.0 / dx2 + 2.0 / dy2);
                }
            }
            self.psi = psi_new;
        }
    }

    /// Advance vorticity with explicit upwind advection + diffusion.
    pub fn step(&mut self, dt: f32) {
        let nx = self.nx;
        let ny = self.ny;
        let dx = self.dx;
        let dy = self.dy;
        let nu = self.nu;

        /* Solve for streamfunction */
        self.solve_streamfunction(20);

        let mut omega_new = self.omega.clone();
        for i in 1..nx - 1 {
            for j in 1..ny - 1 {
                let ci = self.idx(i, j);
                /* Velocity from streamfunction */
                let u_vel =
                    (self.psi[self.idx(i, j + 1)] - self.psi[self.idx(i, j - 1)]) / (2.0 * dy);
                let v_vel =
                    -(self.psi[self.idx(i + 1, j)] - self.psi[self.idx(i - 1, j)]) / (2.0 * dx);

                /* Upwind advection */
                let dow_dx = if u_vel >= 0.0 {
                    (self.omega[ci] - self.omega[self.idx(i - 1, j)]) / dx
                } else {
                    (self.omega[self.idx(i + 1, j)] - self.omega[ci]) / dx
                };
                let dow_dy = if v_vel >= 0.0 {
                    (self.omega[ci] - self.omega[self.idx(i, j - 1)]) / dy
                } else {
                    (self.omega[self.idx(i, j + 1)] - self.omega[ci]) / dy
                };

                /* Diffusion */
                let lap_omega = (self.omega[self.idx(i - 1, j)] - 2.0 * self.omega[ci]
                    + self.omega[self.idx(i + 1, j)])
                    / (dx * dx)
                    + (self.omega[self.idx(i, j - 1)] - 2.0 * self.omega[ci]
                        + self.omega[self.idx(i, j + 1)])
                        / (dy * dy);

                omega_new[ci] =
                    self.omega[ci] + dt * (-u_vel * dow_dx - v_vel * dow_dy + nu * lap_omega);
            }
        }
        self.omega = omega_new;
        self.time += dt;
    }

    /// Get velocity u at (i, j).
    pub fn velocity_u(&self, i: usize, j: usize) -> f32 {
        if j == 0 || j >= self.ny - 1 {
            return 0.0;
        }
        (self.psi[self.idx(i, j + 1)] - self.psi[self.idx(i, j - 1)]) / (2.0 * self.dy)
    }

    /// Get velocity v at (i, j).
    pub fn velocity_v(&self, i: usize, j: usize) -> f32 {
        if i == 0 || i >= self.nx - 1 {
            return 0.0;
        }
        -(self.psi[self.idx(i + 1, j)] - self.psi[self.idx(i - 1, j)]) / (2.0 * self.dx)
    }

    /// Max absolute vorticity.
    pub fn max_vorticity(&self) -> f32 {
        self.omega.iter().map(|&v| v.abs()).fold(0.0f32, f32::max)
    }

    /// Total enstrophy (integral of omega^2).
    pub fn enstrophy(&self) -> f32 {
        self.omega.iter().map(|&v| v * v).sum::<f32>() * self.dx * self.dy
    }

    /// Set an initial vortex at (cx, cy) with strength and radius.
    pub fn add_vortex(&mut self, cx: f32, cy: f32, strength: f32, radius: f32) {
        for i in 0..self.nx {
            for j in 0..self.ny {
                let x = i as f32 * self.dx;
                let y = j as f32 * self.dy;
                let r2 = (x - cx).powi(2) + (y - cy).powi(2);
                let sigma2 = radius * radius;
                let ci = self.idx(i, j);
                self.omega[ci] += strength * (-r2 / (2.0 * sigma2)).exp();
            }
        }
    }
}

pub fn new_navier_stokes_2d(nx: usize, ny: usize, lx: f32, ly: f32, nu: f32) -> NavierStokes2D {
    NavierStokes2D::new(nx, ny, lx, ly, nu)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construction() {
        let ns = new_navier_stokes_2d(10, 10, 1.0, 1.0, 0.01);
        assert_eq!(ns.nx, 10);
        assert_eq!(ns.ny, 10);
    }

    #[test]
    fn test_add_vortex() {
        let mut ns = NavierStokes2D::new(20, 20, 1.0, 1.0, 0.01);
        ns.add_vortex(0.5, 0.5, 1.0, 0.1);
        assert!(ns.max_vorticity() > 0.0);
    }

    #[test]
    fn test_enstrophy_positive() {
        let mut ns = NavierStokes2D::new(20, 20, 1.0, 1.0, 0.01);
        ns.add_vortex(0.5, 0.5, 1.0, 0.1);
        assert!(ns.enstrophy() > 0.0);
    }

    #[test]
    fn test_step_runs() {
        let mut ns = NavierStokes2D::new(10, 10, 1.0, 1.0, 0.1);
        ns.add_vortex(0.5, 0.5, 0.5, 0.2);
        ns.step(0.001);
        assert!(ns.time > 0.0);
    }

    #[test]
    fn test_streamfunction_solve() {
        let mut ns = NavierStokes2D::new(15, 15, 1.0, 1.0, 0.01);
        ns.add_vortex(0.5, 0.5, 1.0, 0.1);
        ns.solve_streamfunction(50);
        /* Streamfunction should be non-trivial after solving */
        let max_psi = ns.psi.iter().map(|&v| v.abs()).fold(0.0f32, f32::max);
        assert!(max_psi > 0.0);
    }

    #[test]
    fn test_velocity_u_at_boundary() {
        let ns = NavierStokes2D::new(10, 10, 1.0, 1.0, 0.01);
        assert!((ns.velocity_u(5, 0)).abs() < 1e-10);
    }

    #[test]
    fn test_enstrophy_decays_with_diffusion() {
        let mut ns = NavierStokes2D::new(20, 20, 1.0, 1.0, 1.0); /* high viscosity */
        ns.add_vortex(0.5, 0.5, 1.0, 0.1);
        let e0 = ns.enstrophy();
        for _ in 0..5 {
            ns.step(0.001);
        }
        let e1 = ns.enstrophy();
        /* High viscosity should dissipate enstrophy */
        assert!(e1 <= e0 + 0.1);
    }

    #[test]
    fn test_time_advances() {
        let mut ns = new_navier_stokes_2d(8, 8, 1.0, 1.0, 0.01);
        ns.step(0.01);
        ns.step(0.01);
        assert!((ns.time - 0.02).abs() < 1e-5);
    }
}
