// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Dolly zoom (vertigo) effect preview stub.

/// Dolly zoom view configuration.
#[derive(Debug, Clone)]
pub struct DollyZoomView {
    pub subject_distance: f32,
    pub target_fov_deg: f32,
    pub start_fov_deg: f32,
    pub progress: f32,
    pub enabled: bool,
}

impl DollyZoomView {
    pub fn new() -> Self {
        DollyZoomView {
            subject_distance: 5.0,
            target_fov_deg: 15.0,
            start_fov_deg: 60.0,
            progress: 0.0,
            enabled: true,
        }
    }
}

impl Default for DollyZoomView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new dolly zoom view.
pub fn new_dolly_zoom_view() -> DollyZoomView {
    DollyZoomView::new()
}

/// Compute current FOV based on progress (stub: linear interpolation).
pub fn dzv_current_fov(dzv: &DollyZoomView) -> f32 {
    /* Stub: linearly interpolates between start and target FOV */
    dzv.start_fov_deg + (dzv.target_fov_deg - dzv.start_fov_deg) * dzv.progress
}

/// Compute camera distance to maintain constant subject size (stub).
pub fn dzv_camera_distance(dzv: &DollyZoomView) -> f32 {
    /* Stub: returns subject_distance adjusted by progress */
    let fov = dzv_current_fov(dzv);
    dzv.subject_distance * (dzv.start_fov_deg / fov.max(1.0))
}

/// Advance progress.
pub fn dzv_set_progress(dzv: &mut DollyZoomView, progress: f32) {
    dzv.progress = progress.clamp(0.0, 1.0);
}

/// Set target FOV.
pub fn dzv_set_target_fov(dzv: &mut DollyZoomView, fov_deg: f32) {
    dzv.target_fov_deg = fov_deg.clamp(1.0, 179.0);
}

/// Enable or disable.
pub fn dzv_set_enabled(dzv: &mut DollyZoomView, enabled: bool) {
    dzv.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn dzv_to_json(dzv: &DollyZoomView) -> String {
    format!(
        r#"{{"start_fov":{},"target_fov":{},"progress":{},"enabled":{}}}"#,
        dzv.start_fov_deg, dzv.target_fov_deg, dzv.progress, dzv.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_progress_zero() {
        let d = new_dolly_zoom_view();
        assert!((d.progress).abs() < 1e-6, /* default progress must be 0 */);
    }

    #[test]
    fn test_current_fov_at_start() {
        let d = new_dolly_zoom_view();
        let fov = dzv_current_fov(&d);
        assert!((fov - d.start_fov_deg).abs() < 1e-5, /* fov at progress=0 must equal start_fov */);
    }

    #[test]
    fn test_current_fov_at_end() {
        let mut d = new_dolly_zoom_view();
        dzv_set_progress(&mut d, 1.0);
        let fov = dzv_current_fov(&d);
        assert!(
            (fov - d.target_fov_deg).abs() < 1e-5, /* fov at progress=1 must equal target_fov */
        );
    }

    #[test]
    fn test_set_progress_clamped() {
        let mut d = new_dolly_zoom_view();
        dzv_set_progress(&mut d, 2.0);
        assert!((d.progress - 1.0).abs() < 1e-6, /* progress clamped to 1.0 */);
    }

    #[test]
    fn test_set_target_fov() {
        let mut d = new_dolly_zoom_view();
        dzv_set_target_fov(&mut d, 30.0);
        assert!((d.target_fov_deg - 30.0).abs() < 1e-5, /* target FOV must be set */);
    }

    #[test]
    fn test_camera_distance_positive() {
        let d = new_dolly_zoom_view();
        assert!(dzv_camera_distance(&d) > 0.0, /* camera distance must be positive */);
    }

    #[test]
    fn test_set_enabled() {
        let mut d = new_dolly_zoom_view();
        dzv_set_enabled(&mut d, false);
        assert!(!d.enabled /* must be disabled */,);
    }

    #[test]
    fn test_to_json_contains_start_fov() {
        let d = new_dolly_zoom_view();
        let j = dzv_to_json(&d);
        assert!(j.contains("\"start_fov\""), /* json must contain start_fov */);
    }

    #[test]
    fn test_enabled_default() {
        let d = new_dolly_zoom_view();
        assert!(d.enabled /* must be enabled by default */,);
    }

    #[test]
    fn test_target_fov_clamped() {
        let mut d = new_dolly_zoom_view();
        dzv_set_target_fov(&mut d, 0.0);
        assert!((d.target_fov_deg - 1.0).abs() < 1e-5, /* target FOV clamped to 1.0 */);
    }
}
