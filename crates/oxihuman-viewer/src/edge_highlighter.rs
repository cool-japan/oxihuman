// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! EdgeHighlighter — highlight specific mesh edges in the viewport.

#![allow(dead_code)]

/// A highlighted edge entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeHighlight {
    pub v0: usize,
    pub v1: usize,
    pub color: [f32; 4],
    pub width: f32,
}

/// A collection of highlighted edges.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct HighlightSet {
    pub edges: Vec<EdgeHighlight>,
}

/// Create a new, empty `HighlightSet`.
#[allow(dead_code)]
pub fn new_highlight_set() -> HighlightSet {
    HighlightSet { edges: Vec::new() }
}

/// Add an edge highlight between vertices `v0` and `v1`.
#[allow(dead_code)]
pub fn add_edge_highlight(set: &mut HighlightSet, v0: usize, v1: usize, color: [f32; 4], width: f32) {
    set.edges.push(EdgeHighlight { v0, v1, color, width });
}

/// Remove all highlights.
#[allow(dead_code)]
pub fn clear_highlights(set: &mut HighlightSet) {
    set.edges.clear();
}

/// Return the number of highlighted edges.
#[allow(dead_code)]
pub fn highlight_count(set: &HighlightSet) -> usize {
    set.edges.len()
}

/// Return the colour of the first highlight (or zero if empty).
#[allow(dead_code)]
pub fn highlight_color(set: &HighlightSet) -> [f32; 4] {
    set.edges.first().map(|e| e.color).unwrap_or([0.0; 4])
}

/// Return the line width of the first highlight (or 0 if empty).
#[allow(dead_code)]
pub fn highlight_width(set: &HighlightSet) -> f32 {
    set.edges.first().map(|e| e.width).unwrap_or(0.0)
}

/// Return a reference to the edge highlight at `index`.
#[allow(dead_code)]
pub fn highlight_edge_at(set: &HighlightSet, index: usize) -> Option<&EdgeHighlight> {
    set.edges.get(index)
}

/// Build a flat list of line pairs `[v0_pos, v1_pos]` from a highlight set and vertex list.
#[allow(dead_code)]
pub fn build_highlight_lines(set: &HighlightSet, vertices: &[[f32; 3]]) -> Vec<[[f32; 3]; 2]> {
    set.edges
        .iter()
        .filter_map(|e| {
            let a = *vertices.get(e.v0)?;
            let b = *vertices.get(e.v1)?;
            Some([a, b])
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_highlight_set_empty() {
        let hs = new_highlight_set();
        assert_eq!(highlight_count(&hs), 0);
    }

    #[test]
    fn test_add_edge_highlight() {
        let mut hs = new_highlight_set();
        add_edge_highlight(&mut hs, 0, 1, [1.0, 0.0, 0.0, 1.0], 2.0);
        assert_eq!(highlight_count(&hs), 1);
    }

    #[test]
    fn test_clear_highlights() {
        let mut hs = new_highlight_set();
        add_edge_highlight(&mut hs, 0, 1, [1.0; 4], 1.0);
        clear_highlights(&mut hs);
        assert_eq!(highlight_count(&hs), 0);
    }

    #[test]
    fn test_highlight_color_first() {
        let mut hs = new_highlight_set();
        add_edge_highlight(&mut hs, 0, 1, [0.0, 1.0, 0.0, 1.0], 1.0);
        let c = highlight_color(&hs);
        assert!((c[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_highlight_width_first() {
        let mut hs = new_highlight_set();
        add_edge_highlight(&mut hs, 0, 1, [1.0; 4], 3.0);
        assert!((highlight_width(&hs) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_highlight_edge_at_some() {
        let mut hs = new_highlight_set();
        add_edge_highlight(&mut hs, 2, 5, [1.0; 4], 1.0);
        let e = highlight_edge_at(&hs, 0).expect("should succeed");
        assert_eq!(e.v0, 2);
        assert_eq!(e.v1, 5);
    }

    #[test]
    fn test_highlight_edge_at_none() {
        let hs = new_highlight_set();
        assert!(highlight_edge_at(&hs, 0).is_none());
    }

    #[test]
    fn test_build_highlight_lines() {
        let mut hs = new_highlight_set();
        add_edge_highlight(&mut hs, 0, 1, [1.0; 4], 1.0);
        let verts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let lines = build_highlight_lines(&hs, &verts);
        assert_eq!(lines.len(), 1);
        assert!((lines[0][1][0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_build_highlight_lines_empty_verts() {
        let mut hs = new_highlight_set();
        add_edge_highlight(&mut hs, 0, 1, [1.0; 4], 1.0);
        let lines = build_highlight_lines(&hs, &[]);
        assert!(lines.is_empty());
    }
}
