// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! 2D grid overlay for UI panels.

#[allow(dead_code)]
pub struct GridOverlayConfig {
    pub cell_width: f32,
    pub cell_height: f32,
    pub origin: [f32; 2],
    pub color: [f32; 4],
    pub major_every: u32,
    pub major_color: [f32; 4],
    pub visible: bool,
}

#[allow(dead_code)]
pub struct GridLine2D {
    pub start: [f32; 2],
    pub end: [f32; 2],
    pub color: [f32; 4],
    pub width: f32,
    pub is_major: bool,
}

#[allow(dead_code)]
pub struct GridOverlay {
    pub config: GridOverlayConfig,
    pub lines: Vec<GridLine2D>,
}

#[allow(dead_code)]
pub fn default_grid_overlay_config() -> GridOverlayConfig {
    GridOverlayConfig {
        cell_width: 32.0,
        cell_height: 32.0,
        origin: [0.0, 0.0],
        color: [0.5, 0.5, 0.5, 0.5],
        major_every: 5,
        major_color: [0.8, 0.8, 0.8, 0.8],
        visible: true,
    }
}

fn build_lines(
    cfg: &GridOverlayConfig,
    viewport_width: f32,
    viewport_height: f32,
) -> Vec<GridLine2D> {
    let mut lines = Vec::new();
    let ox = cfg.origin[0];
    let oy = cfg.origin[1];

    // Vertical lines
    let cols = (viewport_width / cfg.cell_width).ceil() as i32 + 1;
    for col in 0..cols {
        let x = ox + col as f32 * cfg.cell_width;
        let is_major = cfg.major_every > 0 && (col as u32).is_multiple_of(cfg.major_every);
        lines.push(GridLine2D {
            start: [x, 0.0],
            end: [x, viewport_height],
            color: if is_major { cfg.major_color } else { cfg.color },
            width: if is_major { 2.0 } else { 1.0 },
            is_major,
        });
    }

    // Horizontal lines
    let rows = (viewport_height / cfg.cell_height).ceil() as i32 + 1;
    for row in 0..rows {
        let y = oy + row as f32 * cfg.cell_height;
        let is_major = cfg.major_every > 0 && (row as u32).is_multiple_of(cfg.major_every);
        lines.push(GridLine2D {
            start: [0.0, y],
            end: [viewport_width, y],
            color: if is_major { cfg.major_color } else { cfg.color },
            width: if is_major { 2.0 } else { 1.0 },
            is_major,
        });
    }

    lines
}

#[allow(dead_code)]
pub fn build_grid_overlay(
    cfg: &GridOverlayConfig,
    viewport_width: f32,
    viewport_height: f32,
) -> GridOverlay {
    let lines = build_lines(cfg, viewport_width, viewport_height);
    GridOverlay {
        config: GridOverlayConfig {
            cell_width: cfg.cell_width,
            cell_height: cfg.cell_height,
            origin: cfg.origin,
            color: cfg.color,
            major_every: cfg.major_every,
            major_color: cfg.major_color,
            visible: cfg.visible,
        },
        lines,
    }
}

#[allow(dead_code)]
pub fn update_grid_overlay(overlay: &mut GridOverlay, viewport_w: f32, viewport_h: f32) {
    overlay.lines = build_lines(&overlay.config, viewport_w, viewport_h);
}

#[allow(dead_code)]
pub fn grid_line_count_2d(overlay: &GridOverlay) -> usize {
    overlay.lines.len()
}

#[allow(dead_code)]
pub fn major_line_count_2d(overlay: &GridOverlay) -> usize {
    overlay.lines.iter().filter(|l| l.is_major).count()
}

#[allow(dead_code)]
pub fn snap_to_grid_2d(pos: [f32; 2], cfg: &GridOverlayConfig) -> [f32; 2] {
    let ox = cfg.origin[0];
    let oy = cfg.origin[1];
    let sx = ((pos[0] - ox) / cfg.cell_width).round() * cfg.cell_width + ox;
    let sy = ((pos[1] - oy) / cfg.cell_height).round() * cfg.cell_height + oy;
    [sx, sy]
}

#[allow(dead_code)]
pub fn grid_cell_at_2d(pos: [f32; 2], cfg: &GridOverlayConfig) -> [i32; 2] {
    let cx = ((pos[0] - cfg.origin[0]) / cfg.cell_width).floor() as i32;
    let cy = ((pos[1] - cfg.origin[1]) / cfg.cell_height).floor() as i32;
    [cx, cy]
}

