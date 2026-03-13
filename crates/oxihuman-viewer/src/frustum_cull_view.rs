// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Frustum culling debug view — visualizes objects inside/outside the view frustum.

/// Frustum cull debug view configuration.
#[derive(Debug, Clone)]
pub struct FrustumCullView {
    pub enabled: bool,
    pub show_frustum_planes: bool,
    pub highlight_culled: bool,
    pub cull_ratio: f32,
    pub total_objects: u32,
}

impl FrustumCullView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            show_frustum_planes: false,
            highlight_culled: true,
            cull_ratio: 0.0,
            total_objects: 0,
        }
    }
}

impl Default for FrustumCullView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new frustum cull view.
pub fn new_frustum_cull_view() -> FrustumCullView {
    FrustumCullView::new()
}

/// Enable or disable frustum cull debug visualization.
pub fn fcv_set_enabled(v: &mut FrustumCullView, enabled: bool) {
    v.enabled = enabled;
}

/// Show the six frustum clipping planes.
pub fn fcv_set_show_frustum_planes(v: &mut FrustumCullView, show: bool) {
    v.show_frustum_planes = show;
}

/// Toggle highlighting of culled objects.
pub fn fcv_set_highlight_culled(v: &mut FrustumCullView, highlight: bool) {
    v.highlight_culled = highlight;
}

/// Update cull ratio (fraction of objects culled, 0–1).
pub fn fcv_update_cull_ratio(v: &mut FrustumCullView, culled: u32, total: u32) {
    v.total_objects = total;
    if total == 0 {
        v.cull_ratio = 0.0;
    } else {
        v.cull_ratio = (culled as f32 / total as f32).clamp(0.0, 1.0);
    }
}

/// Return number of visible objects estimate.
pub fn fcv_visible_count(v: &FrustumCullView) -> u32 {
    let culled = (v.cull_ratio * v.total_objects as f32) as u32;
    v.total_objects.saturating_sub(culled)
}

/// Serialize to JSON-like string.
pub fn frustum_cull_view_to_json(v: &FrustumCullView) -> String {
    format!(
        r#"{{"enabled":{},"show_frustum_planes":{},"highlight_culled":{},"cull_ratio":{:.4},"total_objects":{}}}"#,
        v.enabled, v.show_frustum_planes, v.highlight_culled, v.cull_ratio, v.total_objects
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_frustum_cull_view();
        assert!(!v.enabled);
        assert_eq!(v.total_objects, 0);
    }

    #[test]
    fn test_enable() {
        let mut v = new_frustum_cull_view();
        fcv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_show_planes() {
        let mut v = new_frustum_cull_view();
        fcv_set_show_frustum_planes(&mut v, true);
        assert!(v.show_frustum_planes);
    }

    #[test]
    fn test_highlight_culled_off() {
        let mut v = new_frustum_cull_view();
        fcv_set_highlight_culled(&mut v, false);
        assert!(!v.highlight_culled);
    }

    #[test]
    fn test_cull_ratio_update() {
        let mut v = new_frustum_cull_view();
        fcv_update_cull_ratio(&mut v, 50, 100);
        assert!((v.cull_ratio - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_cull_ratio_zero_total() {
        let mut v = new_frustum_cull_view();
        fcv_update_cull_ratio(&mut v, 0, 0);
        assert_eq!(v.cull_ratio, 0.0);
    }

    #[test]
    fn test_visible_count() {
        let mut v = new_frustum_cull_view();
        fcv_update_cull_ratio(&mut v, 30, 100);
        let vis = fcv_visible_count(&v);
        assert!(vis <= 100);
    }

    #[test]
    fn test_visible_count_all_culled() {
        let mut v = new_frustum_cull_view();
        fcv_update_cull_ratio(&mut v, 100, 100);
        assert_eq!(fcv_visible_count(&v), 0);
    }

    #[test]
    fn test_json_keys() {
        let v = new_frustum_cull_view();
        let s = frustum_cull_view_to_json(&v);
        assert!(s.contains("cull_ratio"));
    }

    #[test]
    fn test_clone() {
        let v = new_frustum_cull_view();
        let v2 = v.clone();
        assert_eq!(v2.total_objects, v.total_objects);
    }
}
