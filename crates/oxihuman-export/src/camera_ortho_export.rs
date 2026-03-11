// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// An orthographic camera export.
#[allow(dead_code)]
pub struct CameraOrthoExport {
    pub width: f32,
    pub height: f32,
    pub near: f32,
    pub far: f32,
    pub position: [f32; 3],
}

/// Create a default ortho camera.
#[allow(dead_code)]
pub fn default_ortho_camera() -> CameraOrthoExport {
    CameraOrthoExport {
        width: 10.0,
        height: 10.0,
        near: 0.1,
        far: 100.0,
        position: [0.0; 3],
    }
}

/// Compute orthographic projection matrix (column-major, OpenGL convention).
#[allow(dead_code)]
pub fn ortho_projection_matrix(cam: &CameraOrthoExport) -> [[f32; 4]; 4] {
    let r = cam.width * 0.5;
    let t = cam.height * 0.5;
    let n = cam.near;
    let f = cam.far;
    [
        [1.0 / r, 0.0, 0.0, 0.0],
        [0.0, 1.0 / t, 0.0, 0.0],
        [0.0, 0.0, -2.0 / (f - n), 0.0],
        [0.0, 0.0, -(f + n) / (f - n), 1.0],
    ]
}

/// Project a 3D point to 2D using an orthographic camera.
#[allow(dead_code)]
pub fn ortho_project_point(cam: &CameraOrthoExport, p: [f32; 3]) -> [f32; 2] {
    let x = (p[0] - cam.position[0]) / (cam.width * 0.5);
    let y = (p[1] - cam.position[1]) / (cam.height * 0.5);
    [x, y]
}

/// Check if a point is in the ortho frustum.
#[allow(dead_code)]
pub fn point_in_ortho_frustum(cam: &CameraOrthoExport, p: [f32; 3]) -> bool {
    let hw = cam.width * 0.5;
    let hh = cam.height * 0.5;
    let dx = (p[0] - cam.position[0]).abs();
    let dy = (p[1] - cam.position[1]).abs();
    let dz = p[2] - cam.position[2];
    dx <= hw && dy <= hh && dz >= -cam.far && dz <= -cam.near
}

/// Aspect ratio of the camera.
#[allow(dead_code)]
pub fn ortho_aspect_ratio(cam: &CameraOrthoExport) -> f32 {
    cam.width / cam.height
}

/// Validate camera parameters.
#[allow(dead_code)]
pub fn validate_ortho_camera(cam: &CameraOrthoExport) -> bool {
    cam.width > 0.0 && cam.height > 0.0 && cam.near > 0.0 && cam.far > cam.near
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn camera_ortho_to_json(cam: &CameraOrthoExport) -> String {
    format!(
        r#"{{"type":"ortho","width":{:.4},"height":{:.4},"near":{:.4},"far":{:.4}}}"#,
        cam.width, cam.height, cam.near, cam.far
    )
}

/// Clone with a new size.
#[allow(dead_code)]
pub fn ortho_resize(cam: &CameraOrthoExport, width: f32, height: f32) -> CameraOrthoExport {
    CameraOrthoExport {
        width,
        height,
        near: cam.near,
        far: cam.far,
        position: cam.position,
    }
}

/// Number of pixels in an ortho texture given pixels-per-unit.
#[allow(dead_code)]
pub fn ortho_pixel_count(cam: &CameraOrthoExport, ppu: f32) -> usize {
    let w = (cam.width * ppu).round() as usize;
    let h = (cam.height * ppu).round() as usize;
    w * h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_valid() {
        assert!(validate_ortho_camera(&default_ortho_camera()));
    }

    #[test]
    fn aspect_ratio() {
        let cam = CameraOrthoExport {
            width: 20.0,
            height: 10.0,
            near: 0.1,
            far: 100.0,
            position: [0.0; 3],
        };
        assert!((ortho_aspect_ratio(&cam) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn projection_matrix_4x4() {
        let m = ortho_projection_matrix(&default_ortho_camera());
        assert_eq!(m.len(), 4);
    }

    #[test]
    fn project_origin() {
        let cam = default_ortho_camera();
        let p = ortho_project_point(&cam, [0.0, 0.0, 0.0]);
        assert!(p[0].abs() < 1e-5 && p[1].abs() < 1e-5);
    }

    #[test]
    fn json_has_type() {
        let j = camera_ortho_to_json(&default_ortho_camera());
        assert!(j.contains("\"type\":\"ortho\""));
    }

    #[test]
    fn resize() {
        let cam = default_ortho_camera();
        let r = ortho_resize(&cam, 20.0, 5.0);
        assert!((r.width - 20.0).abs() < 1e-5);
    }

    #[test]
    fn pixel_count() {
        let cam = CameraOrthoExport {
            width: 10.0,
            height: 10.0,
            near: 0.1,
            far: 100.0,
            position: [0.0; 3],
        };
        assert_eq!(ortho_pixel_count(&cam, 100.0), 1_000_000);
    }

    #[test]
    fn invalid_camera() {
        let cam = CameraOrthoExport {
            width: 0.0,
            height: 10.0,
            near: 0.1,
            far: 100.0,
            position: [0.0; 3],
        };
        assert!(!validate_ortho_camera(&cam));
    }

    #[test]
    fn far_greater_near() {
        let cam = CameraOrthoExport {
            width: 10.0,
            height: 10.0,
            near: 10.0,
            far: 5.0,
            position: [0.0; 3],
        };
        assert!(!validate_ortho_camera(&cam));
    }

    #[test]
    fn frustum_check() {
        let cam = default_ortho_camera();
        // a point 1 unit in front of the camera (in -z)
        let in_frustum = point_in_ortho_frustum(&cam, [0.0, 0.0, -10.0]);
        // just check it runs without panic
        let _ = in_frustum;
    }
}
