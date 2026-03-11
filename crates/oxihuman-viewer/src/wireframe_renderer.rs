// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Wireframe overlay renderer configuration and draw-call builder.
//!
//! Provides utilities for building edge lists from triangle meshes and
//! assembling draw calls for wireframe overlay rendering.

use std::collections::HashSet;

// ── Types ─────────────────────────────────────────────────────────────────────

/// Configuration for wireframe rendering.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WireframeConfig {
    /// RGBA line color (linear, 0.0–1.0).
    pub color: [f32; 4],
    /// Line width in pixels.
    pub line_width: f32,
    /// Whether to render back-facing edges.
    pub show_backface: bool,
    /// Whether the wireframe overlay is visible.
    pub visible: bool,
}

/// A unique edge defined by two vertex indices.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WireframeEdge {
    pub v0: u32,
    pub v1: u32,
}

/// A resolved draw call: vertex positions plus edges to render.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WireframeDrawCall {
    /// Flattened vertex positions (one `[f32; 3]` per vertex).
    pub positions: Vec<[f32; 3]>,
    /// Edge list referencing indices into `positions`.
    pub edges: Vec<WireframeEdge>,
    /// A snapshot of the config at draw-call creation time.
    pub config: WireframeConfig,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Return a sensible default `WireframeConfig`.
#[allow(dead_code)]
pub fn default_wireframe_config() -> WireframeConfig {
    WireframeConfig {
        color: [0.0, 0.0, 0.0, 1.0],
        line_width: 1.0,
        show_backface: false,
        visible: true,
    }
}

/// Build a deduplicated edge list from a triangle face array.
///
/// Each face is `[v0, v1, v2]`; each edge `(a, b)` is stored with `a < b`
/// so that shared edges appear only once.
#[allow(dead_code)]
pub fn build_wireframe_edges(faces: &[[u32; 3]]) -> Vec<WireframeEdge> {
    let mut seen: HashSet<(u32, u32)> = HashSet::new();
    let mut edges = Vec::new();

    for face in faces {
        let pairs = [
            edge_key(face[0], face[1]),
            edge_key(face[1], face[2]),
            edge_key(face[2], face[0]),
        ];
        for (a, b) in &pairs {
            if seen.insert((*a, *b)) {
                edges.push(WireframeEdge { v0: *a, v1: *b });
            }
        }
    }
    edges
}

/// Assemble a `WireframeDrawCall` from vertices, edges, and configuration.
#[allow(dead_code)]
pub fn wireframe_draw_call(
    verts: &[[f32; 3]],
    edges: &[WireframeEdge],
    cfg: &WireframeConfig,
) -> WireframeDrawCall {
    WireframeDrawCall {
        positions: verts.to_vec(),
        edges: edges.to_vec(),
        config: cfg.clone(),
    }
}

/// Return the number of edges in a draw call.
#[allow(dead_code)]
pub fn wireframe_edge_count(call: &WireframeDrawCall) -> usize {
    call.edges.len()
}

/// Set the RGBA color of the wireframe.
#[allow(dead_code)]
pub fn set_wireframe_color(cfg: &mut WireframeConfig, r: f32, g: f32, b: f32, a: f32) {
    cfg.color = [r, g, b, a];
}

/// Set the line width (in pixels) for the wireframe.
#[allow(dead_code)]
pub fn set_wireframe_line_width(cfg: &mut WireframeConfig, width: f32) {
    cfg.line_width = width.max(0.0);
}

/// Toggle back-face edge visibility.
#[allow(dead_code)]
pub fn wireframe_toggle_backface(cfg: &mut WireframeConfig) {
    cfg.show_backface = !cfg.show_backface;
}

/// Return `true` if the wireframe overlay is visible.
#[allow(dead_code)]
pub fn wireframe_is_visible(cfg: &WireframeConfig) -> bool {
    cfg.visible
}

/// Set wireframe visibility.
#[allow(dead_code)]
pub fn set_wireframe_visible(cfg: &mut WireframeConfig, visible: bool) {
    cfg.visible = visible;
}

/// Clear all edges and positions from a draw call.
#[allow(dead_code)]
pub fn clear_wireframe_draw_call(call: &mut WireframeDrawCall) {
    call.positions.clear();
    call.edges.clear();
}

