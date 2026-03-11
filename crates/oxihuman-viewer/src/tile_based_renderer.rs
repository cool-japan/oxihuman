// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Tile-based deferred rendering parameters.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TileConfig {
    pub tile_width: u32,
    pub tile_height: u32,
    pub screen_width: u32,
    pub screen_height: u32,
}

#[allow(dead_code)]
pub fn new_tile_config(screen_w: u32, screen_h: u32, tile_w: u32, tile_h: u32) -> TileConfig {
    TileConfig {
        tile_width: tile_w.max(1),
        tile_height: tile_h.max(1),
        screen_width: screen_w,
        screen_height: screen_h,
    }
}

#[allow(dead_code)]
pub fn tc_tile_count_x(config: &TileConfig) -> u32 {
    config.screen_width.div_ceil(config.tile_width)
}

#[allow(dead_code)]
pub fn tc_tile_count_y(config: &TileConfig) -> u32 {
    config.screen_height.div_ceil(config.tile_height)
}

#[allow(dead_code)]
pub fn tc_total_tiles(config: &TileConfig) -> u32 {
    tc_tile_count_x(config) * tc_tile_count_y(config)
}

#[allow(dead_code)]
pub fn tc_tile_index(config: &TileConfig, pixel_x: u32, pixel_y: u32) -> u32 {
    let tx = pixel_x / config.tile_width;
    let ty = pixel_y / config.tile_height;
    ty * tc_tile_count_x(config) + tx
}

#[allow(dead_code)]
pub fn tc_tile_origin(config: &TileConfig, tile_idx: u32) -> (u32, u32) {
    let tiles_x = tc_tile_count_x(config);
    let tx = tile_idx % tiles_x;
    let ty = tile_idx / tiles_x;
    (tx * config.tile_width, ty * config.tile_height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_count_x_exact() {
        let c = new_tile_config(640, 480, 16, 16);
        assert_eq!(tc_tile_count_x(&c), 40);
    }

    #[test]
    fn test_tile_count_y_exact() {
        let c = new_tile_config(640, 480, 16, 16);
        assert_eq!(tc_tile_count_y(&c), 30);
    }

    #[test]
    fn test_tile_count_x_non_divisible() {
        let c = new_tile_config(100, 100, 16, 16);
        assert_eq!(tc_tile_count_x(&c), 7);
    }

    #[test]
    fn test_total_tiles() {
        let c = new_tile_config(640, 480, 16, 16);
        assert_eq!(tc_total_tiles(&c), 1200);
    }

    #[test]
    fn test_tile_index_origin() {
        let c = new_tile_config(640, 480, 16, 16);
        assert_eq!(tc_tile_index(&c, 0, 0), 0);
    }

    #[test]
    fn test_tile_index_second_col() {
        let c = new_tile_config(640, 480, 16, 16);
        assert_eq!(tc_tile_index(&c, 16, 0), 1);
    }

    #[test]
    fn test_tile_origin_first() {
        let c = new_tile_config(640, 480, 16, 16);
        assert_eq!(tc_tile_origin(&c, 0), (0, 0));
    }

    #[test]
    fn test_tile_origin_roundtrip() {
        let c = new_tile_config(640, 480, 16, 16);
        let idx = tc_tile_index(&c, 32, 16);
        let (ox, oy) = tc_tile_origin(&c, idx);
        assert_eq!(ox, 32);
        assert_eq!(oy, 16);
    }
}
