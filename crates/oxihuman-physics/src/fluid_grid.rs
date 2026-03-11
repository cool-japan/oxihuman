//! Grid-based (Eulerian) 2D fluid simulation.

#[allow(dead_code)]
pub struct FluidCell {
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub pressure: f32,
    pub density: f32,
    pub obstacle: bool,
}

#[allow(dead_code)]
pub struct FluidGrid {
    pub cells: Vec<FluidCell>,
    pub width: usize,
    pub height: usize,
    pub cell_size: f32,
    pub viscosity: f32,
    pub diffusion: f32,
}

#[allow(dead_code)]
pub struct FluidConfig {
    pub viscosity: f32,
    pub diffusion: f32,
    pub cell_size: f32,
    pub gravity: f32,
}

/// Returns a default fluid configuration.
#[allow(dead_code)]
pub fn default_fluid_config() -> FluidConfig {
    FluidConfig {
        viscosity: 0.01,
        diffusion: 0.001,
        cell_size: 1.0,
        gravity: 0.0,
    }
}

/// Create a new fluid grid.
#[allow(dead_code)]
pub fn new_fluid_grid(width: usize, height: usize, cfg: &FluidConfig) -> FluidGrid {
    let count = width * height;
    let mut cells = Vec::with_capacity(count);
    for _ in 0..count {
        cells.push(FluidCell {
            velocity_x: 0.0,
            velocity_y: 0.0,
            pressure: 0.0,
            density: 0.0,
            obstacle: false,
        });
    }
    FluidGrid {
        cells,
        width,
        height,
        cell_size: cfg.cell_size,
        viscosity: cfg.viscosity,
        diffusion: cfg.diffusion,
    }
}

/// Compute linear cell index from grid coordinates.
#[allow(dead_code)]
pub fn cell_index(grid: &FluidGrid, x: usize, y: usize) -> usize {
    y * grid.width + x
}

/// Get a reference to a cell at (x, y).
#[allow(dead_code)]
pub fn get_cell(grid: &FluidGrid, x: usize, y: usize) -> &FluidCell {
    let idx = cell_index(grid, x, y);
    &grid.cells[idx]
}

/// Get a mutable reference to a cell at (x, y).
#[allow(dead_code)]
pub fn get_cell_mut(grid: &mut FluidGrid, x: usize, y: usize) -> &mut FluidCell {
    let idx = cell_index(grid, x, y);
    &mut grid.cells[idx]
}

/// Add density at a specific cell.
#[allow(dead_code)]
pub fn add_density(grid: &mut FluidGrid, x: usize, y: usize, amount: f32) {
    let idx = cell_index(grid, x, y);
    if !grid.cells[idx].obstacle {
        grid.cells[idx].density += amount;
    }
}

/// Add velocity at a specific cell.
#[allow(dead_code)]
pub fn add_velocity(grid: &mut FluidGrid, x: usize, y: usize, vx: f32, vy: f32) {
    let idx = cell_index(grid, x, y);
    if !grid.cells[idx].obstacle {
        grid.cells[idx].velocity_x += vx;
        grid.cells[idx].velocity_y += vy;
    }
}

/// Simple diffusion step for density.
#[allow(dead_code)]
pub fn diffuse_density(grid: &mut FluidGrid, dt: f32) {
    let w = grid.width;
    let h = grid.height;
    let diff = grid.diffusion;
    let a = dt * diff;

    // Copy current densities
    let old: Vec<f32> = grid.cells.iter().map(|c| c.density).collect();

    for y in 1..h.saturating_sub(1) {
        for x in 1..w.saturating_sub(1) {
            let idx = y * w + x;
            if grid.cells[idx].obstacle {
                continue;
            }
            let neighbors = old[(y - 1) * w + x]
                + old[(y + 1) * w + x]
                + old[y * w + (x - 1)]
                + old[y * w + (x + 1)];
            grid.cells[idx].density = (old[idx] + a * neighbors) / (1.0 + 4.0 * a);
        }
    }
}

/// Semi-Lagrangian advection of density.
#[allow(dead_code)]
pub fn advect_density(grid: &mut FluidGrid, dt: f32) {
    let w = grid.width;
    let h = grid.height;
    let cs = grid.cell_size;

    let old_density: Vec<f32> = grid.cells.iter().map(|c| c.density).collect();
    let vel_x: Vec<f32> = grid.cells.iter().map(|c| c.velocity_x).collect();
    let vel_y: Vec<f32> = grid.cells.iter().map(|c| c.velocity_y).collect();

    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            if grid.cells[idx].obstacle {
                continue;
            }
            // Trace back
            let fx = x as f32 - dt * vel_x[idx] / cs;
            let fy = y as f32 - dt * vel_y[idx] / cs;

            let fx = fx.clamp(0.0, (w - 1) as f32);
            let fy = fy.clamp(0.0, (h - 1) as f32);

            let ix = fx as usize;
            let iy = fy as usize;
            let tx = fx - ix as f32;
            let ty = fy - iy as f32;

            let ix1 = (ix + 1).min(w - 1);
            let iy1 = (iy + 1).min(h - 1);

            // Bilinear interpolation
            let d00 = old_density[iy * w + ix];
            let d10 = old_density[iy * w + ix1];
            let d01 = old_density[iy1 * w + ix];
            let d11 = old_density[iy1 * w + ix1];

            grid.cells[idx].density =
                (1.0 - ty) * ((1.0 - tx) * d00 + tx * d10) + ty * ((1.0 - tx) * d01 + tx * d11);
        }
    }
}

