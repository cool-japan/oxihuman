// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Poisson equation solver using Jacobi iteration.
//!
//! Solves: d2u/dx2 + d2u/dy2 = f(x, y)

/// 2D Poisson solver on a uniform grid using Jacobi iteration.
pub struct PoissonSolver2D {
    pub u: Vec<f32>,
    pub f: Vec<f32>,
    pub nx: usize,
    pub ny: usize,
    pub dx: f32,
    pub dy: f32,
    pub iterations: usize,
}

impl PoissonSolver2D {
    /// Create a new solver on an nx × ny grid over `[0,lx]` × `[0,ly]`.
    pub fn new(nx: usize, ny: usize, lx: f32, ly: f32) -> Self {
        let nx = nx.max(3);
        let ny = ny.max(3);
        PoissonSolver2D {
            u: vec![0.0f32; nx * ny],
            f: vec![0.0f32; nx * ny],
            nx,
            ny,
            dx: lx / (nx - 1) as f32,
            dy: ly / (ny - 1) as f32,
            iterations: 0,
        }
    }

    pub fn idx(&self, i: usize, j: usize) -> usize {
        i * self.ny + j
    }

    pub fn get_u(&self, i: usize, j: usize) -> f32 {
        self.u[self.idx(i, j)]
    }

    pub fn set_f(&mut self, i: usize, j: usize, val: f32) {
        let idx = self.idx(i, j);
        self.f[idx] = val;
    }

    /// Perform one Jacobi iteration (Dirichlet BC: boundary stays 0).
    pub fn jacobi_step(&mut self) {
        let nx = self.nx;
        let ny = self.ny;
        let dx2 = self.dx * self.dx;
        let dy2 = self.dy * self.dy;
        let mut u_new = self.u.clone();
        for i in 1..nx - 1 {
            for j in 1..ny - 1 {
                let ci = self.idx(i, j);
                let left = self.u[self.idx(i - 1, j)];
                let right = self.u[self.idx(i + 1, j)];
                let down = self.u[self.idx(i, j - 1)];
                let up = self.u[self.idx(i, j + 1)];
                u_new[ci] = (left / dx2 + right / dx2 + down / dy2 + up / dy2 - self.f[ci])
                    / (2.0 / dx2 + 2.0 / dy2);
            }
        }
        self.u = u_new;
        self.iterations += 1;
    }

    /// Run `n` Jacobi iterations.
    pub fn solve(&mut self, iterations: usize) {
        for _ in 0..iterations {
            self.jacobi_step();
        }
    }

    /// Compute residual norm ||Au - f||.
    pub fn residual_norm(&self) -> f32 {
        let nx = self.nx;
        let ny = self.ny;
        let dx2 = self.dx * self.dx;
        let dy2 = self.dy * self.dy;
        let mut sum = 0.0f32;
        for i in 1..nx - 1 {
            for j in 1..ny - 1 {
                let ci = self.idx(i, j);
                let laplacian = (self.u[self.idx(i - 1, j)] - 2.0 * self.u[ci]
                    + self.u[self.idx(i + 1, j)])
                    / dx2
                    + (self.u[self.idx(i, j - 1)] - 2.0 * self.u[ci] + self.u[self.idx(i, j + 1)])
                        / dy2;
                let r = laplacian - self.f[ci];
                sum += r * r;
            }
        }
        sum.sqrt()
    }

    /// Reset solution to zero.
    pub fn reset(&mut self) {
        self.u.fill(0.0);
        self.iterations = 0;
    }

    /// Return max absolute value of solution.
    pub fn max_abs(&self) -> f32 {
        self.u.iter().map(|&v| v.abs()).fold(0.0f32, f32::max)
    }

    /// Set Dirichlet boundary condition on all edges to value.
    pub fn set_boundary(&mut self, val: f32) {
        let nx = self.nx;
        let ny = self.ny;
        for j in 0..ny {
            let i0 = self.idx(0, j);
            let i1 = self.idx(nx - 1, j);
            self.u[i0] = val;
            self.u[i1] = val;
        }
        for i in 0..nx {
            let j0 = self.idx(i, 0);
            let j1 = self.idx(i, ny - 1);
            self.u[j0] = val;
            self.u[j1] = val;
        }
    }
}

