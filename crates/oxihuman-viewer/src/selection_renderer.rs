// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Selection highlight and outline renderer for selected objects.
//!
//! Provides vertex-level selection tracking for faces, vertices, edges, and
//! objects, along with bounding box computation and adjacency-based growth.

// ── Types ─────────────────────────────────────────────────────────────────────

/// Granularity at which selection operates.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    /// Per-face selection.
    Face,
    /// Per-vertex selection.
    Vertex,
    /// Per-edge selection.
    Edge,
    /// Whole-object selection.
    Object,
}

/// Tracks which vertices (or faces/edges) are currently selected.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SelectionState {
    /// Current selection granularity.
    pub mode: SelectionMode,
    /// Bit-vector: index `i` is selected when `selected[i]` is `true`.
    selected: Vec<bool>,
    /// Total number of selectable items (vertices, faces, etc.).
    pub item_count: usize,
}

/// Configuration for the selection renderer.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SelectionRendererConfig {
    /// RGBA outline colour for selected items.
    pub outline_color: [f32; 4],
    /// Outline width in pixels.
    pub outline_width: f32,
    /// Whether to draw filled highlight on selected faces.
    pub fill_selected: bool,
    /// RGBA fill colour (used when fill_selected is true).
    pub fill_color: [f32; 4],
    /// Default selection mode on creation.
    pub default_mode: SelectionMode,
}

// ── Constructors ──────────────────────────────────────────────────────────────

/// Returns a default [`SelectionRendererConfig`].
#[allow(dead_code)]
pub fn default_selection_config() -> SelectionRendererConfig {
    SelectionRendererConfig {
        outline_color: [1.0, 0.6, 0.0, 1.0],
        outline_width: 2.0,
        fill_selected: true,
        fill_color: [1.0, 0.6, 0.0, 0.2],
        default_mode: SelectionMode::Vertex,
    }
}

/// Creates a new [`SelectionState`] for `item_count` selectable items.
#[allow(dead_code)]
pub fn new_selection_state(item_count: usize, mode: SelectionMode) -> SelectionState {
    SelectionState {
        mode,
        selected: vec![false; item_count],
        item_count,
    }
}

// ── Selection mutations ───────────────────────────────────────────────────────

/// Marks the given vertex indices as selected.
///
/// Out-of-range indices are silently ignored.
#[allow(dead_code)]
pub fn select_vertices(state: &mut SelectionState, indices: &[usize]) {
    for &i in indices {
        if i < state.item_count {
            state.selected[i] = true;
        }
    }
}

/// Marks the given vertex indices as deselected.
///
/// Out-of-range indices are silently ignored.
#[allow(dead_code)]
pub fn deselect_vertices(state: &mut SelectionState, indices: &[usize]) {
    for &i in indices {
        if i < state.item_count {
            state.selected[i] = false;
        }
    }
}

/// Toggles the selection state of each given index.
#[allow(dead_code)]
pub fn toggle_vertex_selection(state: &mut SelectionState, indices: &[usize]) {
    for &i in indices {
        if i < state.item_count {
            state.selected[i] = !state.selected[i];
        }
    }
}

/// Selects all items.
#[allow(dead_code)]
pub fn select_all(state: &mut SelectionState) {
    state.selected.iter_mut().for_each(|s| *s = true);
}

/// Deselects all items.
#[allow(dead_code)]
pub fn deselect_all(state: &mut SelectionState) {
    state.selected.iter_mut().for_each(|s| *s = false);
}

/// Inverts the current selection (selected ↔ unselected).
#[allow(dead_code)]
pub fn invert_selection(state: &mut SelectionState) {
    state.selected.iter_mut().for_each(|s| *s = !*s);
}

// ── Queries ────────────────────────────────────────────────────────────────────

/// Returns the number of currently selected items.
#[allow(dead_code)]
pub fn selected_vertex_count(state: &SelectionState) -> usize {
    state.selected.iter().filter(|&&s| s).count()
}

