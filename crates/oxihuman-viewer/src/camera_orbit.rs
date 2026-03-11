// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Orbital/turntable camera controller.

#![allow(dead_code)]

use std::f32::consts::PI;

/// Configuration for the orbital camera.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OrbitConfig {
    pub min_distance: f32,
    pub max_distance: f32,
    pub min_elevation: f32,
    pub max_elevation: f32,
}

/// Runtime state for the orbital camera.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OrbitState {
    pub azimuth: f32,
    pub elevation: f32,
    pub distance: f32,
    pub target: [f32; 3],
}

#[allow(dead_code)]
pub fn default_orbit_config() -> OrbitConfig {
    OrbitConfig {
        min_distance: 0.1,
        max_distance: 100.0,
        min_elevation: -PI / 2.0 + 0.01,
        max_elevation: PI / 2.0 - 0.01,
    }
}

#[allow(dead_code)]
pub fn new_orbit_state() -> OrbitState {
    OrbitState {
        azimuth: 0.0,
        elevation: 0.3,
        distance: 3.0,
        target: [0.0, 0.9, 0.0],
    }
}

#[allow(dead_code)]
pub fn orbit_rotate(state: &mut OrbitState, delta_az: f32, delta_el: f32, config: &OrbitConfig) {
    state.azimuth += delta_az;
    state.elevation = (state.elevation + delta_el).clamp(config.min_elevation, config.max_elevation);
}

#[allow(dead_code)]
pub fn orbit_zoom(state: &mut OrbitState, delta: f32, config: &OrbitConfig) {
    state.distance = (state.distance + delta).clamp(config.min_distance, config.max_distance);
}

#[allow(dead_code)]
pub fn orbit_pan(state: &mut OrbitState, dx: f32, dy: f32) {
    state.target[0] += dx;
    state.target[1] += dy;
}

#[allow(dead_code)]
pub fn orbit_camera_position(state: &OrbitState) -> [f32; 3] {
    let cos_el = state.elevation.cos();
    [
        state.target[0] + state.distance * cos_el * state.azimuth.sin(),
        state.target[1] + state.distance * state.elevation.sin(),
        state.target[2] + state.distance * cos_el * state.azimuth.cos(),
    ]
}

#[allow(dead_code)]
pub fn orbit_to_json(state: &OrbitState) -> String {
    format!(
        r#"{{"azimuth":{:.4},"elevation":{:.4},"distance":{:.4}}}"#,
        state.azimuth, state.elevation, state.distance
    )
}

#[allow(dead_code)]
pub fn orbit_reset(state: &mut OrbitState) {
    *state = new_orbit_state();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_orbit_config();
        assert!((cfg.min_distance - 0.1).abs() < 1e-6);
        assert!((cfg.max_distance - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_orbit_state();
        assert!((s.distance - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_orbit_rotate() {
        let cfg = default_orbit_config();
        let mut s = new_orbit_state();
        let before_az = s.azimuth;
        orbit_rotate(&mut s, 0.5, 0.0, &cfg);
        assert!((s.azimuth - (before_az + 0.5)).abs() < 1e-6);
    }

    #[test]
    fn test_elevation_clamps() {
        let cfg = default_orbit_config();
        let mut s = new_orbit_state();
        orbit_rotate(&mut s, 0.0, 100.0, &cfg);
        assert!(s.elevation <= cfg.max_elevation + 1e-5);
    }

    #[test]
    fn test_orbit_zoom() {
        let cfg = default_orbit_config();
        let mut s = new_orbit_state();
        let before = s.distance;
        orbit_zoom(&mut s, -1.0, &cfg);
        assert!(s.distance < before);
    }

    #[test]
    fn test_zoom_clamps_min() {
        let cfg = default_orbit_config();
        let mut s = new_orbit_state();
        orbit_zoom(&mut s, -10000.0, &cfg);
        assert!(s.distance >= cfg.min_distance - 1e-5);
    }

    #[test]
    fn test_orbit_pan() {
        let mut s = new_orbit_state();
        orbit_pan(&mut s, 1.0, 0.5);
        assert!((s.target[0] - 1.0).abs() < 1e-6);
        assert!((s.target[1] - 1.4).abs() < 1e-5);
    }

    #[test]
    fn test_camera_position_differs_from_target() {
        let s = new_orbit_state();
        let pos = orbit_camera_position(&s);
        let target = s.target;
        let diff = (pos[0] - target[0]).abs() + (pos[1] - target[1]).abs() + (pos[2] - target[2]).abs();
        assert!(diff > 0.1);
    }
}
