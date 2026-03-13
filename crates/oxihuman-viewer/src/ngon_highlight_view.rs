// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Highlight n-gon faces (faces with more than 4 vertices) in a quad mesh.
#[derive(Debug, Clone)]
pub struct NgonHighlightView {
    pub enabled: bool,
    /// Minimum vertex count to consider a face an n-gon (usually 5).
    pub min_verts: u32,
    /// Highlight colour [R, G, B].
    pub color: [f32; 3],
    /// Opacity of the highlight overlay.
    pub opacity: f32,
}

pub fn new_ngon_highlight_view() -> NgonHighlightView {
    NgonHighlightView {
        enabled: false,
        min_verts: 5,
        color: [1.0, 0.0, 0.5],
        opacity: 0.6,
    }
}

pub fn nhv_enable(v: &mut NgonHighlightView) {
    v.enabled = true;
}

pub fn nhv_set_color(v: &mut NgonHighlightView, r: f32, g: f32, b: f32) {
    v.color = [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)];
}

pub fn nhv_set_opacity(v: &mut NgonHighlightView, o: f32) {
    v.opacity = o.clamp(0.0, 1.0);
}

pub fn nhv_set_min_verts(v: &mut NgonHighlightView, n: u32) {
    v.min_verts = n.max(3);
}

pub fn nhv_is_ngon(v: &NgonHighlightView, face_vert_count: u32) -> bool {
    face_vert_count >= v.min_verts
}

pub fn nhv_to_json(v: &NgonHighlightView) -> String {
    format!(
        r#"{{"enabled":{},"min_verts":{},"opacity":{:.4}}}"#,
        v.enabled, v.min_verts, v.opacity
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        /* min_verts=5, enabled=false */
        let v = new_ngon_highlight_view();
        assert_eq!(v.min_verts, 5);
        assert!(!v.enabled);
    }

    #[test]
    fn test_enable() {
        /* enable flag */
        let mut v = new_ngon_highlight_view();
        nhv_enable(&mut v);
        assert!(v.enabled);
    }

    #[test]
    fn test_is_ngon_5() {
        /* 5 verts is n-gon */
        let v = new_ngon_highlight_view();
        assert!(nhv_is_ngon(&v, 5));
    }

    #[test]
    fn test_is_ngon_4_not() {
        /* 4 verts is not n-gon with min=5 */
        let v = new_ngon_highlight_view();
        assert!(!nhv_is_ngon(&v, 4));
    }

    #[test]
    fn test_set_opacity() {
        /* opacity stored */
        let mut v = new_ngon_highlight_view();
        nhv_set_opacity(&mut v, 0.8);
        assert!((v.opacity - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_opacity_clamp() {
        /* clamped */
        let mut v = new_ngon_highlight_view();
        nhv_set_opacity(&mut v, 2.0);
        assert_eq!(v.opacity, 1.0);
    }

    #[test]
    fn test_set_min_verts_min_3() {
        /* min enforced at 3 */
        let mut v = new_ngon_highlight_view();
        nhv_set_min_verts(&mut v, 0);
        assert_eq!(v.min_verts, 3);
    }

    #[test]
    fn test_color_clamp() {
        /* colour components clamped */
        let mut v = new_ngon_highlight_view();
        nhv_set_color(&mut v, -1.0, 2.0, 0.5);
        assert_eq!(v.color[0], 0.0);
        assert_eq!(v.color[1], 1.0);
    }

    #[test]
    fn test_to_json() {
        /* JSON has min_verts */
        assert!(nhv_to_json(&new_ngon_highlight_view()).contains("min_verts"));
    }
}
