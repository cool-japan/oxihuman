// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Vertex buffer layout view — visualizes VBO binding, strides, and attribute layout.

/// Vertex buffer view configuration.
#[derive(Debug, Clone)]
pub struct VertexBufferView {
    pub enabled: bool,
    pub total_vbos: u32,
    pub total_bytes: u64,
    pub stride_bytes: u32,
    pub show_attribute_layout: bool,
}

impl VertexBufferView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            total_vbos: 0,
            total_bytes: 0,
            stride_bytes: 32,
            show_attribute_layout: true,
        }
    }
}

impl Default for VertexBufferView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new vertex buffer view.
pub fn new_vertex_buffer_view() -> VertexBufferView {
    VertexBufferView::new()
}

/// Enable or disable vertex buffer debug overlay.
pub fn vbv_set_enabled(v: &mut VertexBufferView, enabled: bool) {
    v.enabled = enabled;
}

/// Update VBO count and total byte size.
pub fn vbv_update_stats(v: &mut VertexBufferView, vbo_count: u32, total_bytes: u64) {
    v.total_vbos = vbo_count;
    v.total_bytes = total_bytes;
}

/// Set per-vertex stride.
pub fn vbv_set_stride(v: &mut VertexBufferView, stride: u32) {
    v.stride_bytes = stride.max(4);
}

/// Toggle attribute layout visualization.
pub fn vbv_set_show_attribute_layout(v: &mut VertexBufferView, show: bool) {
    v.show_attribute_layout = show;
}

/// Estimate total vertex count from bytes and stride.
pub fn vbv_vertex_count(v: &VertexBufferView) -> u64 {
    if v.stride_bytes == 0 {
        return 0;
    }
    v.total_bytes / v.stride_bytes as u64
}

/// Serialize to JSON-like string.
pub fn vertex_buffer_view_to_json(v: &VertexBufferView) -> String {
    format!(
        r#"{{"enabled":{},"total_vbos":{},"total_bytes":{},"stride_bytes":{},"show_attribute_layout":{}}}"#,
        v.enabled, v.total_vbos, v.total_bytes, v.stride_bytes, v.show_attribute_layout
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_vertex_buffer_view();
        assert!(!v.enabled);
        assert_eq!(v.stride_bytes, 32);
    }

    #[test]
    fn test_enable() {
        let mut v = new_vertex_buffer_view();
        vbv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_update_stats() {
        let mut v = new_vertex_buffer_view();
        vbv_update_stats(&mut v, 5, 1024 * 1024);
        assert_eq!(v.total_vbos, 5);
        assert_eq!(v.total_bytes, 1024 * 1024);
    }

    #[test]
    fn test_stride_min() {
        let mut v = new_vertex_buffer_view();
        vbv_set_stride(&mut v, 0);
        assert_eq!(v.stride_bytes, 4);
    }

    #[test]
    fn test_stride_set() {
        let mut v = new_vertex_buffer_view();
        vbv_set_stride(&mut v, 48);
        assert_eq!(v.stride_bytes, 48);
    }

    #[test]
    fn test_show_layout_toggle() {
        let mut v = new_vertex_buffer_view();
        vbv_set_show_attribute_layout(&mut v, false);
        assert!(!v.show_attribute_layout);
    }

    #[test]
    fn test_vertex_count_zero() {
        let v = new_vertex_buffer_view();
        assert_eq!(vbv_vertex_count(&v), 0);
    }

    #[test]
    fn test_vertex_count_computed() {
        let mut v = new_vertex_buffer_view();
        vbv_update_stats(&mut v, 1, 320);
        vbv_set_stride(&mut v, 32);
        assert_eq!(vbv_vertex_count(&v), 10);
    }

    #[test]
    fn test_json_keys() {
        let v = new_vertex_buffer_view();
        let s = vertex_buffer_view_to_json(&v);
        assert!(s.contains("stride_bytes"));
    }

    #[test]
    fn test_clone() {
        let v = new_vertex_buffer_view();
        let v2 = v.clone();
        assert_eq!(v2.stride_bytes, v.stride_bytes);
    }
}
