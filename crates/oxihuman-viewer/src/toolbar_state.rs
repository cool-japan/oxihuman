// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Toolbar button and tool state.

/// A single toolbar button.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ToolbarButton {
    /// Unique identifier for this button.
    pub id: u32,
    /// Display label.
    pub label: String,
    /// Icon type identifier.
    pub icon_type: u8,
    /// Whether the button is enabled.
    pub enabled: bool,
    /// Whether the button is currently active/pressed.
    pub active: bool,
}

/// State for a toolbar containing multiple buttons.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ToolbarState {
    /// All buttons in this toolbar.
    pub buttons: Vec<ToolbarButton>,
    /// The id of the currently active tool.
    pub active_tool: u32,
}

/// Create a new empty toolbar state.
#[allow(dead_code)]
pub fn new_toolbar_state() -> ToolbarState {
    ToolbarState {
        buttons: Vec::new(),
        active_tool: 0,
    }
}

/// Add a button to the toolbar.
#[allow(dead_code)]
pub fn add_button(toolbar: &mut ToolbarState, id: u32, label: &str, icon: u8) {
    toolbar.buttons.push(ToolbarButton {
        id,
        label: label.to_string(),
        icon_type: icon,
        enabled: true,
        active: false,
    });
}

/// Set the active tool by id; updates `active_tool` and marks the matching button active.
#[allow(dead_code)]
pub fn set_active_tool(toolbar: &mut ToolbarState, id: u32) {
    toolbar.active_tool = id;
    for b in &mut toolbar.buttons {
        b.active = b.id == id;
    }
}

/// Return whether a given tool id is currently active.
#[allow(dead_code)]
pub fn is_tool_active(toolbar: &ToolbarState, id: u32) -> bool {
    toolbar.active_tool == id
}

/// Return the number of buttons in the toolbar.
#[allow(dead_code)]
pub fn button_count(toolbar: &ToolbarState) -> usize {
    toolbar.buttons.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_toolbar_is_empty() {
        assert_eq!(button_count(&new_toolbar_state()), 0);
    }

    #[test]
    fn add_button_increases_count() {
        let mut t = new_toolbar_state();
        add_button(&mut t, 1, "Select", 0);
        assert_eq!(button_count(&t), 1);
    }

    #[test]
    fn button_enabled_by_default() {
        let mut t = new_toolbar_state();
        add_button(&mut t, 1, "Move", 1);
        assert!(t.buttons[0].enabled);
    }

    #[test]
    fn button_not_active_by_default() {
        let mut t = new_toolbar_state();
        add_button(&mut t, 1, "Scale", 2);
        assert!(!t.buttons[0].active);
    }

    #[test]
    fn set_active_tool_updates_id() {
        let mut t = new_toolbar_state();
        add_button(&mut t, 1, "Rotate", 3);
        set_active_tool(&mut t, 1);
        assert_eq!(t.active_tool, 1);
    }

    #[test]
    fn set_active_tool_marks_button() {
        let mut t = new_toolbar_state();
        add_button(&mut t, 1, "A", 0);
        add_button(&mut t, 2, "B", 0);
        set_active_tool(&mut t, 2);
        assert!(!t.buttons[0].active);
        assert!(t.buttons[1].active);
    }

    #[test]
    fn is_tool_active_true() {
        let mut t = new_toolbar_state();
        set_active_tool(&mut t, 5);
        assert!(is_tool_active(&t, 5));
    }

    #[test]
    fn is_tool_active_false() {
        let t = new_toolbar_state();
        assert!(!is_tool_active(&t, 99));
    }

    #[test]
    fn button_label_stored() {
        let mut t = new_toolbar_state();
        add_button(&mut t, 7, "Paint", 4);
        assert_eq!(t.buttons[0].label, "Paint");
    }

    #[test]
    fn icon_type_stored() {
        let mut t = new_toolbar_state();
        add_button(&mut t, 3, "Tool", 15);
        assert_eq!(t.buttons[0].icon_type, 15);
    }
}
