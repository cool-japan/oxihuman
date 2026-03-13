// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Eulerian (MAC) grid-based 2-D fluid simulation.
//!
//! Uses a staggered Marker-And-Cell (MAC) grid:
//! - `u` velocities are stored on vertical cell faces (left edge of each cell).
//! - `v` velocities are stored on horizontal cell faces (bottom edge).
//! - Scalar quantities (density, pressure) live at cell centres.
//!
//! The simulation pipeline per step:
//! 1. Semi-Lagrangian advection of velocities.
//! 2. Iterative divergence-free projection (Jacobi iterations).

// ── Configuration ─────────────────────────────────────────────────────────────

/// Parameters for the Eulerian fluid simulation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EulerianConfig {
    /// Grid width in cells.
    pub width: usize,
    /// Grid height in cells.
    pub height: usize,
    /// Physical size of each cell (meters).
    pub cell_size: f32,
    /// Fluid density (kg/m³) used for pressure projection.
    pub density: f32,
    /// Number of pressure-projection Jacobi iterations.
    pub projection_iters: usize,
    /// Velocity dissipation per step (0 = no damping, 1 = full).
    pub dissipation: f32,
}

/// Build an [`EulerianConfig`] with sensible defaults (64×64 grid).
#[allow(dead_code)]
pub fn default_eulerian_config() -> EulerianConfig {
    EulerianConfig {
        width: 64,
        height: 64,
        cell_size: 1.0 / 64.0,
        density: 1.0,
        projection_iters: 20,
        dissipation: 0.99,
    }
}

// ── Grid cell ─────────────────────────────────────────────────────────────────

/// A single grid cell storing per-cell scalars.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Default)]
pub struct GridCell {
    /// Scalar density / smoke tracer.
    pub density: f32,
    /// Pressure at cell centre.
    pub pressure: f32,
    /// Whether this cell is a solid obstacle.
    pub obstacle: bool,
}

// ── Eulerian grid ─────────────────────────────────────────────────────────────

/// The full MAC grid state.
///
/// U velocities are on a `(width+1) × height` staggered grid.
/// V velocities are on a `width × (height+1)` staggered grid.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EulerianGrid {
    /// Config used to build this grid.
    pub config: EulerianConfig,
    /// U (horizontal) velocity components: `u[i + j*(width+1)]`.
    pub u: Vec<f32>,
    /// V (vertical) velocity components: `v[i + j*width]`.
    pub v: Vec<f32>,
    /// Per-cell data.
    pub cells: Vec<GridCell>,
}

// ── Constructor ───────────────────────────────────────────────────────────────

/// Create a new zero-initialized [`EulerianGrid`] from the given config.
#[allow(dead_code)]
pub fn new_eulerian_grid(config: EulerianConfig) -> EulerianGrid {
    let w = config.width;
    let h = config.height;
    EulerianGrid {
        u: vec![0.0; (w + 1) * h],
        v: vec![0.0; w * (h + 1)],
        cells: vec![GridCell::default(); w * h],
        config,
    }
}

// ── Accessors ─────────────────────────────────────────────────────────────────

/// Return the grid width in cells.
#[allow(dead_code)]
pub fn grid_width(grid: &EulerianGrid) -> usize {
    grid.config.width
}

/// Return the grid height in cells.
#[allow(dead_code)]
pub fn grid_height(grid: &EulerianGrid) -> usize {
    grid.config.height
}

/// Set the U (horizontal) velocity at face `(i, j)` where `i ∈ [0, width]`, `j ∈ [0, height)`.
#[allow(dead_code)]
pub fn set_velocity_u(grid: &mut EulerianGrid, i: usize, j: usize, val: f32) {
    let w1 = grid.config.width + 1;
    let h = grid.config.height;
    if i < w1 && j < h {
        grid.u[i + j * w1] = val;
    }
}

/// Set the V (vertical) velocity at face `(i, j)` where `i ∈ [0, width)`, `j ∈ [0, height]`.
#[allow(dead_code)]
pub fn set_velocity_v(grid: &mut EulerianGrid, i: usize, j: usize, val: f32) {
    let w = grid.config.width;
    let h1 = grid.config.height + 1;
    if i < w && j < h1 {
        grid.v[i + j * w] = val;
    }
}

