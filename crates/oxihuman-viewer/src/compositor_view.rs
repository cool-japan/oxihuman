// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Compositor node graph view.

/// A compositor node.
#[derive(Debug, Clone)]
pub struct CompositorNode {
    pub id: u32,
    pub label: String,
    pub muted: bool,
}

/// State for the compositor node graph view.
#[derive(Debug, Clone)]
pub struct CompositorView {
    pub nodes: Vec<CompositorNode>,
    pub zoom: f32,
    pub use_gpu: bool,
    pub enabled: bool,
}

/// Create a new compositor view.
pub fn new_compositor_view() -> CompositorView {
    CompositorView {
        nodes: Vec::new(),
        zoom: 1.0,
        use_gpu: false,
        enabled: true,
    }
}

/// Add a compositor node.
pub fn cv_add_node(v: &mut CompositorView, id: u32, label: &str) {
    v.nodes.push(CompositorNode {
        id,
        label: label.to_string(),
        muted: false,
    });
}

/// Mute/unmute a node by ID.
pub fn cv_set_muted(v: &mut CompositorView, id: u32, muted: bool) {
    if let Some(n) = v.nodes.iter_mut().find(|n| n.id == id) {
        n.muted = muted;
    }
}

/// Count active (non-muted) nodes.
pub fn cv_active_node_count(v: &CompositorView) -> usize {
    v.nodes.iter().filter(|n| !n.muted).count()
}

/// Set zoom level.
pub fn cv_set_zoom(v: &mut CompositorView, zoom: f32) {
    v.zoom = zoom.clamp(0.1, 10.0);
}

/// Serialise to JSON.
pub fn cv_to_json(v: &CompositorView) -> String {
    format!(
        r#"{{"node_count":{},"active":{},"use_gpu":{},"enabled":{}}}"#,
        v.nodes.len(),
        cv_active_node_count(v),
        v.use_gpu,
        v.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_empty() {
        let v = new_compositor_view();
        assert!(v.nodes.is_empty() /* no nodes */);
    }

    #[test]
    fn add_node() {
        let mut v = new_compositor_view();
        cv_add_node(&mut v, 1, "Blur");
        assert_eq!(v.nodes.len(), 1 /* one node */);
    }

    #[test]
    fn mute_node() {
        let mut v = new_compositor_view();
        cv_add_node(&mut v, 1, "Glare");
        cv_set_muted(&mut v, 1, true);
        assert!(v.nodes[0].muted /* muted */);
    }

    #[test]
    fn active_count_excludes_muted() {
        let mut v = new_compositor_view();
        cv_add_node(&mut v, 1, "A");
        cv_add_node(&mut v, 2, "B");
        cv_set_muted(&mut v, 1, true);
        assert_eq!(cv_active_node_count(&v), 1 /* one active */);
    }

    #[test]
    fn zoom_clamp() {
        let mut v = new_compositor_view();
        cv_set_zoom(&mut v, 0.0);
        assert!((v.zoom - 0.1).abs() < 1e-6 /* clamped to 0.1 */);
    }

    #[test]
    fn json_has_active() {
        let v = new_compositor_view();
        assert!(cv_to_json(&v).contains("active") /* json has field */);
    }

    #[test]
    fn use_gpu_default_false() {
        let v = new_compositor_view();
        assert!(!v.use_gpu /* gpu disabled by default */);
    }

    #[test]
    fn enabled_default() {
        let v = new_compositor_view();
        assert!(v.enabled /* enabled */);
    }
}
