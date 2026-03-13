// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Hair and strand render debug view.

/// Hair render debug state.
#[derive(Debug, Clone)]
pub struct HairRenderView {
    pub strand_count: u32,
    pub render_as_mesh: bool,
    pub width: f32,
    pub show_guide_curves: bool,
    pub shadow_mode: u8,
}

impl Default for HairRenderView {
    fn default() -> Self {
        Self {
            strand_count: 0,
            render_as_mesh: false,
            width: 0.005,
            show_guide_curves: false,
            shadow_mode: 0,
        }
    }
}

/// Create a new HairRenderView.
pub fn new_hair_render_view() -> HairRenderView {
    HairRenderView::default()
}

/// Set the strand count for preview.
pub fn hair_render_set_strand_count(view: &mut HairRenderView, n: u32) {
    view.strand_count = n;
}

/// Toggle mesh-based hair rendering.
pub fn hair_render_set_as_mesh(view: &mut HairRenderView, v: bool) {
    view.render_as_mesh = v;
}

/// Set strand width in world units.
pub fn hair_render_set_width(view: &mut HairRenderView, w: f32) {
    view.width = w.clamp(1e-5, 1.0);
}

/// Toggle guide curve visualization.
pub fn hair_render_show_guides(view: &mut HairRenderView, show: bool) {
    view.show_guide_curves = show;
}

/// Estimated triangle count for mesh-mode strands.
pub fn hair_render_estimated_tris(view: &HairRenderView) -> u32 {
    if view.render_as_mesh {
        view.strand_count * 6
    } else {
        0
    }
}

/// Serialize to JSON.
pub fn hair_render_to_json(view: &HairRenderView) -> String {
    format!(
        r#"{{"strand_count":{},"as_mesh":{},"width":{},"show_guides":{}}}"#,
        view.strand_count, view.render_as_mesh, view.width, view.show_guide_curves,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let v = new_hair_render_view();
        assert_eq!(v.strand_count, 0 /* default zero */);
    }

    #[test]
    fn test_strand_count() {
        let mut v = new_hair_render_view();
        hair_render_set_strand_count(&mut v, 10000);
        assert_eq!(v.strand_count, 10000 /* stored */);
    }

    #[test]
    fn test_as_mesh() {
        let mut v = new_hair_render_view();
        hair_render_set_as_mesh(&mut v, true);
        assert!(v.render_as_mesh /* enabled */);
    }

    #[test]
    fn test_width_clamp_low() {
        let mut v = new_hair_render_view();
        hair_render_set_width(&mut v, 0.0);
        assert!(v.width > 0.0 /* clamped above zero */);
    }

    #[test]
    fn test_width_clamp_high() {
        let mut v = new_hair_render_view();
        hair_render_set_width(&mut v, 10.0);
        assert!((v.width - 1.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_show_guides() {
        let mut v = new_hair_render_view();
        hair_render_show_guides(&mut v, true);
        assert!(v.show_guide_curves /* enabled */);
    }

    #[test]
    fn test_estimated_tris_mesh() {
        let mut v = new_hair_render_view();
        hair_render_set_strand_count(&mut v, 100);
        hair_render_set_as_mesh(&mut v, true);
        assert_eq!(hair_render_estimated_tris(&v), 600 /* 100 * 6 */);
    }

    #[test]
    fn test_estimated_tris_curve() {
        let v = new_hair_render_view();
        assert_eq!(hair_render_estimated_tris(&v), 0 /* curves mode */);
    }

    #[test]
    fn test_json_keys() {
        let v = new_hair_render_view();
        let j = hair_render_to_json(&v);
        assert!(j.contains("strand_count") /* key */);
    }
}
