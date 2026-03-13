// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Test pattern (color bars) view for display calibration.

/// Test pattern type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TestPatternType {
    ColorBars,
    Smpte75,
    Smpte100,
    Grayscale,
    CheckerBoard,
    SolidColor,
}

/// Test pattern view configuration.
#[derive(Debug, Clone)]
pub struct TestPatternView {
    pub pattern: TestPatternType,
    pub grid_size: u32,
    pub enabled: bool,
    pub solid_color: [f32; 4],
}

impl TestPatternView {
    pub fn new() -> Self {
        Self {
            pattern: TestPatternType::ColorBars,
            grid_size: 8,
            enabled: false,
            solid_color: [1.0, 0.0, 0.0, 1.0],
        }
    }
}

impl Default for TestPatternView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new test pattern view.
pub fn new_test_pattern_view() -> TestPatternView {
    TestPatternView::new()
}

/// Set test pattern type.
pub fn tpv_set_pattern(view: &mut TestPatternView, pattern: TestPatternType) {
    view.pattern = pattern;
}

/// Set grid size for checker/grid patterns.
pub fn tpv_set_grid_size(view: &mut TestPatternView, size: u32) {
    view.grid_size = size.clamp(2, 256);
}

/// Enable or disable the test pattern overlay.
pub fn tpv_set_enabled(view: &mut TestPatternView, enabled: bool) {
    view.enabled = enabled;
}

/// Set solid color for SolidColor pattern type.
pub fn tpv_set_solid_color(view: &mut TestPatternView, r: f32, g: f32, b: f32, a: f32) {
    view.solid_color = [
        r.clamp(0.0, 1.0),
        g.clamp(0.0, 1.0),
        b.clamp(0.0, 1.0),
        a.clamp(0.0, 1.0),
    ];
}

/// Compute number of color bands for bar patterns.
pub fn tpv_bar_count(view: &TestPatternView) -> u32 {
    match view.pattern {
        TestPatternType::ColorBars | TestPatternType::Smpte75 | TestPatternType::Smpte100 => 7,
        TestPatternType::Grayscale => 10,
        TestPatternType::CheckerBoard => view.grid_size * view.grid_size,
        TestPatternType::SolidColor => 1,
    }
}

/// Serialize to JSON-like string.
pub fn test_pattern_view_to_json(view: &TestPatternView) -> String {
    let pattern_str = match view.pattern {
        TestPatternType::ColorBars => "color_bars",
        TestPatternType::Smpte75 => "smpte_75",
        TestPatternType::Smpte100 => "smpte_100",
        TestPatternType::Grayscale => "grayscale",
        TestPatternType::CheckerBoard => "checker",
        TestPatternType::SolidColor => "solid",
    };
    format!(
        r#"{{"pattern":"{pattern_str}","grid_size":{},"enabled":{}}}"#,
        view.grid_size, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_test_pattern_view();
        assert_eq!(v.pattern, TestPatternType::ColorBars);
        assert!(!v.enabled);
    }

    #[test]
    fn test_set_pattern() {
        let mut v = new_test_pattern_view();
        tpv_set_pattern(&mut v, TestPatternType::Grayscale);
        assert_eq!(v.pattern, TestPatternType::Grayscale);
    }

    #[test]
    fn test_grid_size_clamp() {
        let mut v = new_test_pattern_view();
        tpv_set_grid_size(&mut v, 1);
        assert_eq!(v.grid_size, 2);
    }

    #[test]
    fn test_enabled_toggle() {
        let mut v = new_test_pattern_view();
        tpv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_bar_count_color_bars() {
        let v = new_test_pattern_view();
        assert_eq!(tpv_bar_count(&v), 7);
    }

    #[test]
    fn test_bar_count_grayscale() {
        let mut v = new_test_pattern_view();
        tpv_set_pattern(&mut v, TestPatternType::Grayscale);
        assert_eq!(tpv_bar_count(&v), 10);
    }

    #[test]
    fn test_solid_color_set() {
        let mut v = new_test_pattern_view();
        tpv_set_solid_color(&mut v, 0.0, 1.0, 0.0, 1.0);
        assert!((v.solid_color[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_json() {
        let v = new_test_pattern_view();
        let s = test_pattern_view_to_json(&v);
        assert!(s.contains("color_bars"));
    }

    #[test]
    fn test_clone() {
        let v = new_test_pattern_view();
        let v2 = v.clone();
        assert_eq!(v2.pattern, v.pattern);
    }
}
