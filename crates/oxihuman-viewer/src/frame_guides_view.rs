// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Composition guides overlay (rule of thirds, golden ratio, diagonals).

/// Guide type for composition overlay.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GuideType {
    RuleOfThirds,
    GoldenRatio,
    Diagonals,
    CenterCross,
    Custom,
}

/// Frame guides view configuration.
#[derive(Debug, Clone)]
pub struct FrameGuidesView {
    pub guide_type: GuideType,
    pub color: [f32; 4],
    pub line_width: f32,
    pub enabled: bool,
}

impl FrameGuidesView {
    pub fn new() -> Self {
        Self {
            guide_type: GuideType::RuleOfThirds,
            color: [1.0, 1.0, 1.0, 0.5],
            line_width: 1.0,
            enabled: true,
        }
    }
}

impl Default for FrameGuidesView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new frame guides view.
pub fn new_frame_guides_view() -> FrameGuidesView {
    FrameGuidesView::new()
}

/// Set the guide type.
pub fn fgv_set_guide_type(view: &mut FrameGuidesView, guide_type: GuideType) {
    view.guide_type = guide_type;
}

/// Set overlay color as RGBA.
pub fn fgv_set_color(view: &mut FrameGuidesView, r: f32, g: f32, b: f32, a: f32) {
    view.color = [
        r.clamp(0.0, 1.0),
        g.clamp(0.0, 1.0),
        b.clamp(0.0, 1.0),
        a.clamp(0.0, 1.0),
    ];
}

/// Set line width in pixels.
pub fn fgv_set_line_width(view: &mut FrameGuidesView, width: f32) {
    view.line_width = width.clamp(0.5, 8.0);
}

/// Toggle guide visibility.
pub fn fgv_set_enabled(view: &mut FrameGuidesView, enabled: bool) {
    view.enabled = enabled;
}

/// Return the grid division count for the current guide type.
pub fn fgv_grid_divisions(view: &FrameGuidesView) -> u32 {
    match view.guide_type {
        GuideType::RuleOfThirds => 3,
        GuideType::GoldenRatio => 5,
        GuideType::Diagonals => 2,
        GuideType::CenterCross => 2,
        GuideType::Custom => 4,
    }
}

/// Serialize to JSON-like string.
pub fn frame_guides_view_to_json(view: &FrameGuidesView) -> String {
    let gt = match view.guide_type {
        GuideType::RuleOfThirds => "rule_of_thirds",
        GuideType::GoldenRatio => "golden_ratio",
        GuideType::Diagonals => "diagonals",
        GuideType::CenterCross => "center_cross",
        GuideType::Custom => "custom",
    };
    format!(
        r#"{{"guide_type":"{gt}","line_width":{:.4},"enabled":{}}}"#,
        view.line_width, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_frame_guides_view();
        assert_eq!(v.guide_type, GuideType::RuleOfThirds);
        assert!(v.enabled);
    }

    #[test]
    fn test_set_guide_type() {
        let mut v = new_frame_guides_view();
        fgv_set_guide_type(&mut v, GuideType::GoldenRatio);
        assert_eq!(v.guide_type, GuideType::GoldenRatio);
    }

    #[test]
    fn test_line_width_clamp() {
        let mut v = new_frame_guides_view();
        fgv_set_line_width(&mut v, 0.1);
        assert!((v.line_width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_enabled_false() {
        let mut v = new_frame_guides_view();
        fgv_set_enabled(&mut v, false);
        assert!(!v.enabled);
    }

    #[test]
    fn test_grid_divisions_rule_of_thirds() {
        let v = new_frame_guides_view();
        assert_eq!(fgv_grid_divisions(&v), 3);
    }

    #[test]
    fn test_grid_divisions_golden() {
        let mut v = new_frame_guides_view();
        fgv_set_guide_type(&mut v, GuideType::GoldenRatio);
        assert_eq!(fgv_grid_divisions(&v), 5);
    }

    #[test]
    fn test_color_set() {
        let mut v = new_frame_guides_view();
        fgv_set_color(&mut v, 1.0, 0.0, 0.0, 1.0);
        assert!((v.color[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_json() {
        let v = new_frame_guides_view();
        let s = frame_guides_view_to_json(&v);
        assert!(s.contains("rule_of_thirds"));
    }

    #[test]
    fn test_clone() {
        let v = new_frame_guides_view();
        let v2 = v.clone();
        assert_eq!(v2.guide_type, v.guide_type);
    }
}
