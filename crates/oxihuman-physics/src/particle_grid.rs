// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Particle grid / SPH neighbor search grid.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParticleGridConfig {
    pub cell_size: f32,
    pub max_particles: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParticleGrid {
    pub cells: HashMap<(i32, i32, i32), Vec<usize>>,
    pub config: ParticleGridConfig,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GridStats {
    pub cell_count: usize,
    pub total_entries: usize,
    pub max_per_cell: usize,
}

#[allow(dead_code)]
pub fn default_particle_grid_config() -> ParticleGridConfig {
    ParticleGridConfig {
        cell_size: 1.0,
        max_particles: 1024,
    }
}

#[allow(dead_code)]
pub fn new_particle_grid(config: ParticleGridConfig) -> ParticleGrid {
    ParticleGrid {
        cells: HashMap::new(),
        config,
    }
}

#[allow(dead_code)]
pub fn pg_cell_for_pos(config: &ParticleGridConfig, pos: [f32; 3]) -> (i32, i32, i32) {
    let cs = config.cell_size;
    (
        (pos[0] / cs).floor() as i32,
        (pos[1] / cs).floor() as i32,
        (pos[2] / cs).floor() as i32,
    )
}

#[allow(dead_code)]
pub fn pg_insert(grid: &mut ParticleGrid, idx: usize, pos: [f32; 3]) {
    let cell = pg_cell_for_pos(&grid.config, pos);
    grid.cells.entry(cell).or_default().push(idx);
}

#[allow(dead_code)]
pub fn pg_clear(grid: &mut ParticleGrid) {
    grid.cells.clear();
}

#[allow(dead_code)]
pub fn pg_neighbors(grid: &ParticleGrid, pos: [f32; 3], radius: f32) -> Vec<usize> {
    let cs = grid.config.cell_size;
    let cell_radius = (radius / cs).ceil() as i32;
    let center = pg_cell_for_pos(&grid.config, pos);
    let mut result = Vec::new();
    for dx in -cell_radius..=cell_radius {
        for dy in -cell_radius..=cell_radius {
            for dz in -cell_radius..=cell_radius {
                let key = (center.0 + dx, center.1 + dy, center.2 + dz);
                if let Some(entries) = grid.cells.get(&key) {
                    result.extend_from_slice(entries);
                }
            }
        }
    }
    result
}

#[allow(dead_code)]
pub fn pg_stats(grid: &ParticleGrid) -> GridStats {
    let cell_count = grid.cells.len();
    let total_entries: usize = grid.cells.values().map(|v| v.len()).sum();
    let max_per_cell = grid.cells.values().map(|v| v.len()).max().unwrap_or(0);
    GridStats {
        cell_count,
        total_entries,
        max_per_cell,
    }
}

#[allow(dead_code)]
pub fn pg_total_entries(grid: &ParticleGrid) -> usize {
    grid.cells.values().map(|v| v.len()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_particle_grid_config();
        assert!((cfg.cell_size - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_empty() {
        let cfg = default_particle_grid_config();
        let grid = new_particle_grid(cfg);
        assert_eq!(pg_total_entries(&grid), 0);
    }

    #[test]
    fn test_insert_and_total() {
        let cfg = default_particle_grid_config();
        let mut grid = new_particle_grid(cfg);
        pg_insert(&mut grid, 0, [0.5, 0.5, 0.5]);
        pg_insert(&mut grid, 1, [1.5, 0.5, 0.5]);
        assert_eq!(pg_total_entries(&grid), 2);
    }

    #[test]
    fn test_cell_for_pos() {
        let cfg = ParticleGridConfig {
            cell_size: 2.0,
            max_particles: 16,
        };
        let cell = pg_cell_for_pos(&cfg, [3.0, 5.0, 7.0]);
        assert_eq!(cell, (1, 2, 3));
    }

    #[test]
    fn test_neighbors() {
        let cfg = default_particle_grid_config();
        let mut grid = new_particle_grid(cfg);
        pg_insert(&mut grid, 0, [0.5, 0.5, 0.5]);
        pg_insert(&mut grid, 1, [10.0, 10.0, 10.0]);
        let neighbors = pg_neighbors(&grid, [0.5, 0.5, 0.5], 1.5);
        assert!(neighbors.contains(&0));
        assert!(!neighbors.contains(&1));
    }

    #[test]
    fn test_clear() {
        let cfg = default_particle_grid_config();
        let mut grid = new_particle_grid(cfg);
        pg_insert(&mut grid, 0, [0.0; 3]);
        pg_clear(&mut grid);
        assert_eq!(pg_total_entries(&grid), 0);
    }

    #[test]
    fn test_stats() {
        let cfg = default_particle_grid_config();
        let mut grid = new_particle_grid(cfg);
        pg_insert(&mut grid, 0, [0.5, 0.5, 0.5]);
        pg_insert(&mut grid, 1, [0.6, 0.5, 0.5]);
        let stats = pg_stats(&grid);
        assert_eq!(stats.total_entries, 2);
        assert!(stats.cell_count >= 1);
    }

    #[test]
    fn test_multiple_in_same_cell() {
        let cfg = default_particle_grid_config();
        let mut grid = new_particle_grid(cfg);
        pg_insert(&mut grid, 0, [0.1, 0.1, 0.1]);
        pg_insert(&mut grid, 1, [0.2, 0.2, 0.2]);
        pg_insert(&mut grid, 2, [0.3, 0.3, 0.3]);
        let stats = pg_stats(&grid);
        assert_eq!(stats.max_per_cell, 3);
    }

    #[test]
    fn test_neighbors_empty_grid() {
        let cfg = default_particle_grid_config();
        let grid = new_particle_grid(cfg);
        let neighbors = pg_neighbors(&grid, [0.0; 3], 1.0);
        assert!(neighbors.is_empty());
    }
}
