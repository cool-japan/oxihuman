// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Highlight and tooltip overlay shown when hovering over pickable scene objects.

#![allow(dead_code)]

/// Configuration for the picking overlay.
#[derive(Debug, Clone)]
pub struct PickingOverlayConfig {
    /// Highlight colour for the picked object as `(r, g, b, a)`.
    pub highlight_color: (u8, u8, u8, u8),
    /// Whether to show a tooltip on pick.
    pub show_tooltip: bool,
    /// Tooltip background colour.
    pub tooltip_color: (u8, u8, u8, u8),
}

/// Describes a single picked scene object.
#[derive(Debug, Clone)]
pub struct PickedObject {
    /// Unique object identifier.
    pub id: u64,
    /// Human-readable name.
    pub name: String,
    /// Object type tag (e.g. "mesh", "bone", "light").
    pub object_type: String,
    /// World-space pick position as `(x, y, z)`.
    pub hit_position: (f64, f64, f64),
}

/// Runtime state of the picking overlay.
#[derive(Debug, Clone)]
pub struct PickingOverlayState {
    /// Currently picked object, if any.
    pub picked: Option<PickedObject>,
    /// Highlight colour.
    pub highlight_color: (u8, u8, u8, u8),
    /// Whether the tooltip is shown.
    pub show_tooltip: bool,
    /// Tooltip background colour.
    pub tooltip_color: (u8, u8, u8, u8),
}

/// Returns the default [`PickingOverlayConfig`].
pub fn default_picking_overlay_config() -> PickingOverlayConfig {
    PickingOverlayConfig {
        highlight_color: (255, 200, 0, 180),
        show_tooltip: true,
        tooltip_color: (30, 30, 30, 220),
    }
}

/// Creates a new [`PickingOverlayState`] from configuration.
pub fn new_picking_overlay(cfg: &PickingOverlayConfig) -> PickingOverlayState {
    PickingOverlayState {
        picked: None,
        highlight_color: cfg.highlight_color,
        show_tooltip: cfg.show_tooltip,
        tooltip_color: cfg.tooltip_color,
    }
}

/// Sets the currently picked object.
pub fn picking_overlay_set_picked(state: &mut PickingOverlayState, obj: PickedObject) {
    state.picked = Some(obj);
}

/// Clears the currently picked object.
pub fn picking_overlay_clear(state: &mut PickingOverlayState) {
    state.picked = None;
}

/// Returns `true` if an object is currently picked.
pub fn picking_overlay_is_active(state: &PickingOverlayState) -> bool {
    state.picked.is_some()
}

/// Returns the name of the currently picked object, or an empty string if none.
pub fn picking_overlay_object_name(state: &PickingOverlayState) -> &str {
    match &state.picked {
        Some(obj) => &obj.name,
        None => "",
    }
}

/// Returns `true` if the tooltip is currently configured to be shown.
pub fn picking_overlay_show_tooltip(state: &PickingOverlayState) -> bool {
    state.show_tooltip
}

/// Serialises the overlay state as JSON.
pub fn picking_overlay_to_json(state: &PickingOverlayState) -> String {
    let picked_json = match &state.picked {
        Some(obj) => format!(
            "{{\"id\":{},\"name\":\"{}\",\"type\":\"{}\",\"hit\":[{:.4},{:.4},{:.4}]}}",
            obj.id, obj.name, obj.object_type, obj.hit_position.0, obj.hit_position.1, obj.hit_position.2
        ),
        None => "null".to_string(),
    };
    format!(
        "{{\"highlight_color\":[{},{},{},{}],\"show_tooltip\":{},\"picked\":{}}}",
        state.highlight_color.0,
        state.highlight_color.1,
        state.highlight_color.2,
        state.highlight_color.3,
        state.show_tooltip,
        picked_json
    )
}

/// Resets the overlay to default configuration state.
pub fn picking_overlay_reset(state: &mut PickingOverlayState) {
    let cfg = default_picking_overlay_config();
    state.picked = None;
    state.highlight_color = cfg.highlight_color;
    state.show_tooltip = cfg.show_tooltip;
    state.tooltip_color = cfg.tooltip_color;
}

/// Returns the highlight colour as `(r, g, b, a)`.
pub fn picking_overlay_color(state: &PickingOverlayState) -> (u8, u8, u8, u8) {
    state.highlight_color
}

// ── internal helpers ───────────────────────────────────────────────────────────

fn make_picked(id: u64, name: &str, object_type: &str, hit: (f64, f64, f64)) -> PickedObject {
    PickedObject {
        id,
        name: name.to_string(),
        object_type: object_type.to_string(),
        hit_position: hit,
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_picking_overlay_config();
        assert_eq!(cfg.highlight_color, (255, 200, 0, 180));
        assert!(cfg.show_tooltip);
    }

    #[test]
    fn new_overlay_has_no_pick() {
        let cfg = default_picking_overlay_config();
        let state = new_picking_overlay(&cfg);
        assert!(!picking_overlay_is_active(&state));
        assert_eq!(picking_overlay_object_name(&state), "");
    }

    #[test]
    fn set_picked_activates_overlay() {
        let cfg = default_picking_overlay_config();
        let mut state = new_picking_overlay(&cfg);
        picking_overlay_set_picked(&mut state, make_picked(1, "BodyMesh", "mesh", (0.0, 0.0, 0.0)));
        assert!(picking_overlay_is_active(&state));
        assert_eq!(picking_overlay_object_name(&state), "BodyMesh");
    }

    #[test]
    fn clear_removes_picked() {
        let cfg = default_picking_overlay_config();
        let mut state = new_picking_overlay(&cfg);
        picking_overlay_set_picked(&mut state, make_picked(1, "Bone", "bone", (1.0, 2.0, 3.0)));
        picking_overlay_clear(&mut state);
        assert!(!picking_overlay_is_active(&state));
    }

    #[test]
    fn show_tooltip_returns_config_value() {
        let cfg = default_picking_overlay_config();
        let state = new_picking_overlay(&cfg);
        assert!(picking_overlay_show_tooltip(&state));
    }

    #[test]
    fn color_returns_highlight_color() {
        let cfg = default_picking_overlay_config();
        let state = new_picking_overlay(&cfg);
        assert_eq!(picking_overlay_color(&state), (255, 200, 0, 180));
    }

    #[test]
    fn json_contains_highlight_and_picked() {
        let cfg = default_picking_overlay_config();
        let mut state = new_picking_overlay(&cfg);
        picking_overlay_set_picked(&mut state, make_picked(42, "Light", "light", (1.0, 2.0, 3.0)));
        let json = picking_overlay_to_json(&state);
        assert!(json.contains("highlight_color"));
        assert!(json.contains("\"Light\""));
    }

    #[test]
    fn json_null_when_no_pick() {
        let cfg = default_picking_overlay_config();
        let state = new_picking_overlay(&cfg);
        let json = picking_overlay_to_json(&state);
        assert!(json.contains("\"picked\":null"));
    }

    #[test]
    fn reset_clears_pick_and_restores_defaults() {
        let cfg = default_picking_overlay_config();
        let mut state = new_picking_overlay(&cfg);
        picking_overlay_set_picked(&mut state, make_picked(1, "X", "mesh", (0.0, 0.0, 0.0)));
        state.show_tooltip = false;
        picking_overlay_reset(&mut state);
        assert!(!picking_overlay_is_active(&state));
        assert!(picking_overlay_show_tooltip(&state));
    }
}
