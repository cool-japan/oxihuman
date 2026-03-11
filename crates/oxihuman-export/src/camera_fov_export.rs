// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Camera field-of-view export for scene cameras.

use std::f32::consts::PI;

/// Camera FOV keyframe.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct FovKeyframe {
    pub time: f32,
    pub fov_degrees: f32,
}

/// Camera FOV export data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraFovExport {
    pub camera_name: String,
    pub keyframes: Vec<FovKeyframe>,
}

/// Create new camera FOV export.
#[allow(dead_code)]
pub fn new_camera_fov_export(name: &str) -> CameraFovExport {
    CameraFovExport {
        camera_name: name.to_string(),
        keyframes: vec![],
    }
}

/// Add keyframe.
#[allow(dead_code)]
pub fn add_fov_keyframe(e: &mut CameraFovExport, time: f32, fov: f32) {
    e.keyframes.push(FovKeyframe {
        time,
        fov_degrees: fov,
    });
}

/// Keyframe count.
#[allow(dead_code)]
pub fn fov_keyframe_count(e: &CameraFovExport) -> usize {
    e.keyframes.len()
}

/// Convert FOV degrees to radians.
#[allow(dead_code)]
pub fn fov_to_radians(fov_deg: f32) -> f32 {
    fov_deg * PI / 180.0
}

/// Convert FOV to focal length (given sensor width).
#[allow(dead_code)]
pub fn fov_to_focal_length(fov_deg: f32, sensor_width: f32) -> f32 {
    sensor_width / (2.0 * (fov_to_radians(fov_deg) / 2.0).tan())
}

/// Duration.
#[allow(dead_code)]
pub fn fov_duration(e: &CameraFovExport) -> f32 {
    if e.keyframes.is_empty() {
        return 0.0;
    }
    let min = e.keyframes.iter().map(|k| k.time).fold(f32::MAX, f32::min);
    let max = e.keyframes.iter().map(|k| k.time).fold(f32::MIN, f32::max);
    max - min
}

/// Validate.
#[allow(dead_code)]
pub fn fov_validate(e: &CameraFovExport) -> bool {
    e.keyframes
        .iter()
        .all(|k| k.fov_degrees > 0.0 && k.fov_degrees < 180.0 && k.time >= 0.0)
}

/// Export to JSON.
#[allow(dead_code)]
pub fn camera_fov_to_json(e: &CameraFovExport) -> String {
    format!(
        "{{\"camera\":\"{}\",\"keyframes\":{},\"duration\":{:.6}}}",
        e.camera_name,
        fov_keyframe_count(e),
        fov_duration(e)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new() {
        let e = new_camera_fov_export("cam1");
        assert_eq!(e.camera_name, "cam1");
    }
    #[test]
    fn test_add() {
        let mut e = new_camera_fov_export("c");
        add_fov_keyframe(&mut e, 0.0, 60.0);
        assert_eq!(fov_keyframe_count(&e), 1);
    }
    #[test]
    fn test_to_radians() {
        assert!((fov_to_radians(90.0) - PI / 2.0).abs() < 1e-5);
    }
    #[test]
    fn test_focal_length() {
        let fl = fov_to_focal_length(90.0, 36.0);
        assert!(fl > 0.0);
    }
    #[test]
    fn test_duration() {
        let mut e = new_camera_fov_export("c");
        add_fov_keyframe(&mut e, 0.0, 60.0);
        add_fov_keyframe(&mut e, 2.0, 90.0);
        assert!((fov_duration(&e) - 2.0).abs() < 1e-6);
    }
    #[test]
    fn test_duration_empty() {
        let e = new_camera_fov_export("c");
        assert!((fov_duration(&e)).abs() < 1e-9);
    }
    #[test]
    fn test_validate() {
        let mut e = new_camera_fov_export("c");
        add_fov_keyframe(&mut e, 0.0, 60.0);
        assert!(fov_validate(&e));
    }
    #[test]
    fn test_validate_bad() {
        let mut e = new_camera_fov_export("c");
        add_fov_keyframe(&mut e, 0.0, 200.0);
        assert!(!fov_validate(&e));
    }
    #[test]
    fn test_to_json() {
        let e = new_camera_fov_export("cam");
        let j = camera_fov_to_json(&e);
        assert!(j.contains("\"camera\":\"cam\""));
    }
    #[test]
    fn test_focal_known() {
        let fl = fov_to_focal_length(53.13, 36.0);
        assert!(fl > 15.0 && fl < 40.0);
    }
}
