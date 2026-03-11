// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

//! Properties panel state.

/// State for the properties panel.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct PropertiesPanel {
    /// Index of the currently active tab.
    pub active_tab: u8,
    /// Vertical scroll position in pixels.
    pub scroll_y: f32,
    /// Panel width in pixels.
    pub width: f32,
    /// Expanded/collapsed state for each section.
    pub expanded_sections: Vec<bool>,
}

/// Return the default properties panel with 4 sections, all expanded.
#[allow(dead_code)]
pub fn default_properties_panel() -> PropertiesPanel {
    PropertiesPanel {
        active_tab: 0,
        scroll_y: 0.0,
        width: 280.0,
        expanded_sections: vec![true; 4],
    }
}

/// Switch to a different tab (no-op if tab index is the same).
#[allow(dead_code)]
pub fn switch_tab(panel: &mut PropertiesPanel, tab: u8) {
    panel.active_tab = tab;
}

/// Toggle the expanded/collapsed state of a section by index.
#[allow(dead_code)]
pub fn toggle_section(panel: &mut PropertiesPanel, section: usize) {
    if let Some(s) = panel.expanded_sections.get_mut(section) {
        *s = !*s;
    }
}

/// Scroll the panel vertically by a delta (clamped to >= 0).
#[allow(dead_code)]
pub fn scroll_panel(panel: &mut PropertiesPanel, delta: f32) {
    panel.scroll_y = (panel.scroll_y + delta).max(0.0);
}

/// Return the number of sections in the panel.
#[allow(dead_code)]
pub fn section_count(panel: &PropertiesPanel) -> usize {
    panel.expanded_sections.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_active_tab_is_zero() {
        assert_eq!(default_properties_panel().active_tab, 0);
    }

    #[test]
    fn default_sections_all_expanded() {
        let p = default_properties_panel();
        assert!(p.expanded_sections.iter().all(|&e| e));
    }

    #[test]
    fn section_count_default() {
        assert_eq!(section_count(&default_properties_panel()), 4);
    }

    #[test]
    fn switch_tab_changes_active() {
        let mut p = default_properties_panel();
        switch_tab(&mut p, 3);
        assert_eq!(p.active_tab, 3);
    }

    #[test]
    fn toggle_section_collapses() {
        let mut p = default_properties_panel();
        toggle_section(&mut p, 0);
        assert!(!p.expanded_sections[0]);
    }

    #[test]
    fn toggle_section_expands_again() {
        let mut p = default_properties_panel();
        toggle_section(&mut p, 1);
        toggle_section(&mut p, 1);
        assert!(p.expanded_sections[1]);
    }

    #[test]
    fn toggle_out_of_bounds_no_panic() {
        let mut p = default_properties_panel();
        toggle_section(&mut p, 999);
    }

    #[test]
    fn scroll_increases() {
        let mut p = default_properties_panel();
        scroll_panel(&mut p, 50.0);
        assert!((p.scroll_y - 50.0).abs() < 1e-5);
    }

    #[test]
    fn scroll_clamps_at_zero() {
        let mut p = default_properties_panel();
        scroll_panel(&mut p, -100.0);
        assert_eq!(p.scroll_y, 0.0);
    }

    #[test]
    fn width_default() {
        assert!((default_properties_panel().width - 280.0).abs() < 1e-5);
    }
}
