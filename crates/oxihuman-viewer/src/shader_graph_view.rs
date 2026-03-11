// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Shader node graph view.

/// A shader graph node.
#[derive(Debug, Clone)]
pub struct ShaderNode {
    pub id: u32,
    pub node_type: String,
    pub x: f32,
    pub y: f32,
}

/// An edge connecting two shader nodes.
#[derive(Debug, Clone)]
pub struct ShaderEdge {
    pub from_id: u32,
    pub to_id: u32,
}

/// State for the shader graph view.
#[derive(Debug, Clone)]
pub struct ShaderGraphView {
    pub nodes: Vec<ShaderNode>,
    pub edges: Vec<ShaderEdge>,
    pub zoom: f32,
    pub enabled: bool,
}

/// Create a new shader graph view.
pub fn new_shader_graph_view() -> ShaderGraphView {
    ShaderGraphView {
        nodes: Vec::new(),
        edges: Vec::new(),
        zoom: 1.0,
        enabled: true,
    }
}

/// Add a shader node.
pub fn sgv_add_node(v: &mut ShaderGraphView, id: u32, node_type: &str, x: f32, y: f32) {
    v.nodes.push(ShaderNode {
        id,
        node_type: node_type.to_string(),
        x,
        y,
    });
}

/// Connect two nodes with an edge.
pub fn sgv_connect(v: &mut ShaderGraphView, from: u32, to: u32) {
    v.edges.push(ShaderEdge {
        from_id: from,
        to_id: to,
    });
}

/// Clear all nodes and edges.
pub fn sgv_clear(v: &mut ShaderGraphView) {
    v.nodes.clear();
    v.edges.clear();
}

/// Set zoom level (clamped 0.1–10).
pub fn sgv_set_zoom(v: &mut ShaderGraphView, zoom: f32) {
    v.zoom = zoom.clamp(0.1, 10.0);
}

/// Serialise to JSON.
pub fn sgv_to_json(v: &ShaderGraphView) -> String {
    format!(
        r#"{{"nodes":{},"edges":{},"zoom":{:.2},"enabled":{}}}"#,
        v.nodes.len(),
        v.edges.len(),
        v.zoom,
        v.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_empty() {
        let v = new_shader_graph_view();
        assert!(v.nodes.is_empty() /* no nodes */);
    }

    #[test]
    fn add_node() {
        let mut v = new_shader_graph_view();
        sgv_add_node(&mut v, 1, "Texture2D", 0.0, 0.0);
        assert_eq!(v.nodes.len(), 1 /* one node */);
    }

    #[test]
    fn connect_creates_edge() {
        let mut v = new_shader_graph_view();
        sgv_connect(&mut v, 1, 2);
        assert_eq!(v.edges.len(), 1 /* one edge */);
    }

    #[test]
    fn clear_removes_all() {
        let mut v = new_shader_graph_view();
        sgv_add_node(&mut v, 1, "UV", 0.0, 0.0);
        sgv_connect(&mut v, 1, 2);
        sgv_clear(&mut v);
        assert!(v.nodes.is_empty() && v.edges.is_empty() /* cleared */);
    }

    #[test]
    fn zoom_clamps() {
        let mut v = new_shader_graph_view();
        sgv_set_zoom(&mut v, 0.0);
        assert!((v.zoom - 0.1).abs() < 1e-6 /* clamped to 0.1 */);
    }

    #[test]
    fn json_has_edges() {
        let v = new_shader_graph_view();
        assert!(sgv_to_json(&v).contains("edges") /* json has edges */);
    }

    #[test]
    fn enabled_default() {
        let v = new_shader_graph_view();
        assert!(v.enabled /* enabled */);
    }

    #[test]
    fn node_type_stored() {
        let mut v = new_shader_graph_view();
        sgv_add_node(&mut v, 1, "BRDF", 0.0, 0.0);
        assert_eq!(v.nodes[0].node_type, "BRDF" /* node type stored */);
    }
}
