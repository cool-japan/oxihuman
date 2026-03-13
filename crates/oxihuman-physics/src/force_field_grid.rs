// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A 3D grid-based force field for applying spatial forces to particles.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ForceFieldGrid {
    forces: Vec<[f32; 3]>,
    resolution: [usize; 3],
    origin: [f32; 3],
    cell_size: f32,
}

#[allow(dead_code)]
impl ForceFieldGrid {
    pub fn new(resolution: [usize; 3], origin: [f32; 3], cell_size: f32) -> Self {
        let total = resolution[0] * resolution[1] * resolution[2];
        Self {
            forces: vec![[0.0; 3]; total],
            resolution,
            origin,
            cell_size,
        }
    }

    pub fn uniform(resolution: [usize; 3], origin: [f32; 3], cell_size: f32, force: [f32; 3]) -> Self {
        let total = resolution[0] * resolution[1] * resolution[2];
        Self {
            forces: vec![force; total],
            resolution,
            origin,
            cell_size,
        }
    }

    fn cell_index(&self, ix: usize, iy: usize, iz: usize) -> Option<usize> {
        if ix < self.resolution[0] && iy < self.resolution[1] && iz < self.resolution[2] {
            Some(ix + iy * self.resolution[0] + iz * self.resolution[0] * self.resolution[1])
        } else {
            None
        }
    }

    fn world_to_grid(&self, pos: [f32; 3]) -> Option<(usize, usize, usize)> {
        let lx = (pos[0] - self.origin[0]) / self.cell_size;
        let ly = (pos[1] - self.origin[1]) / self.cell_size;
        let lz = (pos[2] - self.origin[2]) / self.cell_size;
        if lx < 0.0 || ly < 0.0 || lz < 0.0 {
            return None;
        }
        let ix = lx as usize;
        let iy = ly as usize;
        let iz = lz as usize;
        if ix < self.resolution[0] && iy < self.resolution[1] && iz < self.resolution[2] {
            Some((ix, iy, iz))
        } else {
            None
        }
    }

    pub fn set_force(&mut self, ix: usize, iy: usize, iz: usize, force: [f32; 3]) {
        if let Some(idx) = self.cell_index(ix, iy, iz) {
            self.forces[idx] = force;
        }
    }

    pub fn get_force(&self, ix: usize, iy: usize, iz: usize) -> [f32; 3] {
        self.cell_index(ix, iy, iz)
            .map(|idx| self.forces[idx])
            .unwrap_or([0.0; 3])
    }

    pub fn sample(&self, pos: [f32; 3]) -> [f32; 3] {
        self.world_to_grid(pos)
            .and_then(|(ix, iy, iz)| self.cell_index(ix, iy, iz))
            .map(|idx| self.forces[idx])
            .unwrap_or([0.0; 3])
    }

    pub fn add_force(&mut self, ix: usize, iy: usize, iz: usize, force: [f32; 3]) {
        if let Some(idx) = self.cell_index(ix, iy, iz) {
            self.forces[idx][0] += force[0];
            self.forces[idx][1] += force[1];
            self.forces[idx][2] += force[2];
        }
    }

    pub fn clear(&mut self) {
        self.forces.iter_mut().for_each(|f| *f = [0.0; 3]);
    }

    pub fn cell_count(&self) -> usize {
        self.forces.len()
    }

    pub fn max_force_magnitude(&self) -> f32 {
        self.forces
            .iter()
            .map(|f| f[0] * f[0] + f[1] * f[1] + f[2] * f[2])
            .fold(0.0_f32, f32::max)
            .sqrt()
    }

    pub fn resolution(&self) -> [usize; 3] {
        self.resolution
    }

    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_grid() {
        let g = ForceFieldGrid::new([4, 4, 4], [0.0; 3], 1.0);
        assert_eq!(g.cell_count(), 64);
    }

    #[test]
    fn test_set_get_force() {
        let mut g = ForceFieldGrid::new([4, 4, 4], [0.0; 3], 1.0);
        g.set_force(1, 2, 3, [1.0, 2.0, 3.0]);
        let f = g.get_force(1, 2, 3);
        assert!((f[0] - 1.0).abs() < f32::EPSILON);
        assert!((f[1] - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_sample() {
        let mut g = ForceFieldGrid::new([4, 4, 4], [0.0; 3], 1.0);
        g.set_force(1, 0, 0, [5.0, 0.0, 0.0]);
        let f = g.sample([1.5, 0.5, 0.5]);
        assert!((f[0] - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_sample_outside() {
        let g = ForceFieldGrid::new([4, 4, 4], [0.0; 3], 1.0);
        let f = g.sample([-1.0, 0.0, 0.0]);
        assert!((f[0]).abs() < f32::EPSILON);
    }

    #[test]
    fn test_add_force() {
        let mut g = ForceFieldGrid::new([4, 4, 4], [0.0; 3], 1.0);
        g.add_force(0, 0, 0, [1.0, 0.0, 0.0]);
        g.add_force(0, 0, 0, [2.0, 0.0, 0.0]);
        let f = g.get_force(0, 0, 0);
        assert!((f[0] - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_clear() {
        let mut g = ForceFieldGrid::new([2, 2, 2], [0.0; 3], 1.0);
        g.set_force(0, 0, 0, [10.0, 0.0, 0.0]);
        g.clear();
        assert!((g.get_force(0, 0, 0)[0]).abs() < f32::EPSILON);
    }

    #[test]
    fn test_uniform() {
        let g = ForceFieldGrid::uniform([2, 2, 2], [0.0; 3], 1.0, [0.0, -9.81, 0.0]);
        let f = g.get_force(1, 1, 1);
        assert!((f[1] - (-9.81)).abs() < 1e-5);
    }

    #[test]
    fn test_max_force_magnitude() {
        let mut g = ForceFieldGrid::new([2, 2, 2], [0.0; 3], 1.0);
        g.set_force(0, 0, 0, [3.0, 4.0, 0.0]);
        assert!((g.max_force_magnitude() - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_resolution() {
        let g = ForceFieldGrid::new([3, 4, 5], [0.0; 3], 0.5);
        assert_eq!(g.resolution(), [3, 4, 5]);
    }

    #[test]
    fn test_out_of_bounds() {
        let g = ForceFieldGrid::new([2, 2, 2], [0.0; 3], 1.0);
        let f = g.get_force(10, 10, 10);
        assert!((f[0]).abs() < f32::EPSILON);
    }
}
