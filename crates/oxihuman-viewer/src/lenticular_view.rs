// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Lenticular interlace view stub.

/// Lenticular interlace axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterlaceAxis {
    Horizontal,
    Vertical,
}

/// Lenticular view configuration.
#[derive(Debug, Clone)]
pub struct LenticularView {
    pub axis: InterlaceAxis,
    pub lpi: f32,
    pub num_frames: u32,
    pub current_angle: f32,
    pub enabled: bool,
}

impl LenticularView {
    pub fn new() -> Self {
        LenticularView {
            axis: InterlaceAxis::Vertical,
            lpi: 40.0,
            num_frames: 3,
            current_angle: 0.0,
            enabled: true,
        }
    }
}

impl Default for LenticularView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new lenticular view.
pub fn new_lenticular_view() -> LenticularView {
    LenticularView::new()
}

/// Set interlace axis.
pub fn ltv_set_axis(view: &mut LenticularView, axis: InterlaceAxis) {
    view.axis = axis;
}

/// Set lines per inch.
pub fn ltv_set_lpi(view: &mut LenticularView, lpi: f32) {
    view.lpi = lpi.clamp(10.0, 200.0);
}

/// Set number of frames to interlace.
pub fn ltv_set_num_frames(view: &mut LenticularView, frames: u32) {
    view.num_frames = frames.clamp(2, 32);
}

/// Set current viewing angle (for animation).
pub fn ltv_set_angle(view: &mut LenticularView, angle: f32) {
    view.current_angle = angle % 360.0;
}

/// Enable or disable.
pub fn ltv_set_enabled(view: &mut LenticularView, enabled: bool) {
    view.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn ltv_to_json(view: &LenticularView) -> String {
    let axis = match view.axis {
        InterlaceAxis::Horizontal => "horizontal",
        InterlaceAxis::Vertical => "vertical",
    };
    format!(
        r#"{{"axis":"{}","lpi":{},"num_frames":{},"current_angle":{},"enabled":{}}}"#,
        axis, view.lpi, view.num_frames, view.current_angle, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_axis() {
        let v = new_lenticular_view();
        assert_eq!(
            v.axis,
            InterlaceAxis::Vertical /* default axis must be Vertical */
        );
    }

    #[test]
    fn test_set_axis() {
        let mut v = new_lenticular_view();
        ltv_set_axis(&mut v, InterlaceAxis::Horizontal);
        assert_eq!(
            v.axis,
            InterlaceAxis::Horizontal /* axis must be set */
        );
    }

    #[test]
    fn test_lpi_clamped_min() {
        let mut v = new_lenticular_view();
        ltv_set_lpi(&mut v, 0.0);
        assert!((v.lpi - 10.0).abs() < 1e-6 /* lpi clamped to 10.0 */);
    }

    #[test]
    fn test_lpi_clamped_max() {
        let mut v = new_lenticular_view();
        ltv_set_lpi(&mut v, 500.0);
        assert!((v.lpi - 200.0).abs() < 1e-6 /* lpi clamped to 200.0 */);
    }

    #[test]
    fn test_num_frames_clamped() {
        let mut v = new_lenticular_view();
        ltv_set_num_frames(&mut v, 1);
        assert_eq!(v.num_frames, 2 /* num_frames clamped to minimum 2 */);
    }

    #[test]
    fn test_angle_wraps() {
        let mut v = new_lenticular_view();
        ltv_set_angle(&mut v, 370.0);
        assert!((v.current_angle - 10.0).abs() < 1e-4 /* angle must wrap at 360 */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_lenticular_view();
        ltv_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_axis() {
        let v = new_lenticular_view();
        let j = ltv_to_json(&v);
        assert!(j.contains("\"axis\"") /* JSON must have axis */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_lenticular_view();
        assert!(v.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_default_lpi() {
        let v = new_lenticular_view();
        assert!((v.lpi - 40.0).abs() < 1e-6 /* default lpi must be 40.0 */);
    }
}
