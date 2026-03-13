// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! 2D Euler fluid simulation stub (MAC grid).

#[derive(Debug, Clone)]
pub struct FluidGrid2d {
    pub width: usize,
    pub height: usize,
    pub cell_size: f32,
    /// Horizontal velocity at cell edges (u-field, size (w+1)*h).
    pub u: Vec<f32>,
    /// Vertical velocity at cell edges (v-field, size w*(h+1)).
    pub v: Vec<f32>,
    /// Pressure field, size w*h.
    pub pressure: Vec<f32>,
    /// Density field, size w*h.
    pub density: Vec<f32>,
}

impl FluidGrid2d {
    pub fn new(width: usize, height: usize, cell_size: f32) -> Self {
        FluidGrid2d {
            width,
            height,
            cell_size,
            u: vec![0.0; (width + 1) * height],
            v: vec![0.0; width * (height + 1)],
            pressure: vec![0.0; width * height],
            density: vec![0.0; width * height],
        }
    }

    pub fn cell_idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn add_density(&mut self, x: usize, y: usize, amount: f32) {
        if x < self.width && y < self.height {
            let idx = self.cell_idx(x, y);
            self.density[idx] += amount;
        }
    }

    pub fn add_velocity_u(&mut self, x: usize, y: usize, amount: f32) {
        if x <= self.width && y < self.height {
            let idx = y * (self.width + 1) + x;
            self.u[idx] += amount;
        }
    }

    pub fn add_velocity_v(&mut self, x: usize, y: usize, amount: f32) {
        if x < self.width && y <= self.height {
            let idx = y * self.width + x;
            self.v[idx] += amount;
        }
    }

    pub fn diffuse_density(&mut self, diff: f32, dt: f32) {
        let a = dt * diff / (self.cell_size * self.cell_size);
        for d in self.density.iter_mut() {
            *d *= 1.0 - a;
        }
    }

    pub fn total_density(&self) -> f32 {
        self.density.iter().sum()
    }

    pub fn clear(&mut self) {
        self.u.iter_mut().for_each(|v| *v = 0.0);
        self.v.iter_mut().for_each(|v| *v = 0.0);
        self.pressure.iter_mut().for_each(|v| *v = 0.0);
        self.density.iter_mut().for_each(|v| *v = 0.0);
    }

    pub fn step(&mut self, dt: f32) {
        self.diffuse_density(0.001, dt);
    }
}

pub fn divergence_at(grid: &FluidGrid2d, x: usize, y: usize) -> f32 {
    if x >= grid.width || y >= grid.height {
        return 0.0;
    }
    let u_right = grid.u[y * (grid.width + 1) + x + 1];
    let u_left = grid.u[y * (grid.width + 1) + x];
    let v_top = grid.v[(y + 1) * grid.width + x];
    let v_bot = grid.v[y * grid.width + x];
    (u_right - u_left + v_top - v_bot) / grid.cell_size
}

pub fn grid_cell_count(grid: &FluidGrid2d) -> usize {
    grid.width * grid.height
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation() {
        let g = FluidGrid2d::new(10, 10, 1.0);
        assert_eq!(g.density.len(), 100);
    }

    #[test]
    fn test_add_density() {
        let mut g = FluidGrid2d::new(10, 10, 1.0);
        g.add_density(5, 5, 1.0);
        let idx = g.cell_idx(5, 5);
        assert!((g.density[idx] - 1.0).abs() < 1e-5 /* density added */,);
    }

    #[test]
    fn test_total_density() {
        let mut g = FluidGrid2d::new(5, 5, 1.0);
        g.add_density(2, 2, 5.0);
        assert!((g.total_density() - 5.0).abs() < 1e-5, /* total density sum */);
    }

    #[test]
    fn test_diffuse_reduces_density() {
        let mut g = FluidGrid2d::new(5, 5, 1.0);
        g.add_density(2, 2, 10.0);
        let before = g.total_density();
        g.diffuse_density(1.0, 1.0);
        assert!(g.total_density() < before, /* diffusion reduces density */);
    }

    #[test]
    fn test_clear() {
        let mut g = FluidGrid2d::new(5, 5, 1.0);
        g.add_density(2, 2, 5.0);
        g.clear();
        assert!((g.total_density() - 0.0).abs() < 1e-6 /* cleared */,);
    }

    #[test]
    fn test_add_velocity_u() {
        let mut g = FluidGrid2d::new(5, 5, 1.0);
        g.add_velocity_u(2, 2, 1.0);
        let idx = 2 * (5 + 1) + 2;
        assert!((g.u[idx] - 1.0).abs() < 1e-5 /* u velocity added */,);
    }

    #[test]
    fn test_divergence() {
        let g = FluidGrid2d::new(5, 5, 1.0);
        let div = divergence_at(&g, 2, 2);
        assert!(div.abs() < 1e-6, /* zero velocities = zero divergence */);
    }

    #[test]
    fn test_cell_count() {
        let g = FluidGrid2d::new(4, 6, 1.0);
        assert_eq!(grid_cell_count(&g), 24);
    }

    #[test]
    fn test_step_runs() {
        let mut g = FluidGrid2d::new(5, 5, 1.0);
        g.add_density(2, 2, 1.0);
        g.step(0.016);
        /* simulation does not panic */
        assert!(g.total_density().is_finite(), /* density stays finite */);
    }
}
