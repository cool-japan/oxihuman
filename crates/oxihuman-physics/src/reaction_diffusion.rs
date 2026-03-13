// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! 2D Gray-Scott reaction diffusion.

/// Gray-Scott reaction-diffusion parameters.
#[derive(Debug, Clone)]
pub struct GrayScottParams {
    /// Diffusion rate of U.
    pub du: f64,
    /// Diffusion rate of V.
    pub dv: f64,
    /// Feed rate.
    pub f: f64,
    /// Kill rate.
    pub k: f64,
}

impl Default for GrayScottParams {
    fn default() -> Self {
        Self {
            du: 0.16,
            dv: 0.08,
            f: 0.035,
            k: 0.065,
        }
    }
}

/// 2D Gray-Scott reaction-diffusion grid.
pub struct ReactionDiffusion {
    pub width: usize,
    pub height: usize,
    pub u: Vec<f64>,
    pub v: Vec<f64>,
    pub params: GrayScottParams,
}

impl ReactionDiffusion {
    pub fn new(width: usize, height: usize, params: GrayScottParams) -> Self {
        let n = width * height;
        let mut u = vec![1.0; n];
        let mut v = vec![0.0; n];
        /* seed center 10x10 patch */
        let cx = width / 2;
        let cy = height / 2;
        for dy in 0..10 {
            for dx in 0..10 {
                let px = cx.saturating_sub(5) + dx;
                let py = cy.saturating_sub(5) + dy;
                if px < width && py < height {
                    let idx = py * width + px;
                    u[idx] = 0.5;
                    v[idx] = 0.25;
                }
            }
        }
        Self {
            width,
            height,
            u,
            v,
            params,
        }
    }

    fn laplacian(&self, grid: &[f64], idx: usize) -> f64 {
        let w = self.width;
        let h = self.height;
        let x = idx % w;
        let y = idx / w;
        let c = grid[idx];
        let l = if x > 0 { grid[idx - 1] } else { grid[idx] };
        let r = if x < w - 1 { grid[idx + 1] } else { grid[idx] };
        let u = if y > 0 { grid[idx - w] } else { grid[idx] };
        let d = if y < h - 1 { grid[idx + w] } else { grid[idx] };
        l + r + u + d - 4.0 * c
    }

    pub fn step(&mut self, dt: f64) {
        let n = self.width * self.height;
        let mut du_field = Vec::with_capacity(n);
        let mut dv_field = Vec::with_capacity(n);
        for i in 0..n {
            let u = self.u[i];
            let v = self.v[i];
            let uvv = u * v * v;
            let lap_u = self.laplacian(&self.u, i);
            let lap_v = self.laplacian(&self.v, i);
            du_field.push(self.params.du * lap_u - uvv + self.params.f * (1.0 - u));
            dv_field.push(self.params.dv * lap_v + uvv - (self.params.f + self.params.k) * v);
        }
        for i in 0..n {
            self.u[i] = (self.u[i] + dt * du_field[i]).clamp(0.0, 1.0);
            self.v[i] = (self.v[i] + dt * dv_field[i]).clamp(0.0, 1.0);
        }
    }

    pub fn mean_u(&self) -> f64 {
        self.u.iter().sum::<f64>() / self.u.len() as f64
    }

    pub fn mean_v(&self) -> f64 {
        self.v.iter().sum::<f64>() / self.v.len() as f64
    }

    pub fn cell_count(&self) -> usize {
        self.width * self.height
    }
}

pub fn new_reaction_diffusion(w: usize, h: usize) -> ReactionDiffusion {
    ReactionDiffusion::new(w, h, GrayScottParams::default())
}

pub fn rd_step(rd: &mut ReactionDiffusion, dt: f64) {
    rd.step(dt);
}

pub fn rd_mean_u(rd: &ReactionDiffusion) -> f64 {
    rd.mean_u()
}

pub fn rd_mean_v(rd: &ReactionDiffusion) -> f64 {
    rd.mean_v()
}

pub fn rd_cell_count(rd: &ReactionDiffusion) -> usize {
    rd.cell_count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_grid() {
        let rd = new_reaction_diffusion(20, 20);
        assert_eq!(rd_cell_count(&rd), 400);
    }

    #[test]
    fn test_initial_mean_u_near_1() {
        /* Use larger grid so the 10x10 seed patch is a smaller fraction */
        let rd = new_reaction_diffusion(50, 50);
        /* 50x50 = 2500 cells, 100 seeded at 0.5, rest at 1.0 → mean > 0.97 */
        assert!(rd_mean_u(&rd) > 0.95);
    }

    #[test]
    fn test_initial_mean_v_low() {
        let rd = new_reaction_diffusion(20, 20);
        assert!(rd_mean_v(&rd) < 0.1);
    }

    #[test]
    fn test_step_changes_grid() {
        let mut rd = new_reaction_diffusion(20, 20);
        let u_before = rd_mean_u(&rd);
        rd_step(&mut rd, 1.0);
        let u_after = rd_mean_u(&rd);
        assert!((u_before - u_after).abs() > 1e-10);
    }

    #[test]
    fn test_u_stays_bounded() {
        let mut rd = new_reaction_diffusion(20, 20);
        for _ in 0..10 {
            rd_step(&mut rd, 1.0);
        }
        for &u in &rd.u {
            assert!((0.0..=1.0).contains(&u));
        }
    }

    #[test]
    fn test_v_stays_bounded() {
        let mut rd = new_reaction_diffusion(20, 20);
        for _ in 0..10 {
            rd_step(&mut rd, 1.0);
        }
        for &v in &rd.v {
            assert!((0.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn test_custom_params() {
        let p = GrayScottParams {
            du: 0.2,
            dv: 0.1,
            f: 0.04,
            k: 0.06,
        };
        let mut rd = ReactionDiffusion::new(10, 10, p);
        rd_step(&mut rd, 1.0);
        assert!(rd.u.iter().all(|&u| u.is_finite()));
    }

    #[test]
    fn test_width_height() {
        let rd = new_reaction_diffusion(15, 25);
        assert_eq!(rd.width, 15);
        assert_eq!(rd.height, 25);
    }

    #[test]
    fn test_many_steps_stable() {
        let mut rd = new_reaction_diffusion(10, 10);
        for _ in 0..50 {
            rd_step(&mut rd, 0.5);
        }
        assert!(rd.u.iter().all(|&u| u.is_finite()));
        assert!(rd.v.iter().all(|&v| v.is_finite()));
    }
}
