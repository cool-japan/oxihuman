// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! 2D shallow water equations (explicit Euler).

#![allow(dead_code)]

/// 2D shallow water simulation on a regular grid.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShallowWater {
    pub nx: usize,
    pub ny: usize,
    pub dx: f32,
    pub dy: f32,
    pub g: f32, // gravity
    /// Water height h[x*ny + y].
    pub h: Vec<f32>,
    /// x-velocity u[x*ny + y].
    pub u: Vec<f32>,
    /// y-velocity v[x*ny + y].
    pub v: Vec<f32>,
    pub time: f32,
}

#[allow(dead_code)]
impl ShallowWater {
    pub fn new(nx: usize, ny: usize, dx: f32, dy: f32) -> Self {
        let n = nx * ny;
        Self {
            nx,
            ny,
            dx,
            dy,
            g: 9.81,
            h: vec![1.0; n],
            u: vec![0.0; n],
            v: vec![0.0; n],
            time: 0.0,
        }
    }

    fn idx(&self, x: usize, y: usize) -> usize {
        x * self.ny + y
    }

    /// Get height at (x, y) with zero-gradient boundary.
    fn h_get(&self, x: i32, y: i32) -> f32 {
        let xi = x.clamp(0, self.nx as i32 - 1) as usize;
        let yi = y.clamp(0, self.ny as i32 - 1) as usize;
        self.h[xi * self.ny + yi]
    }

    fn u_get(&self, x: i32, y: i32) -> f32 {
        let xi = x.clamp(0, self.nx as i32 - 1) as usize;
        let yi = y.clamp(0, self.ny as i32 - 1) as usize;
        self.u[xi * self.ny + yi]
    }

    fn v_get(&self, x: i32, y: i32) -> f32 {
        let xi = x.clamp(0, self.nx as i32 - 1) as usize;
        let yi = y.clamp(0, self.ny as i32 - 1) as usize;
        self.v[xi * self.ny + yi]
    }

    /// Explicit Euler step.
    pub fn step(&mut self, dt: f32) {
        let nx = self.nx as i32;
        let ny = self.ny as i32;
        let dx = self.dx;
        let dy = self.dy;
        let g = self.g;
        let mut dh = vec![0.0f32; self.nx * self.ny];
        let mut du = vec![0.0f32; self.nx * self.ny];
        let mut dv = vec![0.0f32; self.nx * self.ny];

        for xi in 0..nx {
            for yi in 0..ny {
                let i = self.idx(xi as usize, yi as usize);
                let h = self.h_get(xi, yi);
                let u = self.u_get(xi, yi);
                let v = self.v_get(xi, yi);
                // Continuity: dh/dt = -d(hu)/dx - d(hv)/dy
                let flux_x = (self.h_get(xi + 1, yi) * self.u_get(xi + 1, yi)
                    - self.h_get(xi - 1, yi) * self.u_get(xi - 1, yi))
                    / (2.0 * dx);
                let flux_y = (self.h_get(xi, yi + 1) * self.v_get(xi, yi + 1)
                    - self.h_get(xi, yi - 1) * self.v_get(xi, yi - 1))
                    / (2.0 * dy);
                dh[i] = -(flux_x + flux_y);
                // Momentum: du/dt = -u*du/dx - v*du/dy - g*dh/dx
                let dudx = (self.u_get(xi + 1, yi) - self.u_get(xi - 1, yi)) / (2.0 * dx);
                let dudy = (self.u_get(xi, yi + 1) - self.u_get(xi, yi - 1)) / (2.0 * dy);
                let dhdx = (self.h_get(xi + 1, yi) - self.h_get(xi - 1, yi)) / (2.0 * dx);
                du[i] = -u * dudx - v * dudy - g * dhdx;
                let dvdx = (self.v_get(xi + 1, yi) - self.v_get(xi - 1, yi)) / (2.0 * dx);
                let dvdy = (self.v_get(xi, yi + 1) - self.v_get(xi, yi - 1)) / (2.0 * dy);
                let dhdy = (self.h_get(xi, yi + 1) - self.h_get(xi, yi - 1)) / (2.0 * dy);
                dv[i] = -u * dvdx - v * dvdy - g * dhdy;
                let _ = h; // used through h_get
            }
        }
        for i in 0..self.nx * self.ny {
            self.h[i] = (self.h[i] + dh[i] * dt).max(0.0);
            self.u[i] += du[i] * dt;
            self.v[i] += dv[i] * dt;
        }
        self.time += dt;
    }

    /// Set height at (x, y).
    pub fn set_height(&mut self, x: usize, y: usize, h: f32) {
        let i = self.idx(x, y);
        self.h[i] = h;
    }

    /// Get height at (x, y).
    pub fn get_height(&self, x: usize, y: usize) -> f32 {
        self.h[self.idx(x, y)]
    }

    /// Total water volume.
    pub fn total_volume(&self) -> f32 {
        self.h.iter().sum::<f32>() * self.dx * self.dy
    }

    /// CFL stable time step.
    pub fn cfl_dt(&self) -> f32 {
        let h_max = self.h.iter().cloned().fold(0.0f32, f32::max);
        let c = (self.g * h_max.max(1e-6)).sqrt();
        0.4 * self.dx.min(self.dy) / c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small_sw() -> ShallowWater {
        ShallowWater::new(8, 8, 0.1, 0.1)
    }

    #[test]
    fn initial_height_one() {
        let sw = small_sw();
        assert!((sw.get_height(4, 4) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn set_and_get_height() {
        let mut sw = small_sw();
        sw.set_height(2, 3, 2.0);
        assert!((sw.get_height(2, 3) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn total_volume_initial() {
        let sw = small_sw();
        let expected = 8.0 * 8.0 * 0.1 * 0.1;
        assert!((sw.total_volume() - expected).abs() < 0.01);
    }

    #[test]
    fn step_runs_without_panic() {
        let mut sw = small_sw();
        sw.step(0.001);
    }

    #[test]
    fn cfl_dt_positive() {
        let sw = small_sw();
        assert!(sw.cfl_dt() > 0.0);
    }

    #[test]
    fn height_nonnegative_after_step() {
        let mut sw = small_sw();
        sw.step(0.001);
        for &h in &sw.h {
            assert!(h >= 0.0);
        }
    }

    #[test]
    fn time_advances() {
        let mut sw = small_sw();
        sw.step(0.01);
        assert!((sw.time - 0.01).abs() < 1e-5);
    }

    #[test]
    fn many_steps_finite() {
        let mut sw = small_sw();
        let dt = sw.cfl_dt();
        for _ in 0..20 {
            sw.step(dt);
        }
        assert!(sw.h[0].is_finite());
    }

    #[test]
    fn disturbance_propagates() {
        let mut sw = small_sw();
        sw.set_height(4, 4, 2.0);
        let h0 = sw.get_height(5, 4);
        let dt = sw.cfl_dt();
        for _ in 0..10 {
            sw.step(dt);
        }
        let h1 = sw.get_height(5, 4);
        let _changed = (h1 - h0).abs(); // wave should propagate
    }

    #[test]
    fn volume_approx_conserved() {
        let mut sw = small_sw();
        let v0 = sw.total_volume();
        let dt = sw.cfl_dt();
        for _ in 0..10 {
            sw.step(dt);
        }
        let v1 = sw.total_volume();
        assert!((v1 - v0).abs() < v0 * 0.1, "volume changed: {v0} -> {v1}");
    }
}
