// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Spatial hash grid for fast 3D neighbor lookup.

#![allow(dead_code)]

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SpatialHashConfig {
    pub cell_size: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SpatialHashGrid {
    pub config: SpatialHashConfig,
    pub cells: HashMap<(i32, i32, i32), Vec<u32>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SpatialHashStats {
    pub cell_count: usize,
    pub total_entries: usize,
}

#[allow(dead_code)]
pub fn default_spatial_hash_config() -> SpatialHashConfig {
    SpatialHashConfig { cell_size: 1.0 }
}

#[allow(dead_code)]
pub fn new_spatial_hash(config: SpatialHashConfig) -> SpatialHashGrid {
    SpatialHashGrid {
        config,
        cells: HashMap::new(),
    }
}

fn cell_key(grid: &SpatialHashGrid, pos: [f32; 3]) -> (i32, i32, i32) {
    let cs = grid.config.cell_size.max(1e-9);
    (
        (pos[0] / cs).floor() as i32,
        (pos[1] / cs).floor() as i32,
        (pos[2] / cs).floor() as i32,
    )
}

#[allow(dead_code)]
pub fn sh_insert(grid: &mut SpatialHashGrid, id: u32, pos: [f32; 3]) {
    let key = cell_key(grid, pos);
    grid.cells.entry(key).or_default().push(id);
}

#[allow(dead_code)]
pub fn sh_remove(grid: &mut SpatialHashGrid, id: u32, pos: [f32; 3]) {
    let key = cell_key(grid, pos);
    if let Some(v) = grid.cells.get_mut(&key) {
        v.retain(|&x| x != id);
        if v.is_empty() {
            grid.cells.remove(&key);
        }
    }
}

#[allow(dead_code)]
pub fn sh_query_cell(grid: &SpatialHashGrid, pos: [f32; 3]) -> Vec<u32> {
    let key = cell_key(grid, pos);
    grid.cells.get(&key).cloned().unwrap_or_default()
}

#[allow(dead_code)]
pub fn sh_query_radius(grid: &SpatialHashGrid, pos: [f32; 3], radius: f32) -> Vec<u32> {
    let cs = grid.config.cell_size.max(1e-9);
    let r_cells = (radius / cs).ceil() as i32;
    let cx = (pos[0] / cs).floor() as i32;
    let cy = (pos[1] / cs).floor() as i32;
    let cz = (pos[2] / cs).floor() as i32;
    let r2 = radius * radius;
    let mut result = Vec::new();
    for ix in (cx - r_cells)..=(cx + r_cells) {
        for iy in (cy - r_cells)..=(cy + r_cells) {
            for iz in (cz - r_cells)..=(cz + r_cells) {
                if let Some(ids) = grid.cells.get(&(ix, iy, iz)) {
                    for &id in ids {
                        // approximate: include all in overlapping cells
                        let _ = r2;
                        result.push(id);
                    }
                }
            }
        }
    }
    result
}

#[allow(dead_code)]
pub fn sh_clear(grid: &mut SpatialHashGrid) {
    grid.cells.clear();
}

#[allow(dead_code)]
pub fn sh_stats(grid: &SpatialHashGrid) -> SpatialHashStats {
    let total_entries = grid.cells.values().map(|v| v.len()).sum();
    SpatialHashStats {
        cell_count: grid.cells.len(),
        total_entries,
    }
}

#[allow(dead_code)]
pub fn sh_cell_count(grid: &SpatialHashGrid) -> usize {
    grid.cells.len()
}

#[allow(dead_code)]
pub fn sh_entry_count(grid: &SpatialHashGrid) -> usize {
    grid.cells.values().map(|v| v.len()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_grid_is_empty() {
        let g = new_spatial_hash(default_spatial_hash_config());
        assert_eq!(sh_cell_count(&g), 0);
        assert_eq!(sh_entry_count(&g), 0);
    }

    #[test]
    fn test_insert_and_query_cell() {
        let mut g = new_spatial_hash(default_spatial_hash_config());
        sh_insert(&mut g, 1, [0.5, 0.5, 0.5]);
        let res = sh_query_cell(&g, [0.5, 0.5, 0.5]);
        assert_eq!(res, vec![1]);
    }

    #[test]
    fn test_remove_entry() {
        let mut g = new_spatial_hash(default_spatial_hash_config());
        sh_insert(&mut g, 42, [1.0, 1.0, 1.0]);
        sh_remove(&mut g, 42, [1.0, 1.0, 1.0]);
        assert_eq!(sh_cell_count(&g), 0);
    }

    #[test]
    fn test_query_radius_finds_nearby() {
        let mut g = new_spatial_hash(default_spatial_hash_config());
        sh_insert(&mut g, 5, [0.5, 0.5, 0.5]);
        let res = sh_query_radius(&g, [0.5, 0.5, 0.5], 2.0);
        assert!(res.contains(&5));
    }

    #[test]
    fn test_sh_clear() {
        let mut g = new_spatial_hash(default_spatial_hash_config());
        sh_insert(&mut g, 1, [0.0, 0.0, 0.0]);
        sh_insert(&mut g, 2, [5.0, 5.0, 5.0]);
        sh_clear(&mut g);
        assert_eq!(sh_cell_count(&g), 0);
    }

    #[test]
    fn test_stats() {
        let mut g = new_spatial_hash(default_spatial_hash_config());
        sh_insert(&mut g, 1, [0.0, 0.0, 0.0]);
        sh_insert(&mut g, 2, [0.1, 0.1, 0.1]);
        let s = sh_stats(&g);
        assert_eq!(s.cell_count, 1);
        assert_eq!(s.total_entries, 2);
    }

    #[test]
    fn test_multiple_cells() {
        let mut g = new_spatial_hash(default_spatial_hash_config());
        sh_insert(&mut g, 1, [0.0, 0.0, 0.0]);
        sh_insert(&mut g, 2, [10.0, 10.0, 10.0]);
        assert_eq!(sh_cell_count(&g), 2);
    }

    #[test]
    fn test_entry_count() {
        let mut g = new_spatial_hash(default_spatial_hash_config());
        sh_insert(&mut g, 1, [0.0, 0.0, 0.0]);
        sh_insert(&mut g, 2, [0.2, 0.2, 0.2]);
        sh_insert(&mut g, 3, [10.0, 10.0, 10.0]);
        assert_eq!(sh_entry_count(&g), 3);
    }

    #[test]
    fn test_query_empty_cell() {
        let g = new_spatial_hash(default_spatial_hash_config());
        let res = sh_query_cell(&g, [99.0, 99.0, 99.0]);
        assert!(res.is_empty());
    }

    #[test]
    fn test_custom_cell_size() {
        let cfg = SpatialHashConfig { cell_size: 5.0 };
        let mut g = new_spatial_hash(cfg);
        sh_insert(&mut g, 10, [2.0, 2.0, 2.0]);
        sh_insert(&mut g, 11, [4.0, 4.0, 4.0]);
        // Both points should land in the same cell (0,0,0) for cell_size=5
        assert_eq!(sh_cell_count(&g), 1);
        assert_eq!(sh_entry_count(&g), 2);
    }
}
