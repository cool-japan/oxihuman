// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Virtual cinema camera setup stub.

/// Aspect ratio preset.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AspectPreset {
    Wide235,
    Standard185,
    Academy137,
    Square,
}

/// Virtual cinema camera configuration.
#[derive(Debug, Clone)]
pub struct VirtualCameraView {
    pub focal_length_mm: f32,
    pub sensor_width_mm: f32,
    pub aperture_fstop: f32,
    pub aspect: AspectPreset,
    pub enabled: bool,
}

impl VirtualCameraView {
    pub fn new() -> Self {
        VirtualCameraView {
            focal_length_mm: 50.0,
            sensor_width_mm: 36.0,
            aperture_fstop: 2.8,
            aspect: AspectPreset::Wide235,
            enabled: true,
        }
    }
}

impl Default for VirtualCameraView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new virtual camera view.
pub fn new_virtual_camera_view() -> VirtualCameraView {
    VirtualCameraView::new()
}

/// Set focal length in mm.
pub fn vcv_set_focal_length(vcv: &mut VirtualCameraView, mm: f32) {
    vcv.focal_length_mm = mm.max(1.0);
}

/// Set aperture f-stop.
pub fn vcv_set_aperture(vcv: &mut VirtualCameraView, fstop: f32) {
    vcv.aperture_fstop = fstop.max(0.7);
}

/// Compute horizontal FOV in degrees.
pub fn vcv_hfov_deg(vcv: &VirtualCameraView) -> f32 {
    /* Stub: 2 * arctan(sensor_width / (2 * focal_length)) */
    let r = vcv.sensor_width_mm / (2.0 * vcv.focal_length_mm);
    2.0 * r.atan().to_degrees()
}

/// Set aspect ratio preset.
pub fn vcv_set_aspect(vcv: &mut VirtualCameraView, aspect: AspectPreset) {
    vcv.aspect = aspect;
}

/// Enable or disable.
pub fn vcv_set_enabled(vcv: &mut VirtualCameraView, enabled: bool) {
    vcv.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn vcv_to_json(vcv: &VirtualCameraView) -> String {
    format!(
        r#"{{"focal_length_mm":{},"sensor_width_mm":{},"aperture_fstop":{},"enabled":{}}}"#,
        vcv.focal_length_mm, vcv.sensor_width_mm, vcv.aperture_fstop, vcv.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_focal_length() {
        let v = new_virtual_camera_view();
        assert!((v.focal_length_mm - 50.0).abs() < 1e-5, /* default focal length must be 50 */);
    }

    #[test]
    fn test_set_focal_length() {
        let mut v = new_virtual_camera_view();
        vcv_set_focal_length(&mut v, 85.0);
        assert!((v.focal_length_mm - 85.0).abs() < 1e-5, /* focal length must be set */);
    }

    #[test]
    fn test_focal_length_minimum() {
        let mut v = new_virtual_camera_view();
        vcv_set_focal_length(&mut v, -10.0);
        assert!((v.focal_length_mm - 1.0).abs() < 1e-5, /* focal length clamped to 1 */);
    }

    #[test]
    fn test_set_aperture() {
        let mut v = new_virtual_camera_view();
        vcv_set_aperture(&mut v, 5.6);
        assert!((v.aperture_fstop - 5.6).abs() < 1e-5, /* aperture must be set */);
    }

    #[test]
    fn test_hfov_positive() {
        let v = new_virtual_camera_view();
        let hfov = vcv_hfov_deg(&v);
        assert!(hfov > 0.0 /* horizontal FOV must be positive */,);
    }

    #[test]
    fn test_hfov_decreases_with_focal_length() {
        let mut v = new_virtual_camera_view();
        let hfov_50 = vcv_hfov_deg(&v);
        vcv_set_focal_length(&mut v, 100.0);
        let hfov_100 = vcv_hfov_deg(&v);
        assert!(hfov_50 > hfov_100, /* longer focal length must yield narrower FOV */);
    }

    #[test]
    fn test_set_aspect() {
        let mut v = new_virtual_camera_view();
        vcv_set_aspect(&mut v, AspectPreset::Square);
        assert_eq!(v.aspect, AspectPreset::Square /* aspect must be set */,);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_virtual_camera_view();
        vcv_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */,);
    }

    #[test]
    fn test_to_json_contains_focal_length() {
        let v = new_virtual_camera_view();
        let j = vcv_to_json(&v);
        assert!(j.contains("\"focal_length_mm\""), /* json must contain focal_length_mm */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_virtual_camera_view();
        assert!(v.enabled /* must be enabled by default */,);
    }
}
