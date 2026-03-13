// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Occlusion culling debug view — visualizes occlusion queries and hidden objects.

/// Occlusion cull debug view configuration.
#[derive(Debug, Clone)]
pub struct OcclusionCullView {
    pub enabled: bool,
    pub show_occluder_proxies: bool,
    pub highlight_occluded: bool,
    pub query_latency_frames: u32,
    pub occluded_ratio: f32,
}

impl OcclusionCullView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            show_occluder_proxies: false,
            highlight_occluded: true,
            query_latency_frames: 2,
            occluded_ratio: 0.0,
        }
    }
}

impl Default for OcclusionCullView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new occlusion cull view.
pub fn new_occlusion_cull_view() -> OcclusionCullView {
    OcclusionCullView::new()
}

/// Enable or disable occlusion cull debug view.
pub fn occv_set_enabled(v: &mut OcclusionCullView, enabled: bool) {
    v.enabled = enabled;
}

/// Show simplified occluder proxy geometry.
pub fn occv_set_show_occluder_proxies(v: &mut OcclusionCullView, show: bool) {
    v.show_occluder_proxies = show;
}

/// Toggle highlighting occluded objects.
pub fn occv_set_highlight_occluded(v: &mut OcclusionCullView, highlight: bool) {
    v.highlight_occluded = highlight;
}

/// Set GPU query latency in frames.
pub fn occv_set_query_latency(v: &mut OcclusionCullView, frames: u32) {
    v.query_latency_frames = frames.clamp(0, 8);
}

/// Update occlusion ratio from query results.
pub fn occv_update_occluded_ratio(v: &mut OcclusionCullView, ratio: f32) {
    v.occluded_ratio = ratio.clamp(0.0, 1.0);
}

/// Serialize to JSON-like string.
pub fn occlusion_cull_view_to_json(v: &OcclusionCullView) -> String {
    format!(
        r#"{{"enabled":{},"show_occluder_proxies":{},"highlight_occluded":{},"query_latency_frames":{},"occluded_ratio":{:.4}}}"#,
        v.enabled,
        v.show_occluder_proxies,
        v.highlight_occluded,
        v.query_latency_frames,
        v.occluded_ratio
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_occlusion_cull_view();
        assert!(!v.enabled);
        assert_eq!(v.query_latency_frames, 2);
    }

    #[test]
    fn test_enable() {
        let mut v = new_occlusion_cull_view();
        occv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_show_proxies() {
        let mut v = new_occlusion_cull_view();
        occv_set_show_occluder_proxies(&mut v, true);
        assert!(v.show_occluder_proxies);
    }

    #[test]
    fn test_highlight_toggle() {
        let mut v = new_occlusion_cull_view();
        occv_set_highlight_occluded(&mut v, false);
        assert!(!v.highlight_occluded);
    }

    #[test]
    fn test_query_latency_clamp() {
        let mut v = new_occlusion_cull_view();
        occv_set_query_latency(&mut v, 100);
        assert_eq!(v.query_latency_frames, 8);
    }

    #[test]
    fn test_query_latency_set() {
        let mut v = new_occlusion_cull_view();
        occv_set_query_latency(&mut v, 3);
        assert_eq!(v.query_latency_frames, 3);
    }

    #[test]
    fn test_occluded_ratio_clamp() {
        let mut v = new_occlusion_cull_view();
        occv_update_occluded_ratio(&mut v, 2.0);
        assert_eq!(v.occluded_ratio, 1.0);
    }

    #[test]
    fn test_occluded_ratio_set() {
        let mut v = new_occlusion_cull_view();
        occv_update_occluded_ratio(&mut v, 0.4);
        assert!((v.occluded_ratio - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_json_keys() {
        let v = new_occlusion_cull_view();
        let s = occlusion_cull_view_to_json(&v);
        assert!(s.contains("query_latency_frames"));
    }

    #[test]
    fn test_clone() {
        let v = new_occlusion_cull_view();
        let v2 = v.clone();
        assert_eq!(v2.query_latency_frames, v.query_latency_frames);
    }
}
