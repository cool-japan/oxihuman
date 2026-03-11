// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Configurable grid rendering overlay for the viewport.

#[allow(dead_code)]
pub struct GridLine {
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub color: [f32; 4],
    pub width: f32,
}

#[allow(dead_code)]
pub struct GridConfig {
    pub spacing: f32,
    pub extent: f32,
    pub subdivisions: u32,
    pub major_color: [f32; 4],
    pub minor_color: [f32; 4],
    pub origin_color: [f32; 4],
    pub axis_x_color: [f32; 4],
    pub axis_z_color: [f32; 4],
    pub show_origin: bool,
    pub show_axes: bool,
    pub plane: GridPlane,
}

#[allow(dead_code)]
pub enum GridPlane {
    XZ,
    XY,
    YZ,
}

#[allow(dead_code)]
pub struct ViewportGrid {
    pub config: GridConfig,
    pub lines: Vec<GridLine>,
}

#[allow(dead_code)]
pub fn default_grid_config() -> GridConfig {
    GridConfig {
        spacing: 1.0,
        extent: 10.0,
        subdivisions: 5,
        major_color: [0.4, 0.4, 0.4, 1.0],
        minor_color: [0.2, 0.2, 0.2, 1.0],
        origin_color: [0.8, 0.8, 0.8, 1.0],
        axis_x_color: [1.0, 0.2, 0.2, 1.0],
        axis_z_color: [0.2, 0.2, 1.0, 1.0],
        show_origin: true,
        show_axes: true,
        plane: GridPlane::XZ,
    }
}

/// Generate a grid line in the appropriate plane.
fn make_line(a: f32, b_neg: f32, b_pos: f32, is_axis_a: bool, config: &GridConfig) -> GridLine {
    let color = if is_axis_a {
        config.axis_x_color
    } else {
        config.major_color
    };
    let (start, end) = match config.plane {
        GridPlane::XZ => ([a, 0.0, b_neg], [a, 0.0, b_pos]),
        GridPlane::XY => ([a, b_neg, 0.0], [a, b_pos, 0.0]),
        GridPlane::YZ => ([0.0, a, b_neg], [0.0, a, b_pos]),
    };
    GridLine {
        start,
        end,
        color,
        width: 1.0,
    }
}

fn make_cross_line(
    a: f32,
    b_neg: f32,
    b_pos: f32,
    is_axis_b: bool,
    config: &GridConfig,
) -> GridLine {
    let color = if is_axis_b {
        config.axis_z_color
    } else {
        config.major_color
    };
    let (start, end) = match config.plane {
        GridPlane::XZ => ([b_neg, 0.0, a], [b_pos, 0.0, a]),
        GridPlane::XY => ([b_neg, a, 0.0], [b_pos, a, 0.0]),
        GridPlane::YZ => ([0.0, b_neg, a], [0.0, b_pos, a]),
    };
    GridLine {
        start,
        end,
        color,
        width: 1.0,
    }
}

