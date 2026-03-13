// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A keyframe on a camera path.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct PathKeyframe {
    pub time: f32,
    pub position: [f32; 3],
    pub look_at: [f32; 3],
}

/// A camera path defined by keyframes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraPath {
    pub keyframes: Vec<PathKeyframe>,
    pub looping: bool,
}

/// Create a new empty camera path.
#[allow(dead_code)]
pub fn new_camera_path() -> CameraPath {
    CameraPath {
        keyframes: Vec::new(),
        looping: false,
    }
}

/// Add a keyframe to the path (sorted by time).
#[allow(dead_code)]
pub fn add_camera_keyframe(
    path: &mut CameraPath,
    time: f32,
    position: [f32; 3],
    look_at: [f32; 3],
) {
    let kf = PathKeyframe {
        time,
        position,
        look_at,
    };
    let pos = path.keyframes.partition_point(|k| k.time < time);
    path.keyframes.insert(pos, kf);
}

/// Evaluate the camera path at a given time (linear interpolation).
#[allow(dead_code)]
pub fn evaluate_camera_path(path: &CameraPath, t: f32) -> ([f32; 3], [f32; 3]) {
    if path.keyframes.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    if path.keyframes.len() == 1 || t <= path.keyframes[0].time {
        return (path.keyframes[0].position, path.keyframes[0].look_at);
    }
    let last = path.keyframes.len() - 1;
    if t >= path.keyframes[last].time {
        return (path.keyframes[last].position, path.keyframes[last].look_at);
    }
    for i in 0..last {
        let a = &path.keyframes[i];
        let b = &path.keyframes[i + 1];
        if (a.time..=b.time).contains(&t) {
            let frac = if (b.time - a.time).abs() < 1e-9 {
                0.0
            } else {
                (t - a.time) / (b.time - a.time)
            };
            let mut pos = [0.0_f32; 3];
            let mut look = [0.0_f32; 3];
            for j in 0..3 {
                pos[j] = a.position[j] + (b.position[j] - a.position[j]) * frac;
                look[j] = a.look_at[j] + (b.look_at[j] - a.look_at[j]) * frac;
            }
            return (pos, look);
        }
    }
    (path.keyframes[last].position, path.keyframes[last].look_at)
}

/// Return the total duration of the path.
#[allow(dead_code)]
pub fn path_duration(path: &CameraPath) -> f32 {
    if path.keyframes.len() < 2 {
        return 0.0;
    }
    path.keyframes.last().map_or(0.0, |kf| kf.time) - path.keyframes[0].time
}

/// Return the number of keyframes.
#[allow(dead_code)]
pub fn keyframe_count_cp(path: &CameraPath) -> usize {
    path.keyframes.len()
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn path_to_json(path: &CameraPath) -> String {
    let kfs: Vec<String> = path
        .keyframes
        .iter()
        .map(|k| {
            format!(
                "{{\"t\":{:.4},\"pos\":[{:.4},{:.4},{:.4}]}}",
                k.time, k.position[0], k.position[1], k.position[2]
            )
        })
        .collect();
    format!(
        "{{\"looping\":{},\"keyframes\":[{}]}}",
        path.looping,
        kfs.join(",")
    )
}

/// Remove all keyframes.
#[allow(dead_code)]
pub fn path_clear(path: &mut CameraPath) {
    path.keyframes.clear();
}

/// Check if the path is set to loop.
#[allow(dead_code)]
pub fn path_is_looping(path: &CameraPath) -> bool {
    path.looping
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_path_empty() {
        let p = new_camera_path();
        assert_eq!(keyframe_count_cp(&p), 0);
    }

    #[test]
    fn add_keyframe() {
        let mut p = new_camera_path();
        add_camera_keyframe(&mut p, 0.0, [0.0; 3], [0.0, 0.0, -1.0]);
        assert_eq!(keyframe_count_cp(&p), 1);
    }

    #[test]
    fn evaluate_single() {
        let mut p = new_camera_path();
        add_camera_keyframe(&mut p, 0.0, [1.0, 2.0, 3.0], [0.0; 3]);
        let (pos, _) = evaluate_camera_path(&p, 0.0);
        assert!((pos[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn evaluate_interpolation() {
        let mut p = new_camera_path();
        add_camera_keyframe(&mut p, 0.0, [0.0, 0.0, 0.0], [0.0; 3]);
        add_camera_keyframe(&mut p, 1.0, [10.0, 0.0, 0.0], [0.0; 3]);
        let (pos, _) = evaluate_camera_path(&p, 0.5);
        assert!((pos[0] - 5.0).abs() < 1e-4);
    }

    #[test]
    fn duration() {
        let mut p = new_camera_path();
        add_camera_keyframe(&mut p, 1.0, [0.0; 3], [0.0; 3]);
        add_camera_keyframe(&mut p, 3.0, [0.0; 3], [0.0; 3]);
        assert!((path_duration(&p) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn not_looping_by_default() {
        let p = new_camera_path();
        assert!(!path_is_looping(&p));
    }

    #[test]
    fn clear_path() {
        let mut p = new_camera_path();
        add_camera_keyframe(&mut p, 0.0, [0.0; 3], [0.0; 3]);
        path_clear(&mut p);
        assert_eq!(keyframe_count_cp(&p), 0);
    }

    #[test]
    fn to_json() {
        let mut p = new_camera_path();
        add_camera_keyframe(&mut p, 0.0, [1.0, 0.0, 0.0], [0.0; 3]);
        let j = path_to_json(&p);
        assert!(j.contains("keyframes"));
    }

    #[test]
    fn evaluate_empty() {
        let p = new_camera_path();
        let (pos, _) = evaluate_camera_path(&p, 0.0);
        assert!(pos[0].abs() < 1e-6);
    }

    #[test]
    fn duration_single_keyframe() {
        let mut p = new_camera_path();
        add_camera_keyframe(&mut p, 1.0, [0.0; 3], [0.0; 3]);
        assert!(path_duration(&p).abs() < 1e-6);
    }
}