#[allow(dead_code)]
pub fn set_grid_visible(overlay: &mut GridOverlay, visible: bool) {
    overlay.config.visible = visible;
}

#[allow(dead_code)]
pub fn set_grid_color(overlay: &mut GridOverlay, color: [f32; 4]) {
    overlay.config.color = color;
    for line in overlay.lines.iter_mut() {
        if !line.is_major {
            line.color = color;
        }
    }
}

#[allow(dead_code)]
pub fn grid_spacing_pixels(cfg: &GridOverlayConfig) -> [f32; 2] {
    [cfg.cell_width, cfg.cell_height]
}

#[allow(dead_code)]
pub fn nearest_grid_point(pos: [f32; 2], cfg: &GridOverlayConfig) -> [f32; 2] {
    snap_to_grid_2d(pos, cfg)
}

/// Returns lines intersecting AABB [x, y, w, h].
#[allow(dead_code)]
pub fn grid_lines_in_rect(overlay: &GridOverlay, rect: [f32; 4]) -> Vec<&GridLine2D> {
    let rx = rect[0];
    let ry = rect[1];
    let rw = rect[2];
    let rh = rect[3];
    overlay
        .lines
        .iter()
        .filter(|line| {
            let min_x = line.start[0].min(line.end[0]);
            let max_x = line.start[0].max(line.end[0]);
            let min_y = line.start[1].min(line.end[1]);
            let max_y = line.start[1].max(line.end[1]);
            max_x >= rx && min_x <= rx + rw && max_y >= ry && min_y <= ry + rh
        })
        .collect()
}

// ── 3-D grid overlay spec API ─────────────────────────────────────────────

/// A single 3-D grid line with start/end in world space and colour.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GridLine {
    /// World-space start point.
    pub start: [f32; 3],
    /// World-space end point.
    pub end: [f32; 3],
    /// RGBA colour.
    pub color: [f32; 4],
}

/// Configuration for the 3-D viewport grid overlay.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GridOverlayConfig3D {
    /// Spacing between grid lines in world units.
    pub spacing: f32,
    /// Half-extent of the grid on each axis in world units.
    pub half_extent: f32,
    /// Default grid line colour (RGBA).
    pub color: [f32; 4],
    /// Origin (axis) line colour (RGBA).
    pub origin_line_color: [f32; 4],
    /// Whether the grid overlay is active.
    pub enabled: bool,
}

/// The draw call produced by [`build_grid_lines`] — contains all grid lines.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GridOverlayDrawCall {
    /// All grid lines to render.
    pub lines: Vec<GridLine>,
}

/// Returns a sensible default [`GridOverlayConfig3D`].
#[allow(dead_code)]
pub fn default_grid_overlay_config_3d() -> GridOverlayConfig3D {
    GridOverlayConfig3D {
        spacing: 1.0,
        half_extent: 10.0,
        color: [0.4, 0.4, 0.4, 1.0],
        origin_line_color: [0.8, 0.2, 0.2, 1.0],
        enabled: true,
    }
}

/// Build the full set of grid lines for the given config.
#[allow(dead_code)]
pub fn build_grid_lines(cfg: &GridOverlayConfig3D) -> GridOverlayDrawCall {
    let mut lines = Vec::new();
    if !cfg.enabled {
        return GridOverlayDrawCall { lines };
    }
    let steps = (cfg.half_extent / cfg.spacing.max(f32::EPSILON)).ceil() as i32;
    // Lines parallel to Z axis
    for i in -steps..=steps {
        let x = i as f32 * cfg.spacing;
        let color = if i == 0 {
            cfg.origin_line_color
        } else {
            cfg.color
        };
        lines.push(GridLine {
            start: [x, 0.0, -cfg.half_extent],
            end: [x, 0.0, cfg.half_extent],
            color,
        });
    }
    // Lines parallel to X axis
    for j in -steps..=steps {
        let z = j as f32 * cfg.spacing;
        let color = if j == 0 {
            cfg.origin_line_color
        } else {
            cfg.color
        };
        lines.push(GridLine {
            start: [-cfg.half_extent, 0.0, z],
            end: [cfg.half_extent, 0.0, z],
            color,
        });
    }
    GridOverlayDrawCall { lines }
}

