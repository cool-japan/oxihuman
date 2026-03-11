// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Material point method (MPM) 2D grid transfer.

/// A 2D material point.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MpmPoint {
    pub pos: [f32; 2],
    pub vel: [f32; 2],
    pub mass: f32,
}

impl MpmPoint {
    #[allow(dead_code)]
    pub fn new(pos: [f32; 2], vel: [f32; 2], mass: f32) -> Self {
        Self { pos, vel, mass }
    }
}

/// A 2D MPM grid cell.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct MpmCell {
    pub mass: f32,
    pub momentum: [f32; 2],
}

/// MPM configuration.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MpmConfig {
    pub cell_size: f32,
    pub grid_w: usize,
    pub grid_h: usize,
    pub gravity: [f32; 2],
}

impl Default for MpmConfig {
    fn default() -> Self {
        Self {
            cell_size: 1.0,
            grid_w: 16,
            grid_h: 16,
            gravity: [0.0, -9.8],
        }
    }
}

/// A 2D MPM grid.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MpmGrid {
    pub cells: Vec<MpmCell>,
    pub config: MpmConfig,
}

impl MpmGrid {
    /// Create a new MPM grid.
    #[allow(dead_code)]
    pub fn new(config: MpmConfig) -> Self {
        let n = config.grid_w * config.grid_h;
        Self {
            cells: vec![MpmCell::default(); n],
            config,
        }
    }

    fn cell_idx(&self, xi: usize, yi: usize) -> usize {
        yi * self.config.grid_w + xi
    }

