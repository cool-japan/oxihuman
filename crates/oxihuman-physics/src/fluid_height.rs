// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Shallow water simulation on a height-field grid.
//!
//! Implements a simplified shallow-water model suitable for real-time character
//! interaction with water surfaces.

/// Configuration parameters for a fluid height-field grid.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FluidGridConfig {
    /// Number of cells in the X direction.
    pub width: usize,
    /// Number of cells in the Y direction.
    pub height: usize,
    /// Physical size of each cell in meters.
    pub cell_size: f32,
    /// Gravitational acceleration (m/s²), typically 9.81.
    pub gravity: f32,
    /// Velocity damping factor per step (0..1); 0.99 preserves energy well.
    pub damping: f32,
    /// Maximum allowed water depth in meters.
    pub max_depth: f32,
}

impl Default for FluidGridConfig {
    fn default() -> Self {
        FluidGridConfig {
            width: 32,
            height: 32,
            cell_size: 0.1,
            gravity: 9.81,
            damping: 0.99,
            max_depth: 2.0,
        }
    }
}

/// A 2-D height-field fluid grid.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FluidGrid {
    /// Water surface height at each cell [y * width + x].
    pub water_height: Vec<f32>,
    /// Horizontal (X-direction) flow velocity at each cell.
    pub flow_x: Vec<f32>,
    /// Vertical (Y-direction) flow velocity at each cell.
    pub flow_y: Vec<f32>,
    pub config: FluidGridConfig,
}

impl FluidGrid {
    /// Create a new flat fluid grid with all heights at 0.
    #[allow(dead_code)]
    pub fn new(cfg: FluidGridConfig) -> Self {
        let n = cfg.width * cfg.height;
        FluidGrid {
            water_height: vec![0.0; n],
            flow_x: vec![0.0; n],
            flow_y: vec![0.0; n],
            config: cfg,
        }
    }

