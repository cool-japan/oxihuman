// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! XR/VR viewport stub — simplified stereo projection model.

use std::f32::consts::PI;

#[derive(Debug, Clone, PartialEq)]
pub struct XrViewportV2 {
    pub eye_separation: f32,
    pub fov_deg: f32,
    pub near: f32,
    pub far: f32,
    pub width: u32,
    pub height: u32,
}

pub fn new_xr_viewport_v2(width: u32, height: u32) -> XrViewportV2 {
    XrViewportV2 {
        eye_separation: 0.063,
        fov_deg: 110.0,
        near: 0.01,
        far: 1000.0,
        width,
        height,
    }
}

/// Simple perspective stub — returns a column-major 4x4 matrix.
pub fn xr_projection_matrix_v2(vp: &XrViewportV2, _eye: i32) -> [[f32; 4]; 4] {
    let fov_rad = vp.fov_deg * PI / 180.0;
    let aspect = if vp.height > 0 {
        vp.width as f32 / vp.height as f32
    } else {
        1.0
    };
    let f = 1.0 / (fov_rad * 0.5).tan();
    let range = vp.far - vp.near;
    [
        [f / aspect, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, -(vp.far + vp.near) / range, -1.0],
        [0.0, 0.0, -2.0 * vp.far * vp.near / range, 0.0],
    ]
}

pub fn xr_aspect_ratio_v2(vp: &XrViewportV2) -> f32 {
    if vp.height == 0 {
        return 1.0;
    }
    vp.width as f32 / vp.height as f32
}

pub fn xr_stereo_offset_v2(vp: &XrViewportV2, eye: i32) -> f32 {
    let half = vp.eye_separation * 0.5;
    if eye == 0 {
        -half
    } else {
        half
    }
}

pub fn xr_is_stereo_v2(vp: &XrViewportV2) -> bool {
    vp.eye_separation > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_viewport() {
        /* width/height set */
        let vp = new_xr_viewport_v2(1920, 1080);
        assert_eq!(vp.width, 1920);
        assert_eq!(vp.height, 1080);
    }

    #[test]
    fn test_projection_matrix_is_4x4() {
        /* matrix is 4x4 */
        let vp = new_xr_viewport_v2(1280, 720);
        let m = xr_projection_matrix_v2(&vp, 0);
        assert_eq!(m.len(), 4);
        for col in &m {
            assert_eq!(col.len(), 4);
        }
    }

    #[test]
    fn test_aspect_ratio() {
        /* aspect = w/h */
        let vp = new_xr_viewport_v2(1920, 1080);
        let ar = xr_aspect_ratio_v2(&vp);
        assert!((ar - 16.0 / 9.0).abs() < 1e-4);
    }

    #[test]
    fn test_stereo_offset_left_negative() {
        /* left eye offset is negative */
        let vp = new_xr_viewport_v2(1280, 720);
        assert!(xr_stereo_offset_v2(&vp, 0) < 0.0);
    }

    #[test]
    fn test_stereo_offset_right_positive() {
        /* right eye offset is positive */
        let vp = new_xr_viewport_v2(1280, 720);
        assert!(xr_stereo_offset_v2(&vp, 1) > 0.0);
    }

    #[test]
    fn test_is_stereo() {
        /* default eye_separation > 0 => stereo */
        let vp = new_xr_viewport_v2(1280, 720);
        assert!(xr_is_stereo_v2(&vp));
    }
}