/// Returns the number of lines in a draw call.
#[allow(dead_code)]
pub fn grid_line_count(call: &GridOverlayDrawCall) -> usize {
    call.lines.len()
}

/// Update the grid spacing.
#[allow(dead_code)]
pub fn set_grid_spacing(cfg: &mut GridOverlayConfig3D, spacing: f32) {
    cfg.spacing = spacing.max(f32::EPSILON);
}

/// Update the half-extent of the grid.
#[allow(dead_code)]
pub fn set_grid_extent(cfg: &mut GridOverlayConfig3D, half_extent: f32) {
    cfg.half_extent = half_extent.max(0.0);
}

/// Set the default grid line colour.
#[allow(dead_code)]
pub fn set_grid_color_3d(cfg: &mut GridOverlayConfig3D, r: f32, g: f32, b: f32, a: f32) {
    cfg.color = [r, g, b, a];
}

/// Returns `true` when the grid overlay is enabled.
#[allow(dead_code)]
pub fn grid_overlay_is_enabled(cfg: &GridOverlayConfig3D) -> bool {
    cfg.enabled
}

/// Enable or disable the grid overlay.
#[allow(dead_code)]
pub fn set_grid_overlay_enabled(cfg: &mut GridOverlayConfig3D, enabled: bool) {
    cfg.enabled = enabled;
}

/// Returns the origin line colour from the config.
#[allow(dead_code)]
pub fn grid_origin_line_color(cfg: &GridOverlayConfig3D) -> [f32; 4] {
    cfg.origin_line_color
}

