// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Grid snapping utilities for editor interactions.

// ── Structs ───────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GridSnapConfig {
    pub grid_size: f32,
    pub snap_enabled: bool,
    pub sub_divisions: u32,
    pub world_align: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SnapResult {
    pub snapped_position: [f32; 3],
    pub grid_cell: [i32; 3],
    pub was_snapped: bool,
    pub snap_distance: f32,
}

// ── Functions ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn default_grid_snap_config() -> GridSnapConfig {
    GridSnapConfig {
        grid_size: 1.0,
        snap_enabled: true,
        sub_divisions: 1,
        world_align: true,
    }
}

#[allow(dead_code)]
pub fn snap_to_grid(pos: [f32; 3], cfg: &GridSnapConfig) -> SnapResult {
    if !cfg.snap_enabled {
        return SnapResult {
            snapped_position: pos,
            grid_cell: [0, 0, 0],
            was_snapped: false,
            snap_distance: 0.0,
        };
    }
    let effective_size = cfg.grid_size / (cfg.sub_divisions.max(1) as f32);
    let snapped = [
        snap_component(pos[0], effective_size),
        snap_component(pos[1], effective_size),
        snap_component(pos[2], effective_size),
    ];
    let dist = snap_distance(pos, snapped);
    let cell = grid_cell_for_position(snapped, cfg);
    SnapResult {
        snapped_position: snapped,
        grid_cell: cell,
        was_snapped: true,
        snap_distance: dist,
    }
}

#[allow(dead_code)]
pub fn snap_component(v: f32, grid_size: f32) -> f32 {
    if grid_size <= 0.0 {
        return v;
    }
    (v / grid_size).round() * grid_size
}

#[allow(dead_code)]
pub fn grid_cell_for_position(pos: [f32; 3], cfg: &GridSnapConfig) -> [i32; 3] {
    let s = cfg.grid_size.max(1e-9);
    [
        (pos[0] / s).round() as i32,
        (pos[1] / s).round() as i32,
        (pos[2] / s).round() as i32,
    ]
}

#[allow(dead_code)]
pub fn cell_center(cell: [i32; 3], cfg: &GridSnapConfig) -> [f32; 3] {
    let s = cfg.grid_size;
    [
        cell[0] as f32 * s,
        cell[1] as f32 * s,
        cell[2] as f32 * s,
    ]
}

