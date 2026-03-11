// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Paint-over annotation layer view stub.

/// Brush tool for paint-over.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaintBrushTool {
    Pen,
    Marker,
    Eraser,
    Fill,
}

/// A paint stroke entry.
#[derive(Debug, Clone)]
pub struct PaintStroke {
    pub tool: PaintBrushTool,
    pub color: [f32; 4],
    pub points: Vec<[f32; 2]>,
    pub brush_size: f32,
}

/// Paint-over annotation view.
#[derive(Debug, Clone)]
pub struct PaintOverView {
    pub strokes: Vec<PaintStroke>,
    pub opacity: f32,
    pub enabled: bool,
}

impl PaintOverView {
    pub fn new() -> Self {
        PaintOverView {
            strokes: Vec::new(),
            opacity: 1.0,
            enabled: true,
        }
    }
}

impl Default for PaintOverView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new paint-over view.
pub fn new_paint_over_view() -> PaintOverView {
    PaintOverView::new()
}

/// Add a stroke.
pub fn pov_add_stroke(view: &mut PaintOverView, stroke: PaintStroke) {
    view.strokes.push(stroke);
}

/// Set layer opacity.
pub fn pov_set_opacity(view: &mut PaintOverView, opacity: f32) {
    view.opacity = opacity.clamp(0.0, 1.0);
}

/// Clear all strokes.
pub fn pov_clear(view: &mut PaintOverView) {
    view.strokes.clear();
}

/// Enable or disable.
pub fn pov_set_enabled(view: &mut PaintOverView, enabled: bool) {
    view.enabled = enabled;
}

/// Return stroke count.
pub fn pov_stroke_count(view: &PaintOverView) -> usize {
    view.strokes.len()
}

/// Serialize to JSON-like string.
pub fn pov_to_json(view: &PaintOverView) -> String {
    format!(
        r#"{{"stroke_count":{},"opacity":{},"enabled":{}}}"#,
        view.strokes.len(),
        view.opacity,
        view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_stroke() -> PaintStroke {
        PaintStroke {
            tool: PaintBrushTool::Pen,
            color: [1.0, 0.0, 0.0, 1.0],
            points: vec![[0.0, 0.0], [1.0, 1.0]],
            brush_size: 5.0,
        }
    }

    #[test]
    fn test_initial_empty() {
        let v = new_paint_over_view();
        assert_eq!(pov_stroke_count(&v), 0 /* no strokes initially */);
    }

    #[test]
    fn test_add_stroke() {
        let mut v = new_paint_over_view();
        pov_add_stroke(&mut v, make_stroke());
        assert_eq!(pov_stroke_count(&v), 1 /* one stroke after add */);
    }

    #[test]
    fn test_clear() {
        let mut v = new_paint_over_view();
        pov_add_stroke(&mut v, make_stroke());
        pov_clear(&mut v);
        assert_eq!(pov_stroke_count(&v), 0 /* cleared */);
    }

    #[test]
    fn test_opacity_clamp() {
        let mut v = new_paint_over_view();
        pov_set_opacity(&mut v, 2.0);
        assert!((v.opacity - 1.0).abs() < 1e-6 /* opacity clamped to 1.0 */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_paint_over_view();
        pov_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_stroke_count() {
        let v = new_paint_over_view();
        let j = pov_to_json(&v);
        assert!(j.contains("\"stroke_count\"") /* JSON must have stroke_count */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_paint_over_view();
        assert!(v.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_default_opacity() {
        let v = new_paint_over_view();
        assert!((v.opacity - 1.0).abs() < 1e-6 /* default opacity must be 1.0 */);
    }

    #[test]
    fn test_multiple_strokes() {
        let mut v = new_paint_over_view();
        pov_add_stroke(&mut v, make_stroke());
        pov_add_stroke(&mut v, make_stroke());
        assert_eq!(pov_stroke_count(&v), 2 /* two strokes */);
    }
}
