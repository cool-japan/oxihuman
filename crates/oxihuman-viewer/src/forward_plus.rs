// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Forward+ rendering tile data.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ForwardPlusTile {
    pub lights: Vec<u32>,
    pub tile_x: u32,
    pub tile_y: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ForwardPlusGrid {
    pub tiles: Vec<ForwardPlusTile>,
    pub tile_size: u32,
    pub grid_w: u32,
    pub grid_h: u32,
}

#[allow(dead_code)]
pub fn new_forward_plus_grid(screen_w: u32, screen_h: u32, tile_size: u32) -> ForwardPlusGrid {
    let ts = tile_size.max(1);
    let gw = screen_w.div_ceil(ts);
    let gh = screen_h.div_ceil(ts);
    let mut tiles = Vec::with_capacity((gw * gh) as usize);
    for ty in 0..gh {
        for tx in 0..gw {
            tiles.push(ForwardPlusTile { lights: Vec::new(), tile_x: tx, tile_y: ty });
        }
    }
    ForwardPlusGrid { tiles, tile_size: ts, grid_w: gw, grid_h: gh }
}

#[allow(dead_code)]
pub fn fp_tile_count(grid: &ForwardPlusGrid) -> usize {
    grid.tiles.len()
}

#[allow(dead_code)]
pub fn fp_assign_light(grid: &mut ForwardPlusGrid, tile_idx: usize, light_id: u32) {
    if tile_idx < grid.tiles.len() {
        grid.tiles[tile_idx].lights.push(light_id);
    }
}

#[allow(dead_code)]
pub fn fp_lights_in_tile(grid: &ForwardPlusGrid, tile_idx: usize) -> &[u32] {
    if tile_idx < grid.tiles.len() {
        &grid.tiles[tile_idx].lights
    } else {
        &[]
    }
}

#[allow(dead_code)]
pub fn fp_max_lights_per_tile(grid: &ForwardPlusGrid) -> usize {
    grid.tiles.iter().map(|t| t.lights.len()).max().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_count_from_dimensions() {
        let g = new_forward_plus_grid(640, 480, 16);
        assert_eq!(fp_tile_count(&g), (40 * 30) as usize);
    }

    #[test]
    fn test_tile_count_non_divisible() {
        let g = new_forward_plus_grid(100, 100, 16);
        /* ceil(100/16) = 7, 7*7 = 49 */
        assert_eq!(fp_tile_count(&g), 49);
    }

    #[test]
    fn test_assign_light() {
        let mut g = new_forward_plus_grid(64, 64, 16);
        fp_assign_light(&mut g, 0, 5);
        assert_eq!(fp_lights_in_tile(&g, 0), &[5]);
    }

    #[test]
    fn test_lights_in_tile_empty() {
        let g = new_forward_plus_grid(64, 64, 16);
        assert_eq!(fp_lights_in_tile(&g, 0).len(), 0);
    }

    #[test]
    fn test_max_lights_per_tile() {
        let mut g = new_forward_plus_grid(64, 64, 16);
        fp_assign_light(&mut g, 0, 1);
        fp_assign_light(&mut g, 0, 2);
        fp_assign_light(&mut g, 1, 3);
        assert_eq!(fp_max_lights_per_tile(&g), 2);
    }

    #[test]
    fn test_max_lights_empty() {
        let g = new_forward_plus_grid(64, 64, 16);
        assert_eq!(fp_max_lights_per_tile(&g), 0);
    }

    #[test]
    fn test_tile_xy_set() {
        let g = new_forward_plus_grid(32, 32, 16);
        assert_eq!(g.tiles[0].tile_x, 0);
        assert_eq!(g.tiles[0].tile_y, 0);
    }

    #[test]
    fn test_out_of_range_tile_returns_empty() {
        let g = new_forward_plus_grid(64, 64, 16);
        assert_eq!(fp_lights_in_tile(&g, 9999).len(), 0);
    }
}
