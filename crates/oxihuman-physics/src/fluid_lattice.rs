// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A lattice-Boltzmann-inspired 2D fluid grid for simple fluid simulation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FluidLattice {
    width: usize,
    height: usize,
    density: Vec<f32>,
    velocity_x: Vec<f32>,
    velocity_y: Vec<f32>,
    viscosity: f32,
}

#[allow(dead_code)]
impl FluidLattice {
    pub fn new(width: usize, height: usize, viscosity: f32) -> Self {
        let size = width * height;
        Self {
            width,
            height,
            density: vec![1.0; size],
            velocity_x: vec![0.0; size],
            velocity_y: vec![0.0; size],
            viscosity: viscosity.max(0.0),
        }
    }

    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn set_density(&mut self, x: usize, y: usize, d: f32) {
        if x < self.width && y < self.height {
            let i = self.idx(x, y);
            self.density[i] = d;
        }
    }

    pub fn get_density(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.density[self.idx(x, y)]
        } else {
            0.0
        }
    }

    pub fn set_velocity(&mut self, x: usize, y: usize, vx: f32, vy: f32) {
        if x < self.width && y < self.height {
            let i = self.idx(x, y);
            self.velocity_x[i] = vx;
            self.velocity_y[i] = vy;
        }
    }

    pub fn get_velocity(&self, x: usize, y: usize) -> (f32, f32) {
        if x < self.width && y < self.height {
            let i = self.idx(x, y);
            (self.velocity_x[i], self.velocity_y[i])
        } else {
            (0.0, 0.0)
        }
    }

    /// Simple diffusion step.
    pub fn diffuse(&mut self, dt: f32) {
        let factor = self.viscosity * dt;
        let old_vx = self.velocity_x.clone();
        let old_vy = self.velocity_y.clone();
        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                let i = self.idx(x, y);
                let left = self.idx(x - 1, y);
                let right = self.idx(x + 1, y);
                let up = self.idx(x, y - 1);
                let down = self.idx(x, y + 1);
                let laplacian_x = old_vx[left] + old_vx[right] + old_vx[up] + old_vx[down] - 4.0 * old_vx[i];
                let laplacian_y = old_vy[left] + old_vy[right] + old_vy[up] + old_vy[down] - 4.0 * old_vy[i];
                self.velocity_x[i] += factor * laplacian_x;
                self.velocity_y[i] += factor * laplacian_y;
            }
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn cell_count(&self) -> usize {
        self.width * self.height
    }

    pub fn total_density(&self) -> f32 {
        self.density.iter().sum()
    }

    pub fn average_density(&self) -> f32 {
        let n = self.cell_count() as f32;
        if n < 1.0 {
            return 0.0;
        }
        self.total_density() / n
    }

    pub fn max_speed(&self) -> f32 {
        self.velocity_x
            .iter()
            .zip(self.velocity_y.iter())
            .map(|(&vx, &vy)| vx * vx + vy * vy)
            .fold(0.0_f32, f32::max)
            .sqrt()
    }

    pub fn clear_velocities(&mut self) {
        self.velocity_x.iter_mut().for_each(|v| *v = 0.0);
        self.velocity_y.iter_mut().for_each(|v| *v = 0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let f = FluidLattice::new(10, 10, 0.1);
        assert_eq!(f.cell_count(), 100);
    }

    #[test]
    fn test_set_get_density() {
        let mut f = FluidLattice::new(4, 4, 0.1);
        f.set_density(1, 2, 5.0);
        assert!((f.get_density(1, 2) - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_get_velocity() {
        let mut f = FluidLattice::new(4, 4, 0.1);
        f.set_velocity(1, 1, 2.0, 3.0);
        let (vx, vy) = f.get_velocity(1, 1);
        assert!((vx - 2.0).abs() < f32::EPSILON);
        assert!((vy - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_average_density() {
        let f = FluidLattice::new(4, 4, 0.1);
        assert!((f.average_density() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_max_speed_zero() {
        let f = FluidLattice::new(4, 4, 0.1);
        assert!((f.max_speed()).abs() < f32::EPSILON);
    }

    #[test]
    fn test_max_speed_nonzero() {
        let mut f = FluidLattice::new(4, 4, 0.1);
        f.set_velocity(1, 1, 3.0, 4.0);
        assert!((f.max_speed() - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_clear_velocities() {
        let mut f = FluidLattice::new(4, 4, 0.1);
        f.set_velocity(1, 1, 5.0, 5.0);
        f.clear_velocities();
        assert!((f.max_speed()).abs() < f32::EPSILON);
    }

    #[test]
    fn test_diffuse_no_crash() {
        let mut f = FluidLattice::new(8, 8, 0.1);
        f.set_velocity(4, 4, 1.0, 0.0);
        f.diffuse(0.01);
        // Just verify it doesn't panic
        assert!(f.max_speed() >= 0.0);
    }

    #[test]
    fn test_out_of_bounds() {
        let f = FluidLattice::new(4, 4, 0.1);
        assert!((f.get_density(10, 10)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_dimensions() {
        let f = FluidLattice::new(5, 7, 0.1);
        assert_eq!(f.width(), 5);
        assert_eq!(f.height(), 7);
    }
}