/// Interpolate the velocity vector `(u, v)` at a continuous cell-space position `(x, y)`.
///
/// Uses bilinear interpolation on the staggered MAC faces.
#[allow(dead_code)]
pub fn get_velocity(grid: &EulerianGrid, x: f32, y: f32) -> [f32; 2] {
    let u = sample_u(grid, x - 0.5, y);
    let v = sample_v(grid, x, y - 0.5);
    [u, v]
}

// ── Simulation ────────────────────────────────────────────────────────────────

/// Semi-Lagrangian advection of the velocity field.
///
/// Traces particles backwards from each face position, samples the
/// velocity at the traced-back location, and stores the result.
#[allow(dead_code)]
pub fn advect_velocity(grid: &mut EulerianGrid, dt: f32) {
    let w = grid.config.width;
    let h = grid.config.height;
    let cs = grid.config.cell_size;

    // Advect U faces
    let w1 = w + 1;
    let mut new_u = grid.u.clone();
    for j in 0..h {
        for i in 0..w1 {
            let x = i as f32;
            let y = j as f32 + 0.5;
            let [ux, vy] = get_velocity(grid, x, y);
            let px = x - ux * dt / cs;
            let py = y - vy * dt / cs;
            new_u[i + j * w1] = sample_u(grid, px - 0.5, py);
        }
    }

    // Advect V faces
    let h1 = h + 1;
    let mut new_v = grid.v.clone();
    for j in 0..h1 {
        for i in 0..w {
            let x = i as f32 + 0.5;
            let y = j as f32;
            let [ux, vy] = get_velocity(grid, x, y);
            let px = x - ux * dt / cs;
            let py = y - vy * dt / cs;
            new_v[i + j * w] = sample_v(grid, px, py - 0.5);
        }
    }

    let diss = grid.config.dissipation;
    for v in new_u.iter_mut() {
        *v *= diss;
    }
    for v in new_v.iter_mut() {
        *v *= diss;
    }
    grid.u = new_u;
    grid.v = new_v;
}

/// Make the velocity field divergence-free using Jacobi pressure projection.
#[allow(dead_code)]
pub fn project_velocity(grid: &mut EulerianGrid) {
    let w = grid.config.width;
    let h = grid.config.height;
    let cs = grid.config.cell_size;
    let iters = grid.config.projection_iters;

    // Reset pressure
    for cell in grid.cells.iter_mut() {
        cell.pressure = 0.0;
    }

    let scale = cs; // Δx
    let inv_scale2 = 1.0 / (scale * scale);

    for _ in 0..iters {
        let old_pressure: Vec<f32> = grid.cells.iter().map(|c| c.pressure).collect();
        for j in 0..h {
            for i in 0..w {
                if grid.cells[i + j * w].obstacle {
                    continue;
                }
                let div = cell_divergence(grid, i, j);
                // Jacobi update: p = (sum_neighbours p + rhs) / count
                let mut sum_p = 0.0f32;
                let mut cnt = 0u32;
                if i > 0 && !grid.cells[(i - 1) + j * w].obstacle {
                    sum_p += old_pressure[(i - 1) + j * w];
                    cnt += 1;
                }
                if i + 1 < w && !grid.cells[(i + 1) + j * w].obstacle {
                    sum_p += old_pressure[(i + 1) + j * w];
                    cnt += 1;
                }
                if j > 0 && !grid.cells[i + (j - 1) * w].obstacle {
                    sum_p += old_pressure[i + (j - 1) * w];
                    cnt += 1;
                }
                if j + 1 < h && !grid.cells[i + (j + 1) * w].obstacle {
                    sum_p += old_pressure[i + (j + 1) * w];
                    cnt += 1;
                }
                if cnt == 0 {
                    continue;
                }
                let rhs = div * grid.config.density * inv_scale2;
                grid.cells[i + j * w].pressure =
                    (sum_p + rhs / (cnt as f32 * inv_scale2)) / cnt as f32;
            }
        }

        // Apply pressure gradient to velocities
        for j in 0..h {
            for i in 0..w {
                let p = grid.cells[i + j * w].pressure;
                let w1 = w + 1;
                // Right U face
                if i + 1 < w {
                    let pr = grid.cells[(i + 1) + j * w].pressure;
                    grid.u[(i + 1) + j * w1] -= (pr - p) / scale;
                }
                // Top V face
                if j + 1 < h {
                    let pt = grid.cells[i + (j + 1) * w].pressure;
                    grid.v[i + (j + 1) * w] -= (pt - p) / scale;
                }
            }
        }
    }
}

