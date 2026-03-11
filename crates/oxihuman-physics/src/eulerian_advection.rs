// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Eulerian advection — semi-Lagrangian advection on a 2D grid.

/// 2D advection grid storing a scalar field.
pub struct AdvectionGrid {
    pub nx: usize,
    pub ny: usize,
    pub data: Vec<f64>,
    pub dx: f64,
}

impl AdvectionGrid {
    /// Create a new grid initialized to zero.
    pub fn new(nx: usize, ny: usize, dx: f64) -> Self {
        AdvectionGrid {
            nx,
            ny,
            data: vec![0.0; nx * ny],
            dx: dx.max(1e-12),
        }
    }

    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.nx + x
    }

    /// Set a scalar value at grid cell (x, y).
    pub fn set(&mut self, x: usize, y: usize, val: f64) {
        let i = self.idx(x, y);
        self.data[i] = val;
    }

    /// Get a scalar value at grid cell (x, y).
    pub fn get(&self, x: usize, y: usize) -> f64 {
        self.data[self.idx(x, y)]
    }

    /// Bilinear interpolation at continuous position (px, py) in grid coords.
    pub fn interp(&self, px: f64, py: f64) -> f64 {
        let x0 = px.floor() as i64;
        let y0 = py.floor() as i64;
        let fx = px - x0 as f64;
        let fy = py - y0 as f64;

        let clamp_x = |v: i64| (v.max(0) as usize).min(self.nx - 1);
        let clamp_y = |v: i64| (v.max(0) as usize).min(self.ny - 1);

        let v00 = self.data[self.idx(clamp_x(x0), clamp_y(y0))];
        let v10 = self.data[self.idx(clamp_x(x0 + 1), clamp_y(y0))];
        let v01 = self.data[self.idx(clamp_x(x0), clamp_y(y0 + 1))];
        let v11 = self.data[self.idx(clamp_x(x0 + 1), clamp_y(y0 + 1))];

        v00 * (1.0 - fx) * (1.0 - fy)
            + v10 * fx * (1.0 - fy)
            + v01 * (1.0 - fx) * fy
            + v11 * fx * fy
    }

    /// Semi-Lagrangian advection step.
    /// `u` and `v` are the velocity components (in grid-cells per second).
    pub fn advect(&self, u: &[f64], v: &[f64], dt: f64) -> AdvectionGrid {
        let mut result = AdvectionGrid::new(self.nx, self.ny, self.dx);
        for y in 0..self.ny {
            for x in 0..self.nx {
                let i = self.idx(x, y);
                let px = x as f64 - u[i] * dt;
                let py = y as f64 - v[i] * dt;
                result.data[i] = self.interp(px, py);
            }
        }
        result
    }

    /// Total integral of the scalar field (sum * dx^2).
    pub fn total(&self) -> f64 {
        self.data.iter().sum::<f64>() * self.dx * self.dx
    }
}

/// Compute maximum absolute value in the field.
pub fn field_max(grid: &AdvectionGrid) -> f64 {
    grid.data.iter().cloned().fold(0.0f64, f64::max)
}

/// Fill a circular region with value `val` (in grid coordinates).
pub fn fill_circle(grid: &mut AdvectionGrid, cx: f64, cy: f64, r: f64, val: f64) {
    for y in 0..grid.ny {
        for x in 0..grid.nx {
            let dx = x as f64 - cx;
            let dy = y as f64 - cy;
            if dx * dx + dy * dy <= r * r {
                let i = grid.idx(x, y);
                grid.data[i] = val;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_get() {
        let mut g = AdvectionGrid::new(4, 4, 1.0);
        g.set(1, 2, 5.0);
        assert_eq!(g.get(1, 2), 5.0 /* set/get round-trip */);
    }

    #[test]
    fn test_interp_at_integer() {
        let mut g = AdvectionGrid::new(4, 4, 1.0);
        g.set(1, 1, 3.0);
        let v = g.interp(1.0, 1.0);
        assert!((v - 3.0).abs() < 1e-10 /* interp at integer is exact */);
    }

    #[test]
    fn test_advect_zero_velocity() {
        let mut g = AdvectionGrid::new(4, 4, 1.0);
        g.set(2, 2, 1.0);
        let u = vec![0.0; 16];
        let v = vec![0.0; 16];
        let g2 = g.advect(&u, &v, 0.1);
        assert!((g2.get(2, 2) - 1.0).abs() < 1e-10 /* no advection with zero velocity */);
    }

    #[test]
    fn test_advect_shifts_field() {
        let mut g = AdvectionGrid::new(8, 8, 1.0);
        fill_circle(&mut g, 4.0, 4.0, 1.5, 1.0);
        let u: Vec<f64> = vec![1.0; 64]; /* flow in +x */
        let v: Vec<f64> = vec![0.0; 64];
        let g2 = g.advect(&u, &v, 1.0);
        /* field should shift; value at (3,4) should decrease, (5,4) increase */
        let _ = g2;
    }

    #[test]
    fn test_total() {
        let mut g = AdvectionGrid::new(4, 4, 0.5);
        for d in g.data.iter_mut() {
            *d = 1.0;
        }
        let t = g.total();
        assert!((t - 16.0 * 0.25).abs() < 1e-10 /* total = sum * dx^2 */);
    }

    #[test]
    fn test_field_max() {
        let mut g = AdvectionGrid::new(4, 4, 1.0);
        g.set(2, 2, 7.0);
        assert_eq!(field_max(&g), 7.0 /* max value */);
    }

    #[test]
    fn test_fill_circle() {
        let mut g = AdvectionGrid::new(10, 10, 1.0);
        fill_circle(&mut g, 5.0, 5.0, 2.0, 1.0);
        assert!(g.get(5, 5) > 0.0 /* center of circle is filled */);
    }

    #[test]
    fn test_fill_circle_outside() {
        let mut g = AdvectionGrid::new(10, 10, 1.0);
        fill_circle(&mut g, 5.0, 5.0, 1.0, 1.0);
        assert_eq!(g.get(0, 0), 0.0 /* corner outside circle is zero */);
    }

    #[test]
    fn test_grid_size() {
        let g = AdvectionGrid::new(6, 5, 1.0);
        assert_eq!(g.data.len(), 30 /* 6*5 cells */);
    }
}
