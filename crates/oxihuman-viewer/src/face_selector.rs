// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! FaceSelector — face (triangle/polygon) selection management.

#![allow(dead_code)]

/// Selection mode.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    Replace,
    Add,
    Subtract,
    Toggle,
}

/// Tracks which face indices are selected.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct FaceSelection {
    pub selected: Vec<bool>,
}

/// Create a `FaceSelection` for `n` faces (none selected).
#[allow(dead_code)]
pub fn new_face_selection(n: usize) -> FaceSelection {
    FaceSelection { selected: vec![false; n] }
}

/// Select face `index`.
#[allow(dead_code)]
pub fn select_face(sel: &mut FaceSelection, index: usize) {
    if index < sel.selected.len() {
        sel.selected[index] = true;
    }
}

/// Deselect face `index`.
#[allow(dead_code)]
pub fn deselect_face(sel: &mut FaceSelection, index: usize) {
    if index < sel.selected.len() {
        sel.selected[index] = false;
    }
}

/// Toggle face `index`.
#[allow(dead_code)]
pub fn toggle_face(sel: &mut FaceSelection, index: usize) {
    if index < sel.selected.len() {
        sel.selected[index] = !sel.selected[index];
    }
}

/// Return the number of selected faces.
#[allow(dead_code)]
pub fn selected_count(sel: &FaceSelection) -> usize {
    sel.selected.iter().filter(|&&v| v).count()
}

/// Return whether face `index` is selected.
#[allow(dead_code)]
pub fn is_selected(sel: &FaceSelection, index: usize) -> bool {
    sel.selected.get(index).copied().unwrap_or(false)
}

/// Return a sorted list of selected face indices.
#[allow(dead_code)]
pub fn selection_to_list(sel: &FaceSelection) -> Vec<usize> {
    sel.selected.iter().enumerate().filter_map(|(i, &v)| if v { Some(i) } else { None }).collect()
}

/// Clear all selections.
#[allow(dead_code)]
pub fn clear_selection(sel: &mut FaceSelection) {
    sel.selected.fill(false);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_face_selection_none_selected() {
        let sel = new_face_selection(5);
        assert_eq!(selected_count(&sel), 0);
    }

    #[test]
    fn test_select_face() {
        let mut sel = new_face_selection(5);
        select_face(&mut sel, 2);
        assert!(is_selected(&sel, 2));
        assert_eq!(selected_count(&sel), 1);
    }

    #[test]
    fn test_deselect_face() {
        let mut sel = new_face_selection(5);
        select_face(&mut sel, 1);
        deselect_face(&mut sel, 1);
        assert!(!is_selected(&sel, 1));
    }

    #[test]
    fn test_toggle_face() {
        let mut sel = new_face_selection(3);
        toggle_face(&mut sel, 0);
        assert!(is_selected(&sel, 0));
        toggle_face(&mut sel, 0);
        assert!(!is_selected(&sel, 0));
    }

    #[test]
    fn test_is_selected_oob() {
        let sel = new_face_selection(2);
        assert!(!is_selected(&sel, 99));
    }

    #[test]
    fn test_selection_to_list() {
        let mut sel = new_face_selection(4);
        select_face(&mut sel, 0);
        select_face(&mut sel, 3);
        let list = selection_to_list(&sel);
        assert_eq!(list, vec![0, 3]);
    }

    #[test]
    fn test_clear_selection() {
        let mut sel = new_face_selection(3);
        select_face(&mut sel, 0);
        select_face(&mut sel, 1);
        clear_selection(&mut sel);
        assert_eq!(selected_count(&sel), 0);
    }

    #[test]
    fn test_selection_mode_enum() {
        assert_ne!(SelectionMode::Add, SelectionMode::Subtract);
        assert_eq!(SelectionMode::Toggle, SelectionMode::Toggle);
    }
}
