// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Camera track export: animated camera path keyframes.

/// A camera keyframe.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct CameraKeyframe {
    pub time: f32,
    pub position: [f32; 3],
    pub target: [f32; 3],
    pub fov_deg: f32,
}

/// A camera track export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraTrackExport {
    pub name: String,
    pub keyframes: Vec<CameraKeyframe>,
    pub fps: f32,
}

/// Create a new camera track.
#[allow(dead_code)]
pub fn new_camera_track(name: &str, fps: f32) -> CameraTrackExport {
    CameraTrackExport {
        name: name.to_string(),
        keyframes: Vec::new(),
        fps,
    }
}

/// Add a keyframe.
#[allow(dead_code)]
pub fn add_camera_keyframe(track: &mut CameraTrackExport, kf: CameraKeyframe) {
    track.keyframes.push(kf);
    track.keyframes.sort_by(|a, b| {
        a.time
            .partial_cmp(&b.time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

/// Keyframe count.
#[allow(dead_code)]
pub fn camera_keyframe_count(track: &CameraTrackExport) -> usize {
    track.keyframes.len()
}

/// Duration (time of last keyframe).
#[allow(dead_code)]
pub fn camera_track_duration(track: &CameraTrackExport) -> f32 {
    track.keyframes.last().map_or(0.0, |k| k.time)
}

/// Sample position by linear interpolation at time t.
#[allow(dead_code)]
pub fn sample_camera_position(track: &CameraTrackExport, t: f32) -> [f32; 3] {
    let kfs = &track.keyframes;
    if kfs.is_empty() {
        return [0.0; 3];
    }
    if t <= kfs[0].time {
        return kfs[0].position;
    }
    if t >= kfs[kfs.len() - 1].time {
        return kfs[kfs.len() - 1].position;
    }
    for i in 1..kfs.len() {
        if kfs[i].time >= t {
            let span = kfs[i].time - kfs[i - 1].time;
            let alpha = if span > 0.0 {
                (t - kfs[i - 1].time) / span
            } else {
                0.0
            };
            let a = kfs[i - 1].position;
            let b = kfs[i].position;
            return [
                a[0] + alpha * (b[0] - a[0]),
                a[1] + alpha * (b[1] - a[1]),
                a[2] + alpha * (b[2] - a[2]),
            ];
        }
    }
    kfs[kfs.len() - 1].position
}

/// Validate: fov in (0, 180).
#[allow(dead_code)]
pub fn validate_camera_track(track: &CameraTrackExport) -> bool {
    track
        .keyframes
        .iter()
        .all(|k| k.fov_deg > 0.0 && k.fov_deg < 180.0)
}

/// Serialise to JSON.
#[allow(dead_code)]
pub fn camera_track_to_json(track: &CameraTrackExport) -> String {
    format!(
        "{{\"name\":\"{}\",\"keyframe_count\":{},\"duration\":{}}}",
        track.name,
        camera_keyframe_count(track),
        camera_track_duration(track)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kf(t: f32) -> CameraKeyframe {
        CameraKeyframe {
            time: t,
            position: [t, 0.0, 0.0],
            target: [0.0; 3],
            fov_deg: 60.0,
        }
    }

    #[test]
    fn new_track_empty() {
        let t = new_camera_track("main", 24.0);
        assert_eq!(camera_keyframe_count(&t), 0);
    }

    #[test]
    fn add_keyframe_increments() {
        let mut t = new_camera_track("main", 24.0);
        add_camera_keyframe(&mut t, kf(0.0));
        assert_eq!(camera_keyframe_count(&t), 1);
    }

    #[test]
    fn duration_last_kf() {
        let mut t = new_camera_track("main", 24.0);
        add_camera_keyframe(&mut t, kf(0.0));
        add_camera_keyframe(&mut t, kf(2.0));
        assert!((camera_track_duration(&t) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn sample_before_start() {
        let mut t = new_camera_track("c", 24.0);
        add_camera_keyframe(&mut t, kf(1.0));
        let p = sample_camera_position(&t, 0.0);
        assert!((p[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn sample_midpoint() {
        let mut t = new_camera_track("c", 24.0);
        add_camera_keyframe(&mut t, kf(0.0));
        add_camera_keyframe(&mut t, kf(2.0));
        let p = sample_camera_position(&t, 1.0);
        assert!((p[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn validate_valid_fov() {
        let mut t = new_camera_track("c", 24.0);
        add_camera_keyframe(&mut t, kf(0.0));
        assert!(validate_camera_track(&t));
    }

    #[test]
    fn json_contains_name() {
        let t = new_camera_track("hero_cam", 30.0);
        let j = camera_track_to_json(&t);
        assert!(j.contains("hero_cam"));
    }

    #[test]
    fn fov_in_valid_range() {
        let kf = kf(0.0);
        assert!((0.0..=180.0).contains(&kf.fov_deg));
    }

    #[test]
    fn keyframes_sorted_after_add() {
        let mut t = new_camera_track("c", 24.0);
        add_camera_keyframe(&mut t, kf(2.0));
        add_camera_keyframe(&mut t, kf(0.5));
        assert!(t.keyframes[0].time <= t.keyframes[1].time);
    }

    #[test]
    fn sample_empty_returns_origin() {
        let t = new_camera_track("c", 24.0);
        let p = sample_camera_position(&t, 1.0);
        assert!((p[0]).abs() < 1e-6);
    }
}
