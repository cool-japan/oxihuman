// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Export camera data to JSON-compatible format.

use std::f32::consts::PI;

/* ── legacy API (kept) ── */

#[derive(Debug, Clone)]
pub struct CameraExport {
    pub name: String,
    pub position: [f32; 3],
    pub target: [f32; 3],
    pub up: [f32; 3],
    pub fov_y: f32,
    pub near: f32,
    pub far: f32,
    pub camera_type: u8,
}

pub fn default_camera_export(name: &str) -> CameraExport {
    CameraExport {
        name: name.to_string(),
        position: [0.0, 0.0, 5.0],
        target: [0.0, 0.0, 0.0],
        up: [0.0, 1.0, 0.0],
        fov_y: PI / 3.0,
        near: 0.1,
        far: 1000.0,
        camera_type: 0,
    }
}

/* ── spec functions (wave 150B) ── */

/// Spec-style camera data.
#[derive(Debug, Clone)]
pub struct CameraData {
    pub name: String,
    pub fov_deg: f32,
    pub near: f32,
    pub far: f32,
    pub orthographic: bool,
    pub ortho_scale: f32,
}

/// Create a new `CameraData`.
pub fn new_camera_data(name: &str, fov_deg: f32) -> CameraData {
    CameraData {
        name: name.to_string(),
        fov_deg,
        near: 0.1,
        far: 1000.0,
        orthographic: false,
        ortho_scale: 1.0,
    }
}

/// Serialize to JSON.
pub fn camera_to_json(c: &CameraData) -> String {
    format!(
        "{{\"name\":\"{}\",\"fov_deg\":{},\"near\":{},\"far\":{},\"ortho\":{}}}",
        c.name, c.fov_deg, c.near, c.far, c.orthographic
    )
}

/// Column-major 4×4 perspective projection matrix (right-handed, OpenGL convention).
pub fn camera_projection_matrix(c: &CameraData) -> [f32; 16] {
    let fov_rad = c.fov_deg * PI / 180.0;
    let f = 1.0 / (fov_rad / 2.0).tan();
    let rng = c.near - c.far;
    [
        f,
        0.0,
        0.0,
        0.0,
        0.0,
        f,
        0.0,
        0.0,
        0.0,
        0.0,
        (c.far + c.near) / rng,
        -1.0,
        0.0,
        0.0,
        (2.0 * c.far * c.near) / rng,
        0.0,
    ]
}

/// Maximum view distance (far plane).
pub fn camera_view_distance(c: &CameraData) -> f32 {
    c.far
}

/// Returns true if the camera is orthographic.
pub fn camera_is_orthographic(c: &CameraData) -> bool {
    c.orthographic
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_camera_data() {
        let c = new_camera_data("cam", 60.0);
        assert_eq!(c.name, "cam");
        assert!((c.fov_deg - 60.0).abs() < 1e-5);
    }

    #[test]
    fn test_camera_to_json() {
        let c = new_camera_data("main", 45.0);
        let j = camera_to_json(&c);
        assert!(j.contains("main"));
    }

    #[test]
    fn test_camera_projection_matrix() {
        let c = new_camera_data("c", 60.0);
        let m = camera_projection_matrix(&c);
        /* [0] = f component should be positive */
        assert!(m[0] > 0.0);
    }

    #[test]
    fn test_camera_view_distance() {
        let c = new_camera_data("c", 60.0);
        assert!((camera_view_distance(&c) - 1000.0).abs() < 1e-5);
    }

    #[test]
    fn test_camera_is_orthographic_false() {
        let c = new_camera_data("c", 60.0);
        assert!(!camera_is_orthographic(&c));
    }
}
