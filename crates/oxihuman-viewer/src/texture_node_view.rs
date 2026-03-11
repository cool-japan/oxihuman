// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Texture node graph view.

/// A texture node entry.
#[derive(Debug, Clone)]
pub struct TextureNode {
    pub id: u32,
    pub name: String,
    pub width: u32,
    pub height: u32,
}

/// State for the texture node view.
#[derive(Debug, Clone)]
pub struct TextureNodeView {
    pub nodes: Vec<TextureNode>,
    pub selected_id: Option<u32>,
    pub zoom: f32,
    pub enabled: bool,
}

/// Create a new texture node view.
pub fn new_texture_node_view() -> TextureNodeView {
    TextureNodeView {
        nodes: Vec::new(),
        selected_id: None,
        zoom: 1.0,
        enabled: true,
    }
}

/// Add a texture node.
pub fn tnv_add_node(v: &mut TextureNodeView, id: u32, name: &str, width: u32, height: u32) {
    v.nodes.push(TextureNode {
        id,
        name: name.to_string(),
        width,
        height,
    });
}

/// Select a texture node by ID.
pub fn tnv_select(v: &mut TextureNodeView, id: u32) {
    v.selected_id = Some(id);
}

/// Deselect all.
pub fn tnv_deselect(v: &mut TextureNodeView) {
    v.selected_id = None;
}

/// Set zoom level (clamped 0.1–10).
pub fn tnv_set_zoom(v: &mut TextureNodeView, zoom: f32) {
    v.zoom = zoom.clamp(0.1, 10.0);
}

/// Serialise to JSON.
pub fn tnv_to_json(v: &TextureNodeView) -> String {
    format!(
        r#"{{"node_count":{},"selected":{:?},"zoom":{:.2},"enabled":{}}}"#,
        v.nodes.len(),
        v.selected_id,
        v.zoom,
        v.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_empty() {
        let v = new_texture_node_view();
        assert!(v.nodes.is_empty() /* no nodes */);
    }

    #[test]
    fn add_node() {
        let mut v = new_texture_node_view();
        tnv_add_node(&mut v, 1, "Albedo", 1024, 1024);
        assert_eq!(v.nodes.len(), 1 /* one node */);
    }

    #[test]
    fn select_node() {
        let mut v = new_texture_node_view();
        tnv_select(&mut v, 5);
        assert_eq!(v.selected_id, Some(5) /* selected */);
    }

    #[test]
    fn deselect_clears() {
        let mut v = new_texture_node_view();
        tnv_select(&mut v, 3);
        tnv_deselect(&mut v);
        assert_eq!(v.selected_id, None /* deselected */);
    }

    #[test]
    fn zoom_clamps_upper() {
        let mut v = new_texture_node_view();
        tnv_set_zoom(&mut v, 100.0);
        assert!((v.zoom - 10.0).abs() < 1e-6 /* clamped to 10 */);
    }

    #[test]
    fn zoom_clamps_lower() {
        let mut v = new_texture_node_view();
        tnv_set_zoom(&mut v, 0.0);
        assert!((v.zoom - 0.1).abs() < 1e-6 /* clamped to 0.1 */);
    }

    #[test]
    fn json_has_node_count() {
        let v = new_texture_node_view();
        assert!(tnv_to_json(&v).contains("node_count") /* json has field */);
    }

    #[test]
    fn node_dimensions_stored() {
        let mut v = new_texture_node_view();
        tnv_add_node(&mut v, 2, "Normal", 512, 512);
        assert_eq!(v.nodes[0].width, 512 /* width stored */);
    }
}
