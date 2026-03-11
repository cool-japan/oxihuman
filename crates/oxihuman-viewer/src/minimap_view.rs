// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Minimap / overview panel showing top-down scene thumbnail.

#![allow(dead_code)]

/// Configuration for the minimap view.
#[derive(Debug, Clone)]
pub struct MinimapConfig {
    /// Width of the minimap panel in pixels.
    pub width_px: u32,
    /// Height of the minimap panel in pixels.
    pub height_px: u32,
    /// World-space extent covered by the minimap (half-size).
    pub world_extent: f32,
    /// Background RGBA colour.
    pub background: [f32; 4],
}

/// A marker point on the minimap.
#[derive(Debug, Clone)]
pub struct MinimapMarker {
    /// Unique identifier.
    pub id: u32,
    /// World-space position [x, z] (top-down).
    pub world_pos: [f32; 2],
    /// RGBA colour.
    pub color: [f32; 4],
    /// Label text.
    pub label: String,
}

/// Runtime state for the minimap.
#[derive(Debug, Clone)]
pub struct MinimapView {
    /// Configuration.
    pub config: MinimapConfig,
    /// Current zoom level (1.0 = no zoom).
    pub zoom: f32,
    /// Minimap centre in world space [x, z].
    pub center: [f32; 2],
    /// All markers.
    pub markers: Vec<MinimapMarker>,
}

/// Returns the default [`MinimapConfig`].
pub fn default_minimap_config() -> MinimapConfig {
    MinimapConfig {
        width_px: 200,
        height_px: 200,
        world_extent: 50.0,
        background: [0.05, 0.05, 0.05, 0.8],
    }
}

/// Creates a new, empty [`MinimapView`].
pub fn new_minimap_view(config: MinimapConfig) -> MinimapView {
    MinimapView {
        config,
        zoom: 1.0,
        center: [0.0, 0.0],
        markers: Vec::new(),
    }
}

/// Adds a marker to the minimap.
pub fn minimap_add_marker(view: &mut MinimapView, marker: MinimapMarker) {
    view.markers.push(marker);
}

/// Removes a marker by id. Returns true if found and removed.
pub fn minimap_remove_marker(view: &mut MinimapView, id: u32) -> bool {
    let before = view.markers.len();
    view.markers.retain(|m| m.id != id);
    view.markers.len() < before
}

/// Returns the number of markers.
pub fn minimap_marker_count(view: &MinimapView) -> usize {
    view.markers.len()
}

/// Converts a world-space [x, z] position to minimap screen pixels [px, py].
/// Returns None if outside the minimap area.
pub fn minimap_world_to_screen(view: &MinimapView, world_xz: [f32; 2]) -> Option<[f32; 2]> {
    let half = view.config.world_extent / view.zoom;
    let dx = world_xz[0] - view.center[0];
    let dz = world_xz[1] - view.center[1];
    if dx.abs() > half || dz.abs() > half {
        return None;
    }
    let nx = (dx / half + 1.0) * 0.5;
    let nz = (dz / half + 1.0) * 0.5;
    Some([
        nx * view.config.width_px as f32,
        nz * view.config.height_px as f32,
    ])
}

/// Sets the minimap zoom level (clamped to 0.1..=20.0).
pub fn minimap_set_zoom(view: &mut MinimapView, zoom: f32) {
    view.zoom = zoom.clamp(0.1, 20.0);
}

/// Centres the minimap on a world-space [x, z] position.
pub fn minimap_center_on(view: &mut MinimapView, world_xz: [f32; 2]) {
    view.center = world_xz;
}

/// Clears all markers.
pub fn minimap_clear(view: &mut MinimapView) {
    view.markers.clear();
}

