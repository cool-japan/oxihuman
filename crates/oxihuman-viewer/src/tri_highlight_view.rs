// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Highlight triangles (3-vertex faces) in a predominantly quad mesh.
#[derive(Debug, Clone)]
pub struct TriHighlightView {
    pub enabled: bool,
    /// Highlight colour for triangle faces.
    pub color: [f32; 3],
    /// Opacity of the highlight.
    pub opacity: f32,
    /// If true, also count triangles and show in stats.
    pub show_count: bool,
}

pub fn new_tri_highlight_view() -> TriHighlightView {
    TriHighlightView {
        enabled: false,
        color: [1.0, 0.5, 0.0],
        opacity: 0.7,
        show_count: true,
    }
}

pub fn thv_enable(v: &mut TriHighlightView) {
    v.enabled = true;
}

pub fn thv_set_opacity(v: &mut TriHighlightView, o: f32) {
    v.opacity = o.clamp(0.0, 1.0);
}

pub fn thv_set_color(v: &mut TriHighlightView, r: f32, g: f32, b: f32) {
    v.color = [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)];
}

pub fn thv_set_show_count(v: &mut TriHighlightView, show: bool) {
    v.show_count = show;
}

pub fn thv_is_tri(face_vert_count: u32) -> bool {
    face_vert_count == 3
}

/// Returns ratio of triangles among total faces (0.0 … 1.0).
pub fn thv_tri_ratio(tri_count: u32, total_count: u32) -> f32 {
    if total_count == 0 {
        return 0.0;
    }
    (tri_count as f32 / total_count as f32).clamp(0.0, 1.0)
}

pub fn thv_to_json(v: &TriHighlightView) -> String {
    format!(
        r#"{{"enabled":{},"opacity":{:.4},"show_count":{}}}"#,
        v.enabled, v.opacity, v.show_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        /* enabled=false, show_count=true */
        let v = new_tri_highlight_view();
        assert!(!v.enabled);
        assert!(v.show_count);
    }

    #[test]
    fn test_enable() {
        /* enable */
        let mut v = new_tri_highlight_view();
        thv_enable(&mut v);
        assert!(v.enabled);
    }

    #[test]
    fn test_is_tri_true() {
        /* 3 verts is tri */
        assert!(thv_is_tri(3));
    }

    #[test]
    fn test_is_tri_false() {
        /* 4 verts is not tri */
        assert!(!thv_is_tri(4));
    }

    #[test]
    fn test_tri_ratio() {
        /* 1 out of 4 = 0.25 */
        assert!((thv_tri_ratio(1, 4) - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_tri_ratio_zero_total() {
        /* no faces → ratio 0 */
        assert_eq!(thv_tri_ratio(0, 0), 0.0);
    }

    #[test]
    fn test_set_opacity() {
        /* opacity stored */
        let mut v = new_tri_highlight_view();
        thv_set_opacity(&mut v, 0.5);
        assert!((v.opacity - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_opacity_clamp() {
        /* clamped above 1 */
        let mut v = new_tri_highlight_view();
        thv_set_opacity(&mut v, 3.0);
        assert_eq!(v.opacity, 1.0);
    }

    #[test]
    fn test_color_set() {
        /* colour stored */
        let mut v = new_tri_highlight_view();
        thv_set_color(&mut v, 0.1, 0.2, 0.3);
        assert!((v.color[0] - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        /* JSON has opacity */
        assert!(thv_to_json(&new_tri_highlight_view()).contains("opacity"));
    }
}