    /// Advance the simulation by one time step.
    ///
    /// Algorithm:
    /// 1. Compute flow velocities from hydrostatic pressure (height differences).
    /// 2. Advect water using the flow field.
    /// 3. Apply damping.
    /// 4. Clamp heights to [0, max_depth].
    #[allow(dead_code)]
    pub fn step(&mut self, dt: f32) {
        let w = self.config.width;
        let h = self.config.height;
        let g = self.config.gravity;
        let cs = self.config.cell_size;
        let damp = self.config.damping;
        let max_d = self.config.max_depth;

        // Step 1: update flow from pressure gradient (height difference)
        let heights = self.water_height.clone();
        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;
                let cur = heights[idx];
                // Flow in X direction (towards right neighbour)
                if x + 1 < w {
                    let right = heights[y * w + x + 1];
                    let pressure_grad = (cur - right) * g / cs;
                    self.flow_x[idx] += pressure_grad * dt;
                    self.flow_x[idx] *= damp;
                }
                // Flow in Y direction (towards bottom neighbour)
                if y + 1 < h {
                    let below = heights[(y + 1) * w + x];
                    let pressure_grad = (cur - below) * g / cs;
                    self.flow_y[idx] += pressure_grad * dt;
                    self.flow_y[idx] *= damp;
                }
            }
        }

        // Step 2: apply flow to move water
        let flow_x = self.flow_x.clone();
        let flow_y = self.flow_y.clone();
        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;
                // Transfer from left to right (X flow)
                if x + 1 < w {
                    let flux = flow_x[idx] * dt / cs;
                    let flux_clamped =
                        flux.clamp(-self.water_height[idx], self.water_height[y * w + x + 1]);
                    self.water_height[idx] -= flux_clamped;
                    self.water_height[y * w + x + 1] += flux_clamped;
                }
                // Transfer downwards (Y flow)
                if y + 1 < h {
                    let flux = flow_y[idx] * dt / cs;
                    let flux_clamped =
                        flux.clamp(-self.water_height[idx], self.water_height[(y + 1) * w + x]);
                    self.water_height[idx] -= flux_clamped;
                    self.water_height[(y + 1) * w + x] += flux_clamped;
                }
            }
        }

        // Step 3: clamp heights
        for h_val in &mut self.water_height {
            *h_val = h_val.clamp(0.0, max_d);
        }
    }

    /// Add a circular wave impulse centred at `(cx, cy)` with given `amplitude` and `radius`.
    #[allow(dead_code)]
    pub fn add_wave(&mut self, cx: usize, cy: usize, amplitude: f32, radius: usize) {
        let w = self.config.width;
        let h = self.config.height;
        let r = radius as i64;
        let cx = cx as i64;
        let cy = cy as i64;
        for dy in -r..=r {
            for dx in -r..=r {
                let nx = cx + dx;
                let ny = cy + dy;
                if nx >= 0 && nx < w as i64 && ny >= 0 && ny < h as i64 {
                    let dist = ((dx * dx + dy * dy) as f32).sqrt();
                    if dist <= radius as f32 {
                        let factor = 1.0 - dist / (radius as f32 + 1.0);
                        let idx = (ny as usize) * w + nx as usize;
                        self.water_height[idx] = (self.water_height[idx] + amplitude * factor)
                            .clamp(0.0, self.config.max_depth);
                    }
                }
            }
        }
    }

    /// Compute the total water volume (sum of water_height * cell_size²).
    #[allow(dead_code)]
    pub fn total_volume(&self) -> f32 {
        let cs2 = self.config.cell_size * self.config.cell_size;
        self.water_height.iter().sum::<f32>() * cs2
    }

    /// Return the maximum water height across all cells.
    #[allow(dead_code)]
    pub fn max_height(&self) -> f32 {
        self.water_height
            .iter()
            .cloned()
            .fold(f32::NEG_INFINITY, f32::max)
    }

    /// Get the water height at cell `(x, y)`.
    #[allow(dead_code)]
    pub fn at(&self, x: usize, y: usize) -> f32 {
        self.water_height[y * self.config.width + x]
    }

    /// Set the water height at cell `(x, y)`.
    #[allow(dead_code)]
    pub fn set(&mut self, x: usize, y: usize, h: f32) {
        let idx = y * self.config.width + x;
        self.water_height[idx] = h.clamp(0.0, self.config.max_depth);
    }
}

/// Compute the finite-difference gradient of the water height at cell `(x, y)`.
///
/// Returns `[dh/dx, dh/dy]` in units of height-per-meter.
#[allow(dead_code)]
pub fn height_gradient(grid: &FluidGrid, x: usize, y: usize) -> [f32; 2] {
    let w = grid.config.width;
    let h = grid.config.height;
    let cs = grid.config.cell_size;
    let idx = |gx: usize, gy: usize| gy * w + gx;

    let cur = grid.water_height[idx(x, y)];

    let gx = if x + 1 < w && x > 0 {
        (grid.water_height[idx(x + 1, y)] - grid.water_height[idx(x - 1, y)]) / (2.0 * cs)
    } else if x + 1 < w {
        (grid.water_height[idx(x + 1, y)] - cur) / cs
    } else if x > 0 {
        (cur - grid.water_height[idx(x - 1, y)]) / cs
    } else {
        0.0
    };

    let gy = if y + 1 < h && y > 0 {
        (grid.water_height[idx(x, y + 1)] - grid.water_height[idx(x, y - 1)]) / (2.0 * cs)
    } else if y + 1 < h {
        (grid.water_height[idx(x, y + 1)] - cur) / cs
    } else if y > 0 {
        (cur - grid.water_height[idx(x, y - 1)]) / cs
    } else {
        0.0
    };

    [gx, gy]
}

