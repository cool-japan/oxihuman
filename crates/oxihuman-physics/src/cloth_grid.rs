// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Grid-based cloth simulation helper.

#![allow(dead_code)]

/// Grid-based cloth particle structure.
#[allow(dead_code)]
pub struct ClothGrid {
    pub rows: usize,
    pub cols: usize,
    pub positions: Vec<[f32; 3]>,
    pub velocities: Vec<[f32; 3]>,
    pub masses: Vec<f32>,
    pub fixed: Vec<bool>,
}

/// Create a new cloth grid with `rows` x `cols` particles, spaced by `spacing`.
#[allow(dead_code)]
pub fn new_cloth_grid(rows: usize, cols: usize, spacing: f32) -> ClothGrid {
    let n = rows * cols;
    let mut positions = Vec::with_capacity(n);
    for r in 0..rows {
        for c in 0..cols {
            positions.push([c as f32 * spacing, 0.0, r as f32 * spacing]);
        }
    }
    ClothGrid {
        rows,
        cols,
        positions,
        velocities: vec![[0.0; 3]; n],
        masses: vec![1.0; n],
        fixed: vec![false; n],
    }
}

/// Return the flat index of particle at (row, col).
#[allow(dead_code)]
pub fn cloth_grid_index(grid: &ClothGrid, r: usize, c: usize) -> usize {
    r * grid.cols + c
}

/// Return the rest length of the spring between particles i and j.
#[allow(dead_code)]
pub fn spring_rest_length(grid: &ClothGrid, i: usize, j: usize) -> f32 {
    let pi = grid.positions[i];
    let pj = grid.positions[j];
    let dx = pi[0] - pj[0];
    let dy = pi[1] - pj[1];
    let dz = pi[2] - pj[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Perform a single Euler integration step on the cloth grid.
/// Applies gravity to unfixed particles.
#[allow(dead_code)]
pub fn cloth_grid_step(grid: &mut ClothGrid, gravity: [f32; 3], dt: f32) {
    let n = grid.positions.len();
    for i in 0..n {
        if grid.fixed[i] {
            continue;
        }
        grid.velocities[i][0] += gravity[0] * dt;
        grid.velocities[i][1] += gravity[1] * dt;
        grid.velocities[i][2] += gravity[2] * dt;
        grid.positions[i][0] += grid.velocities[i][0] * dt;
        grid.positions[i][1] += grid.velocities[i][1] * dt;
        grid.positions[i][2] += grid.velocities[i][2] * dt;
    }
}

/// Pin the four corners of the grid (fix them in place).
#[allow(dead_code)]
pub fn pin_corner(grid: &mut ClothGrid) {
    if grid.rows == 0 || grid.cols == 0 {
        return;
    }
    let tl = cloth_grid_index(grid, 0, 0);
    let tr = cloth_grid_index(grid, 0, grid.cols - 1);
    let bl = cloth_grid_index(grid, grid.rows - 1, 0);
    let br = cloth_grid_index(grid, grid.rows - 1, grid.cols - 1);
    for idx in [tl, tr, bl, br] {
        grid.fixed[idx] = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_grid_size() {
        let g = new_cloth_grid(3, 4, 1.0);
        assert_eq!(g.rows, 3);
        assert_eq!(g.cols, 4);
        assert_eq!(g.positions.len(), 12);
    }

    #[test]
    fn grid_index_correct() {
        let g = new_cloth_grid(3, 4, 1.0);
        assert_eq!(cloth_grid_index(&g, 1, 2), 6);
    }

    #[test]
    fn spring_rest_length_adjacent() {
        let g = new_cloth_grid(2, 2, 1.0);
        let i = cloth_grid_index(&g, 0, 0);
        let j = cloth_grid_index(&g, 0, 1);
        let len = spring_rest_length(&g, i, j);
        assert!((len - 1.0).abs() < 1e-5, "expected 1.0 got {len}");
    }

    #[test]
    fn pin_corner_marks_four_corners() {
        let mut g = new_cloth_grid(3, 3, 1.0);
        pin_corner(&mut g);
        assert!(g.fixed[cloth_grid_index(&g, 0, 0)]);
        assert!(g.fixed[cloth_grid_index(&g, 0, 2)]);
        assert!(g.fixed[cloth_grid_index(&g, 2, 0)]);
        assert!(g.fixed[cloth_grid_index(&g, 2, 2)]);
    }

    #[test]
    fn step_moves_unfixed_particles() {
        let mut g = new_cloth_grid(2, 2, 1.0);
        let before_y = g.positions[0][1];
        cloth_grid_step(&mut g, [0.0, -9.8, 0.0], 0.01);
        assert!(g.positions[0][1] < before_y);
    }

    #[test]
    fn step_does_not_move_fixed_particles() {
        let mut g = new_cloth_grid(2, 2, 1.0);
        pin_corner(&mut g);
        let before = g.positions[0];
        cloth_grid_step(&mut g, [0.0, -9.8, 0.0], 0.1);
        assert_eq!(g.positions[0], before);
    }

    #[test]
    fn new_grid_all_velocities_zero() {
        let g = new_cloth_grid(2, 3, 0.5);
        for v in &g.velocities {
            assert_eq!(*v, [0.0; 3]);
        }
    }

    #[test]
    fn new_grid_positions_spaced_correctly() {
        let g = new_cloth_grid(1, 3, 2.0);
        assert!((g.positions[0][0]).abs() < 1e-6);
        assert!((g.positions[1][0] - 2.0).abs() < 1e-6);
        assert!((g.positions[2][0] - 4.0).abs() < 1e-6);
    }

    #[test]
    fn new_grid_masses_are_one() {
        let g = new_cloth_grid(2, 2, 1.0);
        for m in &g.masses {
            assert!((*m - 1.0).abs() < 1e-6);
        }
    }
}