#[allow(dead_code)]
pub fn build_grid(config: &GridConfig) -> ViewportGrid {
    let mut lines = Vec::new();
    let extent = config.extent;
    let spacing = config.spacing;
    let subdivisions = config.subdivisions.max(1);
    let sub_spacing = spacing / subdivisions as f32;

    // Generate lines along two axes
    let mut pos = -extent;
    while pos <= extent + 1e-5 {
        let is_on_major = (pos / spacing).abs().fract() < 1e-4
            || ((pos / spacing).abs().fract() - 1.0).abs() < 1e-4;

        let is_origin = pos.abs() < 1e-5;

        let color = if is_origin {
            config.origin_color
        } else if is_on_major {
            config.major_color
        } else {
            config.minor_color
        };

        let width = if is_origin || is_on_major { 1.5 } else { 0.5 };

        let (start_a, end_a) = match config.plane {
            GridPlane::XZ => ([pos, 0.0, -extent], [pos, 0.0, extent]),
            GridPlane::XY => ([pos, -extent, 0.0], [pos, extent, 0.0]),
            GridPlane::YZ => ([0.0, pos, -extent], [0.0, pos, extent]),
        };

        // Override color for axes when show_axes is enabled
        let line_color_a = if config.show_axes && is_origin {
            match config.plane {
                GridPlane::XZ | GridPlane::XY => config.axis_x_color,
                GridPlane::YZ => config.origin_color,
            }
        } else {
            color
        };

        lines.push(GridLine {
            start: start_a,
            end: end_a,
            color: line_color_a,
            width,
        });

        let (start_b, end_b) = match config.plane {
            GridPlane::XZ => ([-extent, 0.0, pos], [extent, 0.0, pos]),
            GridPlane::XY => ([-extent, pos, 0.0], [extent, pos, 0.0]),
            GridPlane::YZ => ([0.0, -extent, pos], [0.0, extent, pos]),
        };

        let line_color_b = if config.show_axes && is_origin {
            match config.plane {
                GridPlane::XZ => config.axis_z_color,
                GridPlane::XY | GridPlane::YZ => config.axis_z_color,
            }
        } else {
            color
        };

        lines.push(GridLine {
            start: start_b,
            end: end_b,
            color: line_color_b,
            width,
        });

        pos += sub_spacing;
    }

    ViewportGrid {
        config: GridConfig {
            spacing: config.spacing,
            extent: config.extent,
            subdivisions: config.subdivisions,
            major_color: config.major_color,
            minor_color: config.minor_color,
            origin_color: config.origin_color,
            axis_x_color: config.axis_x_color,
            axis_z_color: config.axis_z_color,
            show_origin: config.show_origin,
            show_axes: config.show_axes,
            plane: match config.plane {
                GridPlane::XZ => GridPlane::XZ,
                GridPlane::XY => GridPlane::XY,
                GridPlane::YZ => GridPlane::YZ,
            },
        },
        lines,
    }
}

#[allow(dead_code)]
pub fn grid_line_count(grid: &ViewportGrid) -> usize {
    grid.lines.len()
}

#[allow(dead_code)]
pub fn major_line_count(grid: &ViewportGrid) -> usize {
    let spacing = grid.config.spacing;
    grid.lines
        .iter()
        .filter(|l| {
            // A line is major if its start's first non-zero axis coord is a multiple of spacing
            let coord = match grid.config.plane {
                GridPlane::XZ => l.start[0],
                GridPlane::XY => l.start[0],
                GridPlane::YZ => l.start[1],
            };
            let other_coord = match grid.config.plane {
                GridPlane::XZ => l.start[2],
                GridPlane::XY => l.start[1],
                GridPlane::YZ => l.start[2],
            };
            // Line is major if it runs parallel to one axis at a major spacing value
            let on_major_a = coord.abs() < 1e-4 || (coord.abs() / spacing).fract() < 0.01;
            let on_major_b =
                other_coord.abs() < 1e-4 || (other_coord.abs() / spacing).fract() < 0.01;
            on_major_a || on_major_b
        })
        .count()
}

#[allow(dead_code)]
pub fn rebuild_grid(grid: &mut ViewportGrid) {
    let new_grid = build_grid(&grid.config);
    grid.lines = new_grid.lines;
}

#[allow(dead_code)]
pub fn set_grid_spacing(grid: &mut ViewportGrid, spacing: f32) {
    grid.config.spacing = spacing.max(0.0001);
    rebuild_grid(grid);
}

#[allow(dead_code)]
pub fn snap_to_grid(point: [f32; 3], spacing: f32) -> [f32; 3] {
    if spacing < 1e-9 {
        return point;
    }
    let snap = |v: f32| (v / spacing).round() * spacing;
    [snap(point[0]), point[1], snap(point[2])]
}

