// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export a camera path (v2) as a spline sequence.

/// A single camera path keyframe.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CamPathKeyV2 {
    pub time: f32,
    pub position: [f32; 3],
    pub target: [f32; 3],
    pub fov_deg: f32,
}

/// A camera path v2 export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraPathV2 {
    pub keys: Vec<CamPathKeyV2>,
    pub name: String,
}

/// Create a new camera path v2.
#[allow(dead_code)]
pub fn new_camera_path_v2(name: &str) -> CameraPathV2 {
    CameraPathV2 {
        keys: Vec::new(),
        name: name.to_string(),
    }
}

/// Add a keyframe.
#[allow(dead_code)]
pub fn add_cam_path_key_v2(path: &mut CameraPathV2, key: CamPathKeyV2) {
    path.keys.push(key);
    path.keys.sort_by(|a, b| {
        a.time
            .partial_cmp(&b.time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

/// Count keyframes.
#[allow(dead_code)]
pub fn cam_path_v2_key_count(path: &CameraPathV2) -> usize {
    path.keys.len()
}

/// Duration of the path.
#[allow(dead_code)]
pub fn cam_path_v2_duration(path: &CameraPathV2) -> f32 {
    if path.keys.is_empty() {
        return 0.0;
    }
    path.keys.last().map_or(0.0, |k| k.time) - path.keys[0].time
}

/// Linear interpolate position at time t.
#[allow(dead_code)]
pub fn cam_path_v2_position_at(path: &CameraPathV2, t: f32) -> [f32; 3] {
    if path.keys.is_empty() {
        return [0.0; 3];
    }
    if path.keys.len() == 1 {
        return path.keys[0].position;
    }
    let idx = path.keys.partition_point(|k| k.time <= t).saturating_sub(1);
    let i0 = idx.min(path.keys.len() - 1);
    let i1 = (idx + 1).min(path.keys.len() - 1);
    if i0 == i1 {
        return path.keys[i0].position;
    }
    let k0 = &path.keys[i0];
    let k1 = &path.keys[i1];
    let dt = k1.time - k0.time;
    let f = if dt > 0.0 {
        ((t - k0.time) / dt).clamp(0.0, 1.0)
    } else {
        0.0
    };
    [
        k0.position[0] + f * (k1.position[0] - k0.position[0]),
        k0.position[1] + f * (k1.position[1] - k0.position[1]),
        k0.position[2] + f * (k1.position[2] - k0.position[2]),
    ]
}

/// Validate the path (ascending times, positive fov).
#[allow(dead_code)]
pub fn validate_cam_path_v2(path: &CameraPathV2) -> bool {
    path.keys.windows(2).all(|w| w[1].time > w[0].time) && path.keys.iter().all(|k| k.fov_deg > 0.0)
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn cam_path_v2_to_json(path: &CameraPathV2) -> String {
    format!(
        "{{\"name\":\"{}\",\"key_count\":{}}}",
        path.name,
        path.keys.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_path() -> CameraPathV2 {
        let mut p = new_camera_path_v2("main");
        add_cam_path_key_v2(
            &mut p,
            CamPathKeyV2 {
                time: 0.0,
                position: [0.0, 0.0, 0.0],
                target: [0.0, 0.0, 1.0],
                fov_deg: 60.0,
            },
        );
        add_cam_path_key_v2(
            &mut p,
            CamPathKeyV2 {
                time: 1.0,
                position: [1.0, 0.0, 0.0],
                target: [1.0, 0.0, 1.0],
                fov_deg: 60.0,
            },
        );
        p
    }

    #[test]
    fn test_key_count() {
        let p = sample_path();
        assert_eq!(cam_path_v2_key_count(&p), 2);
    }

    #[test]
    fn test_duration() {
        let p = sample_path();
        assert!((cam_path_v2_duration(&p) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_position_at_start() {
        let p = sample_path();
        let pos = cam_path_v2_position_at(&p, 0.0);
        assert!(pos[0].abs() < 1e-5);
    }

    #[test]
    fn test_position_at_end() {
        let p = sample_path();
        let pos = cam_path_v2_position_at(&p, 1.0);
        assert!((pos[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_position_at_mid() {
        let p = sample_path();
        let pos = cam_path_v2_position_at(&p, 0.5);
        assert!((pos[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_validate_valid() {
        let p = sample_path();
        assert!(validate_cam_path_v2(&p));
    }

    #[test]
    fn test_validate_empty() {
        let p = new_camera_path_v2("x");
        assert!(validate_cam_path_v2(&p));
    }

    #[test]
    fn test_cam_path_v2_to_json() {
        let p = sample_path();
        let j = cam_path_v2_to_json(&p);
        assert!(j.contains("key_count"));
    }

    #[test]
    fn test_empty_path_duration_zero() {
        let p = new_camera_path_v2("x");
        assert!(cam_path_v2_duration(&p).abs() < 1e-6);
    }

    #[test]
    fn test_empty_path_position_zero() {
        let p = new_camera_path_v2("x");
        let pos = cam_path_v2_position_at(&p, 0.5);
        assert_eq!(pos, [0.0; 3]);
    }
}
