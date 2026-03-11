// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Driver/dependency graph view stub.

/// Graph node type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GraphNodeType {
    Driver,
    Driven,
    Intermediate,
}

/// A graph node entry.
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: u32,
    pub node_type: GraphNodeType,
    pub label: String,
    pub position: [f32; 2],
}

/// Driver graph view configuration.
#[derive(Debug, Clone)]
pub struct DriverGraphView {
    pub nodes: Vec<GraphNode>,
    pub show_edge_labels: bool,
    pub zoom: f32,
    pub enabled: bool,
}

impl DriverGraphView {
    pub fn new() -> Self {
        DriverGraphView {
            nodes: Vec::new(),
            show_edge_labels: true,
            zoom: 1.0,
            enabled: true,
        }
    }
}

impl Default for DriverGraphView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new driver graph view.
pub fn new_driver_graph_view() -> DriverGraphView {
    DriverGraphView::new()
}

/// Add a node to the graph.
pub fn dgv_add_node(view: &mut DriverGraphView, node: GraphNode) {
    view.nodes.push(node);
}

/// Clear all nodes.
pub fn dgv_clear(view: &mut DriverGraphView) {
    view.nodes.clear();
}

/// Set zoom level.
pub fn dgv_set_zoom(view: &mut DriverGraphView, zoom: f32) {
    view.zoom = zoom.max(0.01);
}

/// Toggle edge label display.
pub fn dgv_show_edge_labels(view: &mut DriverGraphView, show: bool) {
    view.show_edge_labels = show;
}

/// Enable or disable.
pub fn dgv_set_enabled(view: &mut DriverGraphView, enabled: bool) {
    view.enabled = enabled;
}

/// Return node count.
pub fn dgv_node_count(view: &DriverGraphView) -> usize {
    view.nodes.len()
}

/// Serialize to JSON-like string.
pub fn dgv_to_json(view: &DriverGraphView) -> String {
    format!(
        r#"{{"node_count":{},"show_edge_labels":{},"zoom":{},"enabled":{}}}"#,
        view.nodes.len(),
        view.show_edge_labels,
        view.zoom,
        view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_node(id: u32) -> GraphNode {
        GraphNode {
            id,
            node_type: GraphNodeType::Driver,
            label: "driver".to_string(),
            position: [0.0, 0.0],
        }
    }

    #[test]
    fn test_initial_empty() {
        let v = new_driver_graph_view();
        assert_eq!(dgv_node_count(&v), 0 /* no nodes initially */);
    }

    #[test]
    fn test_add_node() {
        let mut v = new_driver_graph_view();
        dgv_add_node(&mut v, make_node(0));
        assert_eq!(dgv_node_count(&v), 1 /* one node after add */);
    }

    #[test]
    fn test_clear() {
        let mut v = new_driver_graph_view();
        dgv_add_node(&mut v, make_node(0));
        dgv_clear(&mut v);
        assert_eq!(dgv_node_count(&v), 0 /* cleared */);
    }

    #[test]
    fn test_zoom_min() {
        let mut v = new_driver_graph_view();
        dgv_set_zoom(&mut v, 0.0);
        assert!((v.zoom - 0.01).abs() < 1e-6 /* zoom minimum must be 0.01 */);
    }

    #[test]
    fn test_set_zoom() {
        let mut v = new_driver_graph_view();
        dgv_set_zoom(&mut v, 2.5);
        assert!((v.zoom - 2.5).abs() < 1e-6 /* zoom must be set */);
    }

    #[test]
    fn test_edge_labels() {
        let mut v = new_driver_graph_view();
        dgv_show_edge_labels(&mut v, false);
        assert!(!v.show_edge_labels /* edge labels must be hidden */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_driver_graph_view();
        dgv_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_node_count() {
        let v = new_driver_graph_view();
        let j = dgv_to_json(&v);
        assert!(j.contains("\"node_count\"") /* JSON must have node_count */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_driver_graph_view();
        assert!(v.enabled /* must be enabled by default */);
    }
}
