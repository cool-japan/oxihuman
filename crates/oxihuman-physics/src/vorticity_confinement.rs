// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Vorticity confinement — restores fine-scale vortical details in grid-based fluids.

/// 2D velocity grid for vorticity confinement.
pub struct VorticityGrid {
    pub nx: usize,
    pub ny: usize,
    pub u: Vec<f64>, /* x-velocity */
    pub v: Vec<f64>, /* y-velocity */
    pub dx: f64,
}

impl VorticityGrid {
    /// Create a new velocity grid.
    pub fn new(nx: usize, ny: usize, dx: f64) -> Self {
        let n = nx * ny;
        VorticityGrid {
            nx,
            ny,
            u: vec![0.0; n],
            v: vec![0.0; n],
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

    /// Compute vorticity (curl z) at each cell: omega = dv/dx - du/dy.
    pub fn compute_vorticity(&self) -> Vec<f64> {
        let mut omega = vec![0.0f64; self.nx * self.ny];
        let h2 = 2.0 * self.dx;
        for y in 0..self.ny {
            for x in 0..self.nx {
                let xp = self.clamp_x(x as i64 + 1);
                let xm = self.clamp_x(x as i64 - 1);
                let yp = self.clamp_y(y as i64 + 1);
                let ym = self.clamp_y(y as i64 - 1);
                let dv_dx = (self.v[self.idx(xp, y)] - self.v[self.idx(xm, y)]) / h2;
                let du_dy = (self.u[self.idx(x, yp)] - self.u[self.idx(x, ym)]) / h2;
                omega[self.idx(x, y)] = dv_dx - du_dy;
            }
        }
        omega
    }

    /// Compute vorticity confinement force and apply it.
    pub fn apply_vorticity_confinement(&mut self, epsilon: f64, dt: f64) {
        let omega = self.compute_vorticity();
        let mut eta_x = vec![0.0f64; self.nx * self.ny];
        let mut eta_y = vec![0.0f64; self.nx * self.ny];
        let h2 = 2.0 * self.dx;
        /* compute gradient of |omega| */
        for y in 0..self.ny {
            for x in 0..self.nx {
                let xp = self.clamp_x(x as i64 + 1);
                let xm = self.clamp_x(x as i64 - 1);
                let yp = self.clamp_y(y as i64 + 1);
                let ym = self.clamp_y(y as i64 - 1);
                eta_x[self.idx(x, y)] =
                    (omega[self.idx(xp, y)].abs() - omega[self.idx(xm, y)].abs()) / h2;
                eta_y[self.idx(x, y)] =
                    (omega[self.idx(x, yp)].abs() - omega[self.idx(x, ym)].abs()) / h2;
            }
        }
        /* normalize eta and compute confinement force */
        for y in 0..self.ny {
            for x in 0..self.nx {
                let i = self.idx(x, y);
                let len = (eta_x[i] * eta_x[i] + eta_y[i] * eta_y[i]).sqrt();
                if len > 1e-12 {
                    let nx = eta_x[i] / len;
                    let ny = eta_y[i] / len;
                    let w = omega[i];
                    /* confinement force = epsilon * (N x omega) */
                    self.u[i] += epsilon * (ny * w) * dt;
                    self.v[i] += epsilon * (-nx * w) * dt;
                }
            }
        }
    }

    /// Total kinetic energy of the grid.
    pub fn kinetic_energy(&self) -> f64 {
        let sum_sq: f64 = self
            .u
            .iter()
            .zip(self.v.iter())
            .map(|(u, v)| u * u + v * v)
            .sum();
        0.5 * sum_sq * self.dx * self.dx
    }
}

/// Maximum absolute vorticity.
pub fn max_vorticity(omega: &[f64]) -> f64 {
    omega.iter().cloned().fold(0.0f64, |a, b| a.max(b.abs()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vorticity_zero_field() {
        let g = VorticityGrid::new(4, 4, 1.0);
        let omega = g.compute_vorticity();
        assert!(omega.iter().all(|&w| w == 0.0) /* zero velocity => zero vorticity */);
    }

    #[test]
    fn test_vorticity_nonzero() {
        let mut g = VorticityGrid::new(4, 4, 1.0);
        /* set a vortex: u increases with y, v increases with x */
        for y in 0..4usize {
            for x in 0..4usize {
                let i = y * 4 + x;
                g.u[i] = y as f64;
                g.v[i] = x as f64;
            }
        }
        let omega = g.compute_vorticity();
        assert!(omega.iter().any(|&w| w != 0.0) /* non-zero vorticity */);
    }

    #[test]
    fn test_apply_confinement_changes_velocity() {
        let mut g = VorticityGrid::new(4, 4, 1.0);
        g.u[2 * 4 + 2] = 1.0;
        let before_u = g.u.clone();
        g.apply_vorticity_confinement(0.1, 0.01);
        assert!(g.u != before_u || g.v.iter().any(|&v| v != 0.0) /* velocity changed */);
    }

    #[test]
    fn test_kinetic_energy_zero() {
        let g = VorticityGrid::new(4, 4, 1.0);
        assert_eq!(g.kinetic_energy(), 0.0 /* zero velocity => zero KE */);
    }

    #[test]
    fn test_kinetic_energy_nonzero() {
        let mut g = VorticityGrid::new(2, 2, 1.0);
        for u in g.u.iter_mut() {
            *u = 1.0;
        }
        assert!(g.kinetic_energy() > 0.0 /* non-zero velocity => non-zero KE */);
    }

    #[test]
    fn test_max_vorticity() {
        let omega = vec![0.5, -1.0, 0.3];
        assert!((max_vorticity(&omega) - 1.0).abs() < 1e-10 /* max abs vorticity */);
    }

    #[test]
    fn test_max_vorticity_empty() {
        let omega: Vec<f64> = vec![];
        assert_eq!(max_vorticity(&omega), 0.0 /* empty => 0 */);
    }

    #[test]
    fn test_grid_size() {
        let g = VorticityGrid::new(5, 3, 1.0);
        assert_eq!(g.u.len(), 15 /* 5*3 cells */);
    }

    #[test]
    fn test_dx_clamped() {
        let g = VorticityGrid::new(2, 2, 0.0);
        assert!(g.dx > 0.0 /* dx clamped to positive */);
    }
}
