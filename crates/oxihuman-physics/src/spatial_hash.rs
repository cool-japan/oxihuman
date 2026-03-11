// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Spatial hashing for broad-phase collision detection.
//!
//! Divides 3D space into a uniform grid of cells keyed by hash values.
//! Entities are inserted by point or AABB, and neighbor queries return
//! all entities that share the same cell(s).

use std::collections::HashMap;

// ── structs ──────────────────────────────────────────────────────────────────

/// Configuration for the spatial hash grid.
#[allow(dead_code)]
pub struct SpatialHashConfig {
    /// Cell size along each axis.
    pub cell_size: f32,
    /// Hash table capacity hint.
    pub table_size: usize,
}

/// An entry stored in the spatial hash grid.
#[allow(dead_code)]
#[derive(Clone)]
pub struct SpatialHashEntry {
    /// Unique entity identifier.
    pub id: usize,
    /// Position of the entity.
    pub position: [f32; 3],
    /// Half-extents for AABB queries (zero for point entities).
    pub half_extents: [f32; 3],
}

/// Statistics about the spatial hash grid.
#[allow(dead_code)]
pub struct GridStats {
    /// Number of occupied cells.
    pub cell_count: usize,
    /// Total number of entries.
    pub entry_count: usize,
    /// Maximum entries in a single cell.
    pub max_cell_occupancy: usize,
    /// Average entries per occupied cell.
    pub avg_cell_occupancy: f32,
}

/// The spatial hash grid structure.
#[allow(dead_code)]
pub struct SpatialHashGrid {
    cell_size: f32,
    inv_cell_size: f32,
    cells: HashMap<(i32, i32, i32), Vec<SpatialHashEntry>>,
    entry_total: usize,
}

// ── public API ───────────────────────────────────────────────────────────────

/// Returns a default spatial hash configuration.
#[allow(dead_code)]
pub fn default_spatial_hash_config() -> SpatialHashConfig {
    SpatialHashConfig {
        cell_size: 1.0,
        table_size: 1024,
    }
}

/// Create a new spatial hash grid with the given configuration.
#[allow(dead_code)]
pub fn new_spatial_hash(config: &SpatialHashConfig) -> SpatialHashGrid {
    SpatialHashGrid {
        cell_size: config.cell_size,
        inv_cell_size: 1.0 / config.cell_size,
        cells: HashMap::with_capacity(config.table_size),
        entry_total: 0,
    }
}

/// Insert a point entity into the grid.
#[allow(dead_code)]
pub fn insert_point(grid: &mut SpatialHashGrid, id: usize, position: [f32; 3]) {
    let key = hash_position_internal(position, grid.inv_cell_size);
    let entry = SpatialHashEntry {
        id,
        position,
        half_extents: [0.0, 0.0, 0.0],
    };
    grid.cells.entry(key).or_default().push(entry);
    grid.entry_total += 1;
}

/// Insert an AABB entity into the grid (inserted into all overlapping cells).
#[allow(dead_code)]
pub fn insert_aabb(
    grid: &mut SpatialHashGrid,
    id: usize,
    center: [f32; 3],
    half_extents: [f32; 3],
) {
    let entry = SpatialHashEntry {
        id,
        position: center,
        half_extents,
    };
    let min = [
        center[0] - half_extents[0],
        center[1] - half_extents[1],
        center[2] - half_extents[2],
    ];
    let max = [
        center[0] + half_extents[0],
        center[1] + half_extents[1],
        center[2] + half_extents[2],
    ];
    let min_cell = hash_position_internal(min, grid.inv_cell_size);
    let max_cell = hash_position_internal(max, grid.inv_cell_size);

    for iz in min_cell.2..=max_cell.2 {
        for iy in min_cell.1..=max_cell.1 {
            for ix in min_cell.0..=max_cell.0 {
                grid.cells
                    .entry((ix, iy, iz))
                    .or_default()
                    .push(entry.clone());
            }
        }
    }
    grid.entry_total += 1;
}

/// Query all entity IDs in the same cell as the given point.
#[allow(dead_code)]
pub fn query_point(grid: &SpatialHashGrid, position: [f32; 3]) -> Vec<usize> {
    let key = hash_position_internal(position, grid.inv_cell_size);
    match grid.cells.get(&key) {
        Some(entries) => entries.iter().map(|e| e.id).collect(),
        None => Vec::new(),
    }
}

