// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Focus peaking highlight overlay for manual focus assistance.

/// Focus peaking view configuration.
#[derive(Debug, Clone)]
pub struct FocusPeakingView {
    pub threshold: f32,
    pub highlight_color: [f32; 4],
    pub enabled: bool,
    pub sensitivity: f32,
}

impl FocusPeakingView {
    pub fn new() -> Self {
        Self {
            threshold: 0.5,
            highlight_color: [1.0, 0.0, 0.0, 1.0],
            enabled: false,
            sensitivity: 0.5,
        }
    }
}

impl Default for FocusPeakingView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new focus peaking view.
pub fn new_focus_peaking_view() -> FocusPeakingView {
    FocusPeakingView::new()
}

/// Set edge detection threshold for focus peaking.
pub fn fpv_set_threshold(view: &mut FocusPeakingView, threshold: f32) {
    view.threshold = threshold.clamp(0.0, 1.0);
}

/// Set highlight color as RGBA.
pub fn fpv_set_highlight_color(view: &mut FocusPeakingView, r: f32, g: f32, b: f32, a: f32) {
    view.highlight_color = [
        r.clamp(0.0, 1.0),
        g.clamp(0.0, 1.0),
        b.clamp(0.0, 1.0),
        a.clamp(0.0, 1.0),
    ];
}

/// Set detection sensitivity.
pub fn fpv_set_sensitivity(view: &mut FocusPeakingView, sensitivity: f32) {
    view.sensitivity = sensitivity.clamp(0.0, 1.0);
}

/// Toggle focus peaking overlay.
pub fn fpv_set_enabled(view: &mut FocusPeakingView, enabled: bool) {
    view.enabled = enabled;
}

/// Evaluate whether a given edge gradient value would be highlighted.
pub fn fpv_is_in_focus(view: &FocusPeakingView, gradient: f32) -> bool {
    gradient * view.sensitivity >= view.threshold
}

/// Serialize to JSON-like string.
pub fn focus_peaking_view_to_json(view: &FocusPeakingView) -> String {
    format!(
        r#"{{"threshold":{:.4},"sensitivity":{:.4},"enabled":{}}}"#,
        view.threshold, view.sensitivity, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_focus_peaking_view();
        assert!((v.threshold - 0.5).abs() < 1e-6);
        assert!(!v.enabled);
    }

    #[test]
    fn test_threshold_clamp() {
        let mut v = new_focus_peaking_view();
        fpv_set_threshold(&mut v, 2.0);
        assert_eq!(v.threshold, 1.0);
    }

    #[test]
    fn test_sensitivity_set() {
        let mut v = new_focus_peaking_view();
        fpv_set_sensitivity(&mut v, 0.8);
        assert!((v.sensitivity - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_enabled_toggle() {
        let mut v = new_focus_peaking_view();
        fpv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_in_focus_high_gradient() {
        let v = new_focus_peaking_view();
        assert!(fpv_is_in_focus(&v, 1.0));
    }

    #[test]
    fn test_not_in_focus_low_gradient() {
        let v = new_focus_peaking_view();
        assert!(!fpv_is_in_focus(&v, 0.0));
    }

    #[test]
    fn test_highlight_color_set() {
        let mut v = new_focus_peaking_view();
        fpv_set_highlight_color(&mut v, 0.0, 1.0, 0.0, 1.0);
        assert!((v.highlight_color[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_json() {
        let v = new_focus_peaking_view();
        let s = focus_peaking_view_to_json(&v);
        assert!(s.contains("threshold"));
    }

    #[test]
    fn test_clone() {
        let v = new_focus_peaking_view();
        let v2 = v.clone();
        assert!((v2.sensitivity - v.sensitivity).abs() < 1e-6);
    }
}
