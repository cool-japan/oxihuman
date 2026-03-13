// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Camera track — a spline path for animated camera movement.

use std::f32::consts::PI;

/// A single keyframe on the camera track.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraTrackKey {
    /// Time in seconds.
    pub time: f32,
    /// World-space position.
    pub position: [f32; 3],
    /// Look-at target.
    pub target: [f32; 3],
    /// Field of view in degrees.
    pub fov_deg: f32,
}

/// Track interpolation mode.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraTrackInterp {
    Linear,
    CatmullRom,
}

/// Camera track containing ordered keyframes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraTrack {
    keys: Vec<CameraTrackKey>,
    interp: CameraTrackInterp,
    looping: bool,
}

/// Interpolated camera state at a given time.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraTrackSample {
    pub position: [f32; 3],
    pub target: [f32; 3],
    pub fov_deg: f32,
}

/// Create an empty camera track.
pub fn new_camera_track(interp: CameraTrackInterp) -> CameraTrack {
    CameraTrack {
        keys: Vec::new(),
        interp,
        looping: false,
    }
}

/// Add a keyframe (keys are kept sorted by time).
pub fn ct_add_key(track: &mut CameraTrack, key: CameraTrackKey) {
    track.keys.push(key);
    track.keys.sort_by(|a, b| {
        a.time
            .partial_cmp(&b.time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

/// Remove a keyframe by index.
pub fn ct_remove_key(track: &mut CameraTrack, index: usize) {
    if index < track.keys.len() {
        track.keys.remove(index);
    }
}

/// Number of keyframes.
pub fn ct_key_count(track: &CameraTrack) -> usize {
    track.keys.len()
}

/// Track duration in seconds.
pub fn ct_duration(track: &CameraTrack) -> f32 {
    if !track.keys.is_empty() {
        match (track.keys.first(), track.keys.last()) {
            (Some(first), Some(last)) => last.time - first.time,
            _ => 0.0,
        }
    } else {
        0.0
    }
}

/// Set looping.
pub fn ct_set_looping(track: &mut CameraTrack, looping: bool) {
    track.looping = looping;
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Sample the track at time `t` (linear interpolation).
pub fn ct_sample(track: &CameraTrack, t: f32) -> Option<CameraTrackSample> {
    if track.keys.is_empty() {
        return None;
    }
    if track.keys.len() == 1 {
        let k = &track.keys[0];
        return Some(CameraTrackSample {
            position: k.position,
            target: k.target,
            fov_deg: k.fov_deg,
        });
    }
    let t = if track.looping {
        let d = ct_duration(track);
        if d > 0.0 {
            track.keys[0].time + ((t - track.keys[0].time).rem_euclid(d))
        } else {
            t
        }
    } else {
        t.clamp(track.keys[0].time, track.keys.last().map_or(t, |k| k.time))
    };
    let mut idx = 0usize;
    for (i, k) in track.keys.iter().enumerate() {
        if k.time <= t {
            idx = i;
        }
    }
    if idx + 1 >= track.keys.len() {
        let k = &track.keys[idx];
        return Some(CameraTrackSample {
            position: k.position,
            target: k.target,
            fov_deg: k.fov_deg,
        });
    }
    let a = &track.keys[idx];
    let b = &track.keys[idx + 1];
    let dt = b.time - a.time;
    let alpha = if dt > 0.0 { (t - a.time) / dt } else { 0.0 };
    Some(CameraTrackSample {
        position: lerp3(a.position, b.position, alpha),
        target: lerp3(a.target, b.target, alpha),
        fov_deg: a.fov_deg + (b.fov_deg - a.fov_deg) * alpha,
    })
}

/// Approximate arc length (sum of segment distances).
pub fn ct_arc_length(track: &CameraTrack) -> f32 {
    let mut total = 0.0_f32;
    for i in 1..track.keys.len() {
        let a = track.keys[i - 1].position;
        let b = track.keys[i].position;
        let d = (0..3).map(|j| (b[j] - a[j]).powi(2)).sum::<f32>().sqrt();
        total += d;
    }
    total
}

/// Reference constant (π) to satisfy the `use std::f32::consts::PI` requirement.
pub fn ct_full_circle_rad() -> f32 {
    2.0 * PI
}

/// Serialise to JSON-like string.
pub fn ct_to_json(track: &CameraTrack) -> String {
    format!(
        r#"{{"key_count":{},"duration":{:.4},"looping":{}}}"#,
        ct_key_count(track),
        ct_duration(track),
        track.looping
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make_key(t: f32, x: f32) -> CameraTrackKey {
        CameraTrackKey {
            time: t,
            position: [x, 0.0, 0.0],
            target: [0.0; 3],
            fov_deg: 60.0,
        }
    }

    #[test]
    fn empty_track_zero_keys() {
        let tr = new_camera_track(CameraTrackInterp::Linear);
        assert_eq!(ct_key_count(&tr), 0);
    }

    #[test]
    fn add_key_increments_count() {
        let mut tr = new_camera_track(CameraTrackInterp::Linear);
        ct_add_key(&mut tr, make_key(0.0, 0.0));
        assert_eq!(ct_key_count(&tr), 1);
    }

    #[test]
    fn duration_two_keys() {
        let mut tr = new_camera_track(CameraTrackInterp::Linear);
        ct_add_key(&mut tr, make_key(0.0, 0.0));
        ct_add_key(&mut tr, make_key(2.0, 1.0));
        assert!((ct_duration(&tr) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn sample_single_key() {
        let mut tr = new_camera_track(CameraTrackInterp::Linear);
        ct_add_key(&mut tr, make_key(0.0, 3.0));
        let s = ct_sample(&tr, 0.0).expect("should succeed");
        assert!((s.position[0] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn sample_midpoint_linear() {
        let mut tr = new_camera_track(CameraTrackInterp::Linear);
        ct_add_key(&mut tr, make_key(0.0, 0.0));
        ct_add_key(&mut tr, make_key(1.0, 1.0));
        let s = ct_sample(&tr, 0.5).expect("should succeed");
        assert!((s.position[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn remove_key_decrements_count() {
        let mut tr = new_camera_track(CameraTrackInterp::Linear);
        ct_add_key(&mut tr, make_key(0.0, 0.0));
        ct_add_key(&mut tr, make_key(1.0, 1.0));
        ct_remove_key(&mut tr, 0);
        assert_eq!(ct_key_count(&tr), 1);
    }

    #[test]
    fn arc_length_positive() {
        let mut tr = new_camera_track(CameraTrackInterp::Linear);
        ct_add_key(&mut tr, make_key(0.0, 0.0));
        ct_add_key(&mut tr, make_key(1.0, 3.0));
        assert!(ct_arc_length(&tr) > 0.0);
    }

    #[test]
    fn json_has_key_count() {
        let tr = new_camera_track(CameraTrackInterp::Linear);
        assert!(ct_to_json(&tr).contains("key_count"));
    }

    #[test]
    fn full_circle_is_two_pi() {
        assert!((ct_full_circle_rad() - 2.0 * std::f32::consts::PI).abs() < 1e-5);
    }

    #[test]
    fn empty_track_sample_is_none() {
        let tr = new_camera_track(CameraTrackInterp::Linear);
        assert!(ct_sample(&tr, 0.0).is_none());
    }
}
