// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Viewport header bar view.

/// A menu entry in the header bar.
#[derive(Debug, Clone)]
pub struct HeaderMenu {
    pub label: String,
    pub shortcut: String,
    pub enabled: bool,
}

/// State for the header bar view.
#[derive(Debug, Clone)]
pub struct HeaderBarView {
    pub menus: Vec<HeaderMenu>,
    pub editor_type: String,
    pub show_region_header: bool,
    pub enabled: bool,
}

/// Create a new header bar view.
pub fn new_header_bar_view() -> HeaderBarView {
    HeaderBarView {
        menus: Vec::new(),
        editor_type: "3D Viewport".to_string(),
        show_region_header: true,
        enabled: true,
    }
}

/// Add a menu entry.
pub fn hbv_add_menu(v: &mut HeaderBarView, label: &str, shortcut: &str) {
    v.menus.push(HeaderMenu {
        label: label.to_string(),
        shortcut: shortcut.to_string(),
        enabled: true,
    });
}

/// Set the editor type label.
pub fn hbv_set_editor_type(v: &mut HeaderBarView, t: &str) {
    v.editor_type = t.to_string();
}

/// Count enabled menus.
pub fn hbv_enabled_menu_count(v: &HeaderBarView) -> usize {
    v.menus.iter().filter(|m| m.enabled).count()
}

/// Toggle region header visibility.
pub fn hbv_toggle_region_header(v: &mut HeaderBarView) {
    v.show_region_header = !v.show_region_header;
}

/// Serialise to JSON.
pub fn hbv_to_json(v: &HeaderBarView) -> String {
    format!(
        r#"{{"editor_type":"{}","menu_count":{},"enabled":{}}}"#,
        v.editor_type,
        v.menus.len(),
        v.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_editor_type() {
        let v = new_header_bar_view();
        assert_eq!(v.editor_type, "3D Viewport" /* default editor type */);
    }

    #[test]
    fn add_menu() {
        let mut v = new_header_bar_view();
        hbv_add_menu(&mut v, "View", "V");
        assert_eq!(v.menus.len(), 1 /* one menu */);
    }

    #[test]
    fn set_editor_type() {
        let mut v = new_header_bar_view();
        hbv_set_editor_type(&mut v, "Graph Editor");
        assert_eq!(v.editor_type, "Graph Editor" /* updated */);
    }

    #[test]
    fn enabled_menu_count() {
        let mut v = new_header_bar_view();
        hbv_add_menu(&mut v, "A", "a");
        hbv_add_menu(&mut v, "B", "b");
        v.menus[0].enabled = false;
        assert_eq!(hbv_enabled_menu_count(&v), 1 /* one enabled */);
    }

    #[test]
    fn toggle_region_header() {
        let mut v = new_header_bar_view();
        assert!(v.show_region_header /* default shown */);
        hbv_toggle_region_header(&mut v);
        assert!(!v.show_region_header /* toggled off */);
    }

    #[test]
    fn json_has_editor_type() {
        let v = new_header_bar_view();
        assert!(hbv_to_json(&v).contains("editor_type") /* json field */);
    }

    #[test]
    fn enabled_default() {
        let v = new_header_bar_view();
        assert!(v.enabled /* enabled */);
    }

    #[test]
    fn shortcut_stored() {
        let mut v = new_header_bar_view();
        hbv_add_menu(&mut v, "Select", "A");
        assert_eq!(v.menus[0].shortcut, "A" /* shortcut stored */);
    }
}
