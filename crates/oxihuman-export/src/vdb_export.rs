// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! OpenVDB volume stub export.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VdbConfig {
    pub grid_name: String,
    pub voxel_size: f32,
    pub background_value: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VdbGrid {
    pub config: VdbConfig,
    pub active_voxels: Vec<([i32; 3], f32)>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VdbExportResult {
    pub voxel_count: usize,
    pub min_value: f32,
    pub max_value: f32,
}

#[allow(dead_code)]
pub fn default_vdb_config() -> VdbConfig {
    VdbConfig {
        grid_name: "density".to_string(),
        voxel_size: 0.1,
        background_value: 0.0,
    }
}

#[allow(dead_code)]
pub fn new_vdb_grid(config: VdbConfig) -> VdbGrid {
    VdbGrid {
        config,
        active_voxels: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn vdb_set_voxel(grid: &mut VdbGrid, coord: [i32; 3], value: f32) {
    if let Some(entry) = grid.active_voxels.iter_mut().find(|(c, _)| *c == coord) {
        entry.1 = value;
    } else {
        grid.active_voxels.push((coord, value));
    }
}

#[allow(dead_code)]
pub fn vdb_get_voxel(grid: &VdbGrid, coord: [i32; 3]) -> f32 {
    grid.active_voxels
        .iter()
        .find(|(c, _)| *c == coord)
        .map(|(_, v)| *v)
        .unwrap_or(grid.config.background_value)
}

#[allow(dead_code)]
pub fn vdb_active_count(grid: &VdbGrid) -> usize {
    grid.active_voxels.len()
}

#[allow(dead_code)]
pub fn vdb_clear(grid: &mut VdbGrid) {
    grid.active_voxels.clear();
}

#[allow(dead_code)]
pub fn vdb_stats(grid: &VdbGrid) -> VdbExportResult {
    if grid.active_voxels.is_empty() {
        return VdbExportResult {
            voxel_count: 0,
            min_value: grid.config.background_value,
            max_value: grid.config.background_value,
        };
    }
    let values: Vec<f32> = grid.active_voxels.iter().map(|(_, v)| *v).collect();
    let min_value = values.iter().cloned().fold(f32::MAX, f32::min);
    let max_value = values.iter().cloned().fold(f32::MIN, f32::max);
    VdbExportResult {
        voxel_count: grid.active_voxels.len(),
        min_value,
        max_value,
    }
}

#[allow(dead_code)]
pub fn vdb_export_to_bytes(grid: &VdbGrid) -> Vec<u8> {
    // Stub: encode as simple binary (coord + value per active voxel).
    let mut bytes = Vec::with_capacity(grid.active_voxels.len() * 16);
    for (coord, value) in &grid.active_voxels {
        for &c in coord {
            bytes.extend_from_slice(&c.to_le_bytes());
        }
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

#[allow(dead_code)]
pub fn vdb_to_json(grid: &VdbGrid) -> String {
    let stats = vdb_stats(grid);
    format!(
        "{{\"grid_name\":\"{}\",\"voxel_size\":{},\"active_voxels\":{},\"min\":{},\"max\":{}}}",
        grid.config.grid_name,
        grid.config.voxel_size,
        stats.voxel_count,
        stats.min_value,
        stats.max_value
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_vdb_config();
        assert_eq!(cfg.grid_name, "density");
    }

    #[test]
    fn test_set_get_voxel() {
        let mut grid = new_vdb_grid(default_vdb_config());
        vdb_set_voxel(&mut grid, [0, 0, 0], 1.0);
        assert!((vdb_get_voxel(&grid, [0, 0, 0]) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_get_background() {
        let grid = new_vdb_grid(default_vdb_config());
        assert!((vdb_get_voxel(&grid, [5, 5, 5]) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_active_count() {
        let mut grid = new_vdb_grid(default_vdb_config());
        vdb_set_voxel(&mut grid, [0, 0, 0], 1.0);
        vdb_set_voxel(&mut grid, [1, 0, 0], 0.5);
        assert_eq!(vdb_active_count(&grid), 2);
    }

    #[test]
    fn test_set_voxel_updates() {
        let mut grid = new_vdb_grid(default_vdb_config());
        vdb_set_voxel(&mut grid, [0, 0, 0], 1.0);
        vdb_set_voxel(&mut grid, [0, 0, 0], 2.0);
        assert_eq!(vdb_active_count(&grid), 1);
        assert!((vdb_get_voxel(&grid, [0, 0, 0]) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_clear() {
        let mut grid = new_vdb_grid(default_vdb_config());
        vdb_set_voxel(&mut grid, [0, 0, 0], 1.0);
        vdb_clear(&mut grid);
        assert_eq!(vdb_active_count(&grid), 0);
    }

    #[test]
    fn test_stats() {
        let mut grid = new_vdb_grid(default_vdb_config());
        vdb_set_voxel(&mut grid, [0, 0, 0], 0.2);
        vdb_set_voxel(&mut grid, [1, 0, 0], 0.8);
        let stats = vdb_stats(&grid);
        assert!((stats.min_value - 0.2).abs() < 1e-5);
        assert!((stats.max_value - 0.8).abs() < 1e-5);
    }

    #[test]
    fn test_export_bytes() {
        let mut grid = new_vdb_grid(default_vdb_config());
        vdb_set_voxel(&mut grid, [0, 0, 0], 1.0);
        let bytes = vdb_export_to_bytes(&grid);
        assert_eq!(bytes.len(), 16); // 3*4 + 4 bytes
    }

    #[test]
    fn test_to_json() {
        let grid = new_vdb_grid(default_vdb_config());
        let j = vdb_to_json(&grid);
        assert!(j.contains("grid_name"));
    }
}
