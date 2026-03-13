// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Per-tile frustum debug visualization for tiled lighting.

/// Configuration for tile frustum view.
#[derive(Debug, Clone)]
pub struct TileFrustumViewConfig {
    pub tile_size_px: u32,
    pub show_empty_tiles: bool,
    pub opacity: f32,
}

impl Default for TileFrustumViewConfig {
    fn default() -> Self {
        Self { tile_size_px: 16, show_empty_tiles: false, opacity: 0.4 }
    }
}

/// State for tile frustum visualization.
#[derive(Debug, Clone)]
pub struct TileFrustumView {
    pub config: TileFrustumViewConfig,
    pub enabled: bool,
}

impl Default for TileFrustumView {
    fn default() -> Self {
        Self { config: TileFrustumViewConfig::default(), enabled: false }
    }
}

/// Enable the tile frustum view.
pub fn tfv_enable(view: &mut TileFrustumView) {
    view.enabled = true;
}

/// Disable the tile frustum view.
pub fn tfv_disable(view: &mut TileFrustumView) {
    view.enabled = false;
}

/// Set tile size in pixels.
pub fn tfv_set_tile_size(view: &mut TileFrustumView, size: u32) {
    view.config.tile_size_px = size.max(1);
}

/// Return the tile index for a screen pixel position.
pub fn tfv_tile_index(px: u32, py: u32, tile_size: u32) -> (u32, u32) {
    (px / tile_size.max(1), py / tile_size.max(1))
}

/// Return a colour for the tile based on occupancy (0.0–1.0).
pub fn tfv_tile_color(occupancy: f32, config: &TileFrustumViewConfig) -> [f32; 4] {
    let o = occupancy.clamp(0.0, 1.0);
    [o, 1.0 - o, 0.2, config.opacity]
}

/// Return whether a tile with the given occupancy should be rendered.
pub fn tfv_should_render_tile(occupancy: f32, config: &TileFrustumViewConfig) -> bool {
    config.show_empty_tiles || occupancy > 0.0
}

/// Export config to JSON string (stub).
pub fn tfv_to_json(view: &TileFrustumView) -> String {
    format!(
        r#"{{"tile_size_px":{},"show_empty":{},"enabled":{}}}"#,
        view.config.tile_size_px, view.config.show_empty_tiles, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_disabled() {
        /* default should be disabled */
        let v = TileFrustumView::default();
        assert!(!v.enabled);
    }

    #[test]
    fn test_enable_disable() {
        /* enable/disable should toggle */
        let mut v = TileFrustumView::default();
        tfv_enable(&mut v);
        assert!(v.enabled);
        tfv_disable(&mut v);
        assert!(!v.enabled);
    }

    #[test]
    fn test_set_tile_size() {
        /* tile size should be stored */
        let mut v = TileFrustumView::default();
        tfv_set_tile_size(&mut v, 32);
        assert_eq!(v.config.tile_size_px, 32);
    }

    #[test]
    fn test_tile_size_min() {
        /* tile size should be at least 1 */
        let mut v = TileFrustumView::default();
        tfv_set_tile_size(&mut v, 0);
        assert_eq!(v.config.tile_size_px, 1);
    }

    #[test]
    fn test_tile_index() {
        /* tile index should divide by tile size */
        let (tx, ty) = tfv_tile_index(64, 48, 16);
        assert_eq!(tx, 4);
        assert_eq!(ty, 3);
    }

    #[test]
    fn test_tile_color_range() {
        /* tile color components should be in [0, 1] range */
        let cfg = TileFrustumViewConfig::default();
        let c = tfv_tile_color(0.5, &cfg);
        for ch in &c {
            assert!((0.0..=1.0).contains(ch));
        }
    }

    #[test]
    fn test_should_render_empty_false() {
        /* empty tiles should not render when show_empty is false */
        let cfg = TileFrustumViewConfig { show_empty_tiles: false, ..Default::default() };
        assert!(!tfv_should_render_tile(0.0, &cfg));
    }

    #[test]
    fn test_should_render_empty_true() {
        /* empty tiles should render when show_empty is true */
        let cfg = TileFrustumViewConfig { show_empty_tiles: true, ..Default::default() };
        assert!(tfv_should_render_tile(0.0, &cfg));
    }

    #[test]
    fn test_to_json_enabled() {
        /* JSON should contain enabled field */
        let mut v = TileFrustumView::default();
        tfv_enable(&mut v);
        let json = tfv_to_json(&v);
        assert!(json.contains("true"));
    }
}
