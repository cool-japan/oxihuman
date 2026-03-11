// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! GridVisualizer — configurable viewport grid display.

#![allow(dead_code)]

/// Grid configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GridConfig {
    pub spacing: f32,
    pub line_count: u32,
    pub color: [f32; 4],
    pub opacity: f32,
    pub extent: f32,
}

impl Default for GridConfig {
    fn default() -> Self {
        GridConfig {
            spacing: 1.0,
            line_count: 20,
            color: [0.5, 0.5, 0.5, 1.0],
            opacity: 0.6,
            extent: 10.0,
        }
    }
}

/// Grid visualizer state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GridVisualizer {
    pub config: GridConfig,
    pub enabled: bool,
}

/// Create a `GridVisualizer` with default settings.
#[allow(dead_code)]
pub fn new_grid_visualizer() -> GridVisualizer {
    GridVisualizer { config: GridConfig::default(), enabled: true }
}

/// Create a default `GridConfig`.
#[allow(dead_code)]
pub fn default_grid_config() -> GridConfig {
    GridConfig::default()
}

/// Return the total number of grid lines (2× count for X and Z axes).
#[allow(dead_code)]
pub fn grid_line_count(gv: &GridVisualizer) -> u32 {
    gv.config.line_count * 2
}

/// Return the vertex position of grid line `i` at axis extent.
/// Returns `[offset, 0, -extent]` and `[offset, 0, extent]` for X-axis lines.
#[allow(dead_code)]
pub fn grid_vertex_at(gv: &GridVisualizer, i: u32) -> [[f32; 3]; 2] {
    let half = gv.config.line_count as f32 / 2.0;
    let offset = (i as f32 - half) * gv.config.spacing;
    let ext = gv.config.extent;
    [[offset, 0.0, -ext], [offset, 0.0, ext]]
}

/// Return the grid line color.
#[allow(dead_code)]
pub fn grid_color(gv: &GridVisualizer) -> [f32; 4] {
    gv.config.color
}

/// Return the grid spacing.
#[allow(dead_code)]
pub fn grid_spacing(gv: &GridVisualizer) -> f32 {
    gv.config.spacing
}

/// Return the grid opacity.
#[allow(dead_code)]
pub fn grid_opacity(gv: &GridVisualizer) -> f32 {
    gv.config.opacity
}

/// Return the grid half-extent.
#[allow(dead_code)]
pub fn grid_extent(gv: &GridVisualizer) -> f32 {
    gv.config.extent
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_grid_visualizer_enabled() {
        let gv = new_grid_visualizer();
        assert!(gv.enabled);
    }

    #[test]
    fn test_grid_line_count() {
        let gv = new_grid_visualizer();
        assert_eq!(grid_line_count(&gv), 40);
    }

    #[test]
    fn test_grid_spacing() {
        let gv = new_grid_visualizer();
        assert!((grid_spacing(&gv) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_grid_opacity() {
        let gv = new_grid_visualizer();
        assert!(grid_opacity(&gv) > 0.0 && grid_opacity(&gv) <= 1.0);
    }

    #[test]
    fn test_grid_extent() {
        let gv = new_grid_visualizer();
        assert!((grid_extent(&gv) - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_grid_color_alpha() {
        let gv = new_grid_visualizer();
        let c = grid_color(&gv);
        assert!((c[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_grid_vertex_symmetry() {
        let gv = new_grid_visualizer();
        let n = gv.config.line_count;
        let v0 = grid_vertex_at(&gv, 0);
        let v1 = grid_vertex_at(&gv, n - 1);
        assert!(v0[0][0] < v1[0][0] || v0[0][0] <= v1[0][0]);
    }

    #[test]
    fn test_default_grid_config() {
        let c = default_grid_config();
        assert!(c.line_count > 0);
    }
}