/// Add a scalar density source at cell `(i, j)`.
#[allow(dead_code)]
pub fn add_source(grid: &mut EulerianGrid, i: usize, j: usize, amount: f32) {
    let w = grid.config.width;
    let h = grid.config.height;
    if i < w && j < h {
        grid.cells[i + j * w].density += amount;
    }
}

/// Mark a cell as an obstacle (solid wall).
#[allow(dead_code)]
pub fn set_obstacle(grid: &mut EulerianGrid, i: usize, j: usize, is_obstacle: bool) {
    let w = grid.config.width;
    let h = grid.config.height;
    if i < w && j < h {
        grid.cells[i + j * w].obstacle = is_obstacle;
    }
}

/// Compute the velocity divergence at cell `(i, j)`.
#[allow(dead_code)]
pub fn grid_divergence(grid: &EulerianGrid, i: usize, j: usize) -> f32 {
    let w = grid.config.width;
    let h = grid.config.height;
    if i >= w || j >= h {
        return 0.0;
    }
    cell_divergence(grid, i, j)
}

/// Run a full fluid step: advect + project.
#[allow(dead_code)]
pub fn fluid_step(grid: &mut EulerianGrid, dt: f32) {
    advect_velocity(grid, dt);
    project_velocity(grid);
}

/// Compute a scalar measure of total kinetic energy in the grid.
#[allow(dead_code)]
pub fn grid_energy(grid: &EulerianGrid) -> f32 {
    let u_energy: f32 = grid.u.iter().map(|&u| u * u).sum::<f32>() * 0.5;
    let v_energy: f32 = grid.v.iter().map(|&v| v * v).sum::<f32>() * 0.5;
    u_energy + v_energy
}