/// Returns `true` if the item at `index` is selected.
///
/// Returns `false` for out-of-range indices.
#[allow(dead_code)]
pub fn is_vertex_selected(state: &SelectionState, index: usize) -> bool {
    state.selected.get(index).copied().unwrap_or(false)
}

/// Returns a sorted list of selected indices.
#[allow(dead_code)]
pub fn selection_to_index_list(state: &SelectionState) -> Vec<usize> {
    state
        .selected
        .iter()
        .enumerate()
        .filter_map(|(i, &s)| if s { Some(i) } else { None })
        .collect()
}

// ── Bounding box ──────────────────────────────────────────────────────────────

/// Type alias for an axis-aligned bounding box: `[min_x, min_y, min_z, max_x, max_y, max_z]`.
pub type SelectionAabb = [f32; 6];

/// Computes the axis-aligned bounding box of the selected vertices.
///
/// `positions` must have `state.item_count` elements.
/// Returns `None` if nothing is selected or the slice is too short.
#[allow(dead_code)]
pub fn selection_bounding_box(
    state: &SelectionState,
    positions: &[[f32; 3]],
) -> Option<SelectionAabb> {
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    let mut found = false;

    for (i, &sel) in state.selected.iter().enumerate() {
        if !sel {
            continue;
        }
        let p = positions.get(i)?;
        for k in 0..3 {
            if p[k] < min[k] {
                min[k] = p[k];
            }
            if p[k] > max[k] {
                max[k] = p[k];
            }
        }
        found = true;
    }

    if found {
        Some([min[0], min[1], min[2], max[0], max[1], max[2]])
    } else {
        None
    }
}

// ── Serialisation ─────────────────────────────────────────────────────────────

/// Serialises the current selection to a compact JSON string.
#[allow(dead_code)]
pub fn selection_to_json(state: &SelectionState) -> String {
    let mode = match state.mode {
        SelectionMode::Face => "face",
        SelectionMode::Vertex => "vertex",
        SelectionMode::Edge => "edge",
        SelectionMode::Object => "object",
    };
    let indices = selection_to_index_list(state);
    let idx_json: Vec<String> = indices.iter().map(|i| i.to_string()).collect();
    format!(
        "{{\"mode\":\"{}\",\"count\":{},\"indices\":[{}]}}",
        mode,
        indices.len(),
        idx_json.join(",")
    )
}

// ── Grow selection ─────────────────────────────────────────────────────────────

