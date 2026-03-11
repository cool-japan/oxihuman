// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A single motion capture marker.
#[derive(Debug, Clone)]
pub struct MarkerPos {
    pub name: String,
    pub pos: [f32; 3],
}

/// A single frame of motion capture data.
#[derive(Debug, Clone)]
pub struct MotionCaptureFrame {
    pub time: f32,
    pub markers: Vec<MarkerPos>,
}

/// Create a new empty MotionCaptureFrame.
pub fn new_mocap_frame(time: f32) -> MotionCaptureFrame {
    MotionCaptureFrame {
        time,
        markers: Vec::new(),
    }
}

/// Add a marker to the frame.
pub fn mocap_add_marker(frame: &mut MotionCaptureFrame, name: &str, pos: [f32; 3]) {
    frame.markers.push(MarkerPos {
        name: name.to_string(),
        pos,
    });
}

/// Count of markers in the frame.
pub fn mocap_marker_count(frame: &MotionCaptureFrame) -> usize {
    frame.markers.len()
}

/// Find a marker by name, returning its position if found.
pub fn mocap_find_marker(frame: &MotionCaptureFrame, name: &str) -> Option<[f32; 3]> {
    frame.markers.iter().find(|m| m.name == name).map(|m| m.pos)
}

/// Total duration covered by a sequence of frames (last - first time).
pub fn mocap_frame_duration(frames: &[MotionCaptureFrame]) -> f32 {
    if frames.len() < 2 {
        return 0.0;
    }
    frames[frames.len() - 1].time - frames[0].time
}

/// Estimate marker velocity between two consecutive frames.
pub fn mocap_marker_velocity(
    f0: &MotionCaptureFrame,
    f1: &MotionCaptureFrame,
    name: &str,
) -> Option<[f32; 3]> {
    let dt = f1.time - f0.time;
    if dt.abs() < 1e-9 {
        return None;
    }
    let p0 = mocap_find_marker(f0, name)?;
    let p1 = mocap_find_marker(f1, name)?;
    Some([
        (p1[0] - p0[0]) / dt,
        (p1[1] - p0[1]) / dt,
        (p1[2] - p0[2]) / dt,
    ])
}

/// Centroid position of all markers in a frame.
pub fn mocap_centroid(frame: &MotionCaptureFrame) -> [f32; 3] {
    let n = frame.markers.len() as f32;
    if n == 0.0 {
        return [0.0, 0.0, 0.0];
    }
    let sx: f32 = frame.markers.iter().map(|m| m.pos[0]).sum();
    let sy: f32 = frame.markers.iter().map(|m| m.pos[1]).sum();
    let sz: f32 = frame.markers.iter().map(|m| m.pos[2]).sum();
    [sx / n, sy / n, sz / n]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_mocap_frame() {
        /* empty frame */
        let f = new_mocap_frame(0.0);
        assert_eq!(mocap_marker_count(&f), 0);
        assert!((f.time).abs() < 1e-9);
    }

    #[test]
    fn test_mocap_add_marker() {
        /* add one marker */
        let mut f = new_mocap_frame(0.0);
        mocap_add_marker(&mut f, "head", [0.0, 1.7, 0.0]);
        assert_eq!(mocap_marker_count(&f), 1);
    }

    #[test]
    fn test_mocap_find_marker_found() {
        let mut f = new_mocap_frame(0.1);
        mocap_add_marker(&mut f, "lhand", [0.5, 1.0, 0.0]);
        let p = mocap_find_marker(&f, "lhand");
        assert!(p.is_some());
        assert!((p.unwrap()[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_mocap_find_marker_not_found() {
        let f = new_mocap_frame(0.0);
        assert!(mocap_find_marker(&f, "ghost").is_none());
    }

    #[test]
    fn test_mocap_frame_duration_empty() {
        let frames: Vec<MotionCaptureFrame> = vec![];
        assert!(mocap_frame_duration(&frames).abs() < 1e-9);
    }

    #[test]
    fn test_mocap_frame_duration_two() {
        let f0 = new_mocap_frame(0.0);
        let f1 = new_mocap_frame(1.0 / 30.0);
        let d = mocap_frame_duration(&[f0, f1]);
        assert!((d - 1.0 / 30.0).abs() < 1e-6);
    }

    #[test]
    fn test_mocap_marker_velocity() {
        let mut f0 = new_mocap_frame(0.0);
        let mut f1 = new_mocap_frame(1.0);
        mocap_add_marker(&mut f0, "foot", [0.0, 0.0, 0.0]);
        mocap_add_marker(&mut f1, "foot", [1.0, 0.0, 0.0]);
        let vel = mocap_marker_velocity(&f0, &f1, "foot");
        assert!(vel.is_some());
        assert!((vel.unwrap()[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_mocap_velocity_missing_marker() {
        let f0 = new_mocap_frame(0.0);
        let f1 = new_mocap_frame(1.0);
        assert!(mocap_marker_velocity(&f0, &f1, "nonexistent").is_none());
    }

    #[test]
    fn test_mocap_centroid_single() {
        let mut f = new_mocap_frame(0.0);
        mocap_add_marker(&mut f, "p", [3.0, 4.0, 5.0]);
        let c = mocap_centroid(&f);
        assert!((c[0] - 3.0).abs() < 1e-6);
    }
}
