// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Grid axis renderer for viewport coordinate axes.

/// Which axes to display.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GridAxisPlane {
    XY,
    XZ,
    YZ,
}

/// Grid axis configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GridAxisConfig {
    pub plane: GridAxisPlane,
    pub spacing: f32,
    pub subdivisions: u32,
    pub axis_color_x: [f32; 3],
    pub axis_color_y: [f32; 3],
    pub axis_color_z: [f32; 3],
    pub grid_color: [f32; 4],
    pub visible: bool,
}

#[allow(dead_code)]
pub fn default_grid_axis_config() -> GridAxisConfig {
    GridAxisConfig {
        plane: GridAxisPlane::XZ,
        spacing: 1.0,
        subdivisions: 10,
        axis_color_x: [1.0, 0.2, 0.2],
        axis_color_y: [0.2, 1.0, 0.2],
        axis_color_z: [0.2, 0.2, 1.0],
        grid_color: [0.5, 0.5, 0.5, 0.3],
        visible: true,
    }
}

#[allow(dead_code)]
pub fn set_grid_plane(cfg: &mut GridAxisConfig, plane: GridAxisPlane) {
    cfg.plane = plane;
}

#[allow(dead_code)]
pub fn set_grid_spacing(cfg: &mut GridAxisConfig, spacing: f32) {
    cfg.spacing = spacing.max(0.01);
}

#[allow(dead_code)]
pub fn set_grid_subdivisions(cfg: &mut GridAxisConfig, count: u32) {
    cfg.subdivisions = count.clamp(1, 100);
}

#[allow(dead_code)]
pub fn toggle_grid_visibility(cfg: &mut GridAxisConfig) {
    cfg.visible = !cfg.visible;
}

#[allow(dead_code)]
pub fn grid_line_count(cfg: &GridAxisConfig) -> u32 {
    cfg.subdivisions * 2 + 1
}

#[allow(dead_code)]
pub fn grid_extent(cfg: &GridAxisConfig) -> f32 {
    cfg.spacing * cfg.subdivisions as f32
}

#[allow(dead_code)]
pub fn subdivision_spacing(cfg: &GridAxisConfig) -> f32 {
    cfg.spacing
}

#[allow(dead_code)]
pub fn set_grid_color(cfg: &mut GridAxisConfig, r: f32, g: f32, b: f32, a: f32) {
    cfg.grid_color = [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0), a.clamp(0.0, 1.0)];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_grid_axis_config();
        assert_eq!(cfg.plane, GridAxisPlane::XZ);
        assert!(cfg.visible);
    }

    #[test]
    fn test_set_plane() {
        let mut cfg = default_grid_axis_config();
        set_grid_plane(&mut cfg, GridAxisPlane::XY);
        assert_eq!(cfg.plane, GridAxisPlane::XY);
    }

    #[test]
    fn test_set_spacing() {
        let mut cfg = default_grid_axis_config();
        set_grid_spacing(&mut cfg, 2.0);
        assert!((cfg.spacing - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_spacing_min() {
        let mut cfg = default_grid_axis_config();
        set_grid_spacing(&mut cfg, -1.0);
        assert!((cfg.spacing - 0.01).abs() < 1e-6);
    }

    #[test]
    fn test_set_subdivisions_clamp() {
        let mut cfg = default_grid_axis_config();
        set_grid_subdivisions(&mut cfg, 0);
        assert_eq!(cfg.subdivisions, 1);
    }

    #[test]
    fn test_toggle_visibility() {
        let mut cfg = default_grid_axis_config();
        toggle_grid_visibility(&mut cfg);
        assert!(!cfg.visible);
    }

    #[test]
    fn test_grid_line_count() {
        let cfg = default_grid_axis_config();
        assert_eq!(grid_line_count(&cfg), 21);
    }

    #[test]
    fn test_grid_extent() {
        let cfg = default_grid_axis_config();
        assert!((grid_extent(&cfg) - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_subdivision_spacing() {
        let cfg = default_grid_axis_config();
        assert!((subdivision_spacing(&cfg) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_grid_color_clamp() {
        let mut cfg = default_grid_axis_config();
        set_grid_color(&mut cfg, 2.0, -1.0, 0.5, 0.5);
        assert!((cfg.grid_color[0] - 1.0).abs() < 1e-6);
        assert!(cfg.grid_color[1].abs() < 1e-6);
    }
}
