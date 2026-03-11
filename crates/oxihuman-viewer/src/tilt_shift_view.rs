// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Tilt-shift miniature effect stub.

/// Tilt-shift view configuration.
#[derive(Debug, Clone)]
pub struct TiltShiftView {
    pub focus_center_y: f32,
    pub focus_band_height: f32,
    pub blur_amount: f32,
    pub tilt_angle_deg: f32,
    pub saturation_boost: f32,
    pub enabled: bool,
}

impl TiltShiftView {
    pub fn new() -> Self {
        TiltShiftView {
            focus_center_y: 0.5,
            focus_band_height: 0.15,
            blur_amount: 0.8,
            tilt_angle_deg: 0.0,
            saturation_boost: 1.2,
            enabled: true,
        }
    }
}

impl Default for TiltShiftView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new tilt-shift view.
pub fn new_tilt_shift_view() -> TiltShiftView {
    TiltShiftView::new()
}

/// Compute blur weight for a normalized screen Y coordinate.
pub fn tsh_blur_at_y(tsh: &TiltShiftView, y: f32) -> f32 {
    /* Stub: returns blur weight; 0 inside band, blur_amount outside */
    let dist = (y - tsh.focus_center_y).abs();
    let half = tsh.focus_band_height / 2.0;
    if dist <= half {
        0.0
    } else {
        tsh.blur_amount * ((dist - half) / half).min(1.0)
    }
}

/// Set focus center Y (0.0 = top, 1.0 = bottom).
pub fn tsh_set_focus_center(tsh: &mut TiltShiftView, y: f32) {
    tsh.focus_center_y = y.clamp(0.0, 1.0);
}

/// Set focus band height.
pub fn tsh_set_focus_band(tsh: &mut TiltShiftView, height: f32) {
    tsh.focus_band_height = height.clamp(0.0, 1.0);
}

/// Set blur amount.
pub fn tsh_set_blur(tsh: &mut TiltShiftView, blur: f32) {
    tsh.blur_amount = blur.clamp(0.0, 1.0);
}

/// Enable or disable.
pub fn tsh_set_enabled(tsh: &mut TiltShiftView, enabled: bool) {
    tsh.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn tsh_to_json(tsh: &TiltShiftView) -> String {
    format!(
        r#"{{"focus_center_y":{},"focus_band":{},"blur":{},"enabled":{}}}"#,
        tsh.focus_center_y, tsh.focus_band_height, tsh.blur_amount, tsh.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_focus_center() {
        let t = new_tilt_shift_view();
        assert!((t.focus_center_y - 0.5).abs() < 1e-5, /* default focus center must be 0.5 */);
    }

    #[test]
    fn test_blur_inside_band_zero() {
        let t = new_tilt_shift_view();
        let blur = tsh_blur_at_y(&t, 0.5);
        assert!((blur).abs() < 1e-6, /* blur inside focus band must be zero */);
    }

    #[test]
    fn test_blur_outside_band_positive() {
        let t = new_tilt_shift_view();
        let blur = tsh_blur_at_y(&t, 0.0);
        assert!(blur > 0.0, /* blur outside focus band must be positive */);
    }

    #[test]
    fn test_set_focus_center_clamped() {
        let mut t = new_tilt_shift_view();
        tsh_set_focus_center(&mut t, 2.0);
        assert!((t.focus_center_y - 1.0).abs() < 1e-6, /* center clamped to 1.0 */);
    }

    #[test]
    fn test_set_focus_band() {
        let mut t = new_tilt_shift_view();
        tsh_set_focus_band(&mut t, 0.3);
        assert!((t.focus_band_height - 0.3).abs() < 1e-5, /* band height must be set */);
    }

    #[test]
    fn test_set_blur_clamped() {
        let mut t = new_tilt_shift_view();
        tsh_set_blur(&mut t, 1.5);
        assert!((t.blur_amount - 1.0).abs() < 1e-6, /* blur clamped to 1.0 */);
    }

    #[test]
    fn test_set_enabled() {
        let mut t = new_tilt_shift_view();
        tsh_set_enabled(&mut t, false);
        assert!(!t.enabled /* must be disabled */,);
    }

    #[test]
    fn test_to_json_contains_blur() {
        let t = new_tilt_shift_view();
        let j = tsh_to_json(&t);
        assert!(j.contains("\"blur\"") /* json must contain blur */,);
    }

    #[test]
    fn test_enabled_default() {
        let t = new_tilt_shift_view();
        assert!(t.enabled /* must be enabled by default */,);
    }

    #[test]
    fn test_saturation_boost_default() {
        let t = new_tilt_shift_view();
        assert!(t.saturation_boost > 1.0, /* default saturation boost must be > 1 */);
    }
}