/// Query all entity IDs within a given radius of the point.
#[allow(dead_code)]
pub fn query_radius(grid: &SpatialHashGrid, center: [f32; 3], radius: f32) -> Vec<usize> {
    let r2 = radius * radius;
    let min = [center[0] - radius, center[1] - radius, center[2] - radius];
    let max = [center[0] + radius, center[1] + radius, center[2] + radius];
    let min_cell = hash_position_internal(min, grid.inv_cell_size);
    let max_cell = hash_position_internal(max, grid.inv_cell_size);

    let mut result: Vec<usize> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for iz in min_cell.2..=max_cell.2 {
        for iy in min_cell.1..=max_cell.1 {
            for ix in min_cell.0..=max_cell.0 {
                if let Some(entries) = grid.cells.get(&(ix, iy, iz)) {
                    for e in entries {
                        if seen.insert(e.id) {
                            let dx = e.position[0] - center[0];
                            let dy = e.position[1] - center[1];
                            let dz = e.position[2] - center[2];
                            if dx * dx + dy * dy + dz * dz <= r2 {
                                result.push(e.id);
                            }
                        }
                    }
                }
            }
        }
    }
    result
}

/// Query all entity IDs whose entries overlap a given AABB.
#[allow(dead_code)]
pub fn query_aabb(grid: &SpatialHashGrid, aabb_min: [f32; 3], aabb_max: [f32; 3]) -> Vec<usize> {
    let min_cell = hash_position_internal(aabb_min, grid.inv_cell_size);
    let max_cell = hash_position_internal(aabb_max, grid.inv_cell_size);

    let mut result: Vec<usize> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for iz in min_cell.2..=max_cell.2 {
        for iy in min_cell.1..=max_cell.1 {
            for ix in min_cell.0..=max_cell.0 {
                if let Some(entries) = grid.cells.get(&(ix, iy, iz)) {
                    for e in entries {
                        if seen.insert(e.id) {
                            result.push(e.id);
                        }
                    }
                }
            }
        }
    }
    result
}

/// Clear all entries from the grid.
#[allow(dead_code)]
pub fn clear_grid(grid: &mut SpatialHashGrid) {
    grid.cells.clear();
    grid.entry_total = 0;
}

/// Return the number of occupied cells.
#[allow(dead_code)]
pub fn cell_count(grid: &SpatialHashGrid) -> usize {
    grid.cells.len()
}

/// Return the total number of entities inserted.
#[allow(dead_code)]
pub fn entry_count(grid: &SpatialHashGrid) -> usize {
    grid.entry_total
}

/// Compute the cell key for a given position.
#[allow(dead_code)]
pub fn hash_position(grid: &SpatialHashGrid, position: [f32; 3]) -> (i32, i32, i32) {
    hash_position_internal(position, grid.inv_cell_size)
}

/// Rebuild the grid from a list of point entries.
#[allow(dead_code)]
pub fn rebuild_grid(grid: &mut SpatialHashGrid, entries: &[(usize, [f32; 3])]) {
    clear_grid(grid);
    for &(id, pos) in entries {
        insert_point(grid, id, pos);
    }
}

/// Remove all entries with the given ID from the grid.
/// Returns the number of cells affected.
#[allow(dead_code)]
pub fn remove_entry(grid: &mut SpatialHashGrid, id: usize) -> usize {
    let mut affected = 0;
    for entries in grid.cells.values_mut() {
        let before = entries.len();
        entries.retain(|e| e.id != id);
        if entries.len() < before {
            affected += 1;
        }
    }
    if affected > 0 {
        grid.entry_total = grid.entry_total.saturating_sub(1);
    }
    // Remove empty cells
    grid.cells.retain(|_, v| !v.is_empty());
    affected
}

