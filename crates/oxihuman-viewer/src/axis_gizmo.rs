// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! World-space XYZ axis indicator widget (typically shown in viewport corner).

#![allow(dead_code)]

/// Configuration for the axis gizmo.
#[derive(Debug, Clone)]
pub struct AxisGizmoConfig {
    /// Display size in pixels.
    pub size_px: u32,
    /// Screen-space corner position [x, y] in pixels.
    pub screen_position: [f32; 2],
    /// Shaft length (normalised, 0–1).
    pub shaft_length: f32,
}

/// Runtime state of the axis gizmo.
#[derive(Debug, Clone)]
pub struct AxisGizmoState {
    pub config: AxisGizmoConfig,
    /// Current 3×3 rotation matrix (row-major) derived from the view.
    pub rotation: [[f32; 3]; 3],
    /// Whether the gizmo is visible.
    pub visible: bool,
}

/// Returns the default [`AxisGizmoConfig`].
#[allow(dead_code)]
pub fn default_axis_gizmo_config() -> AxisGizmoConfig {
    AxisGizmoConfig {
        size_px: 80,
        screen_position: [60.0, 60.0],
        shaft_length: 0.8,
    }
}

/// Creates a new [`AxisGizmoState`] with identity rotation.
#[allow(dead_code)]
pub fn new_axis_gizmo(cfg: AxisGizmoConfig) -> AxisGizmoState {
    AxisGizmoState {
        config: cfg,
        rotation: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        visible: true,
    }
}

/// Updates the gizmo rotation from a 4×4 view matrix (column-major).
/// Only the upper-left 3×3 is used.
#[allow(dead_code)]
pub fn axis_gizmo_update_matrix(state: &mut AxisGizmoState, view_matrix: &[[f32; 4]; 4]) {
    for (row, dst_row) in state.rotation.iter_mut().enumerate().take(3) {
        for (col, dst) in dst_row.iter_mut().enumerate().take(3) {
            *dst = view_matrix[col][row];
        }
    }
}

/// Returns the RGBA colour for a given axis index (0=X, 1=Y, 2=Z).
#[allow(dead_code)]
pub fn axis_gizmo_axis_color(axis: usize) -> [f32; 4] {
    match axis {
        0 => [1.0, 0.1, 0.1, 1.0], // X — red
        1 => [0.1, 1.0, 0.1, 1.0], // Y — green
        _ => [0.1, 0.1, 1.0, 1.0], // Z — blue
    }
}

/// Returns the text label for a given axis index.
#[allow(dead_code)]
pub fn axis_gizmo_label(axis: usize) -> &'static str {
    match axis {
        0 => "X",
        1 => "Y",
        _ => "Z",
    }
}

/// Sets visibility.
#[allow(dead_code)]
pub fn axis_gizmo_set_visible(state: &mut AxisGizmoState, visible: bool) {
    state.visible = visible;
}

/// Returns whether the gizmo is visible.
#[allow(dead_code)]
pub fn axis_gizmo_is_visible(state: &AxisGizmoState) -> bool {
    state.visible
}

/// Returns the screen position [x, y] of the gizmo centre.
#[allow(dead_code)]
pub fn axis_gizmo_screen_position(state: &AxisGizmoState) -> [f32; 2] {
    state.config.screen_position
}

/// Serialises gizmo state to a JSON string.
#[allow(dead_code)]
pub fn axis_gizmo_to_json(state: &AxisGizmoState) -> String {
    format!(
        "{{\"visible\":{},\"size_px\":{},\"screen_position\":[{:.3},{:.3}]}}",
        state.visible,
        state.config.size_px,
        state.config.screen_position[0],
        state.config.screen_position[1],
    )
}

/// Resets the gizmo rotation to identity and restores visibility.
#[allow(dead_code)]
pub fn axis_gizmo_reset(state: &mut AxisGizmoState) {
    state.rotation = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    state.visible = true;
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_axis_gizmo_config();
        assert_eq!(cfg.size_px, 80);
        assert!((cfg.shaft_length - 0.8).abs() < 1e-5);
    }

    #[test]
    fn new_gizmo_is_visible_with_identity() {
        let cfg = default_axis_gizmo_config();
        let state = new_axis_gizmo(cfg);
        assert!(axis_gizmo_is_visible(&state));
        assert_eq!(state.rotation[0][0], 1.0);
        assert_eq!(state.rotation[1][1], 1.0);
        assert_eq!(state.rotation[2][2], 1.0);
    }

    #[test]
    fn set_visible_false() {
        let mut s = new_axis_gizmo(default_axis_gizmo_config());
        axis_gizmo_set_visible(&mut s, false);
        assert!(!axis_gizmo_is_visible(&s));
    }

    #[test]
    fn reset_restores_identity_and_visibility() {
        let mut s = new_axis_gizmo(default_axis_gizmo_config());
        axis_gizmo_set_visible(&mut s, false);
        s.rotation[0][0] = 0.5;
        axis_gizmo_reset(&mut s);
        assert!(axis_gizmo_is_visible(&s));
        assert!((s.rotation[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn axis_colors_are_distinct() {
        let cx = axis_gizmo_axis_color(0);
        let cy = axis_gizmo_axis_color(1);
        let cz = axis_gizmo_axis_color(2);
        assert_ne!(cx, cy);
        assert_ne!(cy, cz);
    }

    #[test]
    fn axis_labels_are_correct() {
        assert_eq!(axis_gizmo_label(0), "X");
        assert_eq!(axis_gizmo_label(1), "Y");
        assert_eq!(axis_gizmo_label(2), "Z");
    }

    #[test]
    fn screen_position_roundtrip() {
        let mut cfg = default_axis_gizmo_config();
        cfg.screen_position = [100.0, 200.0];
        let s = new_axis_gizmo(cfg);
        let pos = axis_gizmo_screen_position(&s);
        assert!((pos[0] - 100.0).abs() < 1e-5);
        assert!((pos[1] - 200.0).abs() < 1e-5);
    }

    #[test]
    fn update_matrix_reads_upper_left_3x3() {
        let mut s = new_axis_gizmo(default_axis_gizmo_config());
        // Build a simple 90° yaw matrix (column-major)
        let vm: [[f32; 4]; 4] = [
            [0.0, 0.0, -1.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        axis_gizmo_update_matrix(&mut s, &vm);
        // rotation[row][col] = vm[col][row]
        assert!((s.rotation[0][0] - 0.0).abs() < 1e-5);
        assert!((s.rotation[0][2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn to_json_contains_visible() {
        let s = new_axis_gizmo(default_axis_gizmo_config());
        let json = axis_gizmo_to_json(&s);
        assert!(json.contains("\"visible\":true"));
    }
}
