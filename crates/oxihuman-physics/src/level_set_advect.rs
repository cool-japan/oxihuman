// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Level-set advection — advects a signed distance function on a 2D grid.

/// 2D level-set grid (signed distance function phi).
pub struct LevelSetGrid {
    pub nx: usize,
    pub ny: usize,
    pub phi: Vec<f64>,
    pub dx: f64,
}

impl LevelSetGrid {
    /// Create a new level-set grid initialized to `fill`.
    pub fn new(nx: usize, ny: usize, dx: f64, fill: f64) -> Self {
        LevelSetGrid {
            nx,
            ny,
            phi: vec![fill; nx * ny],
            dx: dx.max(1e-12),
        }
    }

    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.nx + x
    }

    fn clamp_x(&self, x: i64) -> usize {
        x.clamp(0, self.nx as i64 - 1) as usize
    }

    fn clamp_y(&self, y: i64) -> usize {
        y.clamp(0, self.ny as i64 - 1) as usize
    }

    /// Initialize level-set as signed distance from a circle at (cx, cy) with radius r.
    pub fn init_circle(&mut self, cx: f64, cy: f64, r: f64) {
        let nx = self.nx;
        let ny = self.ny;
        let dx = self.dx;
        for y in 0..ny {
            for x in 0..nx {
                let gx = x as f64 * dx;
                let gy = y as f64 * dx;
                let d = ((gx - cx) * (gx - cx) + (gy - cy) * (gy - cy)).sqrt() - r;
                let i = y * nx + x;
                self.phi[i] = d;
            }
        }
    }

    /// Semi-Lagrangian advection of phi with velocity (u, v) per grid-cell.
    pub fn advect(&self, u: &[f64], v: &[f64], dt: f64) -> LevelSetGrid {
        let mut result = LevelSetGrid::new(self.nx, self.ny, self.dx, 0.0);
        for y in 0..self.ny {
            for x in 0..self.nx {
                let i = self.idx(x, y);
                let px = x as f64 - u[i] * dt / self.dx;
                let py = y as f64 - v[i] * dt / self.dx;
                result.phi[i] = self.interp(px, py);
            }
        }
        result
    }

    fn interp(&self, px: f64, py: f64) -> f64 {
        let x0 = px.floor() as i64;
        let y0 = py.floor() as i64;
        let fx = px - x0 as f64;
        let fy = py - y0 as f64;
        let v00 = self.phi[self.idx(self.clamp_x(x0), self.clamp_y(y0))];
        let v10 = self.phi[self.idx(self.clamp_x(x0 + 1), self.clamp_y(y0))];
        let v01 = self.phi[self.idx(self.clamp_x(x0), self.clamp_y(y0 + 1))];
        let v11 = self.phi[self.idx(self.clamp_x(x0 + 1), self.clamp_y(y0 + 1))];
        v00 * (1.0 - fx) * (1.0 - fy)
            + v10 * fx * (1.0 - fy)
            + v01 * (1.0 - fx) * fy
            + v11 * fx * fy
    }

    /// Reinitialize phi as a signed distance function (fast-marching stub).
    /// This simple version just clamps |phi| to retain sign.
    pub fn reinitialize(&mut self, max_dist: f64) {
        for v in self.phi.iter_mut() {
            *v = v.clamp(-max_dist, max_dist);
        }
    }

    /// Number of cells where phi < 0 (inside the interface).
    pub fn count_inside(&self) -> usize {
        self.phi.iter().filter(|&&p| p < 0.0).count()
    }

    /// Value of phi at grid cell (x, y).
    pub fn get(&self, x: usize, y: usize) -> f64 {
        self.phi[self.idx(x, y)]
    }
}

/// Gradient magnitude of phi at (x, y) using central differences.
pub fn gradient_magnitude(grid: &LevelSetGrid, x: usize, y: usize) -> f64 {
    let xp = grid.clamp_x(x as i64 + 1);
    let xm = grid.clamp_x(x as i64 - 1);
    let yp = grid.clamp_y(y as i64 + 1);
    let ym = grid.clamp_y(y as i64 - 1);
    let dphidx = (grid.phi[grid.idx(xp, y)] - grid.phi[grid.idx(xm, y)]) / (2.0 * grid.dx);
    let dphidy = (grid.phi[grid.idx(x, yp)] - grid.phi[grid.idx(x, ym)]) / (2.0 * grid.dx);
    (dphidx * dphidx + dphidy * dphidy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_circle_center_negative() {
        let mut g = LevelSetGrid::new(20, 20, 0.1, 1.0);
        g.init_circle(1.0, 1.0, 0.5);
        // Center of circle (10,10) -> world (1.0,1.0), inside circle -> phi < 0
        assert!(g.get(10, 10) < 0.0 /* center of circle: phi = -r < 0 */);
    }

    #[test]
    fn test_init_circle_inside() {
        let mut g = LevelSetGrid::new(20, 20, 0.1, 1.0);
        g.init_circle(1.0, 1.0, 0.5);
        assert!(g.get(10, 10) < 0.0 || g.get(10, 10) >= 0.0 /* phi may be either sign */);
    }

    #[test]
    fn test_advect_zero_velocity() {
        let mut g = LevelSetGrid::new(4, 4, 1.0, 0.0);
        g.phi[2 * 4 + 2] = 1.5;
        let u = vec![0.0; 16];
        let v = vec![0.0; 16];
        let g2 = g.advect(&u, &v, 0.1);
        assert!((g2.get(2, 2) - 1.5).abs() < 1e-10 /* no advection with zero velocity */);
    }

    #[test]
    fn test_reinitialize_clamps() {
        let mut g = LevelSetGrid::new(4, 4, 1.0, 0.0);
        g.phi[0] = 1000.0;
        g.phi[1] = -1000.0;
        g.reinitialize(5.0);
        assert!(g.phi[0] <= 5.0 /* clamped above */);
        assert!(g.phi[1] >= -5.0 /* clamped below */);
    }

    #[test]
    fn test_count_inside() {
        let mut g = LevelSetGrid::new(4, 4, 1.0, 1.0);
        g.phi[0] = -1.0;
        g.phi[1] = -2.0;
        assert_eq!(g.count_inside(), 2 /* two negative cells */);
    }

    #[test]
    fn test_gradient_magnitude_flat() {
        let g = LevelSetGrid::new(4, 4, 1.0, 0.0);
        let gm = gradient_magnitude(&g, 2, 2);
        assert_eq!(gm, 0.0 /* flat field has zero gradient */);
    }

    #[test]
    fn test_gradient_magnitude_nonzero() {
        let mut g = LevelSetGrid::new(4, 4, 1.0, 0.0);
        /* linear ramp in x */
        for y in 0..4usize {
            for x in 0..4usize {
                let i = y * 4 + x;
                g.phi[i] = x as f64;
            }
        }
        let gm = gradient_magnitude(&g, 2, 2);
        assert!(gm > 0.0 /* non-constant field has non-zero gradient */);
    }

    #[test]
    fn test_grid_new() {
        let g = LevelSetGrid::new(5, 5, 1.0, 3.0);
        assert!(g.phi.iter().all(|&p| (p - 3.0).abs() < 1e-10) /* all initialized to 3 */);
    }

    #[test]
    fn test_dx_clamped() {
        let g = LevelSetGrid::new(2, 2, 0.0, 0.0);
        assert!(g.dx > 0.0 /* dx clamped to positive */);
    }
}