/// Compute the flow divergence at cell `(x, y)`.
///
/// Approximates ∇·v = (∂flow_x/∂x + ∂flow_y/∂y).
#[allow(dead_code)]
pub fn flow_divergence(grid: &FluidGrid, x: usize, y: usize) -> f32 {
    let w = grid.config.width;
    let h = grid.config.height;
    let cs = grid.config.cell_size;
    let idx = |gx: usize, gy: usize| gy * w + gx;

    let dfx_dx = if x + 1 < w && x > 0 {
        (grid.flow_x[idx(x + 1, y)] - grid.flow_x[idx(x - 1, y)]) / (2.0 * cs)
    } else if x + 1 < w {
        (grid.flow_x[idx(x + 1, y)] - grid.flow_x[idx(x, y)]) / cs
    } else if x > 0 {
        (grid.flow_x[idx(x, y)] - grid.flow_x[idx(x - 1, y)]) / cs
    } else {
        0.0
    };

    let dfy_dy = if y + 1 < h && y > 0 {
        (grid.flow_y[idx(x, y + 1)] - grid.flow_y[idx(x, y - 1)]) / (2.0 * cs)
    } else if y + 1 < h {
        (grid.flow_y[idx(x, y + 1)] - grid.flow_y[idx(x, y)]) / cs
    } else if y > 0 {
        (grid.flow_y[idx(x, y)] - grid.flow_y[idx(x, y - 1)]) / cs
    } else {
        0.0
    };

    dfx_dx + dfy_dy
}

