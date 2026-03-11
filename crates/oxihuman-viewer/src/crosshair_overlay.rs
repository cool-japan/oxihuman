// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Centered crosshair overlay for precision placement in viewport.

#![allow(dead_code)]

/// Configuration for the crosshair overlay.
#[derive(Debug, Clone)]
pub struct CrosshairConfig {
    /// Arm length in pixels.
    pub arm_length: f32,
    /// Line thickness in pixels.
    pub thickness: f32,
    /// RGBA colour [r, g, b, a] in 0..=1 range.
    pub color: [f32; 4],
    /// Gap size at centre (pixels).
    pub gap: f32,
    /// Grid snap resolution (0 = disabled).
    pub snap_grid: f32,
}

/// Runtime state for the crosshair overlay.
#[derive(Debug, Clone)]
pub struct CrosshairState {
    /// Current crosshair position in screen pixels [x, y].
    pub position: [f32; 2],
    /// Whether the crosshair is visible.
    pub visible: bool,
    /// Configuration in use.
    pub config: CrosshairConfig,
}

/// Returns the default [`CrosshairConfig`].
pub fn default_crosshair_config() -> CrosshairConfig {
    CrosshairConfig {
        arm_length: 12.0,
        thickness: 1.5,
        color: [1.0, 1.0, 1.0, 0.85],
        gap: 3.0,
        snap_grid: 0.0,
    }
}

/// Creates a new [`CrosshairState`] centred at the origin.
pub fn new_crosshair(config: CrosshairConfig) -> CrosshairState {
    CrosshairState {
        position: [0.0, 0.0],
        visible: true,
        config,
    }
}

/// Sets the crosshair position.
pub fn crosshair_set_position(state: &mut CrosshairState, x: f32, y: f32) {
    state.position = [x, y];
}

/// Returns the effective arm size (arm_length).
pub fn crosshair_size(state: &CrosshairState) -> f32 {
    state.config.arm_length
}

/// Returns the current RGBA colour.
pub fn crosshair_color(state: &CrosshairState) -> [f32; 4] {
    state.config.color
}

/// Sets crosshair visibility.
pub fn crosshair_set_visible(state: &mut CrosshairState, visible: bool) {
    state.visible = visible;
}

/// Returns whether the crosshair is currently visible.
pub fn crosshair_is_visible(state: &CrosshairState) -> bool {
    state.visible
}

/// Snaps the crosshair position to the configured grid (no-op if snap_grid == 0).
pub fn crosshair_snap_to_grid(state: &mut CrosshairState) {
    let g = state.config.snap_grid;
    if g <= 0.0 {
        return;
    }
    state.position[0] = (state.position[0] / g).round() * g;
    state.position[1] = (state.position[1] / g).round() * g;
}

/// Resets the crosshair to its default position (0, 0) and makes it visible.
pub fn crosshair_reset(state: &mut CrosshairState) {
    state.position = [0.0, 0.0];
    state.visible = true;
}

/// Serialises the crosshair state as a compact JSON string.
pub fn crosshair_to_json(state: &CrosshairState) -> String {
    let c = &state.config.color;
    format!(
        "{{\"x\":{:.4},\"y\":{:.4},\"visible\":{},\
        \"arm_length\":{:.4},\"thickness\":{:.4},\
        \"color\":[{:.4},{:.4},{:.4},{:.4}],\"gap\":{:.4}}}",
        state.position[0],
        state.position[1],
        state.visible,
        state.config.arm_length,
        state.config.thickness,
        c[0], c[1], c[2], c[3],
        state.config.gap,
    )
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn default_state() -> CrosshairState {
        new_crosshair(default_crosshair_config())
    }

    #[test]
    fn default_config_values() {
        let cfg = default_crosshair_config();
        assert!((cfg.arm_length - 12.0).abs() < 1e-5);
        assert!(cfg.snap_grid <= 0.0);
    }

    #[test]
    fn new_crosshair_is_visible() {
        let s = default_state();
        assert!(crosshair_is_visible(&s));
    }

    #[test]
    fn new_crosshair_at_origin() {
        let s = default_state();
        assert!((s.position[0]).abs() < 1e-5);
        assert!((s.position[1]).abs() < 1e-5);
    }

    #[test]
    fn set_position_updates_state() {
        let mut s = default_state();
        crosshair_set_position(&mut s, 100.0, 200.0);
        assert!((s.position[0] - 100.0).abs() < 1e-5);
        assert!((s.position[1] - 200.0).abs() < 1e-5);
    }

    #[test]
    fn size_returns_arm_length() {
        let s = default_state();
        assert!((crosshair_size(&s) - 12.0).abs() < 1e-5);
    }

    #[test]
    fn color_returns_configured_color() {
        let s = default_state();
        let c = crosshair_color(&s);
        assert!((c[3] - 0.85).abs() < 1e-4);
    }

    #[test]
    fn set_visible_false_hides_crosshair() {
        let mut s = default_state();
        crosshair_set_visible(&mut s, false);
        assert!(!crosshair_is_visible(&s));
    }

    #[test]
    fn snap_to_grid_rounds_position() {
        let mut s = default_state();
        s.config.snap_grid = 10.0;
        crosshair_set_position(&mut s, 13.0, 27.0);
        crosshair_snap_to_grid(&mut s);
        assert!((s.position[0] - 10.0).abs() < 1e-4);
        assert!((s.position[1] - 30.0).abs() < 1e-4);
    }

    #[test]
    fn snap_no_op_when_disabled() {
        let mut s = default_state();
        crosshair_set_position(&mut s, 13.7, 27.2);
        crosshair_snap_to_grid(&mut s);
        assert!((s.position[0] - 13.7).abs() < 1e-4);
    }

    #[test]
    fn reset_returns_to_origin() {
        let mut s = default_state();
        crosshair_set_position(&mut s, 300.0, 400.0);
        crosshair_set_visible(&mut s, false);
        crosshair_reset(&mut s);
        assert!((s.position[0]).abs() < 1e-5);
        assert!(crosshair_is_visible(&s));
    }

    #[test]
    fn to_json_contains_expected_keys() {
        let s = default_state();
        let json = crosshair_to_json(&s);
        assert!(json.contains("\"x\""));
        assert!(json.contains("\"visible\""));
        assert!(json.contains("\"color\""));
        assert!(json.contains("\"arm_length\""));
    }
}