/// Removes all lines from the draw call.
#[allow(dead_code)]
pub fn grid_overlay_clear(call: &mut GridOverlayDrawCall) {
    call.lines.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_grid_overlay_config();
        assert_eq!(cfg.cell_width, 32.0);
        assert_eq!(cfg.cell_height, 32.0);
        assert!(cfg.visible);
        assert_eq!(cfg.major_every, 5);
    }

    #[test]
    fn test_build_overlay_produces_lines() {
        let cfg = default_grid_overlay_config();
        let overlay = build_grid_overlay(&cfg, 320.0, 320.0);
        assert!(!overlay.lines.is_empty());
    }

    #[test]
    fn test_grid_line_count() {
        let cfg = default_grid_overlay_config();
        let overlay = build_grid_overlay(&cfg, 320.0, 320.0);
        assert!(grid_line_count_2d(&overlay) > 0);
    }

    #[test]
    fn test_major_line_count() {
        let cfg = default_grid_overlay_config();
        let overlay = build_grid_overlay(&cfg, 320.0, 320.0);
        let major = major_line_count_2d(&overlay);
        let total = grid_line_count_2d(&overlay);
        assert!(major < total);
        assert!(major > 0);
    }

    #[test]
    fn test_snap_to_grid() {
        let cfg = default_grid_overlay_config();
        let snapped = snap_to_grid_2d([50.0, 50.0], &cfg);
        assert!((snapped[0] - 64.0).abs() < 0.01 || (snapped[0] - 32.0).abs() < 0.01);
    }

    #[test]
    fn test_snap_to_grid_on_origin() {
        let cfg = default_grid_overlay_config();
        let snapped = snap_to_grid_2d([0.0, 0.0], &cfg);
        assert!((snapped[0] - 0.0).abs() < 0.01);
        assert!((snapped[1] - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_grid_cell_at() {
        let cfg = default_grid_overlay_config();
        let cell = grid_cell_at_2d([50.0, 70.0], &cfg);
        assert_eq!(cell[0], 1); // 50/32 = 1.56 => floor = 1
        assert_eq!(cell[1], 2); // 70/32 = 2.18 => floor = 2
    }

    #[test]
    fn test_set_visible() {
        let cfg = default_grid_overlay_config();
        let mut overlay = build_grid_overlay(&cfg, 320.0, 320.0);
        set_grid_visible(&mut overlay, false);
        assert!(!overlay.config.visible);
        set_grid_visible(&mut overlay, true);
        assert!(overlay.config.visible);
    }

    #[test]
    fn test_set_color() {
        let cfg = default_grid_overlay_config();
        let mut overlay = build_grid_overlay(&cfg, 320.0, 320.0);
        let new_color = [1.0, 0.0, 0.0, 1.0];
        set_grid_color(&mut overlay, new_color);
        assert_eq!(overlay.config.color, new_color);
    }

    #[test]
    fn test_grid_spacing_pixels() {
        let cfg = default_grid_overlay_config();
        let spacing = grid_spacing_pixels(&cfg);
        assert_eq!(spacing[0], 32.0);
        assert_eq!(spacing[1], 32.0);
    }

    #[test]
    fn test_nearest_grid_point() {
        let cfg = default_grid_overlay_config();
        let pt = nearest_grid_point([15.0, 15.0], &cfg);
        assert!((pt[0] - 0.0).abs() < 0.01 || (pt[0] - 32.0).abs() < 0.01);
    }

    #[test]
    fn test_grid_lines_in_rect() {
        let cfg = default_grid_overlay_config();
        let overlay = build_grid_overlay(&cfg, 320.0, 320.0);
        let lines = grid_lines_in_rect(&overlay, [0.0, 0.0, 100.0, 100.0]);
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_update_grid_overlay() {
        let cfg = default_grid_overlay_config();
        let mut overlay = build_grid_overlay(&cfg, 320.0, 320.0);
        let old_count = grid_line_count_2d(&overlay);
        update_grid_overlay(&mut overlay, 640.0, 640.0);
        let new_count = grid_line_count_2d(&overlay);
        assert!(new_count >= old_count);
    }

    // ── 3-D grid overlay API ─────────────────────────────────────────────────

    #[test]
    fn test_default_grid_overlay_config_3d() {
        let cfg = default_grid_overlay_config_3d();
        assert!(cfg.enabled);
        assert!((cfg.spacing - 1.0).abs() < 1e-5);
        assert!((cfg.half_extent - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_build_grid_lines_produces_lines() {
        let cfg = default_grid_overlay_config_3d();
        let call = build_grid_lines(&cfg);
        assert!(!call.lines.is_empty());
    }

    #[test]
    fn test_grid_line_count_nonempty() {
        let cfg = default_grid_overlay_config_3d();
        let call = build_grid_lines(&cfg);
        assert!(grid_line_count(&call) > 0);
    }

    #[test]
    fn test_build_grid_lines_disabled_returns_empty() {
        let mut cfg = default_grid_overlay_config_3d();
        cfg.enabled = false;
        let call = build_grid_lines(&cfg);
        assert_eq!(grid_line_count(&call), 0);
    }

    #[test]
    fn test_set_grid_spacing() {
        let mut cfg = default_grid_overlay_config_3d();
        set_grid_spacing(&mut cfg, 2.0);
        assert!((cfg.spacing - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_set_grid_extent() {
        let mut cfg = default_grid_overlay_config_3d();
        set_grid_extent(&mut cfg, 5.0);
        assert!((cfg.half_extent - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_set_grid_color_3d() {
        let mut cfg = default_grid_overlay_config_3d();
        set_grid_color_3d(&mut cfg, 0.1, 0.2, 0.3, 0.9);
        assert!((cfg.color[0] - 0.1).abs() < 1e-5);
        assert!((cfg.color[3] - 0.9).abs() < 1e-5);
    }

    #[test]
    fn test_grid_overlay_is_enabled() {
        let cfg = default_grid_overlay_config_3d();
        assert!(grid_overlay_is_enabled(&cfg));
    }

    #[test]
    fn test_set_grid_overlay_enabled() {
        let mut cfg = default_grid_overlay_config_3d();
        set_grid_overlay_enabled(&mut cfg, false);
        assert!(!grid_overlay_is_enabled(&cfg));
        set_grid_overlay_enabled(&mut cfg, true);
        assert!(grid_overlay_is_enabled(&cfg));
    }

    #[test]
    fn test_grid_origin_line_color() {
        let cfg = default_grid_overlay_config_3d();
        let col = grid_origin_line_color(&cfg);
        assert_eq!(col, cfg.origin_line_color);
    }

    #[test]
    fn test_grid_overlay_clear() {
        let cfg = default_grid_overlay_config_3d();
        let mut call = build_grid_lines(&cfg);
        assert!(!call.lines.is_empty());
        grid_overlay_clear(&mut call);
        assert_eq!(grid_line_count(&call), 0);
    }
}