/// Expands the selection by one ring of adjacency.
///
/// `adjacency[i]` lists the indices adjacent to item `i`.
/// Each currently selected item's neighbours are added to the selection.
#[allow(dead_code)]
pub fn grow_selection(state: &mut SelectionState, adjacency: &[Vec<usize>]) {
    // Collect neighbours of currently selected items first, then apply
    let to_add: Vec<usize> = state
        .selected
        .iter()
        .enumerate()
        .filter(|(_, &s)| s)
        .flat_map(|(i, _)| {
            adjacency
                .get(i)
                .map(|v| v.as_slice())
                .unwrap_or(&[])
                .to_vec()
        })
        .collect();
    for i in to_add {
        if i < state.item_count {
            state.selected[i] = true;
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state(n: usize) -> SelectionState {
        new_selection_state(n, SelectionMode::Vertex)
    }

    #[test]
    fn test_default_config_outline_width() {
        let cfg = default_selection_config();
        assert!((cfg.outline_width - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_default_config_fill_selected() {
        let cfg = default_selection_config();
        assert!(cfg.fill_selected);
    }

    #[test]
    fn test_new_state_all_unselected() {
        let s = make_state(5);
        assert_eq!(selected_vertex_count(&s), 0);
    }

    #[test]
    fn test_new_state_item_count() {
        let s = make_state(10);
        assert_eq!(s.item_count, 10);
    }

    #[test]
    fn test_select_vertices() {
        let mut s = make_state(5);
        select_vertices(&mut s, &[0, 2, 4]);
        assert_eq!(selected_vertex_count(&s), 3);
    }

    #[test]
    fn test_select_out_of_range_ignored() {
        let mut s = make_state(3);
        select_vertices(&mut s, &[100]);
        assert_eq!(selected_vertex_count(&s), 0);
    }

    #[test]
    fn test_deselect_vertices() {
        let mut s = make_state(5);
        select_all(&mut s);
        deselect_vertices(&mut s, &[1, 3]);
        assert_eq!(selected_vertex_count(&s), 3);
    }

    #[test]
    fn test_toggle_vertex_selection() {
        let mut s = make_state(4);
        select_vertices(&mut s, &[0, 1]);
        toggle_vertex_selection(&mut s, &[1, 2]);
        // 0: selected, 1: deselected, 2: selected, 3: unselected
        assert!(is_vertex_selected(&s, 0));
        assert!(!is_vertex_selected(&s, 1));
        assert!(is_vertex_selected(&s, 2));
        assert!(!is_vertex_selected(&s, 3));
    }

    #[test]
    fn test_select_all() {
        let mut s = make_state(4);
        select_all(&mut s);
        assert_eq!(selected_vertex_count(&s), 4);
    }

    #[test]
    fn test_deselect_all() {
        let mut s = make_state(4);
        select_all(&mut s);
        deselect_all(&mut s);
        assert_eq!(selected_vertex_count(&s), 0);
    }

    #[test]
    fn test_invert_selection() {
        let mut s = make_state(4);
        select_vertices(&mut s, &[0, 2]);
        invert_selection(&mut s);
        assert!(is_vertex_selected(&s, 1));
        assert!(is_vertex_selected(&s, 3));
        assert!(!is_vertex_selected(&s, 0));
        assert!(!is_vertex_selected(&s, 2));
    }

    #[test]
    fn test_is_vertex_selected_in_range() {
        let mut s = make_state(3);
        select_vertices(&mut s, &[1]);
        assert!(is_vertex_selected(&s, 1));
        assert!(!is_vertex_selected(&s, 0));
    }

    #[test]
    fn test_is_vertex_selected_out_of_range() {
        let s = make_state(3);
        assert!(!is_vertex_selected(&s, 99));
    }

    #[test]
    fn test_selection_to_index_list() {
        let mut s = make_state(5);
        select_vertices(&mut s, &[1, 3]);
        let list = selection_to_index_list(&s);
        assert_eq!(list, vec![1, 3]);
    }

    #[test]
    fn test_selection_bounding_box() {
        let mut s = make_state(3);
        select_vertices(&mut s, &[0, 2]);
        let positions = vec![[0.0f32, 0.0, 0.0], [5.0, 5.0, 5.0], [2.0, 3.0, 1.0]];
        let aabb = selection_bounding_box(&s, &positions).expect("should succeed");
        assert!((aabb[0] - 0.0).abs() < 1e-6); // min_x
        assert!((aabb[3] - 2.0).abs() < 1e-6); // max_x
    }

    #[test]
    fn test_selection_bounding_box_empty() {
        let s = make_state(3);
        let positions = vec![[0.0f32; 3]; 3];
        assert!(selection_bounding_box(&s, &positions).is_none());
    }

    #[test]
    fn test_selection_to_json_format() {
        let mut s = make_state(4);
        select_vertices(&mut s, &[0, 3]);
        let json = selection_to_json(&s);
        assert!(json.contains("\"mode\":\"vertex\""));
        assert!(json.contains("\"count\":2"));
    }

    #[test]
    fn test_grow_selection() {
        let mut s = make_state(4);
        select_vertices(&mut s, &[0]);
        // adjacency: 0→[1,2], 1→[3], 2→[], 3→[]
        let adj: Vec<Vec<usize>> = vec![vec![1, 2], vec![3], vec![], vec![]];
        grow_selection(&mut s, &adj);
        assert!(is_vertex_selected(&s, 1));
        assert!(is_vertex_selected(&s, 2));
        assert!(!is_vertex_selected(&s, 3)); // 1 ring only
    }
}
