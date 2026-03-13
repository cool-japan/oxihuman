// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Modal dialog state.

#![allow(dead_code)]

/// A button within a modal dialog.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ModalButton {
    /// Button identifier.
    pub id: u32,
    /// Display label.
    pub label: String,
    /// Whether this is the default action button.
    pub is_default: bool,
}

/// Modal dialog state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ModalDialog {
    /// Dialog title.
    pub title: String,
    /// Dialog body message.
    pub message: String,
    /// Available buttons.
    pub buttons: Vec<ModalButton>,
    /// Whether the dialog is currently open.
    pub visible: bool,
    /// The id of the button that was pressed to close the dialog, if any.
    pub result: Option<u32>,
}

/// Create a new modal dialog.
#[allow(dead_code)]
pub fn new_modal_dialog(title: &str, msg: &str) -> ModalDialog {
    ModalDialog {
        title: title.to_string(),
        message: msg.to_string(),
        buttons: Vec::new(),
        visible: true,
        result: None,
    }
}

/// Add a button to the dialog.
#[allow(dead_code)]
pub fn add_modal_button(d: &mut ModalDialog, id: u32, label: &str, default: bool) {
    d.buttons.push(ModalButton {
        id,
        label: label.to_string(),
        is_default: default,
    });
}

/// Create a pre-built OK/Cancel confirmation dialog.
#[allow(dead_code)]
pub fn confirm_dialog() -> ModalDialog {
    let mut d = new_modal_dialog("Confirm", "Are you sure?");
    add_modal_button(&mut d, 1, "OK", true);
    add_modal_button(&mut d, 0, "Cancel", false);
    d
}

/// Close the dialog with the given button id as result.
#[allow(dead_code)]
pub fn close_dialog(d: &mut ModalDialog, btn_id: u32) {
    d.result = Some(btn_id);
    d.visible = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_dialog() {
        let d = new_modal_dialog("Title", "Message");
        assert_eq!(d.title, "Title");
        assert_eq!(d.message, "Message");
        assert!(d.visible);
        assert!(d.result.is_none());
        assert!(d.buttons.is_empty());
    }

    #[test]
    fn test_add_button() {
        let mut d = new_modal_dialog("T", "M");
        add_modal_button(&mut d, 1, "Yes", true);
        assert_eq!(d.buttons.len(), 1);
        assert_eq!(d.buttons[0].label, "Yes");
        assert!(d.buttons[0].is_default);
    }

    #[test]
    fn test_add_non_default_button() {
        let mut d = new_modal_dialog("T", "M");
        add_modal_button(&mut d, 2, "No", false);
        assert!(!d.buttons[0].is_default);
    }

    #[test]
    fn test_confirm_dialog() {
        let d = confirm_dialog();
        assert_eq!(d.buttons.len(), 2);
        assert_eq!(d.title, "Confirm");
        let ok_btn = d.buttons.iter().find(|b| b.id == 1);
        assert!(ok_btn.is_some());
    }

    #[test]
    fn test_close_dialog() {
        let mut d = confirm_dialog();
        close_dialog(&mut d, 1);
        assert!(!d.visible);
        assert_eq!(d.result, Some(1));
    }

    #[test]
    fn test_close_with_cancel() {
        let mut d = confirm_dialog();
        close_dialog(&mut d, 0);
        assert_eq!(d.result, Some(0));
    }

    #[test]
    fn test_multiple_buttons() {
        let mut d = new_modal_dialog("Choice", "Pick one");
        add_modal_button(&mut d, 1, "A", false);
        add_modal_button(&mut d, 2, "B", false);
        add_modal_button(&mut d, 3, "C", true);
        assert_eq!(d.buttons.len(), 3);
    }

    #[test]
    fn test_default_button_in_confirm() {
        let d = confirm_dialog();
        let default_btn = d.buttons.iter().find(|b| b.is_default);
        assert!(default_btn.is_some());
        assert_eq!(default_btn.expect("should succeed").id, 1);
    }

    #[test]
    fn test_result_none_initially() {
        let d = new_modal_dialog("T", "M");
        assert!(d.result.is_none());
    }
}
