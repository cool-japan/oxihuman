// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Bounding volume hierarchy view — visualises AABB/BVH tree levels.

/// Bounding volume view configuration.
#[derive(Debug, Clone)]
pub struct BoundingVolumeView {
    pub enabled: bool,
    pub max_depth: u32,
    pub color_leaf: [f32; 4],
    pub color_inner: [f32; 4],
    pub show_inner_nodes: bool,
}

impl BoundingVolumeView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            max_depth: 4,
            color_leaf: [0.0, 1.0, 0.5, 0.5],
            color_inner: [0.5, 0.5, 1.0, 0.3],
            show_inner_nodes: true,
        }
    }
}

impl Default for BoundingVolumeView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new bounding volume view.
pub fn new_bounding_volume_view() -> BoundingVolumeView {
    BoundingVolumeView::new()
}

/// Enable or disable the BVH overlay.
pub fn bvv_set_enabled(v: &mut BoundingVolumeView, enabled: bool) {
    v.enabled = enabled;
}

/// Set maximum BVH depth to render.
pub fn bvv_set_max_depth(v: &mut BoundingVolumeView, depth: u32) {
    v.max_depth = depth.clamp(1, 32);
}

/// Toggle inner node display.
pub fn bvv_set_show_inner(v: &mut BoundingVolumeView, show: bool) {
    v.show_inner_nodes = show;
}

/// Set leaf node colour.
pub fn bvv_set_color_leaf(v: &mut BoundingVolumeView, color: [f32; 4]) {
    v.color_leaf = color;
}

/// Set inner node colour.
pub fn bvv_set_color_inner(v: &mut BoundingVolumeView, color: [f32; 4]) {
    v.color_inner = color;
}

/// Serialize to JSON-like string.
pub fn bounding_volume_view_to_json(v: &BoundingVolumeView) -> String {
    format!(
        r#"{{"enabled":{},"max_depth":{},"show_inner_nodes":{}}}"#,
        v.enabled, v.max_depth, v.show_inner_nodes
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_bounding_volume_view();
        assert!(!v.enabled);
        assert_eq!(v.max_depth, 4);
    }

    #[test]
    fn test_enable() {
        let mut v = new_bounding_volume_view();
        bvv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_max_depth_clamp() {
        let mut v = new_bounding_volume_view();
        bvv_set_max_depth(&mut v, 0);
        assert_eq!(v.max_depth, 1);
    }

    #[test]
    fn test_max_depth_set() {
        let mut v = new_bounding_volume_view();
        bvv_set_max_depth(&mut v, 8);
        assert_eq!(v.max_depth, 8);
    }

    #[test]
    fn test_inner_toggle() {
        let mut v = new_bounding_volume_view();
        bvv_set_show_inner(&mut v, false);
        assert!(!v.show_inner_nodes);
    }

    #[test]
    fn test_color_leaf() {
        let mut v = new_bounding_volume_view();
        bvv_set_color_leaf(&mut v, [1.0, 0.0, 0.0, 1.0]);
        assert!((v.color_leaf[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_color_inner() {
        let mut v = new_bounding_volume_view();
        bvv_set_color_inner(&mut v, [0.0, 0.5, 0.5, 1.0]);
        assert!((v.color_inner[1] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_json_keys() {
        let v = new_bounding_volume_view();
        let s = bounding_volume_view_to_json(&v);
        assert!(s.contains("max_depth"));
    }

    #[test]
    fn test_clone() {
        let v = new_bounding_volume_view();
        let v2 = v.clone();
        assert_eq!(v2.max_depth, v.max_depth);
    }
}
