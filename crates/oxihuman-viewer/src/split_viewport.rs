// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Split-view layout manager (2×1 or 2×2 viewport panels with independent cameras).

#![allow(dead_code)]

/// Layout preset names understood by the split-viewport manager.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SplitLayout {
    /// Two panels side by side.
    TwoByOne,
    /// Four panels in a 2×2 grid.
    TwoByTwo,
    /// A single full-screen panel.
    Single,
}

/// Configuration for the split-viewport manager.
#[derive(Debug, Clone)]
pub struct SplitViewportConfig {
    /// Initial layout.
    pub layout: SplitLayout,
    /// Total width in pixels.
    pub total_width: u32,
    /// Total height in pixels.
    pub total_height: u32,
    /// Pretty-print JSON output.
    pub pretty: bool,
}

/// A single viewport panel with its own camera label and pixel bounds.
#[derive(Debug, Clone)]
pub struct ViewportPanel {
    /// Panel index (0-based).
    pub index: usize,
    /// Human-readable camera label (e.g. "Front", "Side", "Perspective").
    pub camera_label: String,
    /// Pixel x offset.
    pub x: u32,
    /// Pixel y offset.
    pub y: u32,
    /// Panel width in pixels.
    pub width: u32,
    /// Panel height in pixels.
    pub height: u32,
    /// Whether this panel is currently active (receives input).
    pub active: bool,
}

/// The split-viewport state container.
#[derive(Debug, Clone)]
pub struct SplitViewport {
    /// All viewport panels.
    pub panels: Vec<ViewportPanel>,
    /// Index of the currently active panel.
    pub active_index: usize,
    /// Current layout name.
    pub layout: SplitLayout,
    /// Total width in pixels.
    pub total_width: u32,
    /// Total height in pixels.
    pub total_height: u32,
}

/// Returns the default [`SplitViewportConfig`].
pub fn default_split_viewport_config() -> SplitViewportConfig {
    SplitViewportConfig {
        layout: SplitLayout::TwoByOne,
        total_width: 1280,
        total_height: 720,
        pretty: true,
    }
}

/// Creates a new, empty [`SplitViewport`].
pub fn new_split_viewport(cfg: &SplitViewportConfig) -> SplitViewport {
    SplitViewport {
        panels: Vec::new(),
        active_index: 0,
        layout: cfg.layout.clone(),
        total_width: cfg.total_width,
        total_height: cfg.total_height,
    }
}

/// Adds a viewport panel to the manager.
pub fn split_viewport_add_panel(vp: &mut SplitViewport, panel: ViewportPanel) {
    vp.panels.push(panel);
}

/// Returns the number of panels.
pub fn split_viewport_panel_count(vp: &SplitViewport) -> usize {
    vp.panels.len()
}

/// Sets the active panel by index; marks only that panel as active.
pub fn split_viewport_set_active(vp: &mut SplitViewport, index: usize) {
    if index < vp.panels.len() {
        for panel in &mut vp.panels {
            panel.active = false;
        }
        vp.panels[index].active = true;
        vp.active_index = index;
    }
}

/// Returns a reference to the currently active panel, if any.
pub fn split_viewport_active_panel(vp: &SplitViewport) -> Option<&ViewportPanel> {
    vp.panels.iter().find(|p| p.active)
}

/// Resizes the total viewport and scales all panel dimensions proportionally.
pub fn split_viewport_resize(vp: &mut SplitViewport, new_width: u32, new_height: u32) {
    let old_w = vp.total_width.max(1) as f64;
    let old_h = vp.total_height.max(1) as f64;
    let sx = new_width as f64 / old_w;
    let sy = new_height as f64 / old_h;
    for panel in &mut vp.panels {
        panel.x = (panel.x as f64 * sx).round() as u32;
        panel.y = (panel.y as f64 * sy).round() as u32;
        panel.width = (panel.width as f64 * sx).round() as u32;
        panel.height = (panel.height as f64 * sy).round() as u32;
    }
    vp.total_width = new_width;
    vp.total_height = new_height;
}

/// Serialises the split-viewport state as JSON.
pub fn split_viewport_to_json(vp: &SplitViewport, cfg: &SplitViewportConfig) -> String {
    let indent = if cfg.pretty { "  " } else { "" };
    let nl = if cfg.pretty { "\n" } else { "" };
    let layout = split_viewport_layout_name(vp);
    let mut out = format!(
        "{{{nl}{indent}\"layout\":\"{layout}\",{nl}{indent}\"total_width\":{},{nl}{indent}\"total_height\":{},{nl}{indent}\"active_index\":{},{nl}{indent}\"panels\":[{nl}",
        vp.total_width, vp.total_height, vp.active_index
    );
    for (i, p) in vp.panels.iter().enumerate() {
        let comma = if i + 1 < vp.panels.len() { "," } else { "" };
        out.push_str(&format!(
            "{indent}{indent}{{\"index\":{},\"camera\":\"{}\",\"x\":{},\"y\":{},\"w\":{},\"h\":{},\"active\":{}}}{}{nl}",
            p.index, p.camera_label, p.x, p.y, p.width, p.height, p.active, comma
        ));
    }
    out.push_str(&format!("{indent}]{nl}}}"));
    out
}

