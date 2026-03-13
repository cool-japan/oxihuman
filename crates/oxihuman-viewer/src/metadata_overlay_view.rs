// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Camera metadata overlay view for production monitoring.

/// Metadata overlay view configuration.
#[derive(Debug, Clone)]
pub struct MetadataOverlayView {
    pub show_focal_length: bool,
    pub show_aperture: bool,
    pub show_iso: bool,
    pub show_shutter: bool,
    pub focal_length_mm: f32,
    pub aperture_fstop: f32,
    pub iso: u32,
    pub shutter_speed: f32,
    pub text_scale: f32,
}

impl MetadataOverlayView {
    pub fn new() -> Self {
        Self {
            show_focal_length: true,
            show_aperture: true,
            show_iso: true,
            show_shutter: true,
            focal_length_mm: 50.0,
            aperture_fstop: 2.8,
            iso: 400,
            shutter_speed: 0.02,
            text_scale: 1.0,
        }
    }
}

impl Default for MetadataOverlayView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new metadata overlay view.
pub fn new_metadata_overlay_view() -> MetadataOverlayView {
    MetadataOverlayView::new()
}

/// Set focal length in mm.
pub fn mov_set_focal_length(view: &mut MetadataOverlayView, mm: f32) {
    view.focal_length_mm = mm.clamp(1.0, 2000.0);
}

/// Set aperture f-stop.
pub fn mov_set_aperture(view: &mut MetadataOverlayView, fstop: f32) {
    view.aperture_fstop = fstop.clamp(0.7, 64.0);
}

/// Set ISO sensitivity.
pub fn mov_set_iso(view: &mut MetadataOverlayView, iso: u32) {
    view.iso = iso.clamp(50, 409_600);
}

/// Set shutter speed in seconds.
pub fn mov_set_shutter(view: &mut MetadataOverlayView, seconds: f32) {
    view.shutter_speed = seconds.clamp(1e-6, 30.0);
}

/// Compute exposure value (EV) from the current settings.
pub fn mov_exposure_value(view: &MetadataOverlayView) -> f32 {
    let n = view.aperture_fstop;
    let t = view.shutter_speed;
    if t.abs() < 1e-9 {
        return 0.0;
    }
    (n * n / t).log2()
}

/// Serialize to JSON-like string.
pub fn metadata_overlay_view_to_json(view: &MetadataOverlayView) -> String {
    format!(
        r#"{{"focal_length_mm":{:.1},"aperture_fstop":{:.2},"iso":{},"shutter_speed":{:.6}}}"#,
        view.focal_length_mm, view.aperture_fstop, view.iso, view.shutter_speed
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_metadata_overlay_view();
        assert!((v.focal_length_mm - 50.0).abs() < 1e-6);
        assert!(v.show_focal_length);
    }

    #[test]
    fn test_focal_length_clamp() {
        let mut v = new_metadata_overlay_view();
        mov_set_focal_length(&mut v, 0.0);
        assert!((v.focal_length_mm - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_aperture_set() {
        let mut v = new_metadata_overlay_view();
        mov_set_aperture(&mut v, 5.6);
        assert!((v.aperture_fstop - 5.6).abs() < 1e-5);
    }

    #[test]
    fn test_iso_set() {
        let mut v = new_metadata_overlay_view();
        mov_set_iso(&mut v, 1600);
        assert_eq!(v.iso, 1600);
    }

    #[test]
    fn test_shutter_speed_set() {
        let mut v = new_metadata_overlay_view();
        mov_set_shutter(&mut v, 0.004);
        assert!((v.shutter_speed - 0.004).abs() < 1e-6);
    }

    #[test]
    fn test_exposure_value_finite() {
        let v = new_metadata_overlay_view();
        let ev = mov_exposure_value(&v);
        assert!(ev.is_finite());
    }

    #[test]
    fn test_json() {
        let v = new_metadata_overlay_view();
        let s = metadata_overlay_view_to_json(&v);
        assert!(s.contains("focal_length_mm"));
    }

    #[test]
    fn test_clone() {
        let v = new_metadata_overlay_view();
        let v2 = v.clone();
        assert_eq!(v2.iso, v.iso);
    }

    #[test]
    fn test_default_trait() {
        let v: MetadataOverlayView = Default::default();
        assert!((v.text_scale - 1.0).abs() < 1e-6);
    }
}
