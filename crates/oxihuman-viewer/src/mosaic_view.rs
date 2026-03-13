// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Mosaic / tile blur view stub.

/// Tile shape for mosaic.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TileShape {
    Square,
    Hexagon,
    Triangle,
    Diamond,
}

/// Mosaic view configuration.
#[derive(Debug, Clone)]
pub struct MosaicView {
    pub tile_shape: TileShape,
    pub tile_size: u32,
    pub border_width: f32,
    pub border_color: [f32; 4],
    pub enabled: bool,
}

impl MosaicView {
    pub fn new() -> Self {
        MosaicView {
            tile_shape: TileShape::Square,
            tile_size: 16,
            border_width: 0.0,
            border_color: [0.0, 0.0, 0.0, 1.0],
            enabled: true,
        }
    }
}

impl Default for MosaicView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new mosaic view.
pub fn new_mosaic_view() -> MosaicView {
    MosaicView::new()
}

/// Set tile shape.
pub fn msv_set_tile_shape(view: &mut MosaicView, shape: TileShape) {
    view.tile_shape = shape;
}

/// Set tile size in pixels.
pub fn msv_set_tile_size(view: &mut MosaicView, size: u32) {
    view.tile_size = size.clamp(2, 256);
}

/// Set border width.
pub fn msv_set_border_width(view: &mut MosaicView, width: f32) {
    view.border_width = width.clamp(0.0, 8.0);
}

/// Set border color RGBA.
pub fn msv_set_border_color(view: &mut MosaicView, r: f32, g: f32, b: f32, a: f32) {
    view.border_color = [
        r.clamp(0.0, 1.0),
        g.clamp(0.0, 1.0),
        b.clamp(0.0, 1.0),
        a.clamp(0.0, 1.0),
    ];
}

/// Enable or disable.
pub fn msv_set_enabled(view: &mut MosaicView, enabled: bool) {
    view.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn msv_to_json(view: &MosaicView) -> String {
    let shape = match view.tile_shape {
        TileShape::Square => "square",
        TileShape::Hexagon => "hexagon",
        TileShape::Triangle => "triangle",
        TileShape::Diamond => "diamond",
    };
    format!(
        r#"{{"tile_shape":"{}","tile_size":{},"border_width":{},"enabled":{}}}"#,
        shape, view.tile_size, view.border_width, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_tile_shape() {
        let v = new_mosaic_view();
        assert_eq!(
            v.tile_shape,
            TileShape::Square /* default tile_shape must be Square */
        );
    }

    #[test]
    fn test_set_tile_shape() {
        let mut v = new_mosaic_view();
        msv_set_tile_shape(&mut v, TileShape::Hexagon);
        assert_eq!(
            v.tile_shape,
            TileShape::Hexagon /* tile_shape must be set */
        );
    }

    #[test]
    fn test_tile_size_clamped_min() {
        let mut v = new_mosaic_view();
        msv_set_tile_size(&mut v, 0);
        assert_eq!(v.tile_size, 2 /* tile_size clamped to 2 */);
    }

    #[test]
    fn test_tile_size_clamped_max() {
        let mut v = new_mosaic_view();
        msv_set_tile_size(&mut v, 1000);
        assert_eq!(v.tile_size, 256 /* tile_size clamped to 256 */);
    }

    #[test]
    fn test_border_width_clamped() {
        let mut v = new_mosaic_view();
        msv_set_border_width(&mut v, 100.0);
        assert!((v.border_width - 8.0).abs() < 1e-6 /* border_width clamped to 8.0 */);
    }

    #[test]
    fn test_set_border_color() {
        let mut v = new_mosaic_view();
        msv_set_border_color(&mut v, 1.0, 1.0, 1.0, 0.5);
        assert!((v.border_color[3] - 0.5).abs() < 1e-6 /* border_color alpha must be 0.5 */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_mosaic_view();
        msv_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_tile_shape() {
        let v = new_mosaic_view();
        let j = msv_to_json(&v);
        assert!(j.contains("\"tile_shape\"") /* JSON must have tile_shape */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_mosaic_view();
        assert!(v.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_default_tile_size() {
        let v = new_mosaic_view();
        assert_eq!(v.tile_size, 16 /* default tile_size must be 16 */);
    }
}
