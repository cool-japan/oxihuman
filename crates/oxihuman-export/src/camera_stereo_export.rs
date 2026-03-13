// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Stereo camera rig export (left/right eye separation).

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StereoMode {
    SideBySide,
    Anaglyph,
    TopBottom,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraStereoExport {
    pub mode: StereoMode,
    pub eye_separation: f32,
    pub convergence_distance: f32,
    pub fov_degrees: f32,
}

#[allow(dead_code)]
pub fn default_stereo_camera() -> CameraStereoExport {
    CameraStereoExport {
        mode: StereoMode::SideBySide,
        eye_separation: 0.065,
        convergence_distance: 2.0,
        fov_degrees: 60.0,
    }
}

#[allow(dead_code)]
pub fn stereo_fov_radians(cam: &CameraStereoExport) -> f32 {
    cam.fov_degrees * PI / 180.0
}

#[allow(dead_code)]
pub fn stereo_left_offset(cam: &CameraStereoExport) -> [f32; 3] {
    [-cam.eye_separation * 0.5, 0.0, 0.0]
}

#[allow(dead_code)]
pub fn stereo_right_offset(cam: &CameraStereoExport) -> [f32; 3] {
    [cam.eye_separation * 0.5, 0.0, 0.0]
}

#[allow(dead_code)]
pub fn stereo_mode_name(cam: &CameraStereoExport) -> &'static str {
    match cam.mode {
        StereoMode::SideBySide => "side_by_side",
        StereoMode::Anaglyph => "anaglyph",
        StereoMode::TopBottom => "top_bottom",
    }
}

#[allow(dead_code)]
pub fn validate_stereo(cam: &CameraStereoExport) -> bool {
    cam.eye_separation > 0.0
        && cam.convergence_distance > 0.0
        && (1.0..=180.0).contains(&cam.fov_degrees)
}

#[allow(dead_code)]
pub fn camera_stereo_to_json(cam: &CameraStereoExport) -> String {
    format!(
        "{{\"mode\":\"{}\",\"eye_sep\":{},\"fov\":{}}}",
        stereo_mode_name(cam),
        cam.eye_separation,
        cam.fov_degrees,
    )
}

#[allow(dead_code)]
pub fn parallax_angle_deg(cam: &CameraStereoExport) -> f32 {
    if cam.convergence_distance == 0.0 {
        return 0.0;
    }
    let half_sep = cam.eye_separation * 0.5;
    (half_sep / cam.convergence_distance).atan() * 180.0 / PI
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_camera() {
        let cam = default_stereo_camera();
        assert!(cam.eye_separation > 0.0);
    }

    #[test]
    fn test_fov_radians() {
        let cam = default_stereo_camera();
        let r = stereo_fov_radians(&cam);
        assert!((r - PI / 3.0).abs() < 1e-4);
    }

    #[test]
    fn test_left_offset_negative() {
        let cam = default_stereo_camera();
        let off = stereo_left_offset(&cam);
        assert!(off[0] < 0.0);
    }

    #[test]
    fn test_right_offset_positive() {
        let cam = default_stereo_camera();
        let off = stereo_right_offset(&cam);
        assert!(off[0] > 0.0);
    }

    #[test]
    fn test_mode_name() {
        let cam = default_stereo_camera();
        assert_eq!(stereo_mode_name(&cam), "side_by_side");
    }

    #[test]
    fn test_validate_default() {
        let cam = default_stereo_camera();
        assert!(validate_stereo(&cam));
    }

    #[test]
    fn test_json_output() {
        let cam = default_stereo_camera();
        let j = camera_stereo_to_json(&cam);
        assert!(j.contains("fov"));
    }

    #[test]
    fn test_parallax_angle_positive() {
        let cam = default_stereo_camera();
        let angle = parallax_angle_deg(&cam);
        assert!(angle > 0.0);
    }

    #[test]
    fn test_anaglyph_mode() {
        let mut cam = default_stereo_camera();
        cam.mode = StereoMode::Anaglyph;
        assert_eq!(stereo_mode_name(&cam), "anaglyph");
    }

    #[test]
    fn test_invalid_fov() {
        let mut cam = default_stereo_camera();
        cam.fov_degrees = 0.0;
        assert!(!validate_stereo(&cam));
    }
}