/// Removes all panels.
pub fn split_viewport_clear(vp: &mut SplitViewport) {
    vp.panels.clear();
    vp.active_index = 0;
}

/// Returns a string name for the current layout.
pub fn split_viewport_layout_name(vp: &SplitViewport) -> &'static str {
    match vp.layout {
        SplitLayout::TwoByOne => "2x1",
        SplitLayout::TwoByTwo => "2x2",
        SplitLayout::Single => "single",
    }
}

// ── internal helpers ───────────────────────────────────────────────────────────

fn make_panel(index: usize, camera_label: &str, x: u32, y: u32, width: u32, height: u32) -> ViewportPanel {
    ViewportPanel {
        index,
        camera_label: camera_label.to_string(),
        x,
        y,
        width,
        height,
        active: false,
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_split_viewport_config();
        assert_eq!(cfg.layout, SplitLayout::TwoByOne);
        assert_eq!(cfg.total_width, 1280);
        assert_eq!(cfg.total_height, 720);
    }

    #[test]
    fn new_viewport_is_empty() {
        let cfg = default_split_viewport_config();
        let vp = new_split_viewport(&cfg);
        assert_eq!(split_viewport_panel_count(&vp), 0);
    }

    #[test]
    fn add_panel_increments_count() {
        let cfg = default_split_viewport_config();
        let mut vp = new_split_viewport(&cfg);
        split_viewport_add_panel(&mut vp, make_panel(0, "Front", 0, 0, 640, 720));
        assert_eq!(split_viewport_panel_count(&vp), 1);
    }

    #[test]
    fn set_active_marks_correct_panel() {
        let cfg = default_split_viewport_config();
        let mut vp = new_split_viewport(&cfg);
        split_viewport_add_panel(&mut vp, make_panel(0, "Front", 0, 0, 640, 720));
        split_viewport_add_panel(&mut vp, make_panel(1, "Side", 640, 0, 640, 720));
        split_viewport_set_active(&mut vp, 1);
        assert_eq!(vp.active_index, 1);
        assert!(!vp.panels[0].active);
        assert!(vp.panels[1].active);
    }

    #[test]
    fn active_panel_returns_none_on_empty() {
        let cfg = default_split_viewport_config();
        let vp = new_split_viewport(&cfg);
        assert!(split_viewport_active_panel(&vp).is_none());
    }

    #[test]
    fn active_panel_returns_correct_label() {
        let cfg = default_split_viewport_config();
        let mut vp = new_split_viewport(&cfg);
        split_viewport_add_panel(&mut vp, make_panel(0, "Perspective", 0, 0, 1280, 720));
        split_viewport_set_active(&mut vp, 0);
        let panel = split_viewport_active_panel(&vp).expect("should succeed");
        assert_eq!(panel.camera_label, "Perspective");
    }

    #[test]
    fn resize_scales_panels() {
        let cfg = default_split_viewport_config();
        let mut vp = new_split_viewport(&cfg);
        split_viewport_add_panel(&mut vp, make_panel(0, "Front", 0, 0, 640, 720));
        split_viewport_resize(&mut vp, 2560, 1440);
        assert_eq!(vp.total_width, 2560);
        assert_eq!(vp.panels[0].width, 1280);
    }

    #[test]
    fn json_contains_layout_and_panels() {
        let cfg = default_split_viewport_config();
        let mut vp = new_split_viewport(&cfg);
        split_viewport_add_panel(&mut vp, make_panel(0, "Front", 0, 0, 640, 720));
        let json = split_viewport_to_json(&vp, &cfg);
        assert!(json.contains("\"layout\""));
        assert!(json.contains("\"panels\""));
        assert!(json.contains("Front"));
    }

    #[test]
    fn layout_name_correct() {
        let cfg = SplitViewportConfig {
            layout: SplitLayout::TwoByTwo,
            total_width: 1280,
            total_height: 720,
            pretty: false,
        };
        let vp = new_split_viewport(&cfg);
        assert_eq!(split_viewport_layout_name(&vp), "2x2");
    }

    #[test]
    fn clear_removes_all_panels() {
        let cfg = default_split_viewport_config();
        let mut vp = new_split_viewport(&cfg);
        split_viewport_add_panel(&mut vp, make_panel(0, "Front", 0, 0, 640, 720));
        split_viewport_add_panel(&mut vp, make_panel(1, "Side", 640, 0, 640, 720));
        split_viewport_clear(&mut vp);
        assert_eq!(split_viewport_panel_count(&vp), 0);
    }
}
