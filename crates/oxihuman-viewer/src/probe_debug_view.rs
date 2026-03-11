// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Debug visualization for reflection/light probe placement and influence volumes.

#![allow(dead_code)]

/// Configuration for probe debug view.
#[derive(Debug, Clone)]
pub struct ProbeDebugConfig {
    /// Radius of the sphere drawn at each probe position.
    pub marker_radius: f32,
    /// Whether influence volume boxes are shown.
    pub show_influence: bool,
    /// Maximum number of markers allowed.
    pub max_markers: usize,
}

/// A single probe debug marker.
#[derive(Debug, Clone)]
pub struct ProbeDebugMarker {
    /// Unique marker id.
    pub id: u32,
    /// World-space position (x, y, z).
    pub position: [f32; 3],
    /// Half-extents of the influence volume (x, y, z).
    pub influence: [f32; 3],
    /// Display label.
    pub label: String,
}

/// State for the probe debug overlay.
#[derive(Debug, Clone)]
pub struct ProbeDebugView {
    /// All registered probe markers.
    pub markers: Vec<ProbeDebugMarker>,
    /// Whether the overlay is visible.
    pub visible: bool,
    /// Next marker id.
    next_id: u32,
}

/// Returns the default [`ProbeDebugConfig`].
pub fn default_probe_debug_config() -> ProbeDebugConfig {
    ProbeDebugConfig {
        marker_radius: 0.25,
        show_influence: true,
        max_markers: 64,
    }
}

/// Creates a new, empty [`ProbeDebugView`].
pub fn new_probe_debug_view() -> ProbeDebugView {
    ProbeDebugView {
        markers: Vec::new(),
        visible: true,
        next_id: 1,
    }
}

/// Adds a probe debug marker, returning its assigned id.
/// Silently ignores the add if `max_markers` is already reached.
pub fn probe_debug_add_marker(
    view: &mut ProbeDebugView,
    cfg: &ProbeDebugConfig,
    position: [f32; 3],
    influence: [f32; 3],
    label: &str,
) -> u32 {
    if view.markers.len() >= cfg.max_markers {
        return 0;
    }
    let id = view.next_id;
    view.next_id += 1;
    view.markers.push(ProbeDebugMarker {
        id,
        position,
        influence,
        label: label.to_string(),
    });
    id
}

/// Removes a marker by id. Returns `true` if found and removed.
pub fn probe_debug_remove_marker(view: &mut ProbeDebugView, id: u32) -> bool {
    if let Some(pos) = view.markers.iter().position(|m| m.id == id) {
        view.markers.remove(pos);
        true
    } else {
        false
    }
}

/// Returns the number of markers currently registered.
pub fn probe_debug_marker_count(view: &ProbeDebugView) -> usize {
    view.markers.len()
}

/// Sets visibility of the probe debug overlay.
pub fn probe_debug_set_visible(view: &mut ProbeDebugView, visible: bool) {
    view.visible = visible;
}

/// Returns whether the probe debug overlay is currently visible.
pub fn probe_debug_is_visible(view: &ProbeDebugView) -> bool {
    view.visible
}

/// Serialises the current debug view state as JSON.
pub fn probe_debug_to_json(view: &ProbeDebugView, cfg: &ProbeDebugConfig) -> String {
    let mut out = format!(
        "{{\"visible\":{},\"marker_radius\":{:.4},\"show_influence\":{},\"markers\":[",
        view.visible, cfg.marker_radius, cfg.show_influence
    );
    let len = view.markers.len();
    for (i, m) in view.markers.iter().enumerate() {
        let comma = if i + 1 < len { "," } else { "" };
        out.push_str(&format!(
            "{{\"id\":{},\"pos\":[{:.4},{:.4},{:.4}],\
             \"influence\":[{:.4},{:.4},{:.4}],\"label\":\"{}\"}}{comma}",
            m.id,
            m.position[0], m.position[1], m.position[2],
            m.influence[0], m.influence[1], m.influence[2],
            m.label
        ));
    }
    out.push_str("]}");
    out
}

