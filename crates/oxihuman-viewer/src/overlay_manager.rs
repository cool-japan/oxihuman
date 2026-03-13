// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Overlay visibility manager (which overlays are shown).

#![allow(dead_code)]

/// Flags controlling which viewport overlays are visible.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OverlayFlags {
    pub show_wireframe: bool,
    pub show_normals: bool,
    pub show_weights: bool,
    pub show_grid: bool,
    pub show_axes: bool,
    pub show_stats: bool,
    pub show_bones: bool,
}

/// Returns default overlay flags (grid and axes visible).
#[allow(dead_code)]
pub fn default_overlay_flags() -> OverlayFlags {
    OverlayFlags {
        show_wireframe: false,
        show_normals: false,
        show_weights: false,
        show_grid: true,
        show_axes: true,
        show_stats: false,
        show_bones: false,
    }
}

/// Toggles wireframe overlay.
#[allow(dead_code)]
pub fn toggle_wireframe(flags: &mut OverlayFlags) {
    flags.show_wireframe = !flags.show_wireframe;
}

/// Toggles normals overlay.
#[allow(dead_code)]
pub fn toggle_normals(flags: &mut OverlayFlags) {
    flags.show_normals = !flags.show_normals;
}

/// Toggles weights overlay.
#[allow(dead_code)]
pub fn toggle_weights(flags: &mut OverlayFlags) {
    flags.show_weights = !flags.show_weights;
}

/// Returns true if any overlay is currently active.
#[allow(dead_code)]
pub fn any_overlay_active(flags: &OverlayFlags) -> bool {
    flags.show_wireframe
        || flags.show_normals
        || flags.show_weights
        || flags.show_grid
        || flags.show_axes
        || flags.show_stats
        || flags.show_bones
}

// ── Overlay Manager (layer-based) ─────────────────────────────────────────────

#[allow(dead_code)]
pub struct OverlayLayer {
    pub name: String,
    pub enabled: bool,
    pub z_order: i32,
}

#[allow(dead_code)]
pub struct OverlayManager {
    pub layers: Vec<OverlayLayer>,
}

#[allow(dead_code)]
pub fn new_overlay_manager() -> OverlayManager {
    OverlayManager { layers: Vec::new() }
}

#[allow(dead_code)]
pub fn om_add_layer(m: &mut OverlayManager, name: &str, z_order: i32) -> usize {
    let idx = m.layers.len();
    m.layers.push(OverlayLayer { name: name.to_string(), enabled: true, z_order });
    idx
}

#[allow(dead_code)]
pub fn om_enable_layer(m: &mut OverlayManager, idx: usize) {
    if idx < m.layers.len() {
        m.layers[idx].enabled = true;
    }
}

#[allow(dead_code)]
pub fn om_disable_layer(m: &mut OverlayManager, idx: usize) {
    if idx < m.layers.len() {
        m.layers[idx].enabled = false;
    }
}

#[allow(dead_code)]
pub fn om_layer_count(m: &OverlayManager) -> usize {
    m.layers.len()
}

#[allow(dead_code)]
pub fn om_visible_count(m: &OverlayManager) -> usize {
    m.layers.iter().filter(|l| l.enabled).count()
}

#[allow(dead_code)]
pub fn om_sorted_layers(m: &OverlayManager) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..m.layers.len()).collect();
    indices.sort_by_key(|&i| m.layers[i].z_order);
    indices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_grid_on() {
        let f = default_overlay_flags();
        assert!(f.show_grid);
        assert!(f.show_axes);
    }

    #[test]
    fn test_default_wireframe_off() {
        let f = default_overlay_flags();
        assert!(!f.show_wireframe);
    }

    #[test]
    fn test_toggle_wireframe() {
        let mut f = default_overlay_flags();
        toggle_wireframe(&mut f);
        assert!(f.show_wireframe);
        toggle_wireframe(&mut f);
        assert!(!f.show_wireframe);
    }

    #[test]
    fn test_toggle_normals() {
        let mut f = default_overlay_flags();
        toggle_normals(&mut f);
        assert!(f.show_normals);
    }

    #[test]
    fn test_toggle_weights() {
        let mut f = default_overlay_flags();
        toggle_weights(&mut f);
        assert!(f.show_weights);
    }

    #[test]
    fn test_any_overlay_default_true() {
        let f = default_overlay_flags();
        assert!(any_overlay_active(&f)); // grid/axes on by default
    }

    #[test]
    fn test_any_overlay_all_off() {
        let f = OverlayFlags {
            show_wireframe: false,
            show_normals: false,
            show_weights: false,
            show_grid: false,
            show_axes: false,
            show_stats: false,
            show_bones: false,
        };
        assert!(!any_overlay_active(&f));
    }

    #[test]
    fn test_any_overlay_one_on() {
        let mut f = OverlayFlags {
            show_wireframe: false,
            show_normals: false,
            show_weights: false,
            show_grid: false,
            show_axes: false,
            show_stats: false,
            show_bones: false,
        };
        f.show_bones = true;
        assert!(any_overlay_active(&f));
    }

    #[test]
    fn test_double_toggle_returns_to_default() {
        let mut f = default_overlay_flags();
        let original = f.show_normals;
        toggle_normals(&mut f);
        toggle_normals(&mut f);
        assert_eq!(f.show_normals, original);
    }

    /* OverlayManager tests */
    #[test]
    fn test_om_add_layer() {
        let mut m = new_overlay_manager();
        let idx = om_add_layer(&mut m, "hud", 10);
        assert_eq!(idx, 0);
    }

    #[test]
    fn test_om_layer_count() {
        let mut m = new_overlay_manager();
        om_add_layer(&mut m, "a", 0);
        om_add_layer(&mut m, "b", 1);
        assert_eq!(om_layer_count(&m), 2);
    }

    #[test]
    fn test_om_enable_disable() {
        let mut m = new_overlay_manager();
        let idx = om_add_layer(&mut m, "dbg", 0);
        om_disable_layer(&mut m, idx);
        assert!(!m.layers[idx].enabled);
        om_enable_layer(&mut m, idx);
        assert!(m.layers[idx].enabled);
    }

    #[test]
    fn test_om_visible_count() {
        let mut m = new_overlay_manager();
        om_add_layer(&mut m, "a", 0);
        om_add_layer(&mut m, "b", 1);
        om_disable_layer(&mut m, 1);
        assert_eq!(om_visible_count(&m), 1);
    }

    #[test]
    fn test_om_sorted_layers() {
        let mut m = new_overlay_manager();
        om_add_layer(&mut m, "high", 10);
        om_add_layer(&mut m, "low", 1);
        let sorted = om_sorted_layers(&m);
        assert_eq!(sorted[0], 1);
    }

    #[test]
    fn test_om_enable_oob_safe() {
        let mut m = new_overlay_manager();
        om_enable_layer(&mut m, 99);
    }

    #[test]
    fn test_om_disable_oob_safe() {
        let mut m = new_overlay_manager();
        om_disable_layer(&mut m, 99);
    }

    #[test]
    fn test_om_sorted_empty() {
        let m = new_overlay_manager();
        assert!(om_sorted_layers(&m).is_empty());
    }
}
