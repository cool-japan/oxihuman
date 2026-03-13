// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Camera path visualization for the 3D viewer.

/// A camera path waypoint.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraWaypoint {
    pub position: [f32; 3],
    pub target: [f32; 3],
    pub fov_deg: f32,
    pub time: f32,
}

/// Camera path configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraPathView {
    pub waypoints: Vec<CameraWaypoint>,
    pub show_path: bool,
    pub show_points: bool,
    pub loop_path: bool,
}

/// Default waypoint.
#[allow(dead_code)]
pub fn default_waypoint() -> CameraWaypoint {
    CameraWaypoint {
        position: [0.0, 1.0, -3.0],
        target: [0.0, 0.9, 0.0],
        fov_deg: 60.0,
        time: 0.0,
    }
}

/// Create empty camera path view.
#[allow(dead_code)]
pub fn new_camera_path_view() -> CameraPathView {
    CameraPathView {
        waypoints: Vec::new(),
        show_path: true,
        show_points: true,
        loop_path: false,
    }
}

/// Add a waypoint.
#[allow(dead_code)]
pub fn add_waypoint(path: &mut CameraPathView, wp: CameraWaypoint) {
    path.waypoints.push(wp);
}

/// Remove a waypoint by index.
#[allow(dead_code)]
pub fn remove_waypoint(path: &mut CameraPathView, index: usize) -> bool {
    if index < path.waypoints.len() {
        path.waypoints.remove(index);
        true
    } else {
        false
    }
}

/// Interpolate position at time t using linear interpolation.
#[allow(dead_code)]
pub fn interpolate_path(path: &CameraPathView, t: f32) -> Option<CameraWaypoint> {
    if path.waypoints.len() < 2 {
        return path.waypoints.first().cloned();
    }
    let total_time = path.waypoints.last()?.time;
    if total_time <= 0.0 {
        return path.waypoints.first().cloned();
    }
    let clamped_t = if path.loop_path { t % total_time } else { t.clamp(0.0, total_time) };
    for i in 0..path.waypoints.len() - 1 {
        let a = &path.waypoints[i];
        let b = &path.waypoints[i + 1];
        if clamped_t >= a.time && clamped_t <= b.time {
            let seg_t = if (b.time - a.time).abs() > 1e-6 {
                (clamped_t - a.time) / (b.time - a.time)
            } else {
                0.0
            };
            return Some(lerp_waypoint(a, b, seg_t));
        }
    }
    path.waypoints.last().cloned()
}

#[allow(dead_code)]
fn lerp_waypoint(a: &CameraWaypoint, b: &CameraWaypoint, t: f32) -> CameraWaypoint {
    CameraWaypoint {
        position: [
            a.position[0] + (b.position[0] - a.position[0]) * t,
            a.position[1] + (b.position[1] - a.position[1]) * t,
            a.position[2] + (b.position[2] - a.position[2]) * t,
        ],
        target: [
            a.target[0] + (b.target[0] - a.target[0]) * t,
            a.target[1] + (b.target[1] - a.target[1]) * t,
            a.target[2] + (b.target[2] - a.target[2]) * t,
        ],
        fov_deg: a.fov_deg + (b.fov_deg - a.fov_deg) * t,
        time: a.time + (b.time - a.time) * t,
    }
}

/// Get total path duration.
#[allow(dead_code)]
pub fn path_duration(path: &CameraPathView) -> f32 {
    path.waypoints.last().map_or(0.0, |w| w.time)
}

/// Waypoint count.
#[allow(dead_code)]
pub fn waypoint_count(path: &CameraPathView) -> usize {
    path.waypoints.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_waypoint() {
        let w = default_waypoint();
        assert!((w.fov_deg - 60.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_path() {
        let p = new_camera_path_view();
        assert!(p.waypoints.is_empty());
    }

    #[test]
    fn test_add_waypoint() {
        let mut p = new_camera_path_view();
        add_waypoint(&mut p, default_waypoint());
        assert_eq!(waypoint_count(&p), 1);
    }

    #[test]
    fn test_remove_waypoint() {
        let mut p = new_camera_path_view();
        add_waypoint(&mut p, default_waypoint());
        assert!(remove_waypoint(&mut p, 0));
        assert_eq!(waypoint_count(&p), 0);
    }

    #[test]
    fn test_remove_invalid() {
        let mut p = new_camera_path_view();
        assert!(!remove_waypoint(&mut p, 0));
    }

    #[test]
    fn test_interpolate_single() {
        let mut p = new_camera_path_view();
        add_waypoint(&mut p, default_waypoint());
        let r = interpolate_path(&p, 0.0);
        assert!(r.is_some());
    }

    #[test]
    fn test_interpolate_two() {
        let mut p = new_camera_path_view();
        let mut w1 = default_waypoint();
        w1.time = 0.0;
        let mut w2 = default_waypoint();
        w2.time = 1.0;
        w2.position = [1.0, 1.0, -3.0];
        add_waypoint(&mut p, w1);
        add_waypoint(&mut p, w2);
        let r = interpolate_path(&p, 0.5).expect("should succeed");
        assert!((r.position[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_path_duration() {
        let mut p = new_camera_path_view();
        let mut w = default_waypoint();
        w.time = 5.0;
        add_waypoint(&mut p, w);
        assert!((path_duration(&p) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_empty_duration() {
        let p = new_camera_path_view();
        assert!(path_duration(&p).abs() < 1e-6);
    }

    #[test]
    fn test_interpolate_empty() {
        let p = new_camera_path_view();
        assert!(interpolate_path(&p, 0.0).is_none());
    }
}
