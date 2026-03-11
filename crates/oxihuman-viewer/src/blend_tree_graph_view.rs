// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Blend tree graph view stub.

/// Blend tree node type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlendTreeNodeType {
    Clip,
    Blend1D,
    Blend2D,
    AdditiveBlend,
    StateMachine,
}

/// A blend tree node.
#[derive(Debug, Clone)]
pub struct BlendTreeNode {
    pub id: u32,
    pub node_type: BlendTreeNodeType,
    pub label: String,
    pub position: [f32; 2],
    pub weight: f32,
}

/// Blend tree graph view configuration.
#[derive(Debug, Clone)]
pub struct BlendTreeGraphView {
    pub nodes: Vec<BlendTreeNode>,
    pub show_weights: bool,
    pub zoom: f32,
    pub pan: [f32; 2],
    pub enabled: bool,
}

impl BlendTreeGraphView {
    pub fn new() -> Self {
        BlendTreeGraphView {
            nodes: Vec::new(),
            show_weights: true,
            zoom: 1.0,
            pan: [0.0, 0.0],
            enabled: true,
        }
    }
}

impl Default for BlendTreeGraphView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new blend tree graph view.
pub fn new_blend_tree_graph_view() -> BlendTreeGraphView {
    BlendTreeGraphView::new()
}

/// Add a node.
pub fn btgv_add_node(view: &mut BlendTreeGraphView, node: BlendTreeNode) {
    view.nodes.push(node);
}

/// Clear all nodes.
pub fn btgv_clear(view: &mut BlendTreeGraphView) {
    view.nodes.clear();
}

/// Set zoom level.
pub fn btgv_set_zoom(view: &mut BlendTreeGraphView, zoom: f32) {
    view.zoom = zoom.max(0.05);
}

/// Set pan offset.
pub fn btgv_set_pan(view: &mut BlendTreeGraphView, pan: [f32; 2]) {
    view.pan = pan;
}

/// Toggle weight label display.
pub fn btgv_show_weights(view: &mut BlendTreeGraphView, show: bool) {
    view.show_weights = show;
}

/// Enable or disable.
pub fn btgv_set_enabled(view: &mut BlendTreeGraphView, enabled: bool) {
    view.enabled = enabled;
}

/// Return node count.
pub fn btgv_node_count(view: &BlendTreeGraphView) -> usize {
    view.nodes.len()
}

/// Serialize to JSON-like string.
pub fn btgv_to_json(view: &BlendTreeGraphView) -> String {
    format!(
        r#"{{"node_count":{},"show_weights":{},"zoom":{},"enabled":{}}}"#,
        view.nodes.len(),
        view.show_weights,
        view.zoom,
        view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_node(id: u32) -> BlendTreeNode {
        BlendTreeNode {
            id,
            node_type: BlendTreeNodeType::Clip,
            label: "run".to_string(),
            position: [0.0, 0.0],
            weight: 1.0,
        }
    }

    #[test]
    fn test_initial_empty() {
        let v = new_blend_tree_graph_view();
        assert_eq!(btgv_node_count(&v), 0 /* no nodes initially */);
    }

    #[test]
    fn test_add_node() {
        let mut v = new_blend_tree_graph_view();
        btgv_add_node(&mut v, make_node(0));
        assert_eq!(btgv_node_count(&v), 1 /* one node after add */);
    }

    #[test]
    fn test_clear() {
        let mut v = new_blend_tree_graph_view();
        btgv_add_node(&mut v, make_node(0));
        btgv_clear(&mut v);
        assert_eq!(btgv_node_count(&v), 0 /* cleared */);
    }

    #[test]
    fn test_zoom_min() {
        let mut v = new_blend_tree_graph_view();
        btgv_set_zoom(&mut v, 0.0);
        assert!((v.zoom - 0.05).abs() < 1e-6 /* minimum zoom must be 0.05 */);
    }

    #[test]
    fn test_set_pan() {
        let mut v = new_blend_tree_graph_view();
        btgv_set_pan(&mut v, [100.0, -50.0]);
        assert!((v.pan[0] - 100.0).abs() < 1e-6 /* pan x must be set */);
    }

    #[test]
    fn test_show_weights() {
        let mut v = new_blend_tree_graph_view();
        btgv_show_weights(&mut v, false);
        assert!(!v.show_weights /* weights must be hidden */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_blend_tree_graph_view();
        btgv_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_node_count() {
        let v = new_blend_tree_graph_view();
        let j = btgv_to_json(&v);
        assert!(j.contains("\"node_count\"") /* JSON must have node_count */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_blend_tree_graph_view();
        assert!(v.enabled /* must be enabled by default */);
    }
}
