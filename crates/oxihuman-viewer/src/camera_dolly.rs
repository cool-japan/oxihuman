// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]
//! Camera dolly: moves the camera along a path of points.

/// A point on the dolly path.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct DollyPoint {
    position: [f32; 3],
    time: f32,
}

/// A dolly path.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DollyPath {
    points: Vec<DollyPoint>,
}

/// A camera dolly.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraDolly {
    path: DollyPath,
}

/// Create a new camera dolly.
#[allow(dead_code)]
pub fn new_camera_dolly() -> CameraDolly {
    CameraDolly {
        path: DollyPath { points: Vec::new() },
    }
}

/// Add a point at the given time.
#[allow(dead_code)]
pub fn add_dolly_point(dolly: &mut CameraDolly, position: [f32; 3], time: f32) {
    dolly.path.points.push(DollyPoint { position, time });
    dolly.path.points.sort_by(|a, b| {
        a.time
            .partial_cmp(&b.time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

/// Evaluate the dolly at time `t`, returning interpolated position.
#[allow(dead_code)]
pub fn dolly_evaluate(dolly: &CameraDolly, t: f32) -> [f32; 3] {
    if dolly.path.points.is_empty() {
        return [0.0; 3];
    }
    if dolly.path.points.len() == 1 {
        return dolly.path.points[0].position;
    }
    let clamped = t.clamp(
        dolly.path.points[0].time,
        dolly.path.points.last().map_or(0.0, |p| p.time),
    );
    for i in 0..dolly.path.points.len() - 1 {
        let a = &dolly.path.points[i];
        let b = &dolly.path.points[i + 1];
        if (a.time..=b.time).contains(&clamped) {
            let span = b.time - a.time;
            if span.abs() < 1e-9 {
                return a.position;
            }
            let frac = (clamped - a.time) / span;
            return [
                a.position[0] + (b.position[0] - a.position[0]) * frac,
                a.position[1] + (b.position[1] - a.position[1]) * frac,
                a.position[2] + (b.position[2] - a.position[2]) * frac,
            ];
        }
    }
    dolly.path.points.last().map_or([0.0; 3], |p| p.position)
}

/// Return the number of points.
#[allow(dead_code)]
pub fn dolly_point_count(dolly: &CameraDolly) -> usize {
    dolly.path.points.len()
}

/// Return the total duration.
#[allow(dead_code)]
pub fn dolly_duration(dolly: &CameraDolly) -> f32 {
    if dolly.path.points.len() < 2 {
        return 0.0;
    }
    dolly.path.points.last().map_or(0.0, |p| p.time) - dolly.path.points[0].time
}

/// Return the total path distance.
#[allow(dead_code)]
pub fn dolly_distance(dolly: &CameraDolly) -> f32 {
    let mut dist = 0.0f32;
    for i in 1..dolly.path.points.len() {
        let a = &dolly.path.points[i - 1].position;
        let b = &dolly.path.points[i].position;
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let dz = b[2] - a[2];
        dist += (dx * dx + dy * dy + dz * dz).sqrt();
    }
    dist
}

/// Serialize to JSON-like string.
#[allow(dead_code)]
pub fn dolly_to_json(dolly: &CameraDolly) -> String {
    format!(
        "{{\"point_count\":{},\"duration\":{}}}",
        dolly.path.points.len(),
        dolly_duration(dolly)
    )
}

/// Clear all points.
#[allow(dead_code)]
pub fn dolly_reset(dolly: &mut CameraDolly) {
    dolly.path.points.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_dolly() {
        let d = new_camera_dolly();
        assert_eq!(dolly_point_count(&d), 0);
    }

    #[test]
    fn test_add_point() {
        let mut d = new_camera_dolly();
        add_dolly_point(&mut d, [0.0, 0.0, 0.0], 0.0);
        assert_eq!(dolly_point_count(&d), 1);
    }

    #[test]
    fn test_evaluate_empty() {
        let d = new_camera_dolly();
        let p = dolly_evaluate(&d, 0.0);
        assert!((p[0]).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_single() {
        let mut d = new_camera_dolly();
        add_dolly_point(&mut d, [1.0, 2.0, 3.0], 0.0);
        let p = dolly_evaluate(&d, 0.0);
        assert!((p[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_interpolation() {
        let mut d = new_camera_dolly();
        add_dolly_point(&mut d, [0.0, 0.0, 0.0], 0.0);
        add_dolly_point(&mut d, [10.0, 0.0, 0.0], 1.0);
        let p = dolly_evaluate(&d, 0.5);
        assert!((p[0] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_duration() {
        let mut d = new_camera_dolly();
        add_dolly_point(&mut d, [0.0; 3], 0.0);
        add_dolly_point(&mut d, [0.0; 3], 3.0);
        assert!((dolly_duration(&d) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_distance() {
        let mut d = new_camera_dolly();
        add_dolly_point(&mut d, [0.0, 0.0, 0.0], 0.0);
        add_dolly_point(&mut d, [3.0, 4.0, 0.0], 1.0);
        assert!((dolly_distance(&d) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let d = new_camera_dolly();
        let json = dolly_to_json(&d);
        assert!(json.contains("\"point_count\":0"));
    }

    #[test]
    fn test_reset() {
        let mut d = new_camera_dolly();
        add_dolly_point(&mut d, [0.0; 3], 0.0);
        dolly_reset(&mut d);
        assert_eq!(dolly_point_count(&d), 0);
    }

    #[test]
    fn test_duration_single() {
        let mut d = new_camera_dolly();
        add_dolly_point(&mut d, [0.0; 3], 1.0);
        assert!((dolly_duration(&d) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_clamped() {
        let mut d = new_camera_dolly();
        add_dolly_point(&mut d, [0.0; 3], 0.0);
        add_dolly_point(&mut d, [10.0, 0.0, 0.0], 1.0);
        let p = dolly_evaluate(&d, 5.0);
        assert!((p[0] - 10.0).abs() < 1e-6);
    }
}
