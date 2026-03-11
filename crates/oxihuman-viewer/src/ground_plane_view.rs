// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Ground plane renderer helper (infinite grid + optional checkerboard pattern).

#![allow(dead_code)]

/// Configuration for the ground-plane view.
#[derive(Debug, Clone)]
pub struct GroundPlaneConfig {
    /// Initial visibility.
    pub visible: bool,
    /// Cell size of the grid in world units.
    pub cell_size: f64,
    /// Grid line colour as `(r, g, b, a)` bytes.
    pub color: (u8, u8, u8, u8),
    /// Whether to render a checkerboard pattern instead of lines.
    pub checkerboard: bool,
    /// Y-axis offset for the ground plane (default 0.0).
    pub y_offset: f64,
}

/// Runtime state of the ground-plane view.
#[derive(Debug, Clone)]
pub struct GroundPlaneState {
    /// Whether the ground plane is currently visible.
    pub visible: bool,
    /// Cell size in world units.
    pub cell_size: f64,
    /// Current grid colour.
    pub color: (u8, u8, u8, u8),
    /// Whether checkerboard mode is active.
    pub checkerboard: bool,
    /// Y-axis offset.
    pub y_offset: f64,
}

/// Returns the default [`GroundPlaneConfig`].
pub fn default_ground_plane_config() -> GroundPlaneConfig {
    GroundPlaneConfig {
        visible: true,
        cell_size: 1.0,
        color: (128, 128, 128, 200),
        checkerboard: false,
        y_offset: 0.0,
    }
}

/// Creates a new [`GroundPlaneState`] from configuration.
pub fn new_ground_plane(cfg: &GroundPlaneConfig) -> GroundPlaneState {
    GroundPlaneState {
        visible: cfg.visible,
        cell_size: cfg.cell_size,
        color: cfg.color,
        checkerboard: cfg.checkerboard,
        y_offset: cfg.y_offset,
    }
}

/// Sets the visibility of the ground plane.
pub fn ground_plane_set_visible(state: &mut GroundPlaneState, visible: bool) {
    state.visible = visible;
}

/// Returns whether the ground plane is visible.
pub fn ground_plane_is_visible(state: &GroundPlaneState) -> bool {
    state.visible
}

/// Returns the cell size of the grid.
pub fn ground_plane_cell_size(state: &GroundPlaneState) -> f64 {
    state.cell_size
}

/// Returns the grid colour as `(r, g, b, a)`.
pub fn ground_plane_color(state: &GroundPlaneState) -> (u8, u8, u8, u8) {
    state.color
}

/// Serialises the ground-plane state as JSON.
pub fn ground_plane_to_json(state: &GroundPlaneState) -> String {
    format!(
        "{{\"visible\":{},\"cell_size\":{:.4},\"color\":[{},{},{},{}],\"checkerboard\":{},\"y_offset\":{:.4}}}",
        state.visible,
        state.cell_size,
        state.color.0,
        state.color.1,
        state.color.2,
        state.color.3,
        state.checkerboard,
        state.y_offset
    )
}

/// Resets the ground plane state to default values.
pub fn ground_plane_reset(state: &mut GroundPlaneState) {
    let cfg = default_ground_plane_config();
    state.visible = cfg.visible;
    state.cell_size = cfg.cell_size;
    state.color = cfg.color;
    state.checkerboard = cfg.checkerboard;
    state.y_offset = cfg.y_offset;
}

/// Toggles the visibility of the ground plane.
pub fn ground_plane_toggle(state: &mut GroundPlaneState) {
    state.visible = !state.visible;
}

/// Returns the Y-axis offset of the ground plane.
pub fn ground_plane_y_offset(state: &GroundPlaneState) -> f64 {
    state.y_offset
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_ground_plane_config();
        assert!(cfg.visible);
        assert!((cfg.cell_size - 1.0).abs() < 1e-9);
        assert_eq!(cfg.color, (128, 128, 128, 200));
        assert!(!cfg.checkerboard);
        assert!((cfg.y_offset).abs() < 1e-9);
    }

    #[test]
    fn new_ground_plane_matches_config() {
        let cfg = default_ground_plane_config();
        let state = new_ground_plane(&cfg);
        assert!(ground_plane_is_visible(&state));
        assert!((ground_plane_cell_size(&state) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn set_visible_updates_state() {
        let cfg = default_ground_plane_config();
        let mut state = new_ground_plane(&cfg);
        ground_plane_set_visible(&mut state, false);
        assert!(!ground_plane_is_visible(&state));
    }

    #[test]
    fn toggle_flips_visibility() {
        let cfg = default_ground_plane_config();
        let mut state = new_ground_plane(&cfg);
        assert!(ground_plane_is_visible(&state));
        ground_plane_toggle(&mut state);
        assert!(!ground_plane_is_visible(&state));
        ground_plane_toggle(&mut state);
        assert!(ground_plane_is_visible(&state));
    }

    #[test]
    fn cell_size_returned_correctly() {
        let cfg = GroundPlaneConfig {
            visible: true,
            cell_size: 0.5,
            color: (255, 255, 255, 255),
            checkerboard: false,
            y_offset: 0.0,
        };
        let state = new_ground_plane(&cfg);
        assert!((ground_plane_cell_size(&state) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn color_returned_correctly() {
        let cfg = default_ground_plane_config();
        let state = new_ground_plane(&cfg);
        assert_eq!(ground_plane_color(&state), (128, 128, 128, 200));
    }

    #[test]
    fn y_offset_returned_correctly() {
        let cfg = GroundPlaneConfig {
            visible: true,
            cell_size: 1.0,
            color: (0, 0, 0, 255),
            checkerboard: false,
            y_offset: -0.01,
        };
        let state = new_ground_plane(&cfg);
        assert!((ground_plane_y_offset(&state) - (-0.01)).abs() < 1e-9);
    }

    #[test]
    fn json_contains_expected_fields() {
        let cfg = default_ground_plane_config();
        let state = new_ground_plane(&cfg);
        let json = ground_plane_to_json(&state);
        assert!(json.contains("\"visible\""));
        assert!(json.contains("\"cell_size\""));
        assert!(json.contains("\"color\""));
    }

    #[test]
    fn reset_restores_defaults() {
        let cfg = default_ground_plane_config();
        let mut state = new_ground_plane(&cfg);
        ground_plane_set_visible(&mut state, false);
        state.cell_size = 5.0;
        ground_plane_reset(&mut state);
        assert!(ground_plane_is_visible(&state));
        assert!((ground_plane_cell_size(&state) - 1.0).abs() < 1e-9);
    }
}