#[allow(dead_code)]
pub fn grid_cell_at(point: [f32; 3], spacing: f32) -> [i32; 2] {
    if spacing < 1e-9 {
        return [0, 0];
    }
    let col = (point[0] / spacing).floor() as i32;
    let row = (point[2] / spacing).floor() as i32;
    [col, row]
}

#[allow(dead_code)]
pub fn world_to_grid_coords(point: [f32; 3], config: &GridConfig) -> [f32; 2] {
    let u = point[0] / config.spacing;
    let v = match config.plane {
        GridPlane::XZ => point[2] / config.spacing,
        GridPlane::XY => point[1] / config.spacing,
        GridPlane::YZ => point[1] / config.spacing,
    };
    [u, v]
}

#[allow(dead_code)]
pub fn grid_lines_in_frustum(grid: &ViewportGrid, min: [f32; 3], max: [f32; 3]) -> Vec<&GridLine> {
    grid.lines
        .iter()
        .filter(|l| {
            // A line is visible if either endpoint or midpoint is inside the AABB,
            // or if the line's axis-aligned extent overlaps the AABB on at least 2 axes.
            let in_box = |p: [f32; 3]| {
                p[0] >= min[0]
                    && p[0] <= max[0]
                    && p[1] >= min[1]
                    && p[1] <= max[1]
                    && p[2] >= min[2]
                    && p[2] <= max[2]
            };
            let mid = [
                (l.start[0] + l.end[0]) * 0.5,
                (l.start[1] + l.end[1]) * 0.5,
                (l.start[2] + l.end[2]) * 0.5,
            ];
            // Check if the line segment's AABB overlaps the query AABB
            let line_min = [
                l.start[0].min(l.end[0]),
                l.start[1].min(l.end[1]),
                l.start[2].min(l.end[2]),
            ];
            let line_max = [
                l.start[0].max(l.end[0]),
                l.start[1].max(l.end[1]),
                l.start[2].max(l.end[2]),
            ];
            let aabb_overlap = line_min[0] <= max[0]
                && line_max[0] >= min[0]
                && line_min[1] <= max[1]
                && line_max[1] >= min[1]
                && line_min[2] <= max[2]
                && line_max[2] >= min[2];
            in_box(l.start) || in_box(l.end) || in_box(mid) || aabb_overlap
        })
        .collect()
}

#[allow(dead_code)]
pub fn grid_to_json(grid: &ViewportGrid) -> String {
    format!(
        r#"{{"spacing":{},"extent":{},"subdivisions":{},"line_count":{},"plane":"{}"}}"#,
        grid.config.spacing,
        grid.config.extent,
        grid.config.subdivisions,
        grid.lines.len(),
        match grid.config.plane {
            GridPlane::XZ => "XZ",
            GridPlane::XY => "XY",
            GridPlane::YZ => "YZ",
        }
    )
}

#[allow(dead_code)]
pub fn infinite_grid_lines_near(center: [f32; 3], radius: f32, spacing: f32) -> Vec<GridLine> {
    if spacing < 1e-9 || radius <= 0.0 {
        return Vec::new();
    }
    let mut lines = Vec::new();
    let min_coord = center[0] - radius;
    let max_coord = center[0] + radius;
    let first = (min_coord / spacing).floor() as i32;
    let last = (max_coord / spacing).ceil() as i32;

    let min_z = center[2] - radius;
    let max_z = center[2] + radius;
    let first_z = (min_z / spacing).floor() as i32;
    let last_z = (max_z / spacing).ceil() as i32;

    for i in first..=last {
        let x = i as f32 * spacing;
        lines.push(GridLine {
            start: [x, 0.0, min_z],
            end: [x, 0.0, max_z],
            color: [0.3, 0.3, 0.3, 1.0],
            width: 1.0,
        });
    }
    for j in first_z..=last_z {
        let z = j as f32 * spacing;
        lines.push(GridLine {
            start: [min_coord, 0.0, z],
            end: [max_coord, 0.0, z],
            color: [0.3, 0.3, 0.3, 1.0],
            width: 1.0,
        });
    }
    lines
}

