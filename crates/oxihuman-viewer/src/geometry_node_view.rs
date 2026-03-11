// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Geometry nodes graph view.

/// Type of geometry node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeoNodeType {
    Input,
    Output,
    Transform,
    Mesh,
    Curve,
    Custom,
}

/// A geometry node.
#[derive(Debug, Clone)]
pub struct GeoNode {
    pub id: u32,
    pub node_type: GeoNodeType,
    pub label: String,
}

/// State for the geometry node view.
#[derive(Debug, Clone)]
pub struct GeometryNodeView {
    pub nodes: Vec<GeoNode>,
    pub zoom: f32,
    pub show_sockets: bool,
    pub enabled: bool,
}

/// Create a new geometry node view.
pub fn new_geometry_node_view() -> GeometryNodeView {
    GeometryNodeView {
        nodes: Vec::new(),
        zoom: 1.0,
        show_sockets: true,
        enabled: true,
    }
}

/// Add a geometry node.
pub fn gnv_add_node(v: &mut GeometryNodeView, id: u32, node_type: GeoNodeType, label: &str) {
    v.nodes.push(GeoNode {
        id,
        node_type,
        label: label.to_string(),
    });
}

/// Clear all nodes.
pub fn gnv_clear(v: &mut GeometryNodeView) {
    v.nodes.clear();
}

/// Set zoom level (clamped 0.1–10).
pub fn gnv_set_zoom(v: &mut GeometryNodeView, zoom: f32) {
    v.zoom = zoom.clamp(0.1, 10.0);
}

/// Count nodes of a given type.
pub fn gnv_count_type(v: &GeometryNodeView, t: &GeoNodeType) -> usize {
    v.nodes.iter().filter(|n| &n.node_type == t).count()
}

/// Serialise to JSON.
pub fn gnv_to_json(v: &GeometryNodeView) -> String {
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
        let v = new_geometry_node_view();
        assert!(v.nodes.is_empty() /* no nodes */);
    }

    #[test]
    fn add_node_increases_count() {
        let mut v = new_geometry_node_view();
        gnv_add_node(&mut v, 1, GeoNodeType::Mesh, "Mesh");
        assert_eq!(v.nodes.len(), 1 /* one node */);
    }

    #[test]
    fn clear_removes_all() {
        let mut v = new_geometry_node_view();
        gnv_add_node(&mut v, 1, GeoNodeType::Input, "In");
        gnv_clear(&mut v);
        assert!(v.nodes.is_empty() /* cleared */);
    }

    #[test]
    fn count_type() {
        let mut v = new_geometry_node_view();
        gnv_add_node(&mut v, 1, GeoNodeType::Mesh, "M1");
        gnv_add_node(&mut v, 2, GeoNodeType::Mesh, "M2");
        gnv_add_node(&mut v, 3, GeoNodeType::Curve, "C1");
        assert_eq!(
            gnv_count_type(&v, &GeoNodeType::Mesh),
            2 /* two mesh nodes */
        );
    }

    #[test]
    fn zoom_clamp() {
        let mut v = new_geometry_node_view();
        gnv_set_zoom(&mut v, 50.0);
        assert!((v.zoom - 10.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn show_sockets_default_true() {
        let v = new_geometry_node_view();
        assert!(v.show_sockets /* sockets shown by default */);
    }

    #[test]
    fn json_has_node_count() {
        let v = new_geometry_node_view();
        assert!(gnv_to_json(&v).contains("node_count") /* json field */);
    }

    #[test]
    fn enabled_default() {
        let v = new_geometry_node_view();
        assert!(v.enabled /* enabled */);
    }
}
