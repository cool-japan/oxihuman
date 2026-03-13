// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Light grid (clustered lighting) utilities.

/// Light grid configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LightGridConfig {
    pub tiles_x: u32,
    pub tiles_y: u32,
    pub slices_z: u32,
    pub max_lights_per_cell: u32,
}

/// A light grid cell.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LightGridCell {
    pub light_indices: Vec<u32>,
}

/// Light grid structure.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LightGrid {
    pub config: LightGridConfig,
    pub cells: Vec<LightGridCell>,
}

/// Default config.
#[allow(dead_code)]
pub fn default_light_grid_config() -> LightGridConfig {
    LightGridConfig {
        tiles_x: 16,
        tiles_y: 9,
        slices_z: 24,
        max_lights_per_cell: 32,
    }
}

/// Create a light grid.
#[allow(dead_code)]
pub fn new_light_grid(config: LightGridConfig) -> LightGrid {
    let total = (config.tiles_x * config.tiles_y * config.slices_z) as usize;
    let cells = vec![LightGridCell { light_indices: Vec::new() }; total];
    LightGrid { config, cells }
}

/// Total cell count.
#[allow(dead_code)]
pub fn total_cells(grid: &LightGrid) -> usize {
    grid.cells.len()
}

/// Get cell index from tile coordinates.
#[allow(dead_code)]
pub fn cell_index(config: &LightGridConfig, x: u32, y: u32, z: u32) -> usize {
    (z * config.tiles_y * config.tiles_x + y * config.tiles_x + x) as usize
}

/// Assign a light to a cell.
#[allow(dead_code)]
pub fn assign_light_to_cell(grid: &mut LightGrid, cell_idx: usize, light_id: u32) -> bool {
    if let Some(cell) = grid.cells.get_mut(cell_idx) {
        if cell.light_indices.len() < grid.config.max_lights_per_cell as usize {
            cell.light_indices.push(light_id);
            return true;
        }
    }
    false
}

/// Clear all assignments.
#[allow(dead_code)]
pub fn clear_light_grid(grid: &mut LightGrid) {
    for cell in &mut grid.cells {
        cell.light_indices.clear();
    }
}

/// Get lights in a cell.
#[allow(dead_code)]
pub fn lights_in_cell(grid: &LightGrid, cell_idx: usize) -> &[u32] {
    grid.cells.get(cell_idx).map_or(&[], |c| &c.light_indices)
}

/// Max lights across all cells.
#[allow(dead_code)]
pub fn max_lights_per_cell_used(grid: &LightGrid) -> usize {
    grid.cells.iter().map(|c| c.light_indices.len()).max().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_light_grid_config();
        assert_eq!(c.tiles_x, 16);
    }

    #[test]
    fn test_new_grid() {
        let g = new_light_grid(default_light_grid_config());
        assert_eq!(total_cells(&g), 16 * 9 * 24);
    }

    #[test]
    fn test_cell_index() {
        let c = default_light_grid_config();
        let idx = cell_index(&c, 0, 0, 0);
        assert_eq!(idx, 0);
    }

    #[test]
    fn test_assign_light() {
        let mut g = new_light_grid(default_light_grid_config());
        assert!(assign_light_to_cell(&mut g, 0, 42));
    }

    #[test]
    fn test_lights_in_cell() {
        let mut g = new_light_grid(default_light_grid_config());
        assign_light_to_cell(&mut g, 0, 42);
        let lights = lights_in_cell(&g, 0);
        assert_eq!(lights.len(), 1);
        assert_eq!(lights[0], 42);
    }

    #[test]
    fn test_clear() {
        let mut g = new_light_grid(default_light_grid_config());
        assign_light_to_cell(&mut g, 0, 1);
        clear_light_grid(&mut g);
        assert!(lights_in_cell(&g, 0).is_empty());
    }

    #[test]
    fn test_max_lights() {
        let config = LightGridConfig { tiles_x: 2, tiles_y: 2, slices_z: 1, max_lights_per_cell: 2 };
        let mut g = new_light_grid(config);
        assign_light_to_cell(&mut g, 0, 1);
        assign_light_to_cell(&mut g, 0, 2);
        assert!(!assign_light_to_cell(&mut g, 0, 3));
    }

    #[test]
    fn test_max_used() {
        let config = LightGridConfig { tiles_x: 2, tiles_y: 2, slices_z: 1, max_lights_per_cell: 10 };
        let mut g = new_light_grid(config);
        assign_light_to_cell(&mut g, 0, 1);
        assign_light_to_cell(&mut g, 0, 2);
        assert_eq!(max_lights_per_cell_used(&g), 2);
    }

    #[test]
    fn test_empty_cell() {
        let g = new_light_grid(default_light_grid_config());
        assert!(lights_in_cell(&g, 0).is_empty());
    }

    #[test]
    fn test_out_of_bounds() {
        let g = new_light_grid(default_light_grid_config());
        assert!(lights_in_cell(&g, 999999).is_empty());
    }
}
