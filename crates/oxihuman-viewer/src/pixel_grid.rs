// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

//! Pixel grid overlay: draws a grid at the pixel level for texture inspection.

/// Configuration for the pixel grid.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PixelGridConfig {
    pub enabled: bool,
    pub zoom_threshold: f32,
    pub line_color: [f32; 4],
    pub line_width: f32,
}

#[allow(dead_code)]
pub fn default_pixel_grid_config() -> PixelGridConfig {
    PixelGridConfig {
        enabled: true,
        zoom_threshold: 4.0,
        line_color: [0.5, 0.5, 0.5, 0.5],
        line_width: 1.0,
    }
}

/// Check if the pixel grid should be visible at the given zoom level.
#[allow(dead_code)]
pub fn should_show_grid(cfg: &PixelGridConfig, zoom: f32) -> bool {
    cfg.enabled && zoom >= cfg.zoom_threshold
}

/// Number of grid lines for a viewport size and zoom.
#[allow(dead_code)]
pub fn grid_line_count(viewport_size: u32, zoom: f32) -> u32 {
    if zoom < 1.0 {
        return 0;
    }
    (viewport_size as f32 / zoom).ceil() as u32 + 1
}

/// Pixel coordinate snapped to the nearest grid line.
#[allow(dead_code)]
pub fn snap_to_pixel(coord: f32, zoom: f32) -> f32 {
    if zoom < 1e-6 {
        return coord;
    }
    (coord / zoom).round() * zoom
}

/// Alpha for a grid line based on distance from the line center.
#[allow(dead_code)]
pub fn grid_line_alpha(distance: f32, line_width: f32) -> f32 {
    if distance <= line_width * 0.5 {
        1.0
    } else {
        0.0
    }
}

#[allow(dead_code)]
pub fn set_grid_line_color(cfg: &mut PixelGridConfig, color: [f32; 4]) {
    cfg.line_color = color;
}

#[allow(dead_code)]
pub fn set_grid_line_width(cfg: &mut PixelGridConfig, width: f32) {
    cfg.line_width = width.clamp(0.1, 10.0);
}

#[allow(dead_code)]
pub fn set_zoom_threshold(cfg: &mut PixelGridConfig, threshold: f32) {
    cfg.zoom_threshold = threshold.max(1.0);
}

#[allow(dead_code)]
pub fn pixel_grid_to_json(cfg: &PixelGridConfig) -> String {
    format!(
        r#"{{"enabled":{},"zoom_threshold":{:.1},"line_width":{:.1}}}"#,
        cfg.enabled, cfg.zoom_threshold, cfg.line_width
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_pixel_grid_config();
        assert!(cfg.enabled);
        assert!((cfg.zoom_threshold - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_should_show_grid_below_threshold() {
        let cfg = default_pixel_grid_config();
        assert!(!should_show_grid(&cfg, 2.0));
    }

    #[test]
    fn test_should_show_grid_above_threshold() {
        let cfg = default_pixel_grid_config();
        assert!(should_show_grid(&cfg, 8.0));
    }

    #[test]
    fn test_should_show_grid_disabled() {
        let mut cfg = default_pixel_grid_config();
        cfg.enabled = false;
        assert!(!should_show_grid(&cfg, 8.0));
    }

    #[test]
    fn test_grid_line_count() {
        let c = grid_line_count(800, 10.0);
        assert!(c > 0);
    }

    #[test]
    fn test_grid_line_count_no_zoom() {
        assert_eq!(grid_line_count(800, 0.5), 0);
    }

    #[test]
    fn test_snap_to_pixel() {
        let s = snap_to_pixel(15.3, 10.0);
        assert!((s - 20.0).abs() < 1e-6);
    }

    #[test]
    fn test_grid_line_alpha_on_line() {
        assert!((grid_line_alpha(0.0, 1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_grid_line_alpha_off_line() {
        assert!(grid_line_alpha(2.0, 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_pixel_grid_to_json() {
        let cfg = default_pixel_grid_config();
        let j = pixel_grid_to_json(&cfg);
        assert!(j.contains("zoom_threshold"));
    }
}