    /// Reset all grid cells to zero.
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        for c in &mut self.cells {
            c.mass = 0.0;
            c.momentum = [0.0; 2];
        }
    }

    /// Particle-to-grid (P2G) transfer using bilinear interpolation.
    #[allow(dead_code)]
    pub fn p2g(&mut self, points: &[MpmPoint]) {
        let dx = self.config.cell_size;
        for p in points {
            let gx = (p.pos[0] / dx).floor() as isize;
            let gy = (p.pos[1] / dx).floor() as isize;
            let fx = p.pos[0] / dx - gx as f32;
            let fy = p.pos[1] / dx - gy as f32;
            let wx = [1.0 - fx, fx];
            let wy = [1.0 - fy, fy];
            for dy in 0..2isize {
                for dx2 in 0..2isize {
                    let xi = gx + dx2;
                    let yi = gy + dy;
                    if xi < 0
                        || yi < 0
                        || xi as usize >= self.config.grid_w
                        || yi as usize >= self.config.grid_h
                    {
                        continue;
                    }
                    let w = wx[dx2 as usize] * wy[dy as usize];
                    let idx = self.cell_idx(xi as usize, yi as usize);
                    self.cells[idx].mass += w * p.mass;
                    for k in 0..2 {
                        self.cells[idx].momentum[k] += w * p.mass * p.vel[k];
                    }
                }
            }
        }
    }

    /// Apply gravity to grid momentum.
    #[allow(dead_code)]
    pub fn apply_gravity(&mut self, dt: f32) {
        let g = self.config.gravity;
        for c in &mut self.cells {
            if c.mass < 1e-10 {
                continue;
            }
            for (mom, gk) in c.momentum.iter_mut().zip(g.iter()) {
                *mom += c.mass * gk * dt;
            }
        }
    }

    /// Grid-to-particle (G2P) transfer using bilinear interpolation.
    #[allow(dead_code)]
    pub fn g2p(&self, points: &mut [MpmPoint], dt: f32) {
        let dx = self.config.cell_size;
        for p in points.iter_mut() {
            let gx = (p.pos[0] / dx).floor() as isize;
            let gy = (p.pos[1] / dx).floor() as isize;
            let fx = p.pos[0] / dx - gx as f32;
            let fy = p.pos[1] / dx - gy as f32;
            let wx = [1.0 - fx, fx];
            let wy = [1.0 - fy, fy];
            let mut new_vel = [0.0f32; 2];
            for dy in 0..2isize {
                for dx2 in 0..2isize {
                    let xi = gx + dx2;
                    let yi = gy + dy;
                    if xi < 0
                        || yi < 0
                        || xi as usize >= self.config.grid_w
                        || yi as usize >= self.config.grid_h
                    {
                        continue;
                    }
                    let w = wx[dx2 as usize] * wy[dy as usize];
                    let idx = self.cell_idx(xi as usize, yi as usize);
                    let m = self.cells[idx].mass;
                    if m < 1e-10 {
                        continue;
                    }
                    for (nv, mom) in new_vel.iter_mut().zip(self.cells[idx].momentum.iter()) {
                        *nv += w * mom / m;
                    }
                }
            }
            p.vel = new_vel;
            for (pos_k, vel_k) in p.pos.iter_mut().zip(p.vel.iter()) {
                *pos_k += vel_k * dt;
            }
        }
    }

    /// Total grid mass.
    #[allow(dead_code)]
    pub fn total_mass(&self) -> f32 {
        self.cells.iter().map(|c| c.mass).sum()
    }

    /// Number of cells.
    #[allow(dead_code)]
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_grid() -> MpmGrid {
        MpmGrid::new(MpmConfig {
            cell_size: 1.0,
            grid_w: 8,
            grid_h: 8,
            gravity: [0.0, -9.8],
        })
    }

    fn make_point() -> MpmPoint {
        MpmPoint::new([3.5, 3.5], [1.0, 0.0], 1.0)
    }

    #[test]
    fn grid_cell_count() {
        let g = make_grid();
        assert_eq!(g.cell_count(), 64);
    }

    #[test]
    fn p2g_increases_grid_mass() {
        let mut g = make_grid();
        g.p2g(&[make_point()]);
        assert!(g.total_mass() > 0.0);
    }

    #[test]
    fn reset_clears_mass() {
        let mut g = make_grid();
        g.p2g(&[make_point()]);
        g.reset();
        assert!(g.total_mass() < 1e-8);
    }

    #[test]
    fn p2g_conserves_mass() {
        let mut g = make_grid();
        let p = make_point();
        g.p2g(std::slice::from_ref(&p));
        let total = g.total_mass();
        assert!((total - p.mass).abs() < 1e-5);
    }

    #[test]
    fn apply_gravity_changes_momentum() {
        let mut g = make_grid();
        g.p2g(&[make_point()]);
        let before: f32 = g.cells.iter().map(|c| c.momentum[1]).sum();
        g.apply_gravity(0.01);
        let after: f32 = g.cells.iter().map(|c| c.momentum[1]).sum();
        assert!(after < before);
    }

    #[test]
    fn g2p_moves_particle() {
        let mut g = make_grid();
        let mut ps = vec![make_point()];
        g.p2g(&ps);
        g.apply_gravity(0.01);
        let old_y = ps[0].pos[1];
        g.g2p(&mut ps, 0.01);
        let _ = old_y;
    }

    #[test]
    fn new_mpm_config_has_positive_cell_size() {
        let c = MpmConfig::default();
        assert!(c.cell_size > 0.0);
    }

    #[test]
    fn reset_is_idempotent() {
        let mut g = make_grid();
        g.reset();
        g.reset();
        assert!(g.total_mass() < 1e-8);
    }

    #[test]
    fn point_out_of_bounds_not_transferred() {
        let mut g = make_grid();
        let out = MpmPoint::new([100.0, 100.0], [0.0, 0.0], 1.0);
        g.p2g(&[out]);
        assert!(g.total_mass() < 1e-8);
    }

    #[test]
    fn multiple_points_sum_mass() {
        let mut g = make_grid();
        let ps = vec![make_point(), make_point()];
        g.p2g(&ps);
        assert!((g.total_mass() - 2.0).abs() < 1e-5);
    }
}
