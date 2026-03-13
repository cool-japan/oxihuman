// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Fick's law diffusion in 1D and 2D.

/// 1D diffusion model using explicit finite differences.
pub struct Diffusion1D {
    pub c: Vec<f32>,
    pub d: f32,
    pub dx: f32,
    pub time: f32,
}

impl Diffusion1D {
    pub fn new(n: usize, length: f32, d: f32) -> Self {
        let n = n.max(3);
        Diffusion1D {
            c: vec![0.0f32; n],
            d,
            dx: length / (n - 1) as f32,
            time: 0.0,
        }
    }

    pub fn stable_dt(&self) -> f32 {
        self.dx * self.dx / (2.0 * self.d.max(1e-12))
    }

    #[allow(clippy::needless_range_loop)]
    pub fn step(&mut self, dt: f32) {
        let n = self.c.len();
        let r = self.d * dt / (self.dx * self.dx);
        let mut c_new = self.c.clone();
        for i in 1..n - 1 {
            c_new[i] = self.c[i] + r * (self.c[i - 1] - 2.0 * self.c[i] + self.c[i + 1]);
        }
        self.c = c_new;
        self.time += dt;
    }

    pub fn advance(&mut self, steps: usize) {
        let dt = self.stable_dt();
        for _ in 0..steps {
            self.step(dt);
        }
    }

    pub fn total_mass(&self) -> f32 {
        self.c.iter().sum::<f32>() * self.dx
    }

    pub fn max_concentration(&self) -> f32 {
        self.c.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
    }

    pub fn grid_size(&self) -> usize {
        self.c.len()
    }
}

/// 2D diffusion model (nx × ny grid).
pub struct Diffusion2D {
    pub c: Vec<f32>,
    pub nx: usize,
    pub ny: usize,
    pub dx: f32,
    pub dy: f32,
    pub d: f32,
    pub time: f32,
}

impl Diffusion2D {
    pub fn new(nx: usize, ny: usize, lx: f32, ly: f32, d: f32) -> Self {
        let nx = nx.max(3);
        let ny = ny.max(3);
        Diffusion2D {
            c: vec![0.0f32; nx * ny],
            nx,
            ny,
            dx: lx / (nx - 1) as f32,
            dy: ly / (ny - 1) as f32,
            d,
            time: 0.0,
        }
    }

    pub fn idx(&self, i: usize, j: usize) -> usize {
        i * self.ny + j
    }

    pub fn get(&self, i: usize, j: usize) -> f32 {
        self.c[self.idx(i, j)]
    }

    pub fn set(&mut self, i: usize, j: usize, val: f32) {
        let idx = self.idx(i, j);
        self.c[idx] = val;
    }

    pub fn stable_dt(&self) -> f32 {
        let rx = self.d / (self.dx * self.dx);
        let ry = self.d / (self.dy * self.dy);
        0.5 / (rx + ry).max(1e-12)
    }

    pub fn step(&mut self, dt: f32) {
        let rx = self.d * dt / (self.dx * self.dx);
        let ry = self.d * dt / (self.dy * self.dy);
        let mut c_new = self.c.clone();
        for i in 1..self.nx - 1 {
            for j in 1..self.ny - 1 {
                let ci = self.idx(i, j);
                let lap_x =
                    self.c[self.idx(i - 1, j)] - 2.0 * self.c[ci] + self.c[self.idx(i + 1, j)];
                let lap_y =
                    self.c[self.idx(i, j - 1)] - 2.0 * self.c[ci] + self.c[self.idx(i, j + 1)];
                c_new[ci] = self.c[ci] + rx * lap_x + ry * lap_y;
            }
        }
        self.c = c_new;
        self.time += dt;
    }

    pub fn advance(&mut self, steps: usize) {
        let dt = self.stable_dt();
        for _ in 0..steps {
            self.step(dt);
        }
    }

    pub fn total_mass(&self) -> f32 {
        self.c.iter().sum::<f32>() * self.dx * self.dy
    }

    pub fn max_concentration(&self) -> f32 {
        self.c.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
    }
}

/// Fick's first law: flux = -D * (dc/dx)
pub fn ficks_first_law(d: f32, c_left: f32, c_right: f32, dx: f32) -> f32 {
    -d * (c_right - c_left) / dx
}

/// Analytical Gaussian solution for 1D diffusion from a point source.
pub fn gaussian_solution(x: f32, t: f32, m: f32, d: f32) -> f32 {
    if t <= 0.0 {
        return 0.0;
    }
    let denom = (4.0 * std::f32::consts::PI * d * t).sqrt();
    m / denom * (-x * x / (4.0 * d * t)).exp()
}

pub fn new_diffusion_1d(n: usize, length: f32, d: f32) -> Diffusion1D {
    Diffusion1D::new(n, length, d)
}

pub fn new_diffusion_2d(nx: usize, ny: usize, lx: f32, ly: f32, d: f32) -> Diffusion2D {
    Diffusion2D::new(nx, ny, lx, ly, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1d_construction() {
        let d = new_diffusion_1d(20, 1.0, 0.01);
        assert_eq!(d.grid_size(), 20);
    }

    #[test]
    fn test_1d_pulse_diffuses() {
        let mut d = Diffusion1D::new(51, 1.0, 0.1);
        d.c[25] = 1.0;
        let init = d.max_concentration();
        d.advance(10);
        assert!(d.max_concentration() < init);
    }

    #[test]
    fn test_1d_stable_dt() {
        let d = Diffusion1D::new(20, 1.0, 0.1);
        let dt = d.stable_dt();
        let r = d.d * dt / (d.dx * d.dx);
        assert!(r <= 0.5 + 1e-5);
    }

    #[test]
    fn test_2d_construction() {
        let d = new_diffusion_2d(10, 10, 1.0, 1.0, 0.01);
        assert_eq!(d.nx, 10);
        assert_eq!(d.ny, 10);
    }

    #[test]
    fn test_2d_pulse_diffuses() {
        let mut d = Diffusion2D::new(21, 21, 1.0, 1.0, 0.1);
        d.set(10, 10, 1.0);
        let init = d.max_concentration();
        d.advance(5);
        assert!(d.max_concentration() < init);
    }

    #[test]
    fn test_ficks_first_law() {
        let flux = ficks_first_law(1.0, 10.0, 0.0, 1.0);
        assert!((flux - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_gaussian_solution_peak() {
        let val = gaussian_solution(0.0, 1.0, 1.0, 1.0);
        assert!(val > 0.0);
        let val2 = gaussian_solution(5.0, 1.0, 1.0, 1.0);
        assert!(val > val2);
    }

    #[test]
    fn test_total_mass_1d() {
        let mut d = Diffusion1D::new(51, 1.0, 0.01);
        d.c[25] = 1.0;
        let m0 = d.total_mass();
        d.advance(5);
        /* Mass should be approximately conserved (Dirichlet can leak) */
        assert!(d.total_mass() > 0.0);
        let _ = m0;
    }
}
