// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Outliner panel view state.

/// A single entry in the outliner.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct OutlinerEntry {
    /// Unique identifier for this entry.
    pub id: u32,
    /// Display name.
    pub name: String,
    /// Nesting depth (0 = root).
    pub depth: u32,
    /// Whether this entry is expanded.
    pub expanded: bool,
    /// Whether this entry is selected.
    pub selected: bool,
    /// Whether this entry is visible in the viewport.
    pub visible: bool,
}

/// State for the outliner panel.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct OutlinerView {
    /// All entries in the outliner.
    pub entries: Vec<OutlinerEntry>,
    /// Vertical scroll position in pixels.
    pub scroll_y: f32,
    /// Current filter string.
    pub filter: String,
}

/// Return an empty outliner view.
#[allow(dead_code)]
pub fn default_outliner_view() -> OutlinerView {
    OutlinerView {
        entries: Vec::new(),
        scroll_y: 0.0,
        filter: String::new(),
    }
}

/// Add a new entry to the outliner.
#[allow(dead_code)]
pub fn add_entry(view: &mut OutlinerView, id: u32, name: &str, depth: u32) {
    view.entries.push(OutlinerEntry {
        id,
        name: name.to_string(),
        depth,
        expanded: true,
        selected: false,
        visible: true,
    });
}

/// Toggle the expanded state of the entry with the given id.
#[allow(dead_code)]
pub fn toggle_expand(view: &mut OutlinerView, id: u32) {
    if let Some(e) = view.entries.iter_mut().find(|e| e.id == id) {
        e.expanded = !e.expanded;
    }
}

/// Select the entry with the given id (deselects all others).
#[allow(dead_code)]
pub fn select_entry(view: &mut OutlinerView, id: u32) {
    for e in &mut view.entries {
        e.selected = e.id == id;
    }
}

/// Return all entries that are visible (not hidden by the visible flag).
#[allow(dead_code)]
pub fn visible_entries(view: &OutlinerView) -> Vec<&OutlinerEntry> {
    view.entries.iter().filter(|e| e.visible).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_empty() {
        let v = default_outliner_view();
        assert!(v.entries.is_empty());
    }

    #[test]
    fn add_entry_increases_count() {
        let mut v = default_outliner_view();
        add_entry(&mut v, 1, "Root", 0);
        assert_eq!(v.entries.len(), 1);
    }

    #[test]
    fn added_entry_defaults_expanded() {
        let mut v = default_outliner_view();
        add_entry(&mut v, 1, "Node", 0);
        assert!(v.entries[0].expanded);
    }

    #[test]
    fn toggle_expand_collapses() {
        let mut v = default_outliner_view();
        add_entry(&mut v, 1, "Node", 0);
        toggle_expand(&mut v, 1);
        assert!(!v.entries[0].expanded);
    }

    #[test]
    fn toggle_expand_unknown_id_no_panic() {
        let mut v = default_outliner_view();
        toggle_expand(&mut v, 999);
    }

    #[test]
    fn select_entry_selects_only_one() {
        let mut v = default_outliner_view();
        add_entry(&mut v, 1, "A", 0);
        add_entry(&mut v, 2, "B", 0);
        select_entry(&mut v, 1);
        assert!(v.entries[0].selected);
        assert!(!v.entries[1].selected);
    }

    #[test]
    fn select_entry_deselects_previous() {
        let mut v = default_outliner_view();
        add_entry(&mut v, 1, "A", 0);
        add_entry(&mut v, 2, "B", 0);
        select_entry(&mut v, 1);
        select_entry(&mut v, 2);
        assert!(!v.entries[0].selected);
        assert!(v.entries[1].selected);
    }

    #[test]
    fn visible_entries_returns_all_by_default() {
        let mut v = default_outliner_view();
        add_entry(&mut v, 1, "A", 0);
        add_entry(&mut v, 2, "B", 0);
        assert_eq!(visible_entries(&v).len(), 2);
    }

    #[test]
    fn hidden_entry_excluded_from_visible() {
        let mut v = default_outliner_view();
        add_entry(&mut v, 1, "A", 0);
        v.entries[0].visible = false;
        assert_eq!(visible_entries(&v).len(), 0);
    }

    #[test]
    fn entry_name_stored_correctly() {
        let mut v = default_outliner_view();
        add_entry(&mut v, 42, "Camera", 2);
        assert_eq!(v.entries[0].name, "Camera");
        assert_eq!(v.entries[0].depth, 2);
    }
}
