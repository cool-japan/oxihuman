// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! 3D grid rendering data with configurable cell size and line count.

/// 3D grid configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Grid3DConfig {
    /// Cell size in world units.
    pub cell_size: f32,
    /// Number of cells along each horizontal axis.
    pub cell_count: u32,
    /// Number of cells in the vertical direction (Y).
    pub vertical_count: u32,
    /// Grid line color [R, G, B, A].
    pub color: [f32; 4],
    /// Emphasis line interval (every N cells uses emphasis color).
    pub emphasis_interval: u32,
    /// Emphasis color [R, G, B, A].
    pub emphasis_color: [f32; 4],
    /// Whether to show the vertical grid.
    pub show_vertical: bool,
    /// Grid opacity 0..=1.
    pub opacity: f32,
}

impl Default for Grid3DConfig {
    fn default() -> Self {
        Self {
            cell_size: 1.0,
            cell_count: 10,
            vertical_count: 5,
            color: [0.5, 0.5, 0.5, 0.5],
            emphasis_interval: 5,
            emphasis_color: [0.7, 0.7, 0.7, 0.8],
            show_vertical: false,
            opacity: 0.5,
        }
    }
}

/// A single grid line segment.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GridLine3D {
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub is_emphasis: bool,
}

/// Create default grid config.
#[allow(dead_code)]
pub fn new_grid_3d_config() -> Grid3DConfig {
    Grid3DConfig::default()
}

/// Set grid cell size.
#[allow(dead_code)]
pub fn grid_set_cell_size(cfg: &mut Grid3DConfig, size: f32) {
    cfg.cell_size = size.max(1e-4);
}

/// Set cell count.
#[allow(dead_code)]
pub fn grid_set_cell_count(cfg: &mut Grid3DConfig, count: u32) {
    cfg.cell_count = count.clamp(1, 256);
}

/// Set opacity.
#[allow(dead_code)]
pub fn grid_set_opacity(cfg: &mut Grid3DConfig, opacity: f32) {
    cfg.opacity = opacity.clamp(0.0, 1.0);
}

/// Generate horizontal grid lines for the XZ plane.
#[allow(dead_code)]
pub fn generate_grid_lines_xz(cfg: &Grid3DConfig) -> Vec<GridLine3D> {
    let half = cfg.cell_size * cfg.cell_count as f32;
    let step = cfg.cell_size;
    let n = cfg.cell_count as i32;
    let mut lines = Vec::new();

    for i in -n..=n {
        let pos = i as f32 * step;
        let emph = (i % cfg.emphasis_interval as i32) == 0;
        lines.push(GridLine3D {
            start: [-half, 0.0, pos],
            end: [half, 0.0, pos],
            is_emphasis: emph,
        });
        lines.push(GridLine3D {
            start: [pos, 0.0, -half],
            end: [pos, 0.0, half],
            is_emphasis: emph,
        });
    }
    lines
}

/// Total number of grid lines generated.
#[allow(dead_code)]
pub fn grid_line_count(cfg: &Grid3DConfig) -> u32 {
    (cfg.cell_count * 2 + 1) * 2
}

/// Grid total extent in world units.
#[allow(dead_code)]
pub fn grid_extent(cfg: &Grid3DConfig) -> f32 {
    cfg.cell_size * cfg.cell_count as f32 * 2.0
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn grid_3d_to_json(cfg: &Grid3DConfig) -> String {
    format!(
        r#"{{"cell_size":{:.4},"cell_count":{},"opacity":{:.4},"emphasis_interval":{}}}"#,
        cfg.cell_size, cfg.cell_count, cfg.opacity, cfg.emphasis_interval
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let c = Grid3DConfig::default();
        assert!((c.cell_size - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_cell_size() {
        let mut c = Grid3DConfig::default();
        grid_set_cell_size(&mut c, 2.5);
        assert!((c.cell_size - 2.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_cell_size_min() {
        let mut c = Grid3DConfig::default();
        grid_set_cell_size(&mut c, -1.0);
        assert!(c.cell_size > 0.0);
    }

    #[test]
    fn test_set_cell_count_clamp() {
        let mut c = Grid3DConfig::default();
        grid_set_cell_count(&mut c, 1000);
        assert!(c.cell_count <= 256);
    }

    #[test]
    fn test_set_opacity_clamp() {
        let mut c = Grid3DConfig::default();
        grid_set_opacity(&mut c, 5.0);
        assert!((c.opacity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_generate_lines_not_empty() {
        let c = Grid3DConfig::default();
        let lines = generate_grid_lines_xz(&c);
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_generate_lines_emphasis() {
        let c = Grid3DConfig::default();
        let lines = generate_grid_lines_xz(&c);
        assert!(lines.iter().any(|l| l.is_emphasis));
    }

    #[test]
    fn test_grid_line_count() {
        let c = Grid3DConfig::default();
        assert_eq!(grid_line_count(&c), (c.cell_count * 2 + 1) * 2);
    }

    #[test]
    fn test_grid_extent() {
        let c = Grid3DConfig::default();
        let ext = grid_extent(&c);
        assert!((ext - 20.0).abs() < 1e-5);
    }

    #[test]
    fn test_to_json() {
        let j = grid_3d_to_json(&Grid3DConfig::default());
        assert!(j.contains("cell_size"));
        assert!(j.contains("opacity"));
    }
}