// Suppress unused import warnings for helper functions used only in tests via make_line/make_cross_line
#[allow(dead_code)]
fn _use_helpers() {
    let cfg = default_grid_config();
    let _ = make_line(0.0, -1.0, 1.0, false, &cfg);
    let _ = make_cross_line(0.0, -1.0, 1.0, false, &cfg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_grid_config();
        assert!((cfg.spacing - 1.0).abs() < 1e-6);
        assert!((cfg.extent - 10.0).abs() < 1e-6);
        assert_eq!(cfg.subdivisions, 5);
        assert!(cfg.show_origin);
        assert!(cfg.show_axes);
    }

    #[test]
    fn test_build_grid_has_lines() {
        let cfg = default_grid_config();
        let grid = build_grid(&cfg);
        assert!(!grid.lines.is_empty());
    }

    #[test]
    fn test_grid_line_count() {
        let cfg = default_grid_config();
        let grid = build_grid(&cfg);
        assert_eq!(grid_line_count(&grid), grid.lines.len());
    }

    #[test]
    fn test_major_line_count_less_than_total() {
        let cfg = default_grid_config();
        let grid = build_grid(&cfg);
        let majors = major_line_count(&grid);
        assert!(majors <= grid_line_count(&grid));
    }

    #[test]
    fn test_rebuild_grid() {
        let cfg = default_grid_config();
        let mut grid = build_grid(&cfg);
        let before = grid.lines.len();
        rebuild_grid(&mut grid);
        assert_eq!(grid.lines.len(), before);
    }

    #[test]
    fn test_set_grid_spacing_rebuilds() {
        let cfg = default_grid_config();
        let mut grid = build_grid(&cfg);
        set_grid_spacing(&mut grid, 2.0);
        assert!((grid.config.spacing - 2.0).abs() < 1e-6);
        assert!(!grid.lines.is_empty());
    }

    #[test]
    fn test_snap_to_grid() {
        let pt = [0.3f32, 1.5, 0.7];
        let snapped = snap_to_grid(pt, 1.0);
        assert!((snapped[0] - 0.0).abs() < 0.01);
        assert!((snapped[2] - 1.0).abs() < 0.01);
        assert!((snapped[1] - 1.5).abs() < 1e-6); // Y is not snapped
    }

    #[test]
    fn test_grid_cell_at() {
        let cell = grid_cell_at([1.5, 0.0, -0.5], 1.0);
        assert_eq!(cell[0], 1);
        assert_eq!(cell[1], -1);
    }

    #[test]
    fn test_world_to_grid_coords() {
        let cfg = default_grid_config(); // spacing = 1.0, plane XZ
        let [u, v] = world_to_grid_coords([3.0, 0.0, 5.0], &cfg);
        assert!((u - 3.0).abs() < 1e-6);
        assert!((v - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_grid_lines_in_frustum() {
        let cfg = default_grid_config();
        let grid = build_grid(&cfg);
        let visible = grid_lines_in_frustum(&grid, [-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
        assert!(!visible.is_empty());
    }

    #[test]
    fn test_grid_to_json() {
        let cfg = default_grid_config();
        let grid = build_grid(&cfg);
        let json = grid_to_json(&grid);
        assert!(json.contains("spacing"));
        assert!(json.contains("XZ"));
    }

    #[test]
    fn test_infinite_grid_near() {
        let lines = infinite_grid_lines_near([0.0, 0.0, 0.0], 5.0, 1.0);
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_snap_to_grid_zero_spacing() {
        // Should not panic with zero spacing
        let pt = [1.5f32, 1.5, 1.5];
        let snapped = snap_to_grid(pt, 0.0);
        assert_eq!(snapped, pt);
    }
}
