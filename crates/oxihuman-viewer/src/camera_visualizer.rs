// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Camera frustum visualization.

#![allow(dead_code)]

/// Configuration for camera visualization.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraVisConfig {
    pub color: [f32; 4],
    pub near_plane_color: [f32; 4],
    pub show_dof: bool,
}

/// Runtime state for camera visualization.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraVisState {
    pub fov_deg: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
    pub config: CameraVisConfig,
}

#[allow(dead_code)]
pub fn default_camera_vis_config() -> CameraVisConfig {
    CameraVisConfig {
        color: [0.3, 0.8, 1.0, 1.0],
        near_plane_color: [1.0, 1.0, 0.0, 1.0],
        show_dof: false,
    }
}

#[allow(dead_code)]
pub fn new_camera_vis_state() -> CameraVisState {
    CameraVisState {
        fov_deg: 60.0,
        aspect: 16.0 / 9.0,
        near: 0.1,
        far: 100.0,
        config: default_camera_vis_config(),
    }
}

#[allow(dead_code)]
pub fn cv_set_fov(state: &mut CameraVisState, fov_deg: f32) {
    state.fov_deg = fov_deg.clamp(1.0, 179.0);
}

#[allow(dead_code)]
pub fn cv_set_near_far(state: &mut CameraVisState, near: f32, far: f32) {
    state.near = near.max(0.001);
    state.far = far.max(state.near + 0.001);
}

#[allow(dead_code)]
pub fn cv_frustum_corners(state: &CameraVisState) -> Vec<[f32; 3]> {
    let half_v_near = (state.fov_deg.to_radians() * 0.5).tan() * state.near;
    let half_h_near = half_v_near * state.aspect;
    let half_v_far = (state.fov_deg.to_radians() * 0.5).tan() * state.far;
    let half_h_far = half_v_far * state.aspect;

    vec![
        [-half_h_near, -half_v_near, -state.near],
        [half_h_near, -half_v_near, -state.near],
        [half_h_near, half_v_near, -state.near],
        [-half_h_near, half_v_near, -state.near],
        [-half_h_far, -half_v_far, -state.far],
        [half_h_far, -half_v_far, -state.far],
        [half_h_far, half_v_far, -state.far],
        [-half_h_far, half_v_far, -state.far],
    ]
}

#[allow(dead_code)]
pub fn cv_to_json(state: &CameraVisState) -> String {
    let c = &state.config.color;
    format!(
        r#"{{"fov_deg":{:.4},"aspect":{:.4},"near":{:.4},"far":{:.4},"color":[{:.4},{:.4},{:.4},{:.4}],"show_dof":{}}}"#,
        state.fov_deg, state.aspect, state.near, state.far,
        c[0], c[1], c[2], c[3],
        state.config.show_dof
    )
}

#[allow(dead_code)]
pub fn cv_reset(state: &mut CameraVisState) {
    *state = new_camera_vis_state();
}

#[allow(dead_code)]
pub fn cv_is_orthographic(state: &CameraVisState) -> bool {
    state.fov_deg < 1e-3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_camera_vis_config();
        assert!(!cfg.show_dof);
    }

    #[test]
    fn test_new_state_defaults() {
        let s = new_camera_vis_state();
        assert!((s.fov_deg - 60.0).abs() < 1e-6);
        assert!((s.near - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_set_fov_clamps() {
        let mut s = new_camera_vis_state();
        cv_set_fov(&mut s, 200.0);
        assert!((s.fov_deg - 179.0).abs() < 1e-6);
        cv_set_fov(&mut s, 0.0);
        assert!((s.fov_deg - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_near_far() {
        let mut s = new_camera_vis_state();
        cv_set_near_far(&mut s, 0.5, 200.0);
        assert!((s.near - 0.5).abs() < 1e-6);
        assert!((s.far - 200.0).abs() < 1e-6);
    }

    #[test]
    fn test_frustum_corners_count() {
        let s = new_camera_vis_state();
        let corners = cv_frustum_corners(&s);
        assert_eq!(corners.len(), 8);
    }

    #[test]
    fn test_frustum_near_z() {
        let s = new_camera_vis_state();
        let corners = cv_frustum_corners(&s);
        assert!((corners[0][2] + s.near).abs() < 1e-4);
    }

    #[test]
    fn test_is_orthographic_false() {
        let s = new_camera_vis_state();
        assert!(!cv_is_orthographic(&s));
    }

    #[test]
    fn test_to_json_contains_fields() {
        let s = new_camera_vis_state();
        let j = cv_to_json(&s);
        assert!(j.contains("fov_deg"));
        assert!(j.contains("near"));
        assert!(j.contains("far"));
    }

    #[test]
    fn test_reset() {
        let mut s = new_camera_vis_state();
        cv_set_fov(&mut s, 90.0);
        cv_reset(&mut s);
        assert!((s.fov_deg - 60.0).abs() < 1e-6);
    }
}
