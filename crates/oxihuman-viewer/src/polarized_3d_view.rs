// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Polarized stereo 3D view stub.

/// Stereo output format.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StereoFormat {
    SideBySide,
    TopBottom,
    FrameSequential,
}

/// Polarized 3D view configuration.
#[derive(Debug, Clone)]
pub struct Polarized3dView {
    pub format: StereoFormat,
    pub eye_separation: f32,
    pub convergence: f32,
    pub swap_eyes: bool,
    pub enabled: bool,
}

impl Polarized3dView {
    pub fn new() -> Self {
        Polarized3dView {
            format: StereoFormat::SideBySide,
            eye_separation: 0.065,
            convergence: 2.0,
            swap_eyes: false,
            enabled: true,
        }
    }
}

impl Default for Polarized3dView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new polarized 3D view.
pub fn new_polarized_3d_view() -> Polarized3dView {
    Polarized3dView::new()
}

/// Set stereo format.
pub fn p3v_set_format(view: &mut Polarized3dView, format: StereoFormat) {
    view.format = format;
}

/// Set eye separation.
pub fn p3v_set_eye_separation(view: &mut Polarized3dView, separation: f32) {
    view.eye_separation = separation.clamp(0.01, 0.5);
}

/// Set convergence distance.
pub fn p3v_set_convergence(view: &mut Polarized3dView, convergence: f32) {
    view.convergence = convergence.clamp(0.1, 100.0);
}

/// Swap left/right eye.
pub fn p3v_set_swap_eyes(view: &mut Polarized3dView, swap: bool) {
    view.swap_eyes = swap;
}

/// Enable or disable.
pub fn p3v_set_enabled(view: &mut Polarized3dView, enabled: bool) {
    view.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn p3v_to_json(view: &Polarized3dView) -> String {
    let fmt = match view.format {
        StereoFormat::SideBySide => "side_by_side",
        StereoFormat::TopBottom => "top_bottom",
        StereoFormat::FrameSequential => "frame_sequential",
    };
    format!(
        r#"{{"format":"{}","eye_separation":{},"convergence":{},"swap_eyes":{},"enabled":{}}}"#,
        fmt, view.eye_separation, view.convergence, view.swap_eyes, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_format() {
        let v = new_polarized_3d_view();
        assert_eq!(
            v.format,
            StereoFormat::SideBySide /* default format must be SideBySide */
        );
    }

    #[test]
    fn test_set_format() {
        let mut v = new_polarized_3d_view();
        p3v_set_format(&mut v, StereoFormat::TopBottom);
        assert_eq!(
            v.format,
            StereoFormat::TopBottom /* format must be set */
        );
    }

    #[test]
    fn test_eye_separation_clamped() {
        let mut v = new_polarized_3d_view();
        p3v_set_eye_separation(&mut v, 1.0);
        assert!((v.eye_separation - 0.5).abs() < 1e-6 /* eye_separation clamped to 0.5 */);
    }

    #[test]
    fn test_convergence_clamped() {
        let mut v = new_polarized_3d_view();
        p3v_set_convergence(&mut v, 0.0);
        assert!((v.convergence - 0.1).abs() < 1e-6 /* convergence clamped to 0.1 */);
    }

    #[test]
    fn test_swap_eyes() {
        let mut v = new_polarized_3d_view();
        p3v_set_swap_eyes(&mut v, true);
        assert!(v.swap_eyes /* swap_eyes must be enabled */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_polarized_3d_view();
        p3v_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_format() {
        let v = new_polarized_3d_view();
        let j = p3v_to_json(&v);
        assert!(j.contains("\"format\"") /* JSON must have format */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_polarized_3d_view();
        assert!(v.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_swap_eyes_default_false() {
        let v = new_polarized_3d_view();
        assert!(!v.swap_eyes /* swap_eyes must default to false */);
    }

    #[test]
    fn test_default_trait() {
        let v = Polarized3dView::default();
        assert!(
            (v.eye_separation - 0.065).abs() < 1e-5 /* Default trait must give 0.065 eye_separation */
        );
    }
}
