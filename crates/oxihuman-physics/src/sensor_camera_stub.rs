// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Camera pose/intrinsics stub.

/// Camera intrinsic parameters.
#[derive(Debug, Clone)]
pub struct CameraIntrinsics {
    /// Focal length in pixels [fx, fy].
    pub focal_length_px: [f32; 2],
    /// Principal point in pixels [cx, cy].
    pub principal_point_px: [f32; 2],
    /// Radial distortion coefficients [k1, k2, k3].
    pub radial_distortion: [f32; 3],
    /// Tangential distortion coefficients [p1, p2].
    pub tangential_distortion: [f32; 2],
    /// Image resolution [width, height] in pixels.
    pub resolution_px: [u32; 2],
}

impl Default for CameraIntrinsics {
    fn default() -> Self {
        CameraIntrinsics {
            focal_length_px: [800.0, 800.0],
            principal_point_px: [320.0, 240.0],
            radial_distortion: [0.0; 3],
            tangential_distortion: [0.0; 2],
            resolution_px: [640, 480],
        }
    }
}

/// A camera pose (position + orientation as rotation matrix rows).
#[derive(Debug, Clone)]
pub struct CameraPose {
    /// Camera position in world frame [x, y, z].
    pub position: [f32; 3],
    /// Rotation matrix (row-major, 9 elements).
    pub rotation: [f32; 9],
}

impl Default for CameraPose {
    fn default() -> Self {
        CameraPose {
            position: [0.0; 3],
            rotation: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0], /* identity */
        }
    }
}

/// A camera sensor stub.
#[derive(Debug, Clone)]
pub struct CameraStub {
    pub intrinsics: CameraIntrinsics,
    pub pose: CameraPose,
    pub frame_rate_hz: f32,
}

impl CameraStub {
    /// Create a new camera stub with default parameters.
    pub fn new() -> Self {
        CameraStub {
            intrinsics: CameraIntrinsics::default(),
            pose: CameraPose::default(),
            frame_rate_hz: 30.0,
        }
    }
}

impl Default for CameraStub {
    fn default() -> Self {
        Self::new()
    }
}

/// Project a 3-D world point to image coordinates (pinhole model, no distortion).
pub fn project_point(
    point_world: [f32; 3],
    pose: &CameraPose,
    intrinsics: &CameraIntrinsics,
) -> Option<[f32; 2]> {
    /* transform to camera frame */
    let r = &pose.rotation;
    let t = pose.position;
    let p = point_world;
    let xc = r[0] * (p[0] - t[0]) + r[1] * (p[1] - t[1]) + r[2] * (p[2] - t[2]);
    let yc = r[3] * (p[0] - t[0]) + r[4] * (p[1] - t[1]) + r[5] * (p[2] - t[2]);
    let zc = r[6] * (p[0] - t[0]) + r[7] * (p[1] - t[1]) + r[8] * (p[2] - t[2]);
    if zc <= 0.0 {
        return None;
    }
    let u = intrinsics.focal_length_px[0] * xc / zc + intrinsics.principal_point_px[0];
    let v = intrinsics.focal_length_px[1] * yc / zc + intrinsics.principal_point_px[1];
    Some([u, v])
}

/// Return `true` if a pixel coordinate is within the image bounds.
pub fn pixel_in_bounds(uv: [f32; 2], intrinsics: &CameraIntrinsics) -> bool {
    uv[0] >= 0.0
        && uv[1] >= 0.0
        && uv[0] < intrinsics.resolution_px[0] as f32
        && uv[1] < intrinsics.resolution_px[1] as f32
}

/// Compute the field of view in degrees for a given axis.
pub fn fov_deg(focal_length_px: f32, sensor_size_px: u32) -> f32 {
    2.0 * (sensor_size_px as f32 / (2.0 * focal_length_px))
        .atan()
        .to_degrees()
}

/// Compute the back-projected ray direction for a pixel.
pub fn backproject_ray(uv: [f32; 2], intrinsics: &CameraIntrinsics) -> [f32; 3] {
    let xn = (uv[0] - intrinsics.principal_point_px[0]) / intrinsics.focal_length_px[0];
    let yn = (uv[1] - intrinsics.principal_point_px[1]) / intrinsics.focal_length_px[1];
    let norm = (xn * xn + yn * yn + 1.0).sqrt();
    [xn / norm, yn / norm, 1.0 / norm]
}

/// Return the pixel area in metres² given a depth and intrinsics.
pub fn pixel_area_at_depth(depth_m: f32, intrinsics: &CameraIntrinsics) -> f32 {
    let fx = intrinsics.focal_length_px[0];
    let fy = intrinsics.focal_length_px[1];
    (depth_m / fx) * (depth_m / fy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_along_z() {
        /* point along z-axis projects to principal point */
        let pose = CameraPose::default();
        let intrinsics = CameraIntrinsics::default();
        let uv = project_point([0.0, 0.0, 1.0], &pose, &intrinsics).unwrap();
        assert!((uv[0] - 320.0).abs() < 1e-3);
        assert!((uv[1] - 240.0).abs() < 1e-3);
    }

    #[test]
    fn test_project_behind_camera_none() {
        /* point behind camera returns None */
        let pose = CameraPose::default();
        let intrinsics = CameraIntrinsics::default();
        assert!(project_point([0.0, 0.0, -1.0], &pose, &intrinsics).is_none());
    }

    #[test]
    fn test_pixel_in_bounds_centre() {
        /* centre pixel is in bounds */
        let intrinsics = CameraIntrinsics::default();
        assert!(pixel_in_bounds([320.0, 240.0], &intrinsics));
    }

    #[test]
    fn test_pixel_out_of_bounds() {
        /* pixel at 1000 x is out of bounds */
        let intrinsics = CameraIntrinsics::default();
        assert!(!pixel_in_bounds([1000.0, 240.0], &intrinsics));
    }

    #[test]
    fn test_fov_deg_positive() {
        /* FoV is positive */
        let fov = fov_deg(800.0, 640);
        assert!(fov > 0.0 && fov < 180.0);
    }

    #[test]
    fn test_backproject_ray_centre() {
        /* centre pixel back-projects along z */
        let intrinsics = CameraIntrinsics::default();
        let ray = backproject_ray(intrinsics.principal_point_px, &intrinsics);
        assert!(ray[2] > 0.0); /* forward direction */
    }

    #[test]
    fn test_pixel_area_positive() {
        /* pixel area at 1 m is positive */
        let intrinsics = CameraIntrinsics::default();
        assert!(pixel_area_at_depth(1.0, &intrinsics) > 0.0);
    }

    #[test]
    fn test_camera_stub_default() {
        /* default camera has 30 fps */
        let cam = CameraStub::default();
        assert_eq!(cam.frame_rate_hz, 30.0);
    }

    #[test]
    fn test_default_resolution() {
        /* default resolution is 640×480 */
        let c = CameraIntrinsics::default();
        assert_eq!(c.resolution_px, [640, 480]);
    }

    #[test]
    fn test_identity_rotation_projects_correctly() {
        /* identity rotation matrix used by default */
        let pose = CameraPose::default();
        let sum: f32 = [0, 4, 8].iter().map(|&i| pose.rotation[i]).sum();
        assert!((sum - 3.0).abs() < 1e-6); /* diagonal is all 1s */
    }
}