/// One full simulation step.
#[allow(dead_code)]
pub fn step_fluid(grid: &mut FluidGrid, dt: f32) {
    diffuse_density(grid, dt);
    advect_density(grid, dt);
}

/// Sum of all cell densities.
#[allow(dead_code)]
pub fn total_density(grid: &FluidGrid) -> f32 {
    grid.cells.iter().map(|c| c.density).sum()
}

/// Maximum velocity magnitude across all cells.
#[allow(dead_code)]
pub fn max_velocity(grid: &FluidGrid) -> f32 {
    grid.cells
        .iter()
        .map(|c| (c.velocity_x.powi(2) + c.velocity_y.powi(2)).sqrt())
        .fold(0.0f32, f32::max)
}

/// Set or clear an obstacle flag at a cell.
#[allow(dead_code)]
pub fn set_obstacle(grid: &mut FluidGrid, x: usize, y: usize, obstacle: bool) {
    let idx = cell_index(grid, x, y);
    grid.cells[idx].obstacle = obstacle;
    if obstacle {
        grid.cells[idx].density = 0.0;
        grid.cells[idx].velocity_x = 0.0;
        grid.cells[idx].velocity_y = 0.0;
    }
}

/// Returns (total_density, max_velocity, avg_pressure).
#[allow(dead_code)]
pub fn fluid_grid_stats(grid: &FluidGrid) -> (f32, f32, f32) {
    let td = total_density(grid);
    let mv = max_velocity(grid);
    let avg_p = if grid.cells.is_empty() {
        0.0
    } else {
        grid.cells.iter().map(|c| c.pressure).sum::<f32>() / grid.cells.len() as f32
    };
    (td, mv, avg_p)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_grid(w: usize, h: usize) -> FluidGrid {
        let cfg = default_fluid_config();
        new_fluid_grid(w, h, &cfg)
    }

    #[test]
    fn test_new_grid_correct_size() {
        let grid = make_grid(10, 8);
        assert_eq!(grid.cells.len(), 80);
        assert_eq!(grid.width, 10);
        assert_eq!(grid.height, 8);
    }

    #[test]
    fn test_cell_index_formula() {
        let grid = make_grid(5, 5);
        assert_eq!(cell_index(&grid, 0, 0), 0);
        assert_eq!(cell_index(&grid, 1, 0), 1);
        assert_eq!(cell_index(&grid, 0, 1), 5);
        assert_eq!(cell_index(&grid, 4, 4), 24);
    }

    #[test]
    fn test_add_density() {
        let mut grid = make_grid(5, 5);
        add_density(&mut grid, 2, 2, 1.0);
        assert!((get_cell(&grid, 2, 2).density - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_total_density() {
        let mut grid = make_grid(5, 5);
        add_density(&mut grid, 1, 1, 2.0);
        add_density(&mut grid, 3, 3, 3.0);
        let td = total_density(&grid);
        assert!((td - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_add_velocity() {
        let mut grid = make_grid(5, 5);
        add_velocity(&mut grid, 2, 2, 3.0, 4.0);
        let cell = get_cell(&grid, 2, 2);
        assert!((cell.velocity_x - 3.0).abs() < 1e-6);
        assert!((cell.velocity_y - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_max_velocity() {
        let mut grid = make_grid(5, 5);
        add_velocity(&mut grid, 2, 2, 3.0, 4.0); // magnitude = 5
        let mv = max_velocity(&grid);
        assert!((mv - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_set_obstacle() {
        let mut grid = make_grid(5, 5);
        add_density(&mut grid, 2, 2, 5.0);
        set_obstacle(&mut grid, 2, 2, true);
        assert!(get_cell(&grid, 2, 2).obstacle);
        assert!((get_cell(&grid, 2, 2).density).abs() < 1e-6);
    }

    #[test]
    fn test_obstacle_blocks_density_add() {
        let mut grid = make_grid(5, 5);
        set_obstacle(&mut grid, 2, 2, true);
        add_density(&mut grid, 2, 2, 10.0);
        assert!((get_cell(&grid, 2, 2).density).abs() < 1e-6);
    }

    #[test]
    fn test_step_no_nan() {
        let mut grid = make_grid(10, 10);
        add_density(&mut grid, 5, 5, 1.0);
        add_velocity(&mut grid, 5, 5, 0.1, 0.1);
        step_fluid(&mut grid, 0.016);
        for cell in &grid.cells {
            assert!(!cell.density.is_nan());
        }
    }

    #[test]
    fn test_diffuse_spreads_density() {
        let mut grid = make_grid(7, 7);
        add_density(&mut grid, 3, 3, 100.0);
        diffuse_density(&mut grid, 0.1);
        // Neighbors should now have some density
        let n = get_cell(&grid, 3, 2).density;
        assert!(n > 0.0);
    }

    #[test]
    fn test_fluid_grid_stats() {
        let mut grid = make_grid(5, 5);
        add_density(&mut grid, 2, 2, 3.0);
        add_velocity(&mut grid, 2, 2, 1.0, 0.0);
        let (td, mv, _avg_p) = fluid_grid_stats(&grid);
        assert!((td - 3.0).abs() < 1e-5);
        assert!((mv - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_default_fluid_config() {
        let cfg = default_fluid_config();
        assert!(cfg.viscosity > 0.0);
        assert!(cfg.diffusion > 0.0);
        assert!(cfg.cell_size > 0.0);
    }

    #[test]
    fn test_get_cell_mut() {
        let mut grid = make_grid(5, 5);
        get_cell_mut(&mut grid, 1, 1).density = 42.0;
        assert!((get_cell(&grid, 1, 1).density - 42.0).abs() < 1e-6);
    }
}