/// Serialises the minimap state as JSON.
pub fn minimap_to_json(view: &MinimapView) -> String {
    let mut out = format!(
        "{{\"zoom\":{:.4},\"center\":[{:.4},{:.4}],\"markers\":[",
        view.zoom, view.center[0], view.center[1]
    );
    for (i, m) in view.markers.iter().enumerate() {
        let comma = if i + 1 < view.markers.len() { "," } else { "" };
        out.push_str(&format!(
            "{{\"id\":{},\"world\":[{:.4},{:.4}],\"label\":\"{}\"}}{}",
            m.id, m.world_pos[0], m.world_pos[1], m.label, comma
        ));
    }
    out.push_str("]}");
    out
}

// ── internal helpers ───────────────────────────────────────────────────────────

fn make_marker(id: u32, x: f32, z: f32) -> MinimapMarker {
    MinimapMarker {
        id,
        world_pos: [x, z],
        color: [1.0, 0.0, 0.0, 1.0],
        label: format!("marker_{}", id),
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn default_view() -> MinimapView {
        new_minimap_view(default_minimap_config())
    }

    #[test]
    fn default_config_values() {
        let cfg = default_minimap_config();
        assert_eq!(cfg.width_px, 200);
        assert!((cfg.world_extent - 50.0).abs() < 1e-5);
    }

    #[test]
    fn new_view_has_no_markers() {
        let v = default_view();
        assert_eq!(minimap_marker_count(&v), 0);
    }

    #[test]
    fn add_marker_increments_count() {
        let mut v = default_view();
        minimap_add_marker(&mut v, make_marker(1, 0.0, 0.0));
        assert_eq!(minimap_marker_count(&v), 1);
    }

    #[test]
    fn remove_marker_by_id() {
        let mut v = default_view();
        minimap_add_marker(&mut v, make_marker(1, 0.0, 0.0));
        minimap_add_marker(&mut v, make_marker(2, 5.0, 5.0));
        let removed = minimap_remove_marker(&mut v, 1);
        assert!(removed);
        assert_eq!(minimap_marker_count(&v), 1);
    }

    #[test]
    fn remove_nonexistent_marker_returns_false() {
        let mut v = default_view();
        assert!(!minimap_remove_marker(&mut v, 99));
    }

    #[test]
    fn world_to_screen_centre() {
        let v = default_view();
        let px = minimap_world_to_screen(&v, [0.0, 0.0]);
        assert!(px.is_some());
        let px = px.unwrap();
        assert!((px[0] - 100.0).abs() < 1.0);
        assert!((px[1] - 100.0).abs() < 1.0);
    }

    #[test]
    fn world_to_screen_out_of_bounds_returns_none() {
        let v = default_view();
        let px = minimap_world_to_screen(&v, [200.0, 0.0]);
        assert!(px.is_none());
    }

    #[test]
    fn set_zoom_clamped() {
        let mut v = default_view();
        minimap_set_zoom(&mut v, 0.0);
        assert!((v.zoom - 0.1).abs() < 1e-5);
        minimap_set_zoom(&mut v, 100.0);
        assert!((v.zoom - 20.0).abs() < 1e-5);
    }

    #[test]
    fn center_on_updates_center() {
        let mut v = default_view();
        minimap_center_on(&mut v, [10.0, -5.0]);
        assert!((v.center[0] - 10.0).abs() < 1e-5);
        assert!((v.center[1] - (-5.0)).abs() < 1e-5);
    }

    #[test]
    fn clear_removes_all_markers() {
        let mut v = default_view();
        minimap_add_marker(&mut v, make_marker(1, 0.0, 0.0));
        minimap_add_marker(&mut v, make_marker(2, 1.0, 1.0));
        minimap_clear(&mut v);
        assert_eq!(minimap_marker_count(&v), 0);
    }

    #[test]
    fn to_json_contains_zoom_and_markers() {
        let mut v = default_view();
        minimap_add_marker(&mut v, make_marker(1, 0.0, 0.0));
        let json = minimap_to_json(&v);
        assert!(json.contains("\"zoom\""));
        assert!(json.contains("\"markers\""));
        assert!(json.contains("\"center\""));
    }
}
