#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

/// Current selection in edit mode.
#[derive(Debug, Clone, Default)]
pub struct EditSelection {
    pub selected_verts: Vec<u32>,
    pub selected_edges: Vec<(u32, u32)>,
    pub selected_faces: Vec<u32>,
}

/// Edit mode state (selected verts/edges/faces).
#[derive(Debug, Clone)]
pub struct EditModeState {
    pub selection: EditSelection,
    pub mode: u8,
    pub pivot: [f32; 3],
}

#[allow(dead_code)]
pub fn new_edit_mode_state() -> EditModeState {
    EditModeState {
        selection: EditSelection::default(),
        mode: 0,
        pivot: [0.0, 0.0, 0.0],
    }
}

#[allow(dead_code)]
pub fn select_vert(state: &mut EditModeState, idx: u32) {
    if !state.selection.selected_verts.contains(&idx) {
        state.selection.selected_verts.push(idx);
    }
}

#[allow(dead_code)]
pub fn deselect_all(state: &mut EditModeState) {
    state.selection.selected_verts.clear();
    state.selection.selected_edges.clear();
    state.selection.selected_faces.clear();
}

#[allow(dead_code)]
pub fn selected_vert_count(state: &EditModeState) -> usize {
    state.selection.selected_verts.len()
}

#[allow(dead_code)]
pub fn edit_mode_name(mode: u8) -> &'static str {
    match mode {
        0 => "Vertex",
        1 => "Edge",
        2 => "Face",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_state_empty() {
        let s = new_edit_mode_state();
        assert_eq!(selected_vert_count(&s), 0);
    }

    #[test]
    fn test_select_vert() {
        let mut s = new_edit_mode_state();
        select_vert(&mut s, 3);
        assert_eq!(selected_vert_count(&s), 1);
    }

    #[test]
    fn test_select_vert_no_duplicate() {
        let mut s = new_edit_mode_state();
        select_vert(&mut s, 5);
        select_vert(&mut s, 5);
        assert_eq!(selected_vert_count(&s), 1);
    }

    #[test]
    fn test_deselect_all() {
        let mut s = new_edit_mode_state();
        select_vert(&mut s, 0);
        select_vert(&mut s, 1);
        deselect_all(&mut s);
        assert_eq!(selected_vert_count(&s), 0);
    }

    #[test]
    fn test_edit_mode_name_vertex() {
        assert_eq!(edit_mode_name(0), "Vertex");
    }

    #[test]
    fn test_edit_mode_name_edge() {
        assert_eq!(edit_mode_name(1), "Edge");
    }

    #[test]
    fn test_edit_mode_name_face() {
        assert_eq!(edit_mode_name(2), "Face");
    }

    #[test]
    fn test_edit_mode_name_unknown() {
        assert_eq!(edit_mode_name(99), "Unknown");
    }

    #[test]
    fn test_pivot_default_zero() {
        let s = new_edit_mode_state();
        assert_eq!(s.pivot, [0.0, 0.0, 0.0]);
    }
}