/// Removes all markers and resets state.
pub fn probe_debug_clear(view: &mut ProbeDebugView) {
    view.markers.clear();
    view.next_id = 1;
}

/// Toggles visibility of the overlay.
pub fn probe_debug_toggle(view: &mut ProbeDebugView) {
    view.visible = !view.visible;
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_probe_debug_config();
        assert!((cfg.marker_radius - 0.25).abs() < 1e-6);
        assert!(cfg.show_influence);
        assert_eq!(cfg.max_markers, 64);
    }

    #[test]
    fn new_view_is_empty() {
        let v = new_probe_debug_view();
        assert_eq!(probe_debug_marker_count(&v), 0);
        assert!(probe_debug_is_visible(&v));
    }

    #[test]
    fn add_marker_increments_count() {
        let mut v = new_probe_debug_view();
        let cfg = default_probe_debug_config();
        probe_debug_add_marker(&mut v, &cfg, [1.0, 2.0, 3.0], [2.0, 2.0, 2.0], "probe_0");
        assert_eq!(probe_debug_marker_count(&v), 1);
    }

    #[test]
    fn add_marker_returns_unique_ids() {
        let mut v = new_probe_debug_view();
        let cfg = default_probe_debug_config();
        let id1 = probe_debug_add_marker(&mut v, &cfg, [0.0; 3], [1.0; 3], "a");
        let id2 = probe_debug_add_marker(&mut v, &cfg, [0.0; 3], [1.0; 3], "b");
        assert_ne!(id1, id2);
        assert!(id1 > 0);
        assert!(id2 > 0);
    }

    #[test]
    fn remove_marker_decrements_count() {
        let mut v = new_probe_debug_view();
        let cfg = default_probe_debug_config();
        let id = probe_debug_add_marker(&mut v, &cfg, [0.0; 3], [1.0; 3], "p");
        assert!(probe_debug_remove_marker(&mut v, id));
        assert_eq!(probe_debug_marker_count(&v), 0);
    }

    #[test]
    fn remove_nonexistent_returns_false() {
        let mut v = new_probe_debug_view();
        assert!(!probe_debug_remove_marker(&mut v, 999));
    }

    #[test]
    fn set_visible_false() {
        let mut v = new_probe_debug_view();
        probe_debug_set_visible(&mut v, false);
        assert!(!probe_debug_is_visible(&v));
    }

    #[test]
    fn toggle_visibility() {
        let mut v = new_probe_debug_view();
        assert!(probe_debug_is_visible(&v));
        probe_debug_toggle(&mut v);
        assert!(!probe_debug_is_visible(&v));
        probe_debug_toggle(&mut v);
        assert!(probe_debug_is_visible(&v));
    }

    #[test]
    fn json_contains_markers_key() {
        let mut v = new_probe_debug_view();
        let cfg = default_probe_debug_config();
        probe_debug_add_marker(&mut v, &cfg, [1.0, 0.0, 0.0], [1.0, 1.0, 1.0], "env_probe");
        let json = probe_debug_to_json(&v, &cfg);
        assert!(json.contains("\"markers\""));
        assert!(json.contains("env_probe"));
    }

    #[test]
    fn max_markers_cap() {
        let mut v = new_probe_debug_view();
        let mut cfg = default_probe_debug_config();
        cfg.max_markers = 2;
        probe_debug_add_marker(&mut v, &cfg, [0.0; 3], [1.0; 3], "a");
        probe_debug_add_marker(&mut v, &cfg, [0.0; 3], [1.0; 3], "b");
        let id = probe_debug_add_marker(&mut v, &cfg, [0.0; 3], [1.0; 3], "c");
        assert_eq!(id, 0);
        assert_eq!(probe_debug_marker_count(&v), 2);
    }

    #[test]
    fn clear_resets_state() {
        let mut v = new_probe_debug_view();
        let cfg = default_probe_debug_config();
        probe_debug_add_marker(&mut v, &cfg, [0.0; 3], [1.0; 3], "x");
        probe_debug_clear(&mut v);
        assert_eq!(probe_debug_marker_count(&v), 0);
    }
}