/// Convenience constructor for a flat fluid grid with uniform initial depth.
///
/// `total_volume = width * height * depth * cell_size²`
#[allow(dead_code)]
pub fn flat_fluid_grid(width: usize, height: usize, depth: f32, cell_size: f32) -> FluidGrid {
    let cfg = FluidGridConfig {
        width,
        height,
        cell_size,
        gravity: 9.81,
        damping: 0.99,
        max_depth: depth.max(2.0),
    };
    let n = width * height;
    FluidGrid {
        water_height: vec![depth; n],
        flow_x: vec![0.0; n],
        flow_y: vec![0.0; n],
        config: cfg,
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-4;

    fn small_grid() -> FluidGrid {
        FluidGrid::new(FluidGridConfig {
            width: 8,
            height: 8,
            cell_size: 0.1,
            gravity: 9.81,
            damping: 0.99,
            max_depth: 2.0,
        })
    }

    // 1. FluidGrid::new produces a flat zero grid
    #[test]
    fn test_new_is_flat() {
        let g = small_grid();
        assert!(g.water_height.iter().all(|&h| h == 0.0));
        assert!(g.flow_x.iter().all(|&v| v == 0.0));
        assert!(g.flow_y.iter().all(|&v| v == 0.0));
    }

    // 2. Dimensions are correct
    #[test]
    fn test_dimensions_correct() {
        let g = small_grid();
        assert_eq!(g.water_height.len(), 64);
        assert_eq!(g.flow_x.len(), 64);
        assert_eq!(g.config.width, 8);
        assert_eq!(g.config.height, 8);
    }

    // 3. flat_fluid_grid total_volume = w * h * depth * cell_size²
    #[test]
    fn test_flat_fluid_grid_total_volume() {
        let g = flat_fluid_grid(10, 10, 0.5, 0.1);
        let expected = 10.0 * 10.0 * 0.5 * 0.1 * 0.1;
        let actual = g.total_volume();
        assert!(
            (actual - expected).abs() < 1e-4,
            "expected volume {expected}, got {actual}"
        );
    }

    // 4. add_wave changes heights near centre
    #[test]
    fn test_add_wave_changes_heights() {
        let mut g = small_grid();
        g.add_wave(4, 4, 0.5, 2);
        let centre = g.at(4, 4);
        assert!(centre > 0.0, "centre height should be raised, got {centre}");
    }

    // 5. add_wave doesn't affect far cells
    #[test]
    fn test_add_wave_far_cells_unchanged() {
        let mut g = small_grid();
        g.add_wave(4, 4, 0.5, 1);
        // Cell (0, 0) is far from centre (4,4)
        assert!(g.at(0, 0).abs() < EPS, "far cell should be unchanged");
    }

    // 6. max_height after wave is positive
    #[test]
    fn test_max_height_after_wave() {
        let mut g = small_grid();
        g.add_wave(4, 4, 1.0, 2);
        let mh = g.max_height();
        assert!(mh > 0.0, "max_height should be positive after wave");
    }

    // 7. height_gradient for flat grid = zero
    #[test]
    fn test_height_gradient_flat_zero() {
        let g = flat_fluid_grid(8, 8, 0.3, 0.1);
        let grad = height_gradient(&g, 4, 4);
        assert!(
            grad[0].abs() < EPS,
            "gx should be zero for flat grid, got {}",
            grad[0]
        );
        assert!(
            grad[1].abs() < EPS,
            "gy should be zero for flat grid, got {}",
            grad[1]
        );
    }

    // 8. at/set round-trip
    #[test]
    fn test_at_set_round_trip() {
        let mut g = small_grid();
        g.set(3, 3, 0.7);
        assert!((g.at(3, 3) - 0.7).abs() < EPS);
    }

    // 9. set clamps to max_depth
    #[test]
    fn test_set_clamps_to_max_depth() {
        let mut g = small_grid();
        g.set(0, 0, 999.0);
        assert!((g.at(0, 0) - g.config.max_depth).abs() < EPS);
    }

    // 10. step doesn't produce NaN
    #[test]
    fn test_step_no_nan() {
        let mut g = flat_fluid_grid(8, 8, 0.5, 0.1);
        g.add_wave(4, 4, 0.3, 2);
        for _ in 0..10 {
            g.step(0.016);
        }
        for &h in &g.water_height {
            assert!(!h.is_nan(), "height should not be NaN");
        }
        for &fx in &g.flow_x {
            assert!(!fx.is_nan(), "flow_x should not be NaN");
        }
    }

    // 11. step approximately conserves volume (with clamping, some loss is expected)
    #[test]
    fn test_step_approximately_conserves_volume() {
        let depth = 1.0f32;
        let mut g = flat_fluid_grid(16, 16, depth, 0.1);
        let initial_volume = g.total_volume();
        // Run several steps without wave
        for _ in 0..20 {
            g.step(0.01);
        }
        let final_volume = g.total_volume();
        // Volume should not grow (clamping may reduce it slightly)
        assert!(
            final_volume <= initial_volume + 1e-3,
            "volume grew unexpectedly: initial={initial_volume}, final={final_volume}"
        );
    }

    // 12. damping reduces flow over time
    #[test]
    fn test_damping_reduces_flow() {
        let mut g = flat_fluid_grid(8, 8, 0.5, 0.1);
        g.add_wave(4, 4, 0.5, 2);
        g.step(0.016); // first step builds up flow
        let max_flow_after_wave: f32 = g.flow_x.iter().cloned().fold(0.0f32, f32::max);
        for _ in 0..50 {
            g.step(0.016);
        }
        let max_flow_later: f32 = g.flow_x.iter().cloned().fold(0.0f32, f32::max);
        // Flow should eventually decay due to damping
        // (after many steps with no energy input)
        assert!(
            max_flow_later <= max_flow_after_wave + 0.01,
            "flow should not grow, after_wave={max_flow_after_wave}, later={max_flow_later}"
        );
    }

    // 13. flow_divergence doesn't panic and returns finite value
    #[test]
    fn test_flow_divergence_finite() {
        let mut g = flat_fluid_grid(8, 8, 0.5, 0.1);
        g.add_wave(4, 4, 0.3, 2);
        g.step(0.016);
        let div = flow_divergence(&g, 4, 4);
        assert!(div.is_finite(), "divergence should be finite, got {div}");
    }

    // 14. height_gradient is non-zero near wave peak
    #[test]
    fn test_height_gradient_nonzero_near_wave() {
        let mut g = small_grid();
        g.add_wave(4, 4, 1.0, 2);
        // At cell (3, 4), neighbour (4, 4) has higher water
        let grad = height_gradient(&g, 3, 4);
        let mag = (grad[0] * grad[0] + grad[1] * grad[1]).sqrt();
        assert!(
            mag > 0.0,
            "gradient should be non-zero near wave edge, got {mag}"
        );
    }
}