/// Reset all velocities and densities to zero.
#[allow(dead_code)]
pub fn reset_grid(grid: &mut EulerianGrid) {
    for v in grid.u.iter_mut() {
        *v = 0.0;
    }
    for v in grid.v.iter_mut() {
        *v = 0.0;
    }
    for cell in grid.cells.iter_mut() {
        cell.density = 0.0;
        cell.pressure = 0.0;
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Compute divergence at cell `(i, j)` from current velocities.
fn cell_divergence(grid: &EulerianGrid, i: usize, j: usize) -> f32 {
    let w = grid.config.width;
    let cs = grid.config.cell_size;
    let w1 = w + 1;
    let u_right = grid.u[(i + 1) + j * w1];
    let u_left = grid.u[i + j * w1];
    let v_top = grid.v[i + (j + 1) * w];
    let v_bottom = grid.v[i + j * w];
    (u_right - u_left + v_top - v_bottom) / cs
}

/// Bilinear sample of U velocity at continuous position `(x, y)` in cell space.
fn sample_u(grid: &EulerianGrid, x: f32, y: f32) -> f32 {
    let w1 = grid.config.width + 1;
    let h = grid.config.height;
    bilinear_sample(&grid.u, w1, h, x, y)
}

/// Bilinear sample of V velocity at continuous position `(x, y)` in cell space.
fn sample_v(grid: &EulerianGrid, x: f32, y: f32) -> f32 {
    let w = grid.config.width;
    let h1 = grid.config.height + 1;
    bilinear_sample(&grid.v, w, h1, x, y)
}

/// Generic bilinear sample over a flat `width × height` array.
fn bilinear_sample(data: &[f32], width: usize, height: usize, x: f32, y: f32) -> f32 {
    let x = x.clamp(0.0, (width as f32) - 1.0 - 1e-4);
    let y = y.clamp(0.0, (height as f32) - 1.0 - 1e-4);
    let ix = x as usize;
    let iy = y as usize;
    let fx = x - ix as f32;
    let fy = y - iy as f32;
    let ix1 = (ix + 1).min(width - 1);
    let iy1 = (iy + 1).min(height - 1);
    let v00 = data[ix + iy * width];
    let v10 = data[ix1 + iy * width];
    let v01 = data[ix + iy1 * width];
    let v11 = data[ix1 + iy1 * width];
    v00 * (1.0 - fx) * (1.0 - fy) + v10 * fx * (1.0 - fy) + v01 * (1.0 - fx) * fy + v11 * fx * fy
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn small_grid() -> EulerianGrid {
        let mut cfg = default_eulerian_config();
        cfg.width = 8;
        cfg.height = 8;
        cfg.cell_size = 0.125;
        cfg.projection_iters = 5;
        new_eulerian_grid(cfg)
    }

    #[test]
    fn default_config_positive_dimensions() {
        let c = default_eulerian_config();
        assert!(c.width > 0 && c.height > 0);
    }

    #[test]
    fn new_eulerian_grid_correct_sizes() {
        let grid = small_grid();
        let w = grid.config.width;
        let h = grid.config.height;
        assert_eq!(grid.u.len(), (w + 1) * h);
        assert_eq!(grid.v.len(), w * (h + 1));
        assert_eq!(grid.cells.len(), w * h);
    }

    #[test]
    fn grid_width_and_height_match_config() {
        let grid = small_grid();
        assert_eq!(grid_width(&grid), grid.config.width);
        assert_eq!(grid_height(&grid), grid.config.height);
    }

    #[test]
    fn set_velocity_u_stores_value() {
        let mut grid = small_grid();
        set_velocity_u(&mut grid, 2, 3, 1.5);
        let w1 = grid.config.width + 1;
        assert!((grid.u[2 + 3 * w1] - 1.5).abs() < 1e-6);
    }

    #[test]
    fn set_velocity_v_stores_value() {
        let mut grid = small_grid();
        set_velocity_v(&mut grid, 4, 5, 2.5);
        let w = grid.config.width;
        assert!((grid.v[4 + 5 * w] - 2.5).abs() < 1e-6);
    }

    #[test]
    fn get_velocity_zero_for_fresh_grid() {
        let grid = small_grid();
        let [u, v] = get_velocity(&grid, 3.5, 3.5);
        assert!(u.abs() < 1e-6 && v.abs() < 1e-6);
    }

    #[test]
    fn add_source_increases_density() {
        let mut grid = small_grid();
        add_source(&mut grid, 2, 2, 1.0);
        let w = grid.config.width;
        assert!((grid.cells[2 + 2 * w].density - 1.0).abs() < 1e-6);
    }

    #[test]
    fn grid_divergence_zero_for_fresh_grid() {
        let grid = small_grid();
        let div = grid_divergence(&grid, 3, 3);
        assert!(div.abs() < 1e-6);
    }

    #[test]
    fn advect_velocity_does_not_explode() {
        let mut grid = small_grid();
        set_velocity_u(&mut grid, 4, 4, 0.5);
        advect_velocity(&mut grid, 0.016);
        let max_u = grid.u.iter().cloned().fold(0.0f32, f32::max);
        assert!(max_u.is_finite() && max_u < 10.0);
    }

    #[test]
    fn project_velocity_runs_without_panic() {
        let mut grid = small_grid();
        set_velocity_u(&mut grid, 4, 4, 0.1);
        project_velocity(&mut grid);
    }

    #[test]
    fn fluid_step_runs_without_panic() {
        let mut grid = small_grid();
        set_velocity_u(&mut grid, 2, 2, 0.1);
        fluid_step(&mut grid, 0.016);
    }

    #[test]
    fn grid_energy_zero_for_fresh_grid() {
        let grid = small_grid();
        assert!(grid_energy(&grid).abs() < 1e-8);
    }

    #[test]
    fn grid_energy_positive_after_velocity_set() {
        let mut grid = small_grid();
        set_velocity_u(&mut grid, 3, 3, 1.0);
        assert!(grid_energy(&grid) > 0.0);
    }

    #[test]
    fn reset_grid_zeroes_velocities_and_density() {
        let mut grid = small_grid();
        set_velocity_u(&mut grid, 3, 3, 1.0);
        add_source(&mut grid, 3, 3, 5.0);
        reset_grid(&mut grid);
        let e = grid_energy(&grid);
        let w = grid.config.width;
        let d = grid.cells[3 + 3 * w].density;
        assert!(e.abs() < 1e-8, "energy after reset={e}");
        assert!(d.abs() < 1e-8, "density after reset={d}");
    }

    #[test]
    fn set_obstacle_marks_cell() {
        let mut grid = small_grid();
        set_obstacle(&mut grid, 3, 3, true);
        let w = grid.config.width;
        assert!(grid.cells[3 + 3 * w].obstacle);
    }

    #[test]
    fn set_velocity_u_oob_is_noop() {
        let mut grid = small_grid();
        let before = grid.u.clone();
        set_velocity_u(&mut grid, 999, 999, 99.0);
        assert_eq!(grid.u, before);
    }
}
