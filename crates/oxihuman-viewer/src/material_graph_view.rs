// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Material node graph view.

/// A node in the material graph.
#[derive(Debug, Clone)]
pub struct MaterialNode {
    pub id: u32,
    pub label: String,
    pub x: f32,
    pub y: f32,
}

/// State for the material graph view.
#[derive(Debug, Clone)]
pub struct MaterialGraphView {
    pub nodes: Vec<MaterialNode>,
    pub zoom: f32,
    pub pan: [f32; 2],
    pub enabled: bool,
}

/// Create a new material graph view.
pub fn new_material_graph_view() -> MaterialGraphView {
    MaterialGraphView {
        nodes: Vec::new(),
        zoom: 1.0,
        pan: [0.0, 0.0],
        enabled: true,
    }
}

/// Add a node to the graph.
pub fn mgv_add_node(v: &mut MaterialGraphView, id: u32, label: &str, x: f32, y: f32) {
    v.nodes.push(MaterialNode {
        id,
        label: label.to_string(),
        x,
        y,
    });
}

/// Remove a node by ID. Returns true if found.
pub fn mgv_remove_node(v: &mut MaterialGraphView, id: u32) -> bool {
    let before = v.nodes.len();
    v.nodes.retain(|n| n.id != id);
    v.nodes.len() < before
}

/// Set zoom level (clamped 0.1–10).
pub fn mgv_set_zoom(v: &mut MaterialGraphView, zoom: f32) {
    v.zoom = zoom.clamp(0.1, 10.0);
}

/// Set pan offset.
pub fn mgv_set_pan(v: &mut MaterialGraphView, x: f32, y: f32) {
    v.pan = [x, y];
}

/// Serialise to JSON.
pub fn mgv_to_json(v: &MaterialGraphView) -> String {
    format!(
        r#"{{"node_count":{},"zoom":{:.2},"enabled":{}}}"#,
        v.nodes.len(),
        v.zoom,
        v.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_empty() {
        let v = new_material_graph_view();
        assert!(v.nodes.is_empty() /* no nodes */);
    }

    #[test]
    fn add_node_increases_count() {
        let mut v = new_material_graph_view();
        mgv_add_node(&mut v, 1, "PBR", 0.0, 0.0);
        assert_eq!(v.nodes.len(), 1 /* one node */);
    }

    #[test]
    fn remove_node_by_id() {
        let mut v = new_material_graph_view();
        mgv_add_node(&mut v, 1, "Tex", 0.0, 0.0);
        mgv_add_node(&mut v, 2, "Mix", 10.0, 0.0);
        assert!(mgv_remove_node(&mut v, 1) /* removed */);
        assert_eq!(v.nodes.len(), 1 /* one remains */);
    }

    #[test]
    fn remove_missing_returns_false() {
        let mut v = new_material_graph_view();
        assert!(!mgv_remove_node(&mut v, 99) /* not found */);
    }

    #[test]
    fn set_zoom_clamps() {
        let mut v = new_material_graph_view();
        mgv_set_zoom(&mut v, 999.0);
        assert!((v.zoom - 10.0).abs() < 1e-6 /* clamped to 10 */);
    }

    #[test]
    fn pan_set_correctly() {
        let mut v = new_material_graph_view();
        mgv_set_pan(&mut v, 50.0, -30.0);
        assert!((v.pan[0] - 50.0).abs() < 1e-6 /* pan x */);
    }

    #[test]
    fn json_has_node_count() {
        let v = new_material_graph_view();
        assert!(mgv_to_json(&v).contains("node_count") /* json has field */);
    }

    #[test]
    fn enabled_default() {
        let v = new_material_graph_view();
        assert!(v.enabled /* enabled */);
    }
}
