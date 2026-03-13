// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Context menu state (right-click menus).

#![allow(dead_code)]

/// A single item in a context menu.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ContextMenuItem {
    /// Unique item identifier.
    pub id: u32,
    /// Display label.
    pub label: String,
    /// Whether the item is interactive.
    pub enabled: bool,
    /// Whether the item is currently checked.
    pub checked: bool,
    /// Whether this item renders as a separator.
    pub separator: bool,
}

/// Context menu state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ContextMenu {
    /// List of menu items.
    pub items: Vec<ContextMenuItem>,
    /// Screen position where the menu is anchored.
    pub position: [f32; 2],
    /// Whether the menu is currently visible.
    pub visible: bool,
}

/// Create a new context menu anchored at `pos`.
#[allow(dead_code)]
pub fn new_context_menu(pos: [f32; 2]) -> ContextMenu {
    ContextMenu {
        items: Vec::new(),
        position: pos,
        visible: true,
    }
}

/// Add an item to the context menu.
#[allow(dead_code)]
pub fn add_menu_item(menu: &mut ContextMenu, id: u32, label: &str, enabled: bool) {
    menu.items.push(ContextMenuItem {
        id,
        label: label.to_string(),
        enabled,
        checked: false,
        separator: false,
    });
}

/// Add a separator item to the context menu.
#[allow(dead_code)]
pub fn add_separator(menu: &mut ContextMenu) {
    menu.items.push(ContextMenuItem {
        id: u32::MAX,
        label: String::new(),
        enabled: false,
        checked: false,
        separator: true,
    });
}

/// Hide the context menu.
#[allow(dead_code)]
pub fn hide_menu(menu: &mut ContextMenu) {
    menu.visible = false;
}

/// Find a menu item by id, returning a reference if found.
#[allow(dead_code)]
pub fn find_item(menu: &ContextMenu, id: u32) -> Option<&ContextMenuItem> {
    menu.items.iter().find(|item| item.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_context_menu() {
        let m = new_context_menu([100.0, 200.0]);
        assert_eq!(m.position, [100.0, 200.0]);
        assert!(m.visible);
        assert!(m.items.is_empty());
    }

    #[test]
    fn test_add_menu_item() {
        let mut m = new_context_menu([0.0, 0.0]);
        add_menu_item(&mut m, 1, "Copy", true);
        assert_eq!(m.items.len(), 1);
        assert_eq!(m.items[0].label, "Copy");
        assert!(m.items[0].enabled);
    }

    #[test]
    fn test_add_disabled_item() {
        let mut m = new_context_menu([0.0, 0.0]);
        add_menu_item(&mut m, 2, "Paste", false);
        assert!(!m.items[0].enabled);
    }

    #[test]
    fn test_add_separator() {
        let mut m = new_context_menu([0.0, 0.0]);
        add_menu_item(&mut m, 1, "Cut", true);
        add_separator(&mut m);
        add_menu_item(&mut m, 2, "Delete", true);
        assert_eq!(m.items.len(), 3);
        assert!(m.items[1].separator);
    }

    #[test]
    fn test_hide_menu() {
        let mut m = new_context_menu([10.0, 20.0]);
        assert!(m.visible);
        hide_menu(&mut m);
        assert!(!m.visible);
    }

    #[test]
    fn test_find_item_found() {
        let mut m = new_context_menu([0.0, 0.0]);
        add_menu_item(&mut m, 42, "Rename", true);
        let item = find_item(&m, 42);
        assert!(item.is_some());
        assert_eq!(item.expect("should succeed").label, "Rename");
    }

    #[test]
    fn test_find_item_not_found() {
        let m = new_context_menu([0.0, 0.0]);
        assert!(find_item(&m, 99).is_none());
    }

    #[test]
    fn test_multiple_items() {
        let mut m = new_context_menu([5.0, 5.0]);
        for i in 0..5u32 {
            add_menu_item(&mut m, i, "Item", true);
        }
        assert_eq!(m.items.len(), 5);
    }

    #[test]
    fn test_item_not_checked_by_default() {
        let mut m = new_context_menu([0.0, 0.0]);
        add_menu_item(&mut m, 1, "Option", true);
        assert!(!m.items[0].checked);
    }
}
