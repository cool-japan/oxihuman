// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Tool shelf / sidebar view.

/// A tool shelf item.
#[derive(Debug, Clone)]
pub struct ShelfItem {
    pub id: u32,
    pub label: String,
    pub icon: String,
    pub active: bool,
}

/// State for the tool shelf view.
#[derive(Debug, Clone)]
pub struct ToolShelfView {
    pub items: Vec<ShelfItem>,
    pub selected_id: Option<u32>,
    pub collapsed: bool,
    pub enabled: bool,
}

/// Create a new tool shelf view.
pub fn new_tool_shelf_view() -> ToolShelfView {
    ToolShelfView {
        items: Vec::new(),
        selected_id: None,
        collapsed: false,
        enabled: true,
    }
}

/// Add a shelf item.
pub fn tsv_add_item(v: &mut ToolShelfView, id: u32, label: &str, icon: &str) {
    v.items.push(ShelfItem {
        id,
        label: label.to_string(),
        icon: icon.to_string(),
        active: true,
    });
}

/// Select a shelf item by ID.
pub fn tsv_select(v: &mut ToolShelfView, id: u32) {
    v.selected_id = Some(id);
}

/// Toggle collapse state.
pub fn tsv_toggle_collapse(v: &mut ToolShelfView) {
    v.collapsed = !v.collapsed;
}

/// Count active items.
pub fn tsv_active_count(v: &ToolShelfView) -> usize {
    v.items.iter().filter(|i| i.active).count()
}

/// Serialise to JSON.
pub fn tsv_to_json(v: &ToolShelfView) -> String {
    format!(
        r#"{{"item_count":{},"collapsed":{},"enabled":{}}}"#,
        v.items.len(),
        v.collapsed,
        v.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_empty() {
        let v = new_tool_shelf_view();
        assert!(v.items.is_empty() /* no items */);
    }

    #[test]
    fn add_item() {
        let mut v = new_tool_shelf_view();
        tsv_add_item(&mut v, 1, "Select", "cursor");
        assert_eq!(v.items.len(), 1 /* one item */);
    }

    #[test]
    fn select_item() {
        let mut v = new_tool_shelf_view();
        tsv_select(&mut v, 3);
        assert_eq!(v.selected_id, Some(3) /* selected */);
    }

    #[test]
    fn toggle_collapse() {
        let mut v = new_tool_shelf_view();
        assert!(!v.collapsed /* not collapsed */);
        tsv_toggle_collapse(&mut v);
        assert!(v.collapsed /* collapsed */);
        tsv_toggle_collapse(&mut v);
        assert!(!v.collapsed /* back to expanded */);
    }

    #[test]
    fn active_count() {
        let mut v = new_tool_shelf_view();
        tsv_add_item(&mut v, 1, "A", "a");
        tsv_add_item(&mut v, 2, "B", "b");
        v.items[0].active = false;
        assert_eq!(tsv_active_count(&v), 1 /* one active */);
    }

    #[test]
    fn json_has_item_count() {
        let v = new_tool_shelf_view();
        assert!(tsv_to_json(&v).contains("item_count") /* json field */);
    }

    #[test]
    fn enabled_default() {
        let v = new_tool_shelf_view();
        assert!(v.enabled /* enabled */);
    }

    #[test]
    fn icon_stored() {
        let mut v = new_tool_shelf_view();
        tsv_add_item(&mut v, 1, "Move", "move_icon");
        assert_eq!(v.items[0].icon, "move_icon" /* icon stored */);
    }
}