/// Compute statistics about the grid.
#[allow(dead_code)]
pub fn grid_stats(grid: &SpatialHashGrid) -> GridStats {
    let cell_count = grid.cells.len();
    let entry_count = grid.entry_total;
    let max_occ = grid.cells.values().map(|v| v.len()).max().unwrap_or(0);
    let avg_occ = if cell_count > 0 {
        grid.cells.values().map(|v| v.len()).sum::<usize>() as f32 / cell_count as f32
    } else {
        0.0
    };
    GridStats {
        cell_count,
        entry_count,
        max_cell_occupancy: max_occ,
        avg_cell_occupancy: avg_occ,
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn hash_position_internal(pos: [f32; 3], inv_cell_size: f32) -> (i32, i32, i32) {
    (
        (pos[0] * inv_cell_size).floor() as i32,
        (pos[1] * inv_cell_size).floor() as i32,
        (pos[2] * inv_cell_size).floor() as i32,
    )
}

// ─── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_grid() -> SpatialHashGrid {
        let cfg = default_spatial_hash_config();
        new_spatial_hash(&cfg)
    }

    #[test]
    fn test_default_config() {
        let cfg = default_spatial_hash_config();
        assert!((cfg.cell_size - 1.0).abs() < 1e-6);
        assert!(cfg.table_size > 0);
    }

    #[test]
    fn test_new_spatial_hash() {
        let grid = make_grid();
        assert_eq!(cell_count(&grid), 0);
        assert_eq!(entry_count(&grid), 0);
    }

    #[test]
    fn test_insert_point() {
        let mut grid = make_grid();
        insert_point(&mut grid, 0, [0.5, 0.5, 0.5]);
        assert_eq!(entry_count(&grid), 1);
        assert_eq!(cell_count(&grid), 1);
    }

    #[test]
    fn test_insert_multiple_same_cell() {
        let mut grid = make_grid();
        insert_point(&mut grid, 0, [0.1, 0.1, 0.1]);
        insert_point(&mut grid, 1, [0.2, 0.2, 0.2]);
        assert_eq!(entry_count(&grid), 2);
        assert_eq!(cell_count(&grid), 1);
    }

    #[test]
    fn test_insert_different_cells() {
        let mut grid = make_grid();
        insert_point(&mut grid, 0, [0.5, 0.5, 0.5]);
        insert_point(&mut grid, 1, [1.5, 0.5, 0.5]);
        assert_eq!(entry_count(&grid), 2);
        assert_eq!(cell_count(&grid), 2);
    }

    #[test]
    fn test_query_point_found() {
        let mut grid = make_grid();
        insert_point(&mut grid, 42, [0.5, 0.5, 0.5]);
        let result = query_point(&grid, [0.5, 0.5, 0.5]);
        assert!(result.contains(&42));
    }

    #[test]
    fn test_query_point_not_found() {
        let mut grid = make_grid();
        insert_point(&mut grid, 42, [0.5, 0.5, 0.5]);
        let result = query_point(&grid, [5.5, 5.5, 5.5]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_query_radius() {
        let mut grid = make_grid();
        insert_point(&mut grid, 0, [0.0, 0.0, 0.0]);
        insert_point(&mut grid, 1, [0.5, 0.0, 0.0]);
        insert_point(&mut grid, 2, [5.0, 0.0, 0.0]);
        let result = query_radius(&grid, [0.0, 0.0, 0.0], 1.0);
        assert!(result.contains(&0));
        assert!(result.contains(&1));
        assert!(!result.contains(&2));
    }

    #[test]
    fn test_query_aabb() {
        let mut grid = make_grid();
        insert_point(&mut grid, 0, [0.5, 0.5, 0.5]);
        insert_point(&mut grid, 1, [3.0, 3.0, 3.0]);
        let result = query_aabb(&grid, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert!(result.contains(&0));
        assert!(!result.contains(&1));
    }

    #[test]
    fn test_insert_aabb() {
        let mut grid = make_grid();
        insert_aabb(&mut grid, 0, [0.5, 0.5, 0.5], [0.5, 0.5, 0.5]);
        // Should cover cell (0,0,0)
        let result = query_point(&grid, [0.5, 0.5, 0.5]);
        assert!(result.contains(&0));
    }

    #[test]
    fn test_clear_grid() {
        let mut grid = make_grid();
        insert_point(&mut grid, 0, [0.5, 0.5, 0.5]);
        clear_grid(&mut grid);
        assert_eq!(entry_count(&grid), 0);
        assert_eq!(cell_count(&grid), 0);
    }

    #[test]
    fn test_hash_position() {
        let grid = make_grid();
        let key = hash_position(&grid, [0.5, 0.5, 0.5]);
        assert_eq!(key, (0, 0, 0));
        let key2 = hash_position(&grid, [1.5, 2.5, 3.5]);
        assert_eq!(key2, (1, 2, 3));
    }

    #[test]
    fn test_rebuild_grid() {
        let mut grid = make_grid();
        insert_point(&mut grid, 99, [10.0, 10.0, 10.0]);
        let entries = vec![(0, [0.0, 0.0, 0.0]), (1, [1.0, 1.0, 1.0])];
        rebuild_grid(&mut grid, &entries);
        assert_eq!(entry_count(&grid), 2);
    }

    #[test]
    fn test_remove_entry() {
        let mut grid = make_grid();
        insert_point(&mut grid, 0, [0.5, 0.5, 0.5]);
        insert_point(&mut grid, 1, [0.6, 0.6, 0.6]);
        let affected = remove_entry(&mut grid, 0);
        assert!(affected > 0);
        let result = query_point(&grid, [0.5, 0.5, 0.5]);
        assert!(!result.contains(&0));
        assert!(result.contains(&1));
    }

    #[test]
    fn test_grid_stats() {
        let mut grid = make_grid();
        insert_point(&mut grid, 0, [0.5, 0.5, 0.5]);
        insert_point(&mut grid, 1, [0.6, 0.6, 0.6]);
        insert_point(&mut grid, 2, [5.0, 5.0, 5.0]);
        let stats = grid_stats(&grid);
        assert_eq!(stats.entry_count, 3);
        assert_eq!(stats.cell_count, 2);
        assert_eq!(stats.max_cell_occupancy, 2);
    }

    #[test]
    fn test_negative_coordinates() {
        let mut grid = make_grid();
        insert_point(&mut grid, 0, [-0.5, -0.5, -0.5]);
        let key = hash_position(&grid, [-0.5, -0.5, -0.5]);
        assert_eq!(key, (-1, -1, -1));
        let result = query_point(&grid, [-0.3, -0.3, -0.3]);
        assert!(result.contains(&0));
    }
}
