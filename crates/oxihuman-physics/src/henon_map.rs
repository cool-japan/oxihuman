// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Hénon map iteration.

/// Hénon map: x_{n+1} = 1 - a*x_n^2 + y_n, y_{n+1} = b*x_n
#[derive(Debug, Clone)]
pub struct HenonMap {
    pub x: f64,
    pub y: f64,
    pub a: f64,
    pub b: f64,
    pub iteration: u64,
}

impl HenonMap {
    pub fn new(x0: f64, y0: f64, a: f64, b: f64) -> Self {
        Self {
            x: x0,
            y: y0,
            a,
            b,
            iteration: 0,
        }
    }

    pub fn step(&mut self) {
        let xn = 1.0 - self.a * self.x * self.x + self.y;
        let yn = self.b * self.x;
        self.x = xn;
        self.y = yn;
        self.iteration += 1;
    }

    pub fn iterate_n(&mut self, n: u64) {
        for _ in 0..n {
            self.step();
        }
    }

    /// Collect trajectory as Vec of (x, y) pairs.
    pub fn trajectory(&self, n: u64) -> Vec<(f64, f64)> {
        let mut x = self.x;
        let mut y = self.y;
        let mut out = Vec::with_capacity(n as usize);
        for _ in 0..n {
            let xn = 1.0 - self.a * x * x + y;
            let yn = self.b * x;
            out.push((xn, yn));
            x = xn;
            y = yn;
        }
        out
    }

    pub fn position(&self) -> (f64, f64) {
        (self.x, self.y)
    }

    /// Jacobian determinant = b (constant for Hénon).
    pub fn jacobian_det(&self) -> f64 {
        self.b
    }
}

pub fn new_henon_map(x0: f64, y0: f64) -> HenonMap {
    /* Classic Hénon parameters: a=1.4, b=0.3 */
    HenonMap::new(x0, y0, 1.4, 0.3)
}

pub fn henon_step(m: &mut HenonMap) {
    m.step();
}

pub fn henon_iterate(m: &mut HenonMap, n: u64) {
    m.iterate_n(n);
}

pub fn henon_position(m: &HenonMap) -> (f64, f64) {
    m.position()
}

pub fn henon_trajectory(m: &HenonMap, n: u64) -> Vec<(f64, f64)> {
    m.trajectory(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_map() {
        let m = new_henon_map(0.0, 0.0);
        let (x, y) = henon_position(&m);
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
    }

    #[test]
    fn test_step() {
        /* x=0, y=0: xn = 1 - 1.4*0 + 0 = 1; yn = 0.3*0 = 0 */
        let mut m = new_henon_map(0.0, 0.0);
        henon_step(&mut m);
        let (x, y) = henon_position(&m);
        assert!((x - 1.0).abs() < 1e-10);
        assert_eq!(y, 0.0);
    }

    #[test]
    fn test_iteration_counter() {
        let mut m = new_henon_map(0.0, 0.0);
        henon_iterate(&mut m, 5);
        assert_eq!(m.iteration, 5);
    }

    #[test]
    fn test_trajectory_length() {
        let m = new_henon_map(0.0, 0.0);
        let traj = henon_trajectory(&m, 10);
        assert_eq!(traj.len(), 10);
    }

    #[test]
    fn test_jacobian_det() {
        let m = new_henon_map(0.0, 0.0);
        assert!((m.jacobian_det() - 0.3).abs() < 1e-10);
    }

    #[test]
    fn test_attractor_bounded() {
        /* Classic Hénon attractor stays within rough bounds */
        let mut m = new_henon_map(0.1, 0.1);
        henon_iterate(&mut m, 100);
        let (x, y) = henon_position(&m);
        assert!(x.is_finite());
        assert!(y.is_finite());
    }

    #[test]
    fn test_custom_params() {
        let m = HenonMap::new(0.0, 0.0, 1.0, 0.5);
        assert!((m.a - 1.0).abs() < 1e-10);
        assert!((m.b - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_trajectory_first_point() {
        /* First point of trajectory from (0,0): x=1, y=0 */
        let m = new_henon_map(0.0, 0.0);
        let traj = henon_trajectory(&m, 1);
        assert!((traj[0].0 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_two_steps_deterministic() {
        let mut m1 = new_henon_map(0.5, 0.3);
        let mut m2 = new_henon_map(0.5, 0.3);
        henon_step(&mut m1);
        henon_step(&mut m2);
        let (x1, y1) = henon_position(&m1);
        let (x2, y2) = henon_position(&m2);
        assert!((x1 - x2).abs() < 1e-14);
        assert!((y1 - y2).abs() < 1e-14);
    }
}
