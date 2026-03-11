// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Screen-space tile-based cluster assignment for forward+ rendering.

/// Cluster tile grid configuration.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ClusterTileConfig {
    pub tile_size_px: u32,
    pub num_depth_slices: u32,
    pub screen_width: u32,
    pub screen_height: u32,
}

impl Default for ClusterTileConfig {
    fn default() -> Self {
        Self {
            tile_size_px: 16,
            num_depth_slices: 24,
            screen_width: 1280,
            screen_height: 720,
        }
    }
}

/// A single cluster tile entry.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ClusterTileEntry {
    pub tile_x: u32,
    pub tile_y: u32,
    pub depth_slice: u32,
    pub light_count: u32,
    pub light_offset: u32,
}

/// Cluster tile grid.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClusterTileGrid {
    pub config: ClusterTileConfig,
    tiles_x: u32,
    tiles_y: u32,
    entries: Vec<ClusterTileEntry>,
}

#[allow(dead_code)]
pub fn new_cluster_tile_grid(config: ClusterTileConfig) -> ClusterTileGrid {
    let tiles_x = config.screen_width.div_ceil(config.tile_size_px);
    let tiles_y = config.screen_height.div_ceil(config.tile_size_px);
    let total = (tiles_x * tiles_y * config.num_depth_slices) as usize;
    let entries = (0..total)
        .map(|i| {
            let per_layer = (tiles_x * tiles_y) as usize;
            let depth_slice = (i / per_layer) as u32;
            let xy = i % per_layer;
            ClusterTileEntry {
                tile_x: (xy as u32) % tiles_x,
                tile_y: (xy as u32) / tiles_x,
                depth_slice,
                light_count: 0,
                light_offset: 0,
            }
        })
        .collect();
    ClusterTileGrid {
        config,
        tiles_x,
        tiles_y,
        entries,
    }
}

#[allow(dead_code)]
pub fn ctg_tile_count(grid: &ClusterTileGrid) -> usize {
    grid.entries.len()
}

#[allow(dead_code)]
pub fn ctg_tiles_x(grid: &ClusterTileGrid) -> u32 {
    grid.tiles_x
}

#[allow(dead_code)]
pub fn ctg_tiles_y(grid: &ClusterTileGrid) -> u32 {
    grid.tiles_y
}

/// Flat index into the cluster grid.
#[allow(dead_code)]
pub fn ctg_flat_index(grid: &ClusterTileGrid, tx: u32, ty: u32, depth: u32) -> Option<usize> {
    if tx >= grid.tiles_x || ty >= grid.tiles_y || depth >= grid.config.num_depth_slices {
        return None;
    }
    let per_layer = grid.tiles_x * grid.tiles_y;
    Some((depth * per_layer + ty * grid.tiles_x + tx) as usize)
}

#[allow(dead_code)]
pub fn ctg_set_light_count(grid: &mut ClusterTileGrid, tx: u32, ty: u32, depth: u32, count: u32) {
    if let Some(idx) = ctg_flat_index(grid, tx, ty, depth) {
        grid.entries[idx].light_count = count;
    }
}

#[allow(dead_code)]
pub fn ctg_light_count(grid: &ClusterTileGrid, tx: u32, ty: u32, depth: u32) -> u32 {
    ctg_flat_index(grid, tx, ty, depth)
        .map(|i| grid.entries[i].light_count)
        .unwrap_or(0)
}

#[allow(dead_code)]
pub fn ctg_clear(grid: &mut ClusterTileGrid) {
    for e in grid.entries.iter_mut() {
        e.light_count = 0;
        e.light_offset = 0;
    }
}

#[allow(dead_code)]
pub fn ctg_total_light_assignments(grid: &ClusterTileGrid) -> u32 {
    grid.entries.iter().map(|e| e.light_count).sum()
}

#[allow(dead_code)]
pub fn ctg_to_json(grid: &ClusterTileGrid) -> String {
    format!(
        "{{\"tiles_x\":{},\"tiles_y\":{},\"depth_slices\":{},\"total_tiles\":{}}}",
        grid.tiles_x,
        grid.tiles_y,
        grid.config.num_depth_slices,
        grid.entries.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_grid() -> ClusterTileGrid {
        new_cluster_tile_grid(ClusterTileConfig::default())
    }

    #[test]
    fn tile_count_matches_dimensions() {
        let g = make_grid();
        let expected = ctg_tiles_x(&g) * ctg_tiles_y(&g) * g.config.num_depth_slices;
        assert_eq!(ctg_tile_count(&g), expected as usize);
    }

    #[test]
    fn tiles_x_matches_config() {
        let g = make_grid();
        let cfg = &g.config;
        let expected = cfg.screen_width.div_ceil(cfg.tile_size_px);
        assert_eq!(ctg_tiles_x(&g), expected);
    }

    #[test]
    fn flat_index_valid() {
        let g = make_grid();
        assert!(ctg_flat_index(&g, 0, 0, 0).is_some());
    }

    #[test]
    fn flat_index_out_of_bounds() {
        let g = make_grid();
        assert!(ctg_flat_index(&g, 9999, 0, 0).is_none());
    }

    #[test]
    fn set_and_get_light_count() {
        let mut g = make_grid();
        ctg_set_light_count(&mut g, 0, 0, 0, 5);
        assert_eq!(ctg_light_count(&g, 0, 0, 0), 5);
    }

    #[test]
    fn clear_resets_lights() {
        let mut g = make_grid();
        ctg_set_light_count(&mut g, 0, 0, 0, 3);
        ctg_clear(&mut g);
        assert_eq!(ctg_total_light_assignments(&g), 0);
    }

    #[test]
    fn total_assignments_sums() {
        let mut g = make_grid();
        ctg_set_light_count(&mut g, 0, 0, 0, 2);
        ctg_set_light_count(&mut g, 1, 0, 0, 3);
        assert_eq!(ctg_total_light_assignments(&g), 5);
    }

    #[test]
    fn json_has_tiles_x() {
        let g = make_grid();
        let j = ctg_to_json(&g);
        assert!(j.contains("tiles_x"));
    }

    #[test]
    fn light_count_zero_oob() {
        let g = make_grid();
        assert_eq!(ctg_light_count(&g, 9999, 0, 0), 0);
    }

    #[test]
    fn entry_depth_slice_correct() {
        let g = make_grid();
        // First entry of second depth slice
        let per_layer = (ctg_tiles_x(&g) * ctg_tiles_y(&g)) as usize;
        assert_eq!(g.entries[per_layer].depth_slice, 1);
    }
}