/// 1D Poisson solver: -d2u/dx2 = f, Dirichlet BC u(0)=u(1)=0.
pub fn poisson_1d(f: &[f32], dx: f32) -> Vec<f32> {
    let n = f.len();
    if n < 3 {
        return vec![0.0; n];
    }
    /* Tridiagonal system: -1/dx^2 * u[i-1] + 2/dx^2 * u[i] - 1/dx^2 * u[i+1] = f[i] */
    let mut a = vec![-1.0f32; n];
    let mut b = vec![2.0f32; n];
    let mut c = vec![-1.0f32; n];
    let mut d: Vec<f32> = f.iter().map(|&v| v * dx * dx).collect();
    /* Boundary: u[0] = u[n-1] = 0 */
    b[0] = 1.0;
    c[0] = 0.0;
    d[0] = 0.0;
    a[n - 1] = 0.0;
    b[n - 1] = 1.0;
    d[n - 1] = 0.0;

    /* Thomas algorithm */
    for i in 1..n {
        let m = a[i] / b[i - 1];
        b[i] -= m * c[i - 1];
        d[i] -= m * d[i - 1];
    }
    let mut x = vec![0.0f32; n];
    x[n - 1] = d[n - 1] / b[n - 1];
    for i in (0..n - 1).rev() {
        x[i] = (d[i] - c[i] * x[i + 1]) / b[i];
    }
    x
}

pub fn new_poisson_solver(nx: usize, ny: usize, lx: f32, ly: f32) -> PoissonSolver2D {
    PoissonSolver2D::new(nx, ny, lx, ly)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construction() {
        let s = new_poisson_solver(10, 10, 1.0, 1.0);
        assert_eq!(s.nx, 10);
        assert_eq!(s.ny, 10);
    }

    #[test]
    fn test_jacobi_decreases_residual() {
        let mut s = PoissonSolver2D::new(20, 20, 1.0, 1.0);
        for i in 5..15 {
            for j in 5..15 {
                s.set_f(i, j, 1.0);
            }
        }
        s.solve(1);
        let r0 = s.residual_norm();
        s.solve(100);
        let r1 = s.residual_norm();
        assert!(r1 <= r0 + 1e-3 /* residual should generally decrease */);
    }

    #[test]
    fn test_reset() {
        let mut s = new_poisson_solver(10, 10, 1.0, 1.0);
        s.solve(5);
        s.reset();
        assert!((s.max_abs()).abs() < 1e-10);
        assert_eq!(s.iterations, 0);
    }

    #[test]
    fn test_1d_poisson_uniform() {
        /* f = 1, exact solution: u = x(1-x)/2 */
        let n = 11;
        let dx = 1.0 / (n - 1) as f32;
        let f = vec![1.0f32; n];
        let u = poisson_1d(&f, dx);
        /* Check interior */
        let mid = n / 2;
        let x = mid as f32 * dx;
        let exact = x * (1.0 - x) / 2.0;
        assert!(
            (u[mid] - exact).abs() < 0.05,
            "got {}, expected ~{}",
            u[mid],
            exact
        );
    }

    #[test]
    fn test_boundary_zero() {
        let mut s = PoissonSolver2D::new(10, 10, 1.0, 1.0);
        s.set_f(5, 5, 1.0);
        s.solve(50);
        /* Boundaries should remain near 0 */
        assert!(s.get_u(0, 0).abs() < 1e-6);
    }

    #[test]
    fn test_set_boundary() {
        let mut s = new_poisson_solver(10, 10, 1.0, 1.0);
        s.set_boundary(5.0);
        assert!((s.get_u(0, 0) - 5.0).abs() < 1e-5);
        assert!((s.get_u(9, 9) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_max_abs_after_solve() {
        let mut s = PoissonSolver2D::new(15, 15, 1.0, 1.0);
        s.set_f(7, 7, 1000.0);
        s.solve(10);
        assert!(s.max_abs() > 0.0);
    }

    #[test]
    fn test_iteration_counter() {
        let mut s = new_poisson_solver(8, 8, 1.0, 1.0);
        s.solve(7);
        assert_eq!(s.iterations, 7);
    }
}