#[allow(dead_code)]
pub fn snap_distance(pos: [f32; 3], snapped: [f32; 3]) -> f32 {
    let dx = pos[0] - snapped[0];
    let dy = pos[1] - snapped[1];
    let dz = pos[2] - snapped[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[allow(dead_code)]
pub fn snap_result_to_json(r: &SnapResult) -> String {
    format!(
        r#"{{"snapped_position":[{},{},{}],"grid_cell":[{},{},{}],"was_snapped":{},"snap_distance":{}}}"#,
        r.snapped_position[0], r.snapped_position[1], r.snapped_position[2],
        r.grid_cell[0], r.grid_cell[1], r.grid_cell[2],
        r.was_snapped,
        r.snap_distance
    )
}

#[allow(dead_code)]
pub fn grid_snap_config_to_json(cfg: &GridSnapConfig) -> String {
    format!(
        r#"{{"grid_size":{},"snap_enabled":{},"sub_divisions":{},"world_align":{}}}"#,
        cfg.grid_size, cfg.snap_enabled, cfg.sub_divisions, cfg.world_align
    )
}

#[allow(dead_code)]
pub fn is_on_grid(pos: [f32; 3], cfg: &GridSnapConfig) -> bool {
    let s = cfg.grid_size.max(1e-9);
    let eps = s * 1e-4;
    let rx = (pos[0] / s).fract().abs();
    let ry = (pos[1] / s).fract().abs();
    let rz = (pos[2] / s).fract().abs();
    (rx < eps || rx > 1.0 - eps)
        && (ry < eps || ry > 1.0 - eps)
        && (rz < eps || rz > 1.0 - eps)
}

/// Returns 3 nearest grid line positions — one per axis.
#[allow(dead_code)]
pub fn nearest_grid_lines(pos: [f32; 3], cfg: &GridSnapConfig) -> Vec<[f32; 3]> {
    let s = cfg.grid_size.max(1e-9);
    let snapped_x = snap_component(pos[0], s);
    let snapped_y = snap_component(pos[1], s);
    let snapped_z = snap_component(pos[2], s);
    vec![
        [snapped_x, pos[1], pos[2]],
        [pos[0], snapped_y, pos[2]],
        [pos[0], pos[1], snapped_z],
    ]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_grid_snap_config();
        assert!((cfg.grid_size - 1.0).abs() < 1e-6);
        assert!(cfg.snap_enabled);
        assert_eq!(cfg.sub_divisions, 1);
        assert!(cfg.world_align);
    }

    #[test]
    fn snap_component_rounds_to_grid() {
        let snapped = snap_component(1.3, 1.0);
        assert!((snapped - 1.0).abs() < 1e-5);
        let snapped2 = snap_component(1.6, 1.0);
        assert!((snapped2 - 2.0).abs() < 1e-5);
    }

    #[test]
    fn snap_to_grid_enabled() {
        let cfg = default_grid_snap_config();
        let result = snap_to_grid([1.3, 0.7, 2.1], &cfg);
        assert!(result.was_snapped);
        assert!((result.snapped_position[0] - 1.0).abs() < 1e-5);
        assert!((result.snapped_position[1] - 1.0).abs() < 1e-5);
        assert!((result.snapped_position[2] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn snap_to_grid_disabled_passthrough() {
        let cfg = GridSnapConfig {
            grid_size: 1.0,
            snap_enabled: false,
            sub_divisions: 1,
            world_align: true,
        };
        let pos = [1.3, 0.7, 2.1];
        let result = snap_to_grid(pos, &cfg);
        assert!(!result.was_snapped);
        assert!((result.snapped_position[0] - 1.3).abs() < 1e-5);
    }

    #[test]
    fn grid_cell_for_position_correct() {
        let cfg = default_grid_snap_config();
        let cell = grid_cell_for_position([2.0, -1.0, 3.0], &cfg);
        assert_eq!(cell, [2, -1, 3]);
    }

    #[test]
    fn cell_center_correct() {
        let cfg = default_grid_snap_config();
        let center = cell_center([3, -2, 1], &cfg);
        assert!((center[0] - 3.0).abs() < 1e-5);
        assert!((center[1] - (-2.0)).abs() < 1e-5);
        assert!((center[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn snap_distance_zero_for_on_grid() {
        let pos = [2.0, 3.0, 4.0];
        let snapped = [2.0, 3.0, 4.0];
        let dist = snap_distance(pos, snapped);
        assert!(dist < 1e-6);
    }

    #[test]
    fn is_on_grid_true_for_grid_point() {
        let cfg = default_grid_snap_config();
        assert!(is_on_grid([2.0, 3.0, 4.0], &cfg));
    }

    #[test]
    fn is_on_grid_false_off_grid() {
        let cfg = default_grid_snap_config();
        assert!(!is_on_grid([1.3, 0.5, 2.7], &cfg));
    }

    #[test]
    fn nearest_grid_lines_returns_three() {
        let cfg = default_grid_snap_config();
        let lines = nearest_grid_lines([1.4, 0.6, 2.9], &cfg);
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn snap_result_to_json_valid() {
        let r = SnapResult {
            snapped_position: [1.0, 0.0, 2.0],
            grid_cell: [1, 0, 2],
            was_snapped: true,
            snap_distance: 0.1,
        };
        let json = snap_result_to_json(&r);
        assert!(json.contains("snapped_position"));
        assert!(json.contains("was_snapped"));
    }

    #[test]
    fn grid_snap_config_to_json_valid() {
        let cfg = default_grid_snap_config();
        let json = grid_snap_config_to_json(&cfg);
        assert!(json.contains("grid_size"));
        assert!(json.contains("snap_enabled"));
    }
}
