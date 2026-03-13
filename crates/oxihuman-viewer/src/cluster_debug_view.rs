// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Clustered lighting debug overlay visualization.

/// Configuration for the cluster debug overlay.
#[derive(Debug, Clone)]
pub struct ClusterDebugConfig {
    pub tile_size_px: u32,
    pub depth_slices: u32,
    pub opacity: f32,
    pub show_empty: bool,
}

impl Default for ClusterDebugConfig {
    fn default() -> Self {
        Self { tile_size_px: 64, depth_slices: 16, opacity: 0.6, show_empty: false }
    }
}

/// State for the cluster debug view.
#[derive(Debug, Clone)]
pub struct ClusterDebugView {
    pub config: ClusterDebugConfig,
    pub enabled: bool,
    pub highlight_slice: Option<u32>,
}

impl Default for ClusterDebugView {
    fn default() -> Self {
        Self { config: ClusterDebugConfig::default(), enabled: false, highlight_slice: None }
    }
}

/// Enable the cluster debug overlay.
pub fn cdv_enable(view: &mut ClusterDebugView) {
    view.enabled = true;
}

/// Disable the cluster debug overlay.
pub fn cdv_disable(view: &mut ClusterDebugView) {
    view.enabled = false;
}

/// Set the tile size in pixels.
pub fn cdv_set_tile_size(view: &mut ClusterDebugView, size_px: u32) {
    view.config.tile_size_px = size_px.max(1);
}

/// Highlight a specific depth slice (None = show all).
pub fn cdv_set_highlight_slice(view: &mut ClusterDebugView, slice: Option<u32>) {
    view.highlight_slice = slice.map(|s| s.min(view.config.depth_slices.saturating_sub(1)));
}

/// Return the cluster count for a given screen tile position.
pub fn cdv_cluster_count_at(tile_x: u32, tile_y: u32, tile_size: u32) -> u32 {
    /* placeholder: returns a deterministic value for testing */
    ((tile_x ^ tile_y) % 8).wrapping_add(tile_size % 4)
}

/// Export the debug config to a JSON string (stub).
pub fn cdv_to_json(view: &ClusterDebugView) -> String {
    format!(
        r#"{{"tile_size_px":{},"depth_slices":{},"opacity":{:.2},"enabled":{}}}"#,
        view.config.tile_size_px, view.config.depth_slices, view.config.opacity, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_disabled() {
        /* default should be disabled */
        let v = ClusterDebugView::default();
        assert!(!v.enabled);
    }

    #[test]
    fn test_enable_disable() {
        /* enable and disable should toggle */
        let mut v = ClusterDebugView::default();
        cdv_enable(&mut v);
        assert!(v.enabled);
        cdv_disable(&mut v);
        assert!(!v.enabled);
    }

    #[test]
    fn test_set_tile_size() {
        /* tile size should be updated */
        let mut v = ClusterDebugView::default();
        cdv_set_tile_size(&mut v, 32);
        assert_eq!(v.config.tile_size_px, 32);
    }

    #[test]
    fn test_tile_size_minimum() {
        /* tile size should be at least 1 */
        let mut v = ClusterDebugView::default();
        cdv_set_tile_size(&mut v, 0);
        assert_eq!(v.config.tile_size_px, 1);
    }

    #[test]
    fn test_highlight_slice_set() {
        /* highlight slice should be stored */
        let mut v = ClusterDebugView::default();
        cdv_set_highlight_slice(&mut v, Some(3));
        assert_eq!(v.highlight_slice, Some(3));
    }

    #[test]
    fn test_highlight_slice_clamp() {
        /* highlight slice should be clamped to depth slice count */
        let mut v = ClusterDebugView::default();
        cdv_set_highlight_slice(&mut v, Some(9999));
        assert!(v.highlight_slice.expect("should succeed") < v.config.depth_slices);
    }

    #[test]
    fn test_highlight_slice_none() {
        /* none highlight should clear the highlight */
        let mut v = ClusterDebugView::default();
        cdv_set_highlight_slice(&mut v, Some(2));
        cdv_set_highlight_slice(&mut v, None);
        assert!(v.highlight_slice.is_none());
    }

    #[test]
    fn test_cluster_count_deterministic() {
        /* same inputs should produce same output */
        let a = cdv_cluster_count_at(3, 5, 64);
        let b = cdv_cluster_count_at(3, 5, 64);
        assert_eq!(a, b);
    }

    #[test]
    fn test_to_json_contains_enabled() {
        /* JSON output should contain enabled field */
        let mut v = ClusterDebugView::default();
        cdv_enable(&mut v);
        let json = cdv_to_json(&v);
        assert!(json.contains("true"));
    }
}