// ── Private helpers ───────────────────────────────────────────────────────────

#[inline]
fn edge_key(a: u32, b: u32) -> (u32, u32) {
    if a <= b { (a, b) } else { (b, a) }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn quad_faces() -> Vec<[u32; 3]> {
        // Two triangles forming a quad: verts 0,1,2,3
        vec![[0, 1, 2], [0, 2, 3]]
    }

    fn quad_verts() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
    }

    #[test]
    fn test_default_wireframe_config() {
        let cfg = default_wireframe_config();
        assert_eq!(cfg.color, [0.0, 0.0, 0.0, 1.0]);
        assert!((cfg.line_width - 1.0).abs() < 1e-6);
        assert!(!cfg.show_backface);
        assert!(cfg.visible);
    }

    #[test]
    fn test_build_wireframe_edges_deduplication() {
        // The quad shares edge (0, 2) between both triangles — should appear once.
        let faces = quad_faces();
        let edges = build_wireframe_edges(&faces);
        // 2 tris × 3 edges = 6 raw; shared edge (0,2) deduplicated → 5 unique
        assert_eq!(edges.len(), 5);
    }

    #[test]
    fn test_build_wireframe_edges_empty() {
        let edges = build_wireframe_edges(&[]);
        assert!(edges.is_empty());
    }

    #[test]
    fn test_wireframe_draw_call_edge_count() {
        let faces = quad_faces();
        let verts = quad_verts();
        let cfg = default_wireframe_config();
        let edges = build_wireframe_edges(&faces);
        let call = wireframe_draw_call(&verts, &edges, &cfg);
        assert_eq!(wireframe_edge_count(&call), edges.len());
    }

    #[test]
    fn test_set_wireframe_color() {
        let mut cfg = default_wireframe_config();
        set_wireframe_color(&mut cfg, 1.0, 0.0, 0.0, 0.5);
        assert_eq!(cfg.color, [1.0, 0.0, 0.0, 0.5]);
    }

    #[test]
    fn test_set_wireframe_line_width() {
        let mut cfg = default_wireframe_config();
        set_wireframe_line_width(&mut cfg, 3.5);
        assert!((cfg.line_width - 3.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_wireframe_line_width_negative_clamped() {
        let mut cfg = default_wireframe_config();
        set_wireframe_line_width(&mut cfg, -1.0);
        assert!((cfg.line_width - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_wireframe_toggle_backface() {
        let mut cfg = default_wireframe_config();
        assert!(!cfg.show_backface);
        wireframe_toggle_backface(&mut cfg);
        assert!(cfg.show_backface);
        wireframe_toggle_backface(&mut cfg);
        assert!(!cfg.show_backface);
    }

    #[test]
    fn test_wireframe_is_visible() {
        let cfg = default_wireframe_config();
        assert!(wireframe_is_visible(&cfg));
    }

    #[test]
    fn test_set_wireframe_visible() {
        let mut cfg = default_wireframe_config();
        set_wireframe_visible(&mut cfg, false);
        assert!(!wireframe_is_visible(&cfg));
        set_wireframe_visible(&mut cfg, true);
        assert!(wireframe_is_visible(&cfg));
    }

    #[test]
    fn test_clear_wireframe_draw_call() {
        let faces = quad_faces();
        let verts = quad_verts();
        let cfg = default_wireframe_config();
        let edges = build_wireframe_edges(&faces);
        let mut call = wireframe_draw_call(&verts, &edges, &cfg);
        clear_wireframe_draw_call(&mut call);
        assert!(call.positions.is_empty());
        assert!(call.edges.is_empty());
    }

    #[test]
    fn test_wireframe_draw_call_positions_count() {
        let verts = quad_verts();
        let cfg = default_wireframe_config();
        let call = wireframe_draw_call(&verts, &[], &cfg);
        assert_eq!(call.positions.len(), verts.len());
    }

    #[test]
    fn test_wireframe_edge_sorted_order() {
        // Edges should always have v0 <= v1 due to edge_key.
        let faces = vec![[3u32, 1, 0]];
        let edges = build_wireframe_edges(&faces);
        for e in &edges {
            assert!(e.v0 <= e.v1, "edge v0 should be <= v1");
        }
    }
}
