// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[allow(dead_code)]
pub struct LatticeGasCell {
    pub density: f32,
    pub velocities: [f32; 4],
}

#[allow(dead_code)]
pub struct LatticeGasGrid {
    pub cells: Vec<LatticeGasCell>,
    pub width: usize,
    pub height: usize,
}

#[allow(dead_code)]
pub fn new_lattice_gas_grid(w: usize, h: usize) -> LatticeGasGrid {
    let cells = (0..w * h).map(|_| LatticeGasCell { density: 0.0, velocities: [0.0; 4] }).collect();
    LatticeGasGrid { cells, width: w, height: h }
}

#[allow(dead_code)]
pub fn lg_cell(g: &LatticeGasGrid, x: usize, y: usize) -> &LatticeGasCell {
    &g.cells[y * g.width + x]
}

#[allow(dead_code)]
pub fn lg_set_density(g: &mut LatticeGasGrid, x: usize, y: usize, d: f32) {
    g.cells[y * g.width + x].density = d;
}

#[allow(dead_code)]
pub fn lg_total_density(g: &LatticeGasGrid) -> f32 {
    g.cells.iter().map(|c| c.density).sum()
}

#[allow(dead_code)]
pub fn lg_step(g: &mut LatticeGasGrid) {
    for cell in g.cells.iter_mut() {
        for v in cell.velocities.iter_mut() {
            *v *= 0.99;
        }
    }
}

#[allow(dead_code)]
pub fn lg_max_density(g: &LatticeGasGrid) -> f32 {
    g.cells.iter().map(|c| c.density).fold(0.0f32, f32::max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_grid() {
        let g = new_lattice_gas_grid(4, 4);
        assert_eq!(g.cells.len(), 16);
    }

    #[test]
    fn test_set_density() {
        let mut g = new_lattice_gas_grid(3, 3);
        lg_set_density(&mut g, 1, 1, 2.5);
        assert!((lg_cell(&g, 1, 1).density - 2.5).abs() < 1e-6);
    }

    #[test]
    fn test_total_density() {
        let mut g = new_lattice_gas_grid(2, 2);
        lg_set_density(&mut g, 0, 0, 1.0);
        lg_set_density(&mut g, 1, 1, 1.0);
        assert!((lg_total_density(&g) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_max_density() {
        let mut g = new_lattice_gas_grid(3, 3);
        lg_set_density(&mut g, 0, 0, 5.0);
        lg_set_density(&mut g, 2, 2, 3.0);
        assert!((lg_max_density(&g) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_step_no_crash() {
        let mut g = new_lattice_gas_grid(4, 4);
        lg_set_density(&mut g, 0, 0, 1.0);
        lg_step(&mut g);
        assert!(lg_total_density(&g) >= 0.0);
    }

    #[test]
    fn test_zero_density_initially() {
        let g = new_lattice_gas_grid(2, 2);
        assert!((lg_total_density(&g)).abs() < 1e-6);
    }

    #[test]
    fn test_cell_accessor() {
        let g = new_lattice_gas_grid(5, 5);
        let cell = lg_cell(&g, 4, 4);
        assert!((cell.density).abs() < 1e-6);
    }

    #[test]
    fn test_step_decays_velocities() {
        let mut g = new_lattice_gas_grid(2, 2);
        for v in g.cells[0].velocities.iter_mut() {
            *v = 1.0;
        }
        lg_step(&mut g);
        assert!(g.cells[0].velocities[0] < 1.0);
    }
}
