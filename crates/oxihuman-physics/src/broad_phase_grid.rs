// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Uniform grid broad-phase collision detection.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BpGridConfig {
    pub cell_size: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BpAabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub id: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BpGrid {
    pub cells: HashMap<(i32, i32, i32), Vec<u32>>,
    pub config: BpGridConfig,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BpPair {
    pub a: u32,
    pub b: u32,
}

#[allow(dead_code)]
pub fn default_bp_grid_config() -> BpGridConfig {
    BpGridConfig { cell_size: 1.0 }
}

#[allow(dead_code)]
pub fn new_bp_grid(config: BpGridConfig) -> BpGrid {
    BpGrid { cells: HashMap::new(), config }
}

#[allow(dead_code)]
pub fn bp_cell_for_point(config: &BpGridConfig, p: [f32; 3]) -> (i32, i32, i32) {
    let cell = |v: f32| (v / config.cell_size).floor() as i32;
    (cell(p[0]), cell(p[1]), cell(p[2]))
}

#[allow(dead_code)]
pub fn bp_aabb_cells(config: &BpGridConfig, aabb: &BpAabb) -> Vec<(i32, i32, i32)> {
    let min_cell = bp_cell_for_point(config, aabb.min);
    let max_cell = bp_cell_for_point(config, aabb.max);
    let mut cells = Vec::new();
    for x in min_cell.0..=max_cell.0 {
        for y in min_cell.1..=max_cell.1 {
            for z in min_cell.2..=max_cell.2 {
                cells.push((x, y, z));
            }
        }
    }
    cells
}

#[allow(dead_code)]
pub fn bp_insert_aabb(grid: &mut BpGrid, aabb: &BpAabb) {
    let config = grid.config.clone();
    let cells = bp_aabb_cells(&config, aabb);
    for cell in cells {
        grid.cells.entry(cell).or_default().push(aabb.id);
    }
}

#[allow(dead_code)]
pub fn bp_clear(grid: &mut BpGrid) {
    grid.cells.clear();
}

#[allow(dead_code)]
pub fn bp_collect_pairs(grid: &BpGrid) -> Vec<BpPair> {
    let mut pairs = Vec::new();
    for ids in grid.cells.values() {
        for i in 0..ids.len() {
            for j in (i + 1)..ids.len() {
                let (a, b) = (ids[i].min(ids[j]), ids[i].max(ids[j]));
                let pair = BpPair { a, b };
                if !pairs.contains(&pair) {
                    pairs.push(pair);
                }
            }
        }
    }
    pairs
}

#[allow(dead_code)]
pub fn bp_pair_count(grid: &BpGrid) -> usize {
    bp_collect_pairs(grid).len()
}

#[allow(dead_code)]
pub struct BroadPhaseGrid {
    pub cell_size: f32,
    pub cells: std::collections::HashMap<(i32, i32, i32), Vec<usize>>,
}

#[allow(dead_code)]
pub fn new_broad_phase_grid(cell_size: f32) -> BroadPhaseGrid {
    BroadPhaseGrid { cell_size, cells: std::collections::HashMap::new() }
}

fn bpg_cell_key(cell_size: f32, pos: [f32; 3]) -> (i32, i32, i32) {
    let f = |v: f32| (v / cell_size).floor() as i32;
    (f(pos[0]), f(pos[1]), f(pos[2]))
}

#[allow(dead_code)]
pub fn bpg_insert(g: &mut BroadPhaseGrid, id: usize, pos: [f32; 3]) {
    let key = bpg_cell_key(g.cell_size, pos);
    g.cells.entry(key).or_default().push(id);
}

#[allow(dead_code)]
pub fn bpg_query(g: &BroadPhaseGrid, pos: [f32; 3], radius: f32) -> Vec<usize> {
    let steps = (radius / g.cell_size).ceil() as i32 + 1;
    let center = bpg_cell_key(g.cell_size, pos);
    let mut result = Vec::new();
    for dx in -steps..=steps {
        for dy in -steps..=steps {
            for dz in -steps..=steps {
                let key = (center.0 + dx, center.1 + dy, center.2 + dz);
                if let Some(ids) = g.cells.get(&key) {
                    for &id in ids {
                        if !result.contains(&id) {
                            result.push(id);
                        }
                    }
                }
            }
        }
    }
    result
}

#[allow(dead_code)]
pub fn bpg_cell_count(g: &BroadPhaseGrid) -> usize {
    g.cells.len()
}

#[allow(dead_code)]
pub fn bpg_clear(g: &mut BroadPhaseGrid) {
    g.cells.clear();
}

#[allow(dead_code)]
pub fn bpg_object_count(g: &BroadPhaseGrid) -> usize {
    g.cells.values().map(|v| v.len()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn aabb(id: u32, min: [f32; 3], max: [f32; 3]) -> BpAabb {
        BpAabb { min, max, id }
    }

    #[test]
    fn test_default_config() {
        let cfg = default_bp_grid_config();
        assert_eq!(cfg.cell_size, 1.0);
    }

    #[test]
    fn test_new_grid_empty() {
        let cfg = default_bp_grid_config();
        let grid = new_bp_grid(cfg);
        assert!(grid.cells.is_empty());
    }

    #[test]
    fn test_cell_for_point() {
        let cfg = default_bp_grid_config();
        let cell = bp_cell_for_point(&cfg, [1.5, 2.5, 3.5]);
        assert_eq!(cell, (1, 2, 3));
    }

    #[test]
    fn test_insert_aabb() {
        let cfg = default_bp_grid_config();
        let mut grid = new_bp_grid(cfg);
        bp_insert_aabb(&mut grid, &aabb(1, [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]));
        assert!(!grid.cells.is_empty());
    }

    #[test]
    fn test_collect_pairs_overlapping() {
        let cfg = default_bp_grid_config();
        let mut grid = new_bp_grid(cfg);
        bp_insert_aabb(&mut grid, &aabb(1, [0.0, 0.0, 0.0], [0.9, 0.9, 0.9]));
        bp_insert_aabb(&mut grid, &aabb(2, [0.0, 0.0, 0.0], [0.9, 0.9, 0.9]));
        let pairs = bp_collect_pairs(&grid);
        assert!(!pairs.is_empty());
    }

    #[test]
    fn test_collect_pairs_no_overlap() {
        let cfg = default_bp_grid_config();
        let mut grid = new_bp_grid(cfg);
        bp_insert_aabb(&mut grid, &aabb(1, [0.0, 0.0, 0.0], [0.4, 0.4, 0.4]));
        bp_insert_aabb(&mut grid, &aabb(2, [5.0, 5.0, 5.0], [5.4, 5.4, 5.4]));
        let pairs = bp_collect_pairs(&grid);
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_bp_clear() {
        let cfg = default_bp_grid_config();
        let mut grid = new_bp_grid(cfg);
        bp_insert_aabb(&mut grid, &aabb(1, [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]));
        bp_clear(&mut grid);
        assert!(grid.cells.is_empty());
    }

    #[test]
    fn test_aabb_cells_single() {
        let cfg = default_bp_grid_config();
        let a = aabb(1, [0.1, 0.1, 0.1], [0.9, 0.9, 0.9]);
        let cells = bp_aabb_cells(&cfg, &a);
        assert_eq!(cells.len(), 1);
    }

    #[test]
    fn test_pair_count() {
        let cfg = default_bp_grid_config();
        let mut grid = new_bp_grid(cfg);
        bp_insert_aabb(&mut grid, &aabb(1, [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]));
        bp_insert_aabb(&mut grid, &aabb(2, [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]));
        assert_eq!(bp_pair_count(&grid), 1);
    }
}
