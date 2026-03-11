// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// An elastic surface modeled as a grid of mass-spring nodes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ElasticSurface {
    positions: Vec<[f32; 3]>,
    velocities: Vec<[f32; 3]>,
    rest_positions: Vec<[f32; 3]>,
    width: usize,
    height: usize,
    stiffness: f32,
    damping: f32,
}

#[allow(dead_code)]
impl ElasticSurface {
    pub fn new(width: usize, height: usize, spacing: f32, stiffness: f32, damping: f32) -> Self {
        let mut positions = Vec::with_capacity(width * height);
        for row in 0..height {
            for col in 0..width {
                positions.push([col as f32 * spacing, 0.0, row as f32 * spacing]);
            }
        }
        let rest_positions = positions.clone();
        let velocities = vec![[0.0f32; 3]; width * height];
        Self {
            positions,
            velocities,
            rest_positions,
            width,
            height,
            stiffness,
            damping,
        }
    }

    pub fn node_count(&self) -> usize {
        self.positions.len()
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn position(&self, index: usize) -> Option<[f32; 3]> {
        self.positions.get(index).copied()
    }

    pub fn set_position(&mut self, index: usize, pos: [f32; 3]) {
        if let Some(p) = self.positions.get_mut(index) {
            *p = pos;
        }
    }

    pub fn apply_force(&mut self, index: usize, force: [f32; 3], dt: f32) {
        if let Some(vel) = self.velocities.get_mut(index) {
            for i in 0..3 {
                vel[i] += force[i] * dt;
            }
        }
    }

    #[allow(clippy::needless_range_loop)]
    pub fn step(&mut self, dt: f32) {
        let n = self.positions.len();
        let mut forces = vec![[0.0f32; 3]; n];
        // Structural springs to rest
        for i in 0..n {
            for axis in 0..3 {
                let disp = self.positions[i][axis] - self.rest_positions[i][axis];
                forces[i][axis] -= self.stiffness * disp + self.damping * self.velocities[i][axis];
            }
        }
        // Integrate
        for i in 0..n {
            for axis in 0..3 {
                self.velocities[i][axis] += forces[i][axis] * dt;
                self.positions[i][axis] += self.velocities[i][axis] * dt;
            }
        }
    }

    pub fn total_displacement(&self) -> f32 {
        let mut total = 0.0f32;
        for i in 0..self.positions.len() {
            for axis in 0..3 {
                let d = self.positions[i][axis] - self.rest_positions[i][axis];
                total += d * d;
            }
        }
        total.sqrt()
    }

    pub fn reset(&mut self) {
        self.positions = self.rest_positions.clone();
        self.velocities.fill([0.0; 3]);
    }

    pub fn center_of_mass(&self) -> [f32; 3] {
        if self.positions.is_empty() {
            return [0.0; 3];
        }
        let mut sum = [0.0f32; 3];
        for p in &self.positions {
            for i in 0..3 {
                sum[i] += p[i];
            }
        }
        let n = self.positions.len() as f32;
        [sum[0] / n, sum[1] / n, sum[2] / n]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let s = ElasticSurface::new(3, 3, 1.0, 100.0, 10.0);
        assert_eq!(s.node_count(), 9);
        assert_eq!(s.width(), 3);
        assert_eq!(s.height(), 3);
    }

    #[test]
    fn test_position() {
        let s = ElasticSurface::new(2, 2, 1.0, 100.0, 10.0);
        let p = s.position(0).unwrap();
        assert!((p[0]).abs() < 1e-6);
    }

    #[test]
    fn test_set_position() {
        let mut s = ElasticSurface::new(2, 2, 1.0, 100.0, 10.0);
        s.set_position(0, [5.0, 5.0, 5.0]);
        assert_eq!(s.position(0), Some([5.0, 5.0, 5.0]));
    }

    #[test]
    fn test_step_at_rest() {
        let mut s = ElasticSurface::new(2, 2, 1.0, 100.0, 10.0);
        s.step(0.01);
        assert!(s.total_displacement() < 1e-6);
    }

    #[test]
    fn test_step_displaced() {
        let mut s = ElasticSurface::new(2, 2, 1.0, 100.0, 10.0);
        s.set_position(0, [0.0, 1.0, 0.0]);
        s.step(0.01);
        let p = s.position(0).unwrap();
        assert!(p[1] < 1.0);
    }

    #[test]
    fn test_apply_force() {
        let mut s = ElasticSurface::new(2, 2, 1.0, 100.0, 10.0);
        s.apply_force(0, [0.0, 10.0, 0.0], 0.1);
        s.step(0.01);
        let p = s.position(0).unwrap();
        assert!(p[1] > 0.0);
    }

    #[test]
    fn test_reset() {
        let mut s = ElasticSurface::new(2, 2, 1.0, 100.0, 10.0);
        s.set_position(0, [99.0, 99.0, 99.0]);
        s.reset();
        assert!(s.total_displacement() < 1e-6);
    }

    #[test]
    fn test_total_displacement() {
        let s = ElasticSurface::new(2, 2, 1.0, 100.0, 10.0);
        assert!(s.total_displacement() < 1e-6);
    }

    #[test]
    fn test_center_of_mass() {
        let s = ElasticSurface::new(2, 1, 2.0, 100.0, 10.0);
        let com = s.center_of_mass();
        assert!((com[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_out_of_bounds() {
        let s = ElasticSurface::new(2, 2, 1.0, 100.0, 10.0);
        assert_eq!(s.position(100), None);
    }
}
