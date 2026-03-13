// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cage edit wireframe view stub.

/// Cage vertex handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CageVertexId(pub u32);

/// Cage edit view configuration.
#[derive(Debug, Clone)]
pub struct CageEditView {
    pub cage_color: [f32; 4],
    pub selected_color: [f32; 4],
    pub line_width: f32,
    pub show_cage_vertices: bool,
    pub selected_vertices: Vec<CageVertexId>,
    pub enabled: bool,
}

impl CageEditView {
    pub fn new() -> Self {
        CageEditView {
            cage_color: [0.8, 0.8, 0.0, 1.0],
            selected_color: [1.0, 0.5, 0.0, 1.0],
            line_width: 1.5,
            show_cage_vertices: true,
            selected_vertices: Vec::new(),
            enabled: true,
        }
    }
}

impl Default for CageEditView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new cage edit view.
pub fn new_cage_edit_view() -> CageEditView {
    CageEditView::new()
}

/// Set cage wireframe color.
pub fn cev_set_cage_color(view: &mut CageEditView, color: [f32; 4]) {
    view.cage_color = color;
}

/// Set selected vertex highlight color.
pub fn cev_set_selected_color(view: &mut CageEditView, color: [f32; 4]) {
    view.selected_color = color;
}

/// Set wireframe line width.
pub fn cev_set_line_width(view: &mut CageEditView, width: f32) {
    view.line_width = width.max(0.1);
}

/// Select a vertex by ID.
pub fn cev_select_vertex(view: &mut CageEditView, id: CageVertexId) {
    if !view.selected_vertices.contains(&id) {
        view.selected_vertices.push(id);
    }
}

/// Clear selection.
pub fn cev_clear_selection(view: &mut CageEditView) {
    view.selected_vertices.clear();
}

/// Enable or disable.
pub fn cev_set_enabled(view: &mut CageEditView, enabled: bool) {
    view.enabled = enabled;
}

/// Return selected vertex count.
pub fn cev_selected_count(view: &CageEditView) -> usize {
    view.selected_vertices.len()
}

/// Serialize to JSON-like string.
pub fn cev_to_json(view: &CageEditView) -> String {
    format!(
        r#"{{"line_width":{},"show_cage_vertices":{},"selected_count":{},"enabled":{}}}"#,
        view.line_width,
        view.show_cage_vertices,
        view.selected_vertices.len(),
        view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_no_selection() {
        let v = new_cage_edit_view();
        assert_eq!(
            cev_selected_count(&v),
            0 /* no vertices selected initially */
        );
    }

    #[test]
    fn test_select_vertex() {
        let mut v = new_cage_edit_view();
        cev_select_vertex(&mut v, CageVertexId(0));
        assert_eq!(cev_selected_count(&v), 1 /* one vertex selected */);
    }

    #[test]
    fn test_no_duplicate_selection() {
        let mut v = new_cage_edit_view();
        cev_select_vertex(&mut v, CageVertexId(0));
        cev_select_vertex(&mut v, CageVertexId(0));
        assert_eq!(cev_selected_count(&v), 1 /* no duplicate selection */);
    }

    #[test]
    fn test_clear_selection() {
        let mut v = new_cage_edit_view();
        cev_select_vertex(&mut v, CageVertexId(1));
        cev_clear_selection(&mut v);
        assert_eq!(cev_selected_count(&v), 0 /* cleared */);
    }

    #[test]
    fn test_line_width_min() {
        let mut v = new_cage_edit_view();
        cev_set_line_width(&mut v, -5.0);
        assert!((v.line_width - 0.1).abs() < 1e-6 /* line_width minimum must be 0.1 */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_cage_edit_view();
        cev_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_line_width() {
        let v = new_cage_edit_view();
        let j = cev_to_json(&v);
        assert!(j.contains("\"line_width\"") /* JSON must have line_width */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_cage_edit_view();
        assert!(v.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_show_vertices_default() {
        let v = new_cage_edit_view();
        assert!(v.show_cage_vertices /* cage vertices must be shown by default */);
    }
}
