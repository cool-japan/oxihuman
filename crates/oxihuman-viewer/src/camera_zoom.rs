// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Camera zoom controls with smooth interpolation.

use std::f32::consts::E;

/// Camera zoom state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraZoomState {
    pub current: f32,
    pub target: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub speed: f32,
}

#[allow(dead_code)]
pub fn default_camera_zoom() -> CameraZoomState {
    CameraZoomState { current: 1.0, target: 1.0, min_zoom: 0.1, max_zoom: 10.0, speed: 5.0 }
}

#[allow(dead_code)]
pub fn set_zoom_target(state: &mut CameraZoomState, target: f32) {
    state.target = target.clamp(state.min_zoom, state.max_zoom);
}

#[allow(dead_code)]
pub fn zoom_in(state: &mut CameraZoomState, amount: f32) {
    let new_target = state.target + amount;
    state.target = new_target.clamp(state.min_zoom, state.max_zoom);
}

#[allow(dead_code)]
pub fn zoom_out(state: &mut CameraZoomState, amount: f32) {
    let new_target = state.target - amount;
    state.target = new_target.clamp(state.min_zoom, state.max_zoom);
}

#[allow(dead_code)]
pub fn update_zoom(state: &mut CameraZoomState, dt: f32) {
    let diff = state.target - state.current;
    let decay = (-state.speed * dt).exp();
    state.current = state.target - diff * decay;
    state.current = state.current.clamp(state.min_zoom, state.max_zoom);
}

#[allow(dead_code)]
pub fn reset_zoom(state: &mut CameraZoomState) {
    state.current = 1.0;
    state.target = 1.0;
}

#[allow(dead_code)]
pub fn zoom_fraction(state: &CameraZoomState) -> f32 {
    if (state.max_zoom - state.min_zoom).abs() < 1e-9 {
        return 0.0;
    }
    (state.current - state.min_zoom) / (state.max_zoom - state.min_zoom)
}

#[allow(dead_code)]
pub fn is_zooming(state: &CameraZoomState) -> bool {
    (state.current - state.target).abs() > 0.001
}

#[allow(dead_code)]
pub fn fov_from_zoom(zoom: f32, base_fov: f32) -> f32 {
    let _ = E; // use E constant
    (base_fov / zoom).clamp(1.0, 179.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_zoom() {
        let z = default_camera_zoom();
        assert!((z.current - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_target_clamp() {
        let mut z = default_camera_zoom();
        set_zoom_target(&mut z, 100.0);
        assert!((z.target - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_zoom_in() {
        let mut z = default_camera_zoom();
        zoom_in(&mut z, 0.5);
        assert!((z.target - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_zoom_out() {
        let mut z = default_camera_zoom();
        zoom_out(&mut z, 0.5);
        assert!((z.target - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_update_zoom() {
        let mut z = default_camera_zoom();
        z.target = 2.0;
        update_zoom(&mut z, 0.1);
        assert!(z.current > 1.0);
        assert!(z.current < 2.0);
    }

    #[test]
    fn test_reset_zoom() {
        let mut z = default_camera_zoom();
        z.current = 5.0;
        z.target = 5.0;
        reset_zoom(&mut z);
        assert!((z.current - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_zoom_fraction() {
        let mut z = default_camera_zoom();
        z.current = 5.05;
        let f = zoom_fraction(&z);
        assert!((0.0..=1.0).contains(&f));
    }

    #[test]
    fn test_is_zooming() {
        let mut z = default_camera_zoom();
        assert!(!is_zooming(&z));
        z.target = 3.0;
        assert!(is_zooming(&z));
    }

    #[test]
    fn test_fov_from_zoom() {
        let fov = fov_from_zoom(2.0, 60.0);
        assert!((fov - 30.0).abs() < 1e-6);
    }

    #[test]
    fn test_zoom_out_clamp() {
        let mut z = default_camera_zoom();
        zoom_out(&mut z, 100.0);
        assert!((z.target - 0.1).abs() < 1e-6);
    }
}
