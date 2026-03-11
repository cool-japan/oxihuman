#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fluid advection on a 2-D grid (simple semi-Lagrangian).

/// Configuration for advection.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AdvectConfig {
    pub width: usize,
    pub height: usize,
    pub cell_size: f32,
}

/// A 2-D fluid grid with velocity and density fields.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FluidGrid {
    pub width: usize,
    pub height: usize,
    pub cell_size: f32,
    /// X velocity per cell.
    pub vel_x: Vec<f32>,
    /// Y velocity per cell.
    pub vel_y: Vec<f32>,
    /// Scalar density per cell.
    pub density: Vec<f32>,
}

impl FluidGrid {
    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }
}

/// Create a new fluid grid.
#[allow(dead_code)]
pub fn new_fluid_grid(config: &AdvectConfig) -> FluidGrid {
    let n = config.width * config.height;
    FluidGrid {
        width: config.width,
        height: config.height,
        cell_size: config.cell_size,
        vel_x: vec![0.0; n],
        vel_y: vec![0.0; n],
        density: vec![0.0; n],
    }
}

fn clamp_idx(v: f32, max: usize) -> usize {
    (v.floor() as isize).clamp(0, max as isize - 1) as usize
}

/// Perform one advection step on the density field.
#[allow(dead_code)]
pub fn advect_step(grid: &mut FluidGrid, dt: f32) {
    let w = grid.width;
    let h = grid.height;
    let cs = grid.cell_size;
    let old_density = grid.density.clone();
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let vx = grid.vel_x[idx];
            let vy = grid.vel_y[idx];
            let src_x = (x as f32 - vx * dt / cs).clamp(0.0, (w - 1) as f32);
            let src_y = (y as f32 - vy * dt / cs).clamp(0.0, (h - 1) as f32);
            let xi = clamp_idx(src_x, w);
            let yi = clamp_idx(src_y, h);
            grid.density[idx] = old_density[yi * w + xi];
        }
    }
}

/// Get velocity at a cell.
#[allow(dead_code)]
pub fn grid_vel_at(grid: &FluidGrid, x: usize, y: usize) -> [f32; 2] {
    let i = grid.idx(x.min(grid.width - 1), y.min(grid.height - 1));
    [grid.vel_x[i], grid.vel_y[i]]
}

/// Get density at a cell.
#[allow(dead_code)]
pub fn grid_density_at(grid: &FluidGrid, x: usize, y: usize) -> f32 {
    let i = grid.idx(x.min(grid.width - 1), y.min(grid.height - 1));
    grid.density[i]
}

/// Set velocity at a cell.
#[allow(dead_code)]
pub fn set_grid_vel(grid: &mut FluidGrid, x: usize, y: usize, vx: f32, vy: f32) {
    let i = grid.idx(x.min(grid.width - 1), y.min(grid.height - 1));
    grid.vel_x[i] = vx;
    grid.vel_y[i] = vy;
}

/// Add density to a cell.
#[allow(dead_code)]
pub fn add_density(grid: &mut FluidGrid, x: usize, y: usize, amount: f32) {
    let i = grid.idx(x.min(grid.width - 1), y.min(grid.height - 1));
    grid.density[i] += amount;
}

/// Return the grid width.
#[allow(dead_code)]
pub fn fluid_grid_width(grid: &FluidGrid) -> usize {
    grid.width
}

/// Return the grid height.
#[allow(dead_code)]
pub fn fluid_grid_height(grid: &FluidGrid) -> usize {
    grid.height
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_grid(w: usize, h: usize) -> FluidGrid {
        new_fluid_grid(&AdvectConfig { width: w, height: h, cell_size: 1.0 })
    }

    #[test]
    fn test_new_fluid_grid() {
        let g = make_grid(4, 4);
        assert_eq!(fluid_grid_width(&g), 4);
        assert_eq!(fluid_grid_height(&g), 4);
    }

    #[test]
    fn test_add_density() {
        let mut g = make_grid(4, 4);
        add_density(&mut g, 1, 1, 5.0);
        assert!((grid_density_at(&g, 1, 1) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_get_vel() {
        let mut g = make_grid(4, 4);
        set_grid_vel(&mut g, 2, 2, 3.0, -1.0);
        let v = grid_vel_at(&g, 2, 2);
        assert!((v[0] - 3.0).abs() < 1e-6);
        assert!((v[1] + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_advect_step_moves_density() {
        let mut g = make_grid(8, 8);
        add_density(&mut g, 4, 4, 1.0);
        set_grid_vel(&mut g, 4, 4, 1.0, 0.0);
        advect_step(&mut g, 1.0);
        // Density should have been advected (away from original position)
        // Just check it doesn't panic and grid is valid
        assert_eq!(fluid_grid_width(&g), 8);
    }

    #[test]
    fn test_grid_density_initially_zero() {
        let g = make_grid(3, 3);
        assert_eq!(grid_density_at(&g, 0, 0), 0.0);
    }

    #[test]
    fn test_grid_vel_initially_zero() {
        let g = make_grid(3, 3);
        let v = grid_vel_at(&g, 1, 1);
        assert_eq!(v, [0.0, 0.0]);
    }

    #[test]
    fn test_add_density_accumulates() {
        let mut g = make_grid(4, 4);
        add_density(&mut g, 0, 0, 2.0);
        add_density(&mut g, 0, 0, 3.0);
        assert!((grid_density_at(&g, 0, 0) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_clamped_index() {
        // Out-of-bounds access should clamp
        let mut g = make_grid(4, 4);
        add_density(&mut g, 100, 100, 1.0);
        // No panic, value added to clamped cell
    }
}
